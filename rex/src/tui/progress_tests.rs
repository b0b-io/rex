//! Tests for progress bar functionality.

use super::*;

#[test]
fn test_progress_bar_new() {
    let progress = ProgressBar::new(45, 115, "Fetching repositories");

    assert_eq!(progress.current(), 45);
    assert_eq!(progress.total(), 115);
}

#[test]
fn test_progress_bar_percentage() {
    let progress = ProgressBar::new(45, 115, "Test");
    assert_eq!(progress.percentage(), 39); // 45/115 ≈ 39.13%

    let progress = ProgressBar::new(50, 100, "Test");
    assert_eq!(progress.percentage(), 50);

    let progress = ProgressBar::new(100, 100, "Test");
    assert_eq!(progress.percentage(), 100);

    let progress = ProgressBar::new(0, 100, "Test");
    assert_eq!(progress.percentage(), 0);
}

#[test]
fn test_progress_bar_percentage_zero_total() {
    let progress = ProgressBar::new(0, 0, "Test");
    assert_eq!(progress.percentage(), 0);
}

#[test]
fn test_progress_bar_percentage_over_100() {
    // If current > total, cap at 100%
    let progress = ProgressBar::new(150, 100, "Test");
    assert_eq!(progress.percentage(), 100);
}

#[test]
fn test_progress_bar_is_complete() {
    let progress = ProgressBar::new(100, 100, "Test");
    assert!(progress.is_complete());

    let progress = ProgressBar::new(50, 100, "Test");
    assert!(!progress.is_complete());

    let progress = ProgressBar::new(101, 100, "Test");
    assert!(progress.is_complete());
}

#[test]
fn test_progress_bar_message() {
    let progress = ProgressBar::new(45, 115, "Fetching repositories");
    assert_eq!(progress.message, "Fetching repositories");

    let progress = ProgressBar::new(0, 100, String::from("Loading..."));
    assert_eq!(progress.message, "Loading...");
}

#[test]
fn test_progress_bar_various_ranges() {
    // Small range
    let progress = ProgressBar::new(1, 5, "Test");
    assert_eq!(progress.percentage(), 20);

    // Large range
    let progress = ProgressBar::new(500, 1000, "Test");
    assert_eq!(progress.percentage(), 50);

    // Single item
    let progress = ProgressBar::new(1, 1, "Test");
    assert_eq!(progress.percentage(), 100);
}
