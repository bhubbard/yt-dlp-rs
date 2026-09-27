use regex::Regex;

/// Formats byte count into human-readable representation.
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2}GiB", b / GB)
    } else if b >= MB {
        format!("{:.2}MiB", b / MB)
    } else if b >= KB {
        format!("{:.2}KiB", b / KB)
    } else {
        format!("{}B", bytes)
    }
}

/// Formats duration in seconds into standard HH:MM:SS or MM:SS format.
pub fn format_duration(seconds: f64) -> String {
    let total_secs = seconds.max(0.0) as u64;
    let hrs = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hrs > 0 {
        format!("{:02}:{:02}:{:02}", hrs, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

/// Replaces invalid filesystem characters with underscores.
pub fn sanitize_filename(name: &str) -> String {
    let invalid = ['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>', '\r', '\n'];
    let res: String = name
        .chars()
        .map(|c| if invalid.contains(&c) { '_' } else { c })
        .collect();
    let trimmed = res.trim();
    if trimmed.is_empty() {
        "video".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Matches the first capture group of a regex pattern.
pub fn match_group(text: &str, pattern: &str) -> Option<String> {
    Regex::new(pattern).ok().and_then(|re| {
        re.captures(text)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00KiB");
        assert_eq!(format_bytes(1048576 * 10), "10.00MiB");
        assert_eq!(format_bytes(500), "500B");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(65.0), "01:05");
        assert_eq!(format_duration(3665.0), "01:01:05");
        assert_eq!(format_duration(15.0), "00:15");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Rick: Astley / Never?"), "Rick_ Astley _ Never_");
        assert_eq!(sanitize_filename("  "), "video");
    }
}
