use super::encryption::{decrypt_string, encrypt_string};
use crate::entity::platform_session;
use chrono::Utc;

use sea_orm::{DatabaseConnection, EntityTrait, Set};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub struct CookieManager {
    db: Arc<DatabaseConnection>,
    temp_dir: PathBuf,
}

impl CookieManager {
    pub fn new(db: Arc<DatabaseConnection>, app_data_dir: PathBuf) -> Self {
        let temp_dir = app_data_dir.join("system").join("temp");
        // Ensure temp dir exists (in a real app, this should probably be done at startup)
        // For now we'll assume the caller handles or we handle it lazily
        Self { db, temp_dir }
    }

    /// initialize temp directory
    pub async fn init(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.temp_dir).await
    }

    pub async fn set_session(
        &self,
        platform_id: String,
        mut cookies_str: String,
        method: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!(
            "Setting session for platform: {}, method: {}",
            platform_id,
            method
        );

        // Validate cookies string (basic check)
        if cookies_str.trim().is_empty() {
            tracing::warn!("Cookies string is empty for {}", platform_id);
            return Err("Cookies string is empty".into());
        }

        // Check for JSON format (starts with [ or {) and convert to Netscape
        let trimmed = cookies_str.trim();
        if trimmed.starts_with('[') || trimmed.starts_with('{') {
            tracing::info!("Detected JSON cookies, attempting conversion to Netscape format");
            match Self::convert_json_to_netscape(trimmed) {
                Ok(netscape) => {
                    cookies_str = netscape;
                    tracing::info!("Successfully converted JSON cookies to Netscape format");
                }
                Err(msg) => {
                    tracing::warn!("JSON conversion failed: {}", msg);
                    return Err(msg.into());
                }
            }
        }

        // Validate that the cookies actually contain the required auth tokens
        self.validate_session_cookies(&platform_id, &cookies_str)
            .await?;

        // Extract username and avatar via network API
        let mut username: Option<String> = None;
        let mut avatar_url: Option<String> = None;

        if let Some((handle, avatar)) =
            crate::auth::api::UsernameFetcher::fetch_profile(&platform_id, &cookies_str).await
        {
            username = Some(handle);
            avatar_url = avatar;
        }

        let encrypted_data = encrypt_string(&cookies_str)?;

        // Parse expiration from cookies
        // format is domain \t flag \t path \t secure \t expiration \t name \t value
        let mut max_expiration: Option<i64> = None;
        for line in cookies_str.lines() {
            let line = line.trim();
            if line.is_empty() || (line.starts_with('#') && !line.starts_with("#HttpOnly_")) {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 7 {
                if let Ok(exp) = parts[4].parse::<i64>() {
                    if exp > 0 {
                        max_expiration = Some(max_expiration.unwrap_or(0).max(exp));
                    }
                }
            }
        }
        let parsed_expires_at =
            max_expiration.and_then(|exp| chrono::DateTime::from_timestamp(exp, 0));

        let now = Utc::now();

        let session = platform_session::ActiveModel {
            platform_id: Set(platform_id.clone()),
            status: Set("ACTIVE".to_string()),
            username: Set(username),
            avatar_url: Set(avatar_url),
            encrypted_cookies: Set(Some(encrypted_data)),
            cookie_method: Set(method),
            expires_at: Set(parsed_expires_at),
            last_verified: Set(Some(now)),
            error_message: Set(None),
            updated_at: Set(now),
            created_at: Set(now), // sea-orm handles ignore on update for created_at if configured, or we can query first
        };

        // Upsert implementation
        use sea_orm::sea_query::OnConflict;

        let res = platform_session::Entity::insert(session)
            .on_conflict(
                OnConflict::column(platform_session::Column::PlatformId)
                    .update_columns([
                        platform_session::Column::Status,
                        platform_session::Column::Username,
                        platform_session::Column::AvatarUrl,
                        platform_session::Column::EncryptedCookies,
                        platform_session::Column::CookieMethod,
                        platform_session::Column::ExpiresAt,
                        platform_session::Column::LastVerified,
                        platform_session::Column::ErrorMessage,
                        platform_session::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(self.db.as_ref())
            .await;

        match res {
            Ok(_) => {
                tracing::info!("Successfully saved session for {}", platform_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to save session for {}: {}", platform_id, e);
                Err(e.into())
            }
        }
    }

    pub async fn get_session(
        &self,
        platform_id: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let session = platform_session::Entity::find_by_id(platform_id)
            .one(self.db.as_ref())
            .await?;

        if let Some(session) = session {
            tracing::info!("Found session for platform: {}", platform_id);
            if let Some(encrypted) = session.encrypted_cookies {
                let decrypted = decrypt_string(&encrypted)?;
                return Ok(Some(decrypted));
            } else {
                tracing::warn!(
                    "Session found for {} but encrypted_cookies is None",
                    platform_id
                );
            }
        } else {
            tracing::warn!("No session found in DB for platform: {}", platform_id);
        }
        Ok(None)
    }

    pub async fn get_netscape_cookies(
        &self,
        platform_id: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let cookies_str = self.get_session(platform_id).await?;

        if let Some(raw_cookies) = cookies_str {
            // Basic validation: check if it looks like netscape
            if raw_cookies.contains("\t") && !raw_cookies.trim().starts_with("{") {
                return Ok(Some(raw_cookies));
            }

            // If it's JSON or other format, we might want to log it
            tracing::debug!(
                "Cookies for {} do not look like Netscape format (starts with '{{'?)",
                platform_id
            );

            // TODO: improved parsing if we support JSON cookies
            return Ok(Some(raw_cookies));
        }

        Ok(None)
    }

    pub async fn create_temp_cookie_file(
        &self,
        platform_id: &str,
    ) -> Result<Option<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Attempting to create temp cookie file for {}", platform_id);
        if let Some(content) = self.get_netscape_cookies(platform_id).await? {
            // Ensure temp directory exists
            if let Err(e) = fs::create_dir_all(&self.temp_dir).await {
                tracing::error!("Failed to create temp directory: {}", e);
                return Err(e.into());
            }

            let filename = format!("{}_{}.txt", platform_id, Uuid::new_v4());
            let file_path = self.temp_dir.join(filename);

            let mut file = fs::File::create(&file_path).await?;
            file.write_all(content.as_bytes()).await?;

            tracing::info!("Created temp cookie file at {:?}", file_path);
            return Ok(Some(file_path));
        } else {
            tracing::warn!("Could not get netscape cookies for {}", platform_id);
        }
        Ok(None)
    }

    pub async fn cleanup_temp_file(
        &self,
        path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if path.exists() {
            fs::remove_file(path).await?;
        }
        Ok(())
    }

    pub async fn delete_session(
        &self,
        platform_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        platform_session::Entity::delete_by_id(platform_id.to_string())
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn validate_session_cookies(
        &self,
        platform_id: &str,
        cookies_str: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Parse the Netscape format to find cookie names
        // Format: domain \t flag \t path \t secure \t expiration \t name \t value
        let cookie_names: Vec<&str> = cookies_str
            .lines()
            .filter(|l| {
                let trimmed = l.trim();
                !trimmed.is_empty()
                    && (!trimmed.starts_with('#') || trimmed.starts_with("#HttpOnly_"))
            })
            .filter_map(|line| {
                // For #HttpOnly_ lines, the domain is embedded after the prefix,
                // but the tab-separated fields remain the same (the #HttpOnly_ is
                // part of the domain field), so index 5 is still the cookie name.
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 6 {
                    Some(parts[5]) // Name is at index 5
                } else {
                    None
                }
            })
            .collect();

        let required_cookie = match platform_id {
            "youtube" => vec!["__Secure-3PSID", "SID"], // Either is usually sufficient, prefer 3PSID
            "tiktok" => vec!["sessionid_ss", "sessionid"], // sessionid usually
            "instagram" => vec!["sessionid"],
            "x" => vec!["auth_token"],
            _ => vec![], // No validation for unknown platforms yet
        };

        if required_cookie.is_empty() {
            return Ok(());
        }

        let has_required = required_cookie
            .iter()
            .any(|&req| cookie_names.contains(&req));

        if !has_required {
            tracing::warn!(
                "Session validation failed for {}. Missing required cookie from: {:?}",
                platform_id,
                required_cookie
            );
            return Err(format!(
                "Session validation failed. Missing required authentication cookie ({:?}). Please ensure you are logged in.",
                required_cookie
            )
            .into());
        }

        Ok(())
    }

    /// Helper method to convert JSON cookies formatted text into Netscape cookie jar format
    fn convert_json_to_netscape(json_str: &str) -> Result<String, String> {
        fn default_path() -> String {
            "/".to_string()
        }

        #[derive(Debug, serde::Deserialize)]
        struct CookieJson {
            domain: String,
            #[serde(default = "default_path")]
            path: String,
            #[serde(default)]
            secure: bool,
            #[serde(rename = "expirationDate", alias = "expires")]
            expiration_date: Option<f64>,
            name: String,
            value: String,
        }

        let json_value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse JSON string: {}", e))?;

        // Determine the list of cookies based on structure
        let cookies_list: Result<Vec<CookieJson>, String> = if json_value.is_array() {
            serde_json::from_value(json_value).map_err(|e| format!("Invalid cookie list: {}", e))
        } else if let Some(cookies_array) = json_value.get("cookies").and_then(|c| c.as_array()) {
            tracing::info!("Detected wrapper object with 'cookies' field");
            serde_json::from_value(serde_json::Value::Array(cookies_array.clone()))
                .map_err(|e| format!("Invalid wrapper content: {}", e))
        } else {
            // Try single object last
            match serde_json::from_value::<CookieJson>(json_value) {
                Ok(cookie) => {
                    tracing::info!("Detected single JSON cookie object");
                    Ok(vec![cookie])
                }
                Err(e) => Err(format!(
                    "Invalid JSON structure. Expected a cookie list, a wrapper object with 'cookies' list, or a valid single object. Error: {}",
                    e
                )),
            }
        };

        let json_cookies = cookies_list?;
        let mut netscape_lines = String::new();
        netscape_lines.push_str("# Netscape HTTP Cookie File\n");
        netscape_lines.push_str("# This file is generated by VideoDownloaderPro\n\n");

        for cookie in json_cookies {
            let domain = cookie.domain;
            let flag = if domain.starts_with('.') {
                "TRUE"
            } else {
                "FALSE"
            };
            let path = cookie.path;
            let secure = if cookie.secure { "TRUE" } else { "FALSE" };
            let expiration = cookie.expiration_date.unwrap_or(0.0) as i64;
            let name = cookie.name;
            let value = cookie.value;

            netscape_lines.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                domain, flag, path, secure, expiration, name, value
            ));
        }

        Ok(netscape_lines)
    }
}
