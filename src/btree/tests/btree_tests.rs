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
            val: format!("value-{}", i),
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
