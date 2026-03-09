//! Utility functions for TikTok URL parsing and cookie format conversion.

// ── Section detection ──────────────────────────────────────────────────

/// Supported TikTok profile sections that use the internal API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TikTokSection {
    Liked,
    Saved,
}

/// Detects the TikTok section type from a URL.
///
/// - `/@user/liked` → `Some(Liked)`
/// - `/@user/saved` → `Some(Saved)`
/// - `/@user`, `/@user/video/123`, etc. → `None`
pub fn detect_tiktok_section(url: &str) -> Option<TikTokSection> {
    let url_lower = url.to_lowercase();
    if !url_lower.contains("tiktok.com") {
        return None;
    }

    // Find the path segment after the username
    let at_pos = url_lower.find('@')?;
    let after_at = &url_lower[at_pos + 1..];
    // Skip the username itself
    let slash_pos = after_at.find('/')?;
    let section_path = &after_at[slash_pos + 1..];
    // Trim trailing slashes or query params
    let section = section_path
        .split(&['/', '?', '#'][..])
        .next()
        .unwrap_or("");

    match section {
        "liked" => Some(TikTokSection::Liked),
        "saved" => Some(TikTokSection::Saved),
        _ => None,
    }
}

// ── Cookie conversion ──────────────────────────────────────────────────

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

// ── Username extraction ────────────────────────────────────────────────

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

    // ── Section detection tests ────────────────────────────────────────

    #[test]
    fn test_detect_section_liked() {
        let url = "https://www.tiktok.com/@user/liked";
        assert_eq!(detect_tiktok_section(url), Some(TikTokSection::Liked));
    }

    #[test]
    fn test_detect_section_saved() {
        let url = "https://www.tiktok.com/@user/saved";
        assert_eq!(detect_tiktok_section(url), Some(TikTokSection::Saved));
    }

    #[test]
    fn test_detect_section_profile_only() {
        let url = "https://www.tiktok.com/@user";
        assert_eq!(detect_tiktok_section(url), None);
    }

    #[test]
    fn test_detect_section_video_url() {
        let url = "https://www.tiktok.com/@user/video/7300000000";
        assert_eq!(detect_tiktok_section(url), None);
    }

    #[test]
    fn test_detect_section_non_tiktok() {
        let url = "https://www.youtube.com/@user/liked";
        assert_eq!(detect_tiktok_section(url), None);
    }

    #[test]
    fn test_detect_section_case_insensitive() {
        let url = "https://www.tiktok.com/@User/Liked";
        assert_eq!(detect_tiktok_section(url), Some(TikTokSection::Liked));
    }

    #[test]
    fn test_detect_section_with_query_params() {
        let url = "https://www.tiktok.com/@user/liked?lang=en";
        assert_eq!(detect_tiktok_section(url), Some(TikTokSection::Liked));
    }

    // ── Cookie conversion tests ────────────────────────────────────────

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

    // ── Username extraction tests ──────────────────────────────────────

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
            Some("anotheruser".to_string()),
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
