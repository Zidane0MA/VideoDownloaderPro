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

pub struct Parser {
    progress_regex: Regex,
}

impl Parser {
    pub fn new() -> Self {
        // [download]  45.0% of 10.00MiB at  2.00MiB/s ETA 00:05
        // Needs to be robust.
        // Capture groups:
        // 1. Percentage (f64)
        // 2. Total Size (str) - optional
        // 3. Speed (str) - optional
        // 4. ETA (str) - optional

        // Example: [download]  23.5% of ~1.23GiB at  5.67MiB/s ETA 03:45
        let re = Regex::new(
            r"\[download\]\s+(\d+(?:\.\d+)?)%\s+of\s+(?:~)?(\S+)\s+at\s+(\S+)\s+ETA\s+(\S+)",
        )
        .unwrap();

        Self { progress_regex: re }
    }

    pub fn parse_line(&self, line: &str) -> Option<ProgressUpdate> {
        if let Some(caps) = self.progress_regex.captures(line) {
            let progress_str = caps.get(1)?.as_str();
            let progress = progress_str.parse::<f64>().ok()?;

            let total_size_str = caps.get(2).map(|m| m.as_str().to_string());
            let speed = caps.get(3).map(|m| m.as_str().to_string());
            let eta = caps.get(4).map(|m| m.as_str().to_string());

            // Helper to parse human readable size to bytes could be added here
            // For now we just return the strings or None for bytes to be filled later/computed
            // Actually, the contract asks for bytes. We should probably parse the size string.
            // But let's start with parsing the structure and maybe adding a size parser helper.

            let total_bytes = total_size_str.as_ref().and_then(|s| parse_size(s));
            let downloaded_bytes = if let Some(total) = total_bytes {
                Some((total as f64 * (progress / 100.0)) as u64)
            } else {
                None
            };

            Some(ProgressUpdate {
                progress,
                downloaded_bytes,
                total_bytes,
                speed,
                eta,
            })
        } else {
            None
        }
    }
}

fn parse_size(size_str: &str) -> Option<u64> {
    // 10.00MiB
    let re = Regex::new(r"(\d+(?:\.\d+)?)([KMGT]i?B)").unwrap();
    if let Some(caps) = re.captures(size_str) {
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
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_progress() {
        let parser = Parser::new();
        let line = "[download]  45.0% of 10.00MiB at  2.00MiB/s ETA 00:05";
        let update = parser.parse_line(line).unwrap();

        assert_eq!(update.progress, 45.0);
        assert_eq!(update.speed, Some("2.00MiB/s".to_string()));
        assert_eq!(update.eta, Some("00:05".to_string()));

        // 10 MiB = 10 * 1024 * 1024 = 10485760 bytes
        // 45% of 10 MiB = 4,718,592
        assert_eq!(update.total_bytes, Some(10485760));
        assert_eq!(update.downloaded_bytes, Some(4718592));
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1.00KiB"), Some(1024));
        assert_eq!(parse_size("1.5MiB"), Some(1572864));
    }
}
