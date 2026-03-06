//! Utility functions for TikTok URL parsing and cookie format conversion.

/// Converts Netscape cookie jar text (tab-separated) to a `Cookie` header string.
///
/// Input format: domain \t flag \t path \t secure \t expiry \t name \t value
/// Output format: name=value; name2=value2
pub fn netscape_to_header(netscape: &str) -> String {
    netscape
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && (!trimmed.starts_with('#') || trimmed.starts_with("#HttpOnly_"))
        })
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 7 {
                Some(format!("{}={}", parts[5], parts[6]))
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join("; ")
}

/// Extracts the username from a TikTok URL.
///
/// Handles: `https://www.tiktok.com/@username/liked`, `https://tiktok.com/@user`
pub fn extract_tiktok_username(url: &str) -> Option<String> {
    // Find "@" in the URL path and extract username until next "/" or end
    let at_pos = url.find('@')?;
    let rest = &url[at_pos + 1..];
    let end = rest.find('/').unwrap_or(rest.len());
    let username = &rest[..end];

    if username.is_empty() {
        None
    } else {
        Some(username.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netscape_to_header() {
        let netscape = "\
# Netscape HTTP Cookie File
.tiktok.com\tTRUE\t/\tTRUE\t0\tsessionid\tabc123
#HttpOnly_.tiktok.com\tTRUE\t/\tTRUE\t0\tsessionid_ss\txyz789
.tiktok.com\tTRUE\t/\tTRUE\t0\ttt_webid_v2\tqqq000
";
        let header = netscape_to_header(netscape);
        assert!(header.contains("sessionid=abc123"));
        assert!(header.contains("sessionid_ss=xyz789"));
        assert!(header.contains("tt_webid_v2=qqq000"));
        // Should be semicolon-separated
        assert_eq!(header.matches(';').count(), 2);
    }

    #[test]
    fn test_netscape_to_header_skips_comments() {
        let netscape = "# Just a comment\n# Another comment\n";
        let header = netscape_to_header(netscape);
        assert!(header.is_empty());
    }

    #[test]
    fn test_extract_username_liked_url() {
        let url = "https://www.tiktok.com/@someuser/liked";
        assert_eq!(extract_tiktok_username(url), Some("someuser".to_string()));
    }

    #[test]
    fn test_extract_username_profile_url() {
        let url = "https://tiktok.com/@anotheruser";
        assert_eq!(
            extract_tiktok_username(url),
            Some("anotheruser".to_string())
        );
    }

    #[test]
    fn test_extract_username_no_at() {
        let url = "https://tiktok.com/foryou";
        assert_eq!(extract_tiktok_username(url), None);
    }

    #[test]
    fn test_extract_username_video_url() {
        let url = "https://www.tiktok.com/@user123/video/7300000000";
        assert_eq!(extract_tiktok_username(url), Some("user123".to_string()));
    }
}
