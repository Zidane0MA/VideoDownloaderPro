/// Shared platform detection from URL.
///
/// Returns the platform identifier (`"youtube"`, `"tiktok"`, `"instagram"`, `"x"`)
/// or `None` if the URL doesn't match any known platform.
pub fn detect_platform(url: &str) -> Option<&'static str> {
    if url.contains("youtube.com") || url.contains("youtu.be") {
        Some("youtube")
    } else if url.contains("tiktok.com") {
        Some("tiktok")
    } else if url.contains("instagram.com") {
        Some("instagram")
    } else if url.contains("x.com") || url.contains("twitter.com") {
        Some("x")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_youtube() {
        assert_eq!(
            detect_platform("https://www.youtube.com/watch?v=abc"),
            Some("youtube")
        );
        assert_eq!(detect_platform("https://youtu.be/abc"), Some("youtube"));
    }

    #[test]
    fn test_detect_tiktok() {
        assert_eq!(
            detect_platform("https://www.tiktok.com/@user/video/123"),
            Some("tiktok")
        );
    }

    #[test]
    fn test_detect_instagram() {
        assert_eq!(
            detect_platform("https://www.instagram.com/reel/abc"),
            Some("instagram")
        );
    }

    #[test]
    fn test_detect_x() {
        assert_eq!(detect_platform("https://x.com/user/status/123"), Some("x"));
        assert_eq!(
            detect_platform("https://twitter.com/user/status/123"),
            Some("x")
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_platform("https://example.com/video"), None);
    }
}
