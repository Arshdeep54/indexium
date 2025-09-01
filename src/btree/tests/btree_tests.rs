use super::super::{Btree, Item, MAX_ITEMS};
use tempfile::NamedTempFile;

fn create_test_btree() -> (Btree, NamedTempFile) {
    let temp_file = NamedTempFile::new().unwrap();
    let btree = Btree::new(temp_file.path().to_str().unwrap(), 4096).unwrap();
    (btree, temp_file)
}

#[test]
fn test_new_btree() {
    let (btree, _temp_file) = create_test_btree();
    assert!(btree.root.is_none());
    assert_eq!(btree.pager.page_size, 4096);
}

#[test]
fn test_insert_single_item() {
    let (mut btree, _temp_file) = create_test_btree();

    let item = Item {
        key: 42,
        val: "test".to_string(),
    };
    btree.insert(item);

    assert!(btree.root.is_some());
    let root = btree.root.as_ref().unwrap();
    assert_eq!(root.num_items, 1);
    assert_eq!(root.items[0].key, 42);
    assert_eq!(root.items[0].val, "test");
}

#[test]
fn test_insert_multiple_items() {
    let (mut btree, _temp_file) = create_test_btree();

    // Insert items in random order
    btree.insert(Item {
        key: 50,
        val: "fifty".to_string(),
    });
    btree.insert(Item {
        key: 30,
        val: "thirty".to_string(),
    });
    btree.insert(Item {
        key: 70,
        val: "seventy".to_string(),
    });

    // Verify they're stored in sorted order
    let root = btree.root.as_ref().unwrap();
    assert_eq!(root.num_items, 3);
    assert_eq!(root.items[0].key, 30);
    assert_eq!(root.items[1].key, 50);
    assert_eq!(root.items[2].key, 70);
}

#[test]
fn test_search_existing_key() {
    let (mut btree, _temp_file) = create_test_btree();

    btree.insert(Item {
        key: 25,
        val: "twenty-five".to_string(),
    });

    let result = btree.search(25).unwrap();
    assert_eq!(result, "twenty-five");
}

#[test]
fn test_search_nonexistent_key() {
    let (mut btree, _temp_file) = create_test_btree();

    btree.insert(Item {
        key: 10,
        val: "ten".to_string(),
    });

    let result = btree.search(99);
    assert!(result.is_err());
}

#[test]
fn test_tree_splitting() {
    let (mut btree, _temp_file) = create_test_btree();

    for i in 0..(MAX_ITEMS + 1) {
        btree.insert(Item {
            key: i,
            val: format!("value-{i}"),
        });
    }

    let root = btree.root.as_ref().unwrap();
    assert_eq!(root.num_items, 1);
    assert_eq!(root.children.len(), 2);
    assert!(!root.is_leaf());
}

#[test]
fn test_duplicate_key_handling() {
    let (mut btree, _temp_file) = create_test_btree();

    btree.insert(Item {
        key: 100,
        val: "first".to_string(),
    });
    btree.insert(Item {
        key: 100,
        val: "second".to_string(),
    });

    let root = btree.root.as_ref().unwrap();
    assert_eq!(root.num_items, 1);
    assert_eq!(root.items[0].val, "first"); // First value should be preserved
}

#[test]
fn test_delete_leaf_node() {
    let (mut btree, _temp_file) = create_test_btree();

    btree.insert(Item {
        key: 10,
        val: "ten".to_string(),
    });
    btree.insert(Item {
        key: 20,
        val: "twenty".to_string(),
    });

    assert!(btree.delete(10).is_ok());

    let root = btree.root.as_ref().unwrap();
    assert_eq!(root.num_items, 1);
    assert_eq!(root.items[0].key, 20);
    assert!(btree.search(10).is_err());
}

#[test]
fn test_delete_internal_node() {
    let (mut btree, _temp_file) = create_test_btree();

    for i in 0..7 {
        btree.insert(Item {
            key: i,
            val: format!("value-{i}"),
        });
    }

    assert!(btree.delete(3).is_ok());

    assert!(btree.search(3).is_err());

    assert_eq!(btree.search(2).unwrap(), "value-2");
    assert_eq!(btree.search(4).unwrap(), "value-4");
}

#[test]
fn test_delete_with_merge() {
    let (mut btree, _temp_file) = create_test_btree();

    for i in 0..7 {
        btree.insert(Item {
            key: i * 10,
            val: format!("value-{}", i * 10),
        });
    }

    assert!(btree.delete(0).is_ok());
    assert!(btree.delete(10).is_ok());
    assert!(btree.delete(20).is_ok());

    assert!(btree.search(0).is_err());
    assert!(btree.search(10).is_err());
    assert!(btree.search(20).is_err());
    assert_eq!(btree.search(30).unwrap(), "value-30");
}

#[test]
fn test_internal_node_values() {
    let (mut btree, _temp_file) = create_test_btree();

    for i in 0..7 {
        btree.insert(Item {
            key: i,
            val: format!("value-{i}"),
        });
    }

    let root = btree.root.as_ref().unwrap();
    assert!(!root.is_leaf());
    assert_eq!(root.items[0].val, "value-2");
    assert_eq!(btree.search(2).unwrap(), "value-2");
}

#[test]
fn test_split_preserves_values() {
    let (mut btree, _temp_file) = create_test_btree();

    for i in 0..10 {
        btree.insert(Item {
            key: i,
            val: format!("value-{i}"),
        });
    }

    for i in 0..10 {
        assert_eq!(btree.search(i).unwrap(), format!("value-{i}"));
    }
}

#[test]
fn test_merge_preserves_values() {
    let (mut btree, _temp_file) = create_test_btree();

    for i in 0..7 {
        btree.insert(Item {
            key: i * 10,
            val: format!("value-{}", i * 10),
        });
    }

    assert!(btree.delete(0).is_ok());
    assert!(btree.delete(10).is_ok());

    assert_eq!(btree.search(20).unwrap(), "value-20");
    assert_eq!(btree.search(30).unwrap(), "value-30");
}
