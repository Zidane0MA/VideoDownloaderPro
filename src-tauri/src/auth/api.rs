use reqwest::header::{HeaderMap, HeaderValue, COOKIE, USER_AGENT};
use reqwest::Client;
use serde_json::Value;

pub struct UsernameFetcher;

impl UsernameFetcher {
    /// Attempts to fetch the TikTok username (unique_id) using the session cookies and user ID.
    pub async fn fetch_tiktok_username(cookies: &str, _uid: &str) -> Option<String> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        // This API endpoint often returns the logged-in user's info
        let url = "https://www.tiktok.com/passport/web/account/info/";

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));

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

        let res = client.get(url).headers(headers).send().await.ok()?;

        if res.status().is_success() {
            let text = res.text().await.ok()?;
            let json: Value = serde_json::from_str(&text).ok()?;

            if let Some(data) = json.get("data") {
                if let Some(handle) = data.get("username").and_then(|v| v.as_str()) {
                    if !handle.is_empty() {
                        return Some(handle.to_string());
                    }
                }
                // Try screen_name as fallback?
                if let Some(screen_name) = data.get("screen_name").and_then(|v| v.as_str()) {
                    if !screen_name.is_empty() {
                        return Some(screen_name.to_string());
                    }
                }
            }
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
                    {
                        return Some(handle.to_string());
                    }
                }
            }
        }

        None
    }
}
