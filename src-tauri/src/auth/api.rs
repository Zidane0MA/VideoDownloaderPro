use reqwest::header::{HeaderMap, HeaderValue, COOKIE, USER_AGENT};
use reqwest::Client;
use serde_json::Value;

pub struct UsernameFetcher;

impl UsernameFetcher {
    /// Attempts to fetch the TikTok username (unique_id) using the session cookies and Passport API.
    pub async fn fetch_tiktok_username(cookies: &str) -> Option<String> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        // This API endpoint uses cookies to return logged-in user's info
        let url = "https://www.tiktok.com/passport/web/account/info/";

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
        headers.insert(
            "Referer",
            HeaderValue::from_static("https://www.tiktok.com/"),
        );

        // Simple Netscape to Cookie header converter
        let cookie_header_value = cookies
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 7 {
                    Some(format!("{}={}", parts[5], parts[6]))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join("; ");

        if let Ok(val) = HeaderValue::from_str(&cookie_header_value) {
            headers.insert(COOKIE, val);
        }

        println!("  [DEBUG] TikTok API: Sending request to {}", url);
        let res = client.get(url).headers(headers).send().await.ok()?;

        println!("  [DEBUG] TikTok API Response Status: {}", res.status());

        if res.status().is_success() {
            let text = res.text().await.ok()?;
            println!("  [DEBUG] TikTok API Response Context:");
            println!("  [DEBUG] {}", &text[0..std::cmp::min(1000, text.len())]);

            let json: Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(e) => {
                    println!("  [DEBUG] TikTok API: JSON Parse Error: {}", e);
                    return None;
                }
            };

            if let Some(data) = json.get("data") {
                if let Some(username) = data.get("username").and_then(|v| v.as_str()) {
                    if !username.is_empty() {
                        return Some(username.to_string());
                    }
                }

                if let Some(screen_name) = data.get("screen_name").and_then(|v| v.as_str()) {
                    if !screen_name.is_empty() {
                        return Some(screen_name.to_string());
                    }
                }
            } else {
                println!("  [DEBUG] TikTok API: No 'data' object found in response");
            }
        } else {
            println!("  [DEBUG] TikTok API: Failed with status {}", res.status());
        }

        None
    }

    /// Attempts to fetch the X (Twitter) username (screen_name) using cookies.
    pub async fn fetch_x_username(cookies: &str, user_id: &str) -> Option<String> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .ok()?;

        // Strategy: Fetch home page and scrape screen_name
        let url = "https://x.com/home";

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));

        // Netscape to Cookie header
        let cookie_header_value = cookies
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 7 {
                    Some(format!("{}={}", parts[5], parts[6]))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join("; ");

        if let Ok(val) = HeaderValue::from_str(&cookie_header_value) {
            headers.insert(COOKIE, val);
        }

        let res = client.get(url).headers(headers).send().await.ok()?;

        if res.status().is_success() {
            let text = res.text().await.unwrap_or_default();

            // Generic search for "screen_name":"..."
            // We skip the first part because split gives us the part *before* the first delimiter
            let broken_str: Vec<&str> = text.split(r#""screen_name":""#).collect();
            for part in broken_str.iter().skip(1) {
                if let Some(end) = part.find('"') {
                    let handle = &part[0..end];
                    if !handle.is_empty()
                        && handle != "home"
                        && handle != "login"
                        && handle != "user"
                        && handle != user_id
                    {
                        return Some(handle.to_string());
                    }
                }
            }
        }

        None
    }

    /// Attempts to fetch the YouTube username/channel name using cookies and the InnerTube API.
    pub async fn fetch_youtube_username(cookies: &str) -> Option<String> {
        use sha1::{Digest, Sha1};
        use std::time::{SystemTime, UNIX_EPOCH};

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        // 1. Extract SAPISID or __Secure-3PAPISID
        let mut sapisid = None;
        for line in cookies.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 7 {
                let name = parts[5];
                let value = parts[6];
                if name == "SAPISID" || name == "__Secure-3PAPISID" {
                    sapisid = Some(value.to_string());
                    // Prefer SAPISID, but if we found __Secure-3PAPISID first, keep looking just in case
                    if name == "SAPISID" {
                        break;
                    }
                }
            }
        }

        let sapisid = sapisid?;
        println!("  [DEBUG] YouTube: Found SAPISID cookie");

        // 2. Generate Auth Header (SAPISIDHASH)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let origin = "https://www.youtube.com";
        let input_str = format!("{} {} {}", timestamp, sapisid, origin);

        let mut hasher = Sha1::new();
        hasher.update(input_str.as_bytes());
        let sha1_hash = format!("{:x}", hasher.finalize());
        let auth_header = format!("SAPISIDHASH {}_{}", timestamp, sha1_hash);

        println!("  [DEBUG] YouTube: Generated SAPISIDHASH auth header");

        // 3. Call InnerTube API
        let url = "https://www.youtube.com/youtubei/v1/account/account_menu?prettyPrint=false";

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));

        if let Ok(auth_val) = HeaderValue::from_str(&auth_header) {
            headers.insert("Authorization", auth_val);
        }
        headers.insert(
            "X-Origin",
            HeaderValue::from_static("https://www.youtube.com"),
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        // Netscape to Cookie header
        let cookie_header_value = cookies
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 7 {
                    Some(format!("{}={}", parts[5], parts[6]))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join("; ");

        if let Ok(val) = HeaderValue::from_str(&cookie_header_value) {
            headers.insert(COOKIE, val);
        }

        let payload = serde_json::json!({
            "context": {
                "client": {
                    "hl": "es",
                    "gl": "ES",
                    "clientName": "WEB",
                    "clientVersion": "2.20240118.01.00"
                }
            }
        });

        println!("  [DEBUG] YouTube: Sending POST request to InnerTube API...");
        let res = client
            .post(url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .ok()?;

        println!("  [DEBUG] YouTube Response Status: {}", res.status());

        if res.status().is_success() {
            let json_body: Value = res.json().await.ok()?;

            // 4. Recursive search for accountName -> simpleText
            fn find_account_name(val: &Value) -> Option<String> {
                match val {
                    Value::Object(map) => {
                        // Check if this object is an "accountName" holding a "simpleText" or runs
                        if let Some(account_name) = map.get("accountName") {
                            if let Some(simple_text) =
                                account_name.get("simpleText").and_then(|s| s.as_str())
                            {
                                return Some(simple_text.to_string());
                            }
                        }

                        // Or if we are already inside an object that might have simpleText, though we prefer being explicit
                        // about the parent key if possible to avoid false positives. Look for account fallback.
                        if map.contains_key("accountName") {
                            // already checked above
                        }

                        // Also sometimes the channel name is under channelName -> simpleText
                        if let Some(channel_name) = map.get("channelName") {
                            if let Some(simple_text) =
                                channel_name.get("simpleText").and_then(|s| s.as_str())
                            {
                                return Some(simple_text.to_string());
                            }
                        }

                        // Recurse into all object values
                        for (_, v) in map {
                            if let Some(found) = find_account_name(v) {
                                return Some(found);
                            }
                        }
                        None
                    }
                    Value::Array(arr) => {
                        for item in arr {
                            if let Some(found) = find_account_name(item) {
                                return Some(found);
                            }
                        }
                        None
                    }
                    _ => None,
                }
            }

            if let Some(name) = find_account_name(&json_body) {
                println!(
                    "  [DEBUG] YouTube: Successfully extracted account name: {}",
                    name
                );
                return Some(name);
            } else {
                println!(
                    "  [DEBUG] YouTube: Request successful, but no account name found in response."
                );
            }
        }

        None
    }

    /// Verifies if a session is still valid by making a lightweight request.
    pub async fn check_session_validity(platform_id: &str, cookies: &str) -> Result<bool, String> {
        let is_valid = match platform_id {
            "tiktok" => Self::fetch_tiktok_username(cookies).await.is_some(),
            "youtube" => Self::fetch_youtube_username(cookies).await.is_some(),
            "x" | "twitter" => Self::fetch_x_username(cookies, "").await.is_some(),
            "instagram" => {
                let client = Client::builder()
                    .timeout(std::time::Duration::from_secs(10))
                    .build()
                    .map_err(|e| e.to_string())?;

                let mut headers = HeaderMap::new();
                headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
                let cookie_header_value = cookies
                    .lines()
                    .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.len() >= 7 {
                            Some(format!("{}={}", parts[5], parts[6]))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("; ");

                if let Ok(val) = HeaderValue::from_str(&cookie_header_value) {
                    headers.insert(COOKIE, val);
                }

                if let Ok(res) = client
                    .get("https://www.instagram.com/")
                    .headers(headers)
                    .send()
                    .await
                {
                    res.status().is_success() && !res.url().path().contains("login")
                } else {
                    false
                }
            }
            _ => true, // Fallback
        };
        Ok(is_valid)
    }
}
