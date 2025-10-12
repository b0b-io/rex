use super::*;

#[test]
fn test_format_size_binary() {
    let size = 1024 * 5; // 5 KiB
    let formatted = format_size(size as u64);
    assert_eq!(formatted, "5 KiB");
}

#[test]
fn test_format_size_decimal() {
    let size = 1000 * 5; // 5 kB
    let formatted = format_size_decimal(size as u64);
    assert_eq!(formatted, "5 kB");
}

#[test]
fn test_format_size_megabytes() {
    let size = 1024 * 1024 * 2; // 2 MiB
    let formatted = format_size(size as u64);
    assert_eq!(formatted, "2 MiB");
}

#[test]
fn test_format_timestamp_relative() {
    let now = chrono::Utc::now();
    let one_day_ago = now - chrono::Duration::days(1);
    let formatted = format_timestamp(&one_day_ago);
    assert_eq!(formatted, "a day ago");
}
