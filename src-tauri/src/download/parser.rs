use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressUpdate {
    pub progress: f64, // 0.0 to 100.0
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub speed: Option<String>,
    pub eta: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum ParseResult {
    Progress(ProgressUpdate),
    Ignore,
}

pub struct Parser {
    progress_regex: Regex,
    completion_regex: Regex,
}

impl Parser {
    pub fn new() -> Self {
        // [download]  45.0% of 10.00MiB at  2.00MiB/s ETA 00:05
        let re = Regex::new(
            r"\[download\]\s+(\d+(?:\.\d+)?)%\s+of\s+(?:~)?(\S+)\s+at\s+(\S+)\s+ETA\s+(\S+)",
        )
        .unwrap();

        // [download] 100% of 10.00MiB in 00:03
        let completion_re =
            Regex::new(r"\[download\]\s+100(?:\.0)?%\s+of\s+(?:~)?(\S+)\s+in\s+(\S+)").unwrap();

        Self {
            progress_regex: re,
            completion_regex: completion_re,
        }
    }

    pub fn parse_line(&self, line: &str) -> ParseResult {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return ParseResult::Ignore;
        }

        // Try the normal progress line first
        if let Some(caps) = self.progress_regex.captures(trimmed) {
            let progress_str = caps.get(1).map(|m| m.as_str()).unwrap_or("0");
            let progress = progress_str.parse::<f64>().unwrap_or(0.0);

            let total_size_str = caps.get(2).map(|m| m.as_str().to_string());
            let speed = caps.get(3).map(|m| m.as_str().to_string());
            let eta = caps.get(4).map(|m| m.as_str().to_string());

            let total_bytes = total_size_str.as_ref().and_then(|s| parse_size(s));
            let downloaded_bytes = if let Some(total) = total_bytes {
                Some((total as f64 * (progress / 100.0)) as u64)
            } else {
                None
            };

            ParseResult::Progress(ProgressUpdate {
                progress,
                downloaded_bytes,
                total_bytes,
                speed,
                eta,
            })
        } else if let Some(caps) = self.completion_regex.captures(trimmed) {
            // Completion line: [download] 100% of 10.00MiB in 00:03
            let total_size_str = caps.get(1).map(|m| m.as_str().to_string());
            let total_bytes = total_size_str.as_ref().and_then(|s| parse_size(s));

            ParseResult::Progress(ProgressUpdate {
                progress: 100.0,
                downloaded_bytes: total_bytes, // 100% means all bytes downloaded
                total_bytes,
                speed: None,
                eta: None,
            })
        } else {
            ParseResult::Ignore
        }
    }
}

/// Lazily compiled regex for parsing human-readable size strings.
static SIZE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+(?:\.\d+)?)([KMGT]i?B)").unwrap());

fn parse_size(size_str: &str) -> Option<u64> {
    let caps = SIZE_REGEX.captures(size_str)?;
    let value = caps.get(1)?.as_str().parse::<f64>().ok()?;
    let unit = caps.get(2)?.as_str();

    let multiplier = match unit {
        "KiB" | "KB" => 1024.0,
        "MiB" | "MB" => 1024.0 * 1024.0,
        "GiB" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "TiB" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    Some((value * multiplier) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_progress() {
        let parser = Parser::new();
        let line = "[download]  45.0% of 10.00MiB at  2.00MiB/s ETA 00:05";
        let result = parser.parse_line(line);

        if let ParseResult::Progress(update) = result {
            assert_eq!(update.progress, 45.0);
            assert_eq!(update.speed, Some("2.00MiB/s".to_string()));
            assert_eq!(update.eta, Some("00:05".to_string()));

            // 10 MiB = 10 * 1024 * 1024 = 10485760 bytes
            // 45% of 10 MiB = 4,718,592
            assert_eq!(update.total_bytes, Some(10485760));
            assert_eq!(update.downloaded_bytes, Some(4718592));
        } else {
            panic!("Expected Progress, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_progress_with_tilde() {
        let parser = Parser::new();
        let line = "[download]  23.5% of ~1.23GiB at  5.67MiB/s ETA 03:45";
        let result = parser.parse_line(line);

        if let ParseResult::Progress(update) = result {
            assert_eq!(update.progress, 23.5);
            assert_eq!(update.speed, Some("5.67MiB/s".to_string()));
            assert_eq!(update.eta, Some("03:45".to_string()));
        } else {
            panic!("Expected Progress, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_non_matching_line() {
        let parser = Parser::new();
        assert_eq!(
            parser.parse_line("[info] Extracting URL"),
            ParseResult::Ignore
        );
        assert_eq!(parser.parse_line(""), ParseResult::Ignore);
    }

    #[test]
    fn test_tagged_lines_are_ignored() {
        let parser = Parser::new();
        assert_eq!(
            parser.parse_line("[info] Extracting URL"),
            ParseResult::Ignore
        );
        assert_eq!(parser.parse_line(""), ParseResult::Ignore);

        // Filename/merger lines are now intentionally ignored (fs-scan is canonical)
        assert_eq!(
            parser.parse_line("[download] Destination: video.mp4"),
            ParseResult::Ignore
        );
        assert_eq!(
            parser.parse_line("[Merger] Merging formats into \"video.mkv\""),
            ParseResult::Ignore
        );
    }
}
