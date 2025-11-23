//! Tests for the worker module.

use super::*;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

// Helper for test cache directory
fn test_cache_dir() -> &'static Path {
    Path::new("/tmp/rex-test-cache")
}

// Note: These tests verify the worker function signatures and message passing.
// Full integration tests with actual registry connections are done separately.

#[test]
fn test_fetch_repositories_sends_message() {
    let (tx, rx) = channel();

    // Spawn in a thread to test async behavior
    let handle = std::thread::spawn(move || {
        fetch_repositories("invalid-url".to_string(), test_cache_dir(), None, tx, 5);
    });

    // Wait for worker to complete
    handle.join().unwrap();

    // Should receive a message (even if it's an error)
    let msg = rx.recv_timeout(Duration::from_secs(1));
    assert!(msg.is_ok(), "Should receive a message from worker");

    // Verify it's the right message type
    if let Ok(Message::RepositoriesLoaded(_)) = msg {
        // Correct message type
    } else {
        panic!("Expected RepositoriesLoaded message");
    }
}

#[test]
fn test_fetch_tags_sends_message() {
    let (tx, rx) = channel();

    let handle = std::thread::spawn(move || {
        fetch_tags(
            "invalid-url".to_string(),
            "alpine".to_string(),
            test_cache_dir(),
            None,
            tx,
        );
    });

    handle.join().unwrap();

    let msg = rx.recv_timeout(Duration::from_secs(1));
    assert!(msg.is_ok(), "Should receive a message from worker");

    // Verify message type and repository name
    if let Ok(Message::TagsLoaded(repo, _)) = msg {
        assert_eq!(repo, "alpine");
    } else {
        panic!("Expected TagsLoaded message");
    }
}

#[test]
fn test_fetch_manifest_and_config_sends_messages() {
    let (tx, rx) = channel();

    let handle = std::thread::spawn(move || {
        fetch_manifest_and_config(
            "invalid-url".to_string(),
            "alpine".to_string(),
            "latest".to_string(),
            test_cache_dir(),
            None,
            tx,
        );
    });

    handle.join().unwrap();

    let msg = rx.recv_timeout(Duration::from_secs(1));
    assert!(msg.is_ok(), "Should receive a message from worker");

    // Verify message type and parameters
    if let Ok(Message::ManifestLoaded(repo, tag, _)) = msg {
        assert_eq!(repo, "alpine");
        assert_eq!(tag, "latest");
    } else {
        panic!("Expected ManifestLoaded message");
    }
}

#[test]
fn test_fetch_repositories_handles_connection_error() {
    let (tx, rx) = channel();

    fetch_repositories(
        "http://nonexistent.invalid:9999".to_string(),
        test_cache_dir(),
        None,
        tx,
        5,
    );

    let msg = rx.recv_timeout(Duration::from_secs(2));
    assert!(msg.is_ok());

    // Should receive error result
    if let Ok(Message::RepositoriesLoaded(result)) = msg {
        assert!(result.is_err(), "Should return error for invalid URL");
    } else {
        panic!("Expected RepositoriesLoaded message");
    }
}

#[test]
fn test_fetch_tags_handles_connection_error() {
    let (tx, rx) = channel();

    fetch_tags(
        "http://nonexistent.invalid:9999".to_string(),
        "alpine".to_string(),
        test_cache_dir(),
        None,
        tx,
    );

    let msg = rx.recv_timeout(Duration::from_secs(2));
    assert!(msg.is_ok());

    if let Ok(Message::TagsLoaded(repo, result)) = msg {
        assert_eq!(repo, "alpine");
        assert!(result.is_err(), "Should return error for invalid URL");
    } else {
        panic!("Expected TagsLoaded message");
    }
}

#[test]
fn test_fetch_manifest_and_config_handles_connection_error() {
    let (tx, rx) = channel();

    fetch_manifest_and_config(
        "http://nonexistent.invalid:9999".to_string(),
        "alpine".to_string(),
        "latest".to_string(),
        test_cache_dir(),
        None,
        tx,
    );

    let msg = rx.recv_timeout(Duration::from_secs(2));
    assert!(msg.is_ok());

    if let Ok(Message::ManifestLoaded(repo, tag, result)) = msg {
        assert_eq!(repo, "alpine");
        assert_eq!(tag, "latest");
        assert!(result.is_err(), "Should return error for invalid URL");
    } else {
        panic!("Expected ManifestLoaded message");
    }
}

#[test]
fn test_multiple_workers_can_run_concurrently() {
    let (tx, rx) = channel();

    // Spawn multiple workers
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let tx3 = tx;

    let h1 = std::thread::spawn(move || {
        fetch_repositories(
            "http://invalid1.test".to_string(),
            test_cache_dir(),
            None,
            tx1,
            5,
        );
    });

    let h2 = std::thread::spawn(move || {
        fetch_tags(
            "http://invalid2.test".to_string(),
            "repo1".to_string(),
            test_cache_dir(),
            None,
            tx2,
        );
    });

    let h3 = std::thread::spawn(move || {
        fetch_manifest_and_config(
            "http://invalid3.test".to_string(),
            "repo2".to_string(),
            "tag1".to_string(),
            test_cache_dir(),
            None,
            tx3,
        );
    });

    // Wait for all workers
    h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();

    // Should receive 3 messages
    let mut count = 0;
    while rx.recv_timeout(Duration::from_millis(100)).is_ok() {
        count += 1;
    }

    assert_eq!(count, 3, "Should receive messages from all workers");
}

#[test]
fn test_worker_thread_exits_after_completion() {
    let (tx, rx) = channel();

    let handle = std::thread::spawn(move || {
        fetch_repositories(
            "http://invalid.test".to_string(),
            test_cache_dir(),
            None,
            tx,
            5,
        );
        // Thread should exit here
    });

    // Thread should complete and exit
    let result = handle.join();
    assert!(result.is_ok(), "Worker thread should exit cleanly");

    // Should have received a message
    let msg = rx.recv_timeout(Duration::from_secs(1));
    assert!(msg.is_ok());
}

#[test]
fn test_sender_can_be_cloned_for_multiple_workers() {
    let (tx, rx) = channel();

    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let tx3 = tx;

    // All senders should work
    let _ = tx1.send(Message::Error("test1".to_string()));
    let _ = tx2.send(Message::Error("test2".to_string()));
    let _ = tx3.send(Message::Error("test3".to_string()));

    // Should receive all 3 messages
    assert!(rx.recv_timeout(Duration::from_millis(100)).is_ok());
    assert!(rx.recv_timeout(Duration::from_millis(100)).is_ok());
    assert!(rx.recv_timeout(Duration::from_millis(100)).is_ok());
}
