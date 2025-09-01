// src/btree/tests/snapshot_tests.rs
use super::super::{Btree, Item};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_snapshot_creation() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut btree = Btree::new(temp_file.path().to_str().unwrap(), 4096).unwrap();

    btree.insert(Item {
        key: 1,
        val: "one".to_string(),
    });
    btree.insert(Item {
        key: 2,
        val: "two".to_string(),
    });

    let result = btree.snapshot();
    assert!(result.is_ok());

    let metadata = fs::metadata(temp_file.path()).unwrap();
    assert!(metadata.len() > 0);
}

#[test]
fn test_snapshot_loading() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let mut btree = Btree::new(path, 4096).unwrap();
    btree.insert(Item {
        key: 10,
        val: "ten".to_string(),
    });
    btree.insert(Item {
        key: 20,
        val: "twenty".to_string(),
    });

    btree.snapshot().unwrap();

    let loaded_btree = Btree::load_snapshot(path, 4096).unwrap();

    let value1 = loaded_btree.search(10).unwrap();
    let value2 = loaded_btree.search(20).unwrap();

    assert_eq!(value1, "ten");
    assert_eq!(value2, "twenty");
}

#[test]
fn test_snapshot_validation() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    assert!(!Btree::is_valid_snapshot(path, 4096));

    let mut btree = Btree::new(path, 4096).unwrap();
    btree.insert(Item {
        key: 5,
        val: "five".to_string(),
    });
    btree.snapshot().unwrap();

    assert!(Btree::is_valid_snapshot(path, 4096));
}

#[test]
fn test_snapshot_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let mut btree = Btree::new(path, 4096).unwrap();
    for i in 0..10 {
        btree.insert(Item {
            key: i,
            val: format!("value-{i}"),
        });
    }

    btree.snapshot().unwrap();

    let loaded_btree = Btree::load_snapshot(path, 4096).unwrap();

    for i in 0..10 {
        match loaded_btree.search(i) {
            Ok(value) => println!("Key {i}: Found '{value}'"),
            Err(e) => println!("Key {i}: ERROR - {e}"),
        }
    }

    for i in 0..10 {
        let value = loaded_btree.search(i).unwrap();
        assert_eq!(value, format!("value-{i}"));
    }
}
