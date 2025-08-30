use core::fmt;
use node::Node;
use paging::Pager;
use std::io::{self, Result};
mod node;
mod paging;
pub const DEGREE: i32 = 3;
pub const MIN_ITEMS: i32 = DEGREE - 1;
pub const MAX_ITEMS: i32 = DEGREE * 2;

#[derive(Clone, Debug)]
pub struct Item {
    pub key: i32,
    pub val: String,
}
impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.key, self.val)
    }
}

#[derive(Debug)]
pub struct Btree {
    pager: Pager,
    pub root: Option<Box<Node>>,
}

impl fmt::Display for Btree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.root {
            Some(node) => write!(f, "{node}"),
            None => write!(f, "<empty tree>"),
        }
    }
}
impl Btree {
    pub fn new(filename: &str, page_size: usize) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)?;

        Ok(Btree {
            pager: Pager {
                file,
                page_size,
                num_pages: 0,
            },
            root: None,
        })
    }

    pub fn insert(&mut self, item: Item) {
        println!("Inserting {:?}", item.key);
        if self.root.is_none() {
            let id = self.pager.allocate_page().unwrap();
            self.root = Some(Box::new(Node::new(id)));
        }
        let root_is_full = if let Some(root_node) = self.root.as_ref() {
            root_node.num_items >= MAX_ITEMS
        } else {
            false
        };

        if root_is_full {
            println!("Root is full {:?}", self.root.clone().unwrap().num_items);
            self.split_root();
        }

        if let Some(root_node) = self.root.as_mut() {
            root_node.insert(item, &mut self.pager);
        }
    }

    fn split_root(&mut self) {
        let mut old_root = self
            .root
            .take()
            .expect("Called split_root on an empty tree.");

        let (mid_item, new_node) = old_root.split(&mut self.pager).unwrap();
        let id = self.pager.allocate_page().unwrap();
        let mut new_root = Node::new(id);
        new_root.insert_item_at(0, mid_item);
        new_root.insert_child_at(0, *old_root);
        new_root.insert_child_at(1, new_node);
        self.root = Some(Box::new(new_root));

        println!(
            "Root split. New num_items {:?}",
            self.root.clone().unwrap().num_items
        );
    }

    pub fn search(&self, key: i32) -> Result<String> {
        let mut current_node_opt = self.root.as_deref();

        while let Some(current_node) = current_node_opt {
            let (pos, found) = current_node.search(key);

            if found {
                let val = &current_node.items[pos as usize].val;
                return Ok(val.clone());
            }

            current_node_opt = current_node
                .children
                .get(pos as usize)
                .map(|boxed_node| &**boxed_node);
        }

        Err(io::Error::new(io::ErrorKind::NotFound, "Key not found"))
    }
    pub fn delete(&mut self, key: i32) {
        println!("Deleting {key}")
    }
    pub fn snapshot(&mut self) -> Result<()> {
        if let Some(root) = self.root.clone() {
            self.snapshot_node(&root)?;
        }
        Ok(())
    }
    fn snapshot_node(&mut self, node: &Node) -> std::io::Result<()> {
        let page = node.to_page();

        self.pager.write_page(&page)?;

        if !node.is_leaf() {
            for child in &node.children {
                self.snapshot_node(child)?;
            }
        }

        Ok(())
    }
}
