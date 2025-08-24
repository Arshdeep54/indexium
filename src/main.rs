use btree::{Btree, Item};

mod btree;
fn main() {
    let item = Item {
        key: 3,
        val: "Hey its 3".to_string(),
    };
    let mut btree = Btree::new();
    btree.insert(item);
    btree.insert(Item {
        key: 4,
        val: "hey its 4".to_string(),
    });
    btree.insert(Item {
        key: 5,
        val: "hey its 5".to_string(),
    });
    btree.insert(Item {
        key: 6,
        val: "hey its 6".to_string(),
    });
    btree.insert(Item {
        key: 7,
        val: "hey its 7".to_string(),
    });
    btree.insert(Item {
        key: 8,
        val: "hey its 8".to_string(),
    });
    btree.insert(Item {
        key: 9,
        val: "hey its 9".to_string(),
    });
    btree.insert(Item {
        key: 10,
        val: "hey its 10".to_string(),
    });
    btree.insert(Item {
        key: 11,
        val: "hey its 11".to_string(),
    });

    match btree.search(8) {
        Ok(val) => println!("{val}"),
        Err(_) => println!("Key not found"),
    }

    if let Some(ref root_node) = btree.root {
        println!("No of items in root: {:?}", root_node.num_items);
    } else {
        println!("The root node is empty.");
    }
}
