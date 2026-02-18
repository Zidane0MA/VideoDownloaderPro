use super::extractor::UsernameExtractor;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_extract_from_local_db() -> Result<(), Box<dyn std::error::Error>> {
        use crate::auth::{encryption, extractor::UsernameExtractor};
        use crate::entity::platform_session;
        use sea_orm::{Database, EntityTrait};
        use std::path::PathBuf;

        println!("Starting local DB test...");

        // Start with %APPDATA% (Roaming)
        let app_data = std::env::var("APPDATA").map_err(|_| "Could not find APPDATA env var")?;

        // Correct path construction for Tauri app
        let db_path = PathBuf::from(app_data)
            .join("com.videodownloaderpro.app")
            .join("videodownloaderpro.db");

        println!("Looking for DB at: {:?}", db_path);

        if !db_path.exists() {
            println!("Skipping test: DB not found at {:?}", db_path);
            return Ok(());
        }

        let db_url = format!("sqlite://{}?mode=ro", db_path.to_string_lossy());
        println!("Connecting to DB URL: {}", db_url);

        let db = Database::connect(db_url).await?;

        let sessions = platform_session::Entity::find().all(&db).await?;
        println!("Found {} sessions in DB.", sessions.len());

        for session in sessions {
            println!("Processing session for platform: {}", session.platform_id);

            if let Some(encrypted) = session.encrypted_cookies {
                match encryption::decrypt_string(&encrypted) {
                    Ok(cookies) => {
                        println!(
                            "  -> Decrypted cookies successfully. Length: {}",
                            cookies.len()
                        );

                        // Verbose Debugging: Print all cookies
                        println!("  -> Cookie Dump for {}:", session.platform_id);
                        for line in cookies.lines() {
                            if line.starts_with('#') || line.trim().is_empty() {
                                continue;
                            }
                            let parts: Vec<&str> = line.split('\t').collect();
                            if parts.len() >= 7 {
                                let domain = parts[0];
                                let name = parts[5];
                                let value = parts[6];
                                let masked_value = if value.len() > 5 {
                                    format!("{}...", &value[0..5])
                                } else {
                                    "***".to_string()
                                };
                                println!("     - {} | {} = {}", domain, name, masked_value);
                            }
                        }

                        let username = UsernameExtractor::extract_from_netscape(
                            &cookies,
                            &session.platform_id,
                        );
                        println!("  -> Extracted Username: {:?}", username);

                        if let Some(db_user) = &session.username {
                            println!("  -> DB Stored Username: {}", db_user);
                        }
                    }
                    Err(e) => println!("  -> Failed to decrypt cookies: {}", e),
                }
            } else {
                println!("  -> No encrypted cookies found for this session.");
            }
        }

        Ok(())
    }
}
