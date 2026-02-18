use serde_json::Value;

pub struct UsernameExtractor;

impl UsernameExtractor {
    pub fn extract_from_netscape(cookies: &str, platform: &str) -> Option<String> {
        // Helper to find cookie value in Netscape format
        // Format: domain \t flag \t path \t secure \t expiration \t name \t value
        let find_cookie = |name: &str| -> Option<String> {
            cookies.lines().find_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 7 && parts[5] == name {
                    Some(parts[6].to_string())
                } else {
                    None
                }
            })
        };

        match platform {
            "instagram" => {
                // 'ds_user' is sometimes present as username
                // 'ds_user_id' is user ID
                find_cookie("ds_user").or_else(|| find_cookie("ds_user_id"))
            }
            "tiktok" => {
                // 'unique_id' often holds the handle in some contexts
                // 'user_id' is numeric ID
                // 'uid_tt' seems to be the user ID in newer sessions
                find_cookie("unique_id")
                    .or_else(|| find_cookie("user_id"))
                    .or_else(|| find_cookie("uid_tt"))
            }
            "x" | "twitter" => {
                // X header 'twid' contains 'u=123456' (often URL encoded as u%3D123456)
                if let Some(twid) = find_cookie("twid") {
                    // Simple decode if needed
                    let decoded = twid.replace("%3D", "=");
                    if let Some(id_part) = decoded.strip_prefix("u=") {
                        Some(id_part.to_string())
                    } else {
                        Some(decoded)
                    }
                } else if find_cookie("auth_token").is_some() {
                    // We don't have the handle, but we can verify it's connected
                    None
                } else {
                    None
                }
            }
            "youtube" => {
                // Google doesn't put email in cookies easily verifyable.
                None
            }
            _ => None,
        }
    }

    pub fn extract_from_json(json_str: &str, platform: &str) -> Option<String> {
        let json_value: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let cookies = if json_value.is_array() {
            json_value.as_array()?
        } else if let Some(wrapper) = json_value.as_object() {
            if let Some(list) = wrapper.get("cookies").and_then(|c| c.as_array()) {
                list
            } else {
                return None;
            }
        } else {
            return None;
        };

        let find_cookie_json = |name: &str| -> Option<String> {
            cookies.iter().find_map(|c| {
                let c_name = c.get("name")?.as_str()?;
                if c_name == name {
                    c.get("value")?.as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        };

        match platform {
            "instagram" => find_cookie_json("ds_user").or_else(|| find_cookie_json("ds_user_id")),
            "tiktok" => find_cookie_json("unique_id")
                .or_else(|| find_cookie_json("user_id"))
                .or_else(|| find_cookie_json("uid_tt")),
            "x" | "twitter" => {
                if let Some(twid) = find_cookie_json("twid") {
                    let decoded = twid.replace("%3D", "=");
                    if let Some(id_part) = decoded.strip_prefix("u=") {
                        Some(id_part.to_string())
                    } else {
                        Some(decoded)
                    }
                } else if find_cookie_json("auth_token").is_some() {
                    None
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
