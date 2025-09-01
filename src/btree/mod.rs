use core::fmt;
use metadata::BtreeMetadata;
use node::Node;
use paging::{Page, Pager};
use std::io::{self, Result};
mod metadata;
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
        let file_exists = std::path::Path::new(filename).exists();

        let file = if file_exists {
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(filename)?
        } else {
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(filename)?
        };

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
        let new_root_id = self.pager.allocate_page().unwrap();
        let mut new_root = Node::new(new_root_id);

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

    pub fn delete(&mut self, key: i32) -> Result<()> {
        if self.root.is_none() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Tree is empty"));
        }

        let mut root = self.root.take().unwrap();
        let result = self.delete_recursive(&mut root, key);

        if root.num_items == 0 && !root.is_leaf() {
            self.root = Some(root.children.remove(0));
        } else {
            self.root = Some(root);
        }

        result
    }

    fn delete_recursive(&mut self, node: &mut Node, key: i32) -> Result<()> {
        let (pos, found) = node.search(key);

        if node.is_leaf() {
            if found {
                self.delete_from_leaf(node, pos)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Key {key} not found"),
                ))
            }
        } else if found {
            self.delete_from_internal(node, pos)
        } else {
            self.delete_from_subtree(node, pos, key)
        }
    }
    pub fn snapshot(&mut self) -> Result<()> {
        if self.root.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot snapshot empty tree",
            ));
        }

        let root_page_id = self.root.as_ref().unwrap().id;
        let metadata = BtreeMetadata::new(
            root_page_id,
            self.pager.page_size as u32,
            self.pager.num_pages,
        );
        self.pager.write_metadata(&metadata)?;

        if let Some(root) = self.root.clone() {
            self.snapshot_node(&root)?;
        }

        self.pager.file.sync_all()?;

        Ok(())
    }

    fn snapshot_node(&mut self, node: &Node) -> std::io::Result<()> {
        if !node.is_leaf() {
            for child in &node.children {
                self.snapshot_node(child)?;
            }
        }
        let page = node.to_page();
        self.pager.write_page(&page)?;

        Ok(())
    }

    pub fn load_snapshot(filename: &str, page_size: usize) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(filename)?;

        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot load snapshot from empty file",
            ));
        }

        let mut pager = Pager {
            file,
            page_size,
            num_pages: 0,
        };
        let metadata = pager.read_metadata()?;

        if metadata.page_size as usize != page_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Page size mismatch: expected {}, got {}",
                    page_size, metadata.page_size
                ),
            ));
        }
        pager.num_pages = metadata.num_pages;

        let root_page = if metadata.root_page_id == 0 {
            pager.read_page(1)?
        } else {
            pager.read_page(metadata.root_page_id)?
        };
        let root_node = Self::load_node(&mut pager, &root_page)?;

        if root_node.num_items < 0 || root_node.num_items > MAX_ITEMS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid root node: invalid item count {}",
                    root_node.num_items
                ),
            ));
        }

        Ok(Btree {
            pager,
            root: Some(Box::new(root_node)),
        })
    }

    fn load_node(pager: &mut Pager, page: &Page) -> std::io::Result<Node> {
        let mut node = Node::from_page(page);

        if let Page::Internal { children, .. } = page {
            let mut loaded_children = Vec::new();
            for &child_page_id in children {
                if child_page_id == 0 || child_page_id > pager.num_pages {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid child page ID: {child_page_id}"),
                    ));
                }
                let child_page = pager.read_page(child_page_id)?;
                let child_node = Self::load_node(pager, &child_page)?;
                loaded_children.push(Box::new(child_node));
            }
            node.children = loaded_children;
        }

        Ok(node)
    }

    pub fn is_valid_snapshot(filename: &str, page_size: usize) -> bool {
        let file = match std::fs::OpenOptions::new().read(true).open(filename) {
            Ok(file) => file,
            Err(_) => return false,
        };

        let metadata = match file.metadata() {
            Ok(meta) => meta,
            Err(_) => return false,
        };

        if metadata.len() < page_size as u64 {
            return false;
        }

        let mut pager = Pager {
            file,
            page_size,
            num_pages: 0,
        };

        match pager.read_metadata() {
            Ok(metadata) => metadata.page_size as usize == page_size,
            Err(_) => false,
        }
    }
}

impl Btree {
    fn delete_from_leaf(&mut self, node: &mut Node, pos: i32) -> Result<()> {
        let key = node.items[pos as usize].key;
        let mut indices_to_remove = Vec::new();

        for (i, item) in node.items.iter().enumerate() {
            if item.key == key {
                indices_to_remove.push(i);
            }
        }

        for &index in indices_to_remove.iter().rev() {
            node.items.remove(index);
            node.num_items -= 1;
        }

        Ok(())
    }

    fn delete_from_internal(&mut self, node: &mut Node, pos: i32) -> Result<()> {
        let key = node.items[pos as usize].key;

        if node.children[pos as usize].num_items > MIN_ITEMS {
            let predecessor = node.get_predecessor(pos);
            node.items[pos as usize] = predecessor.clone();
            self.delete_recursive(&mut node.children[pos as usize], predecessor.key)
        } else if node.children[pos as usize + 1].num_items > MIN_ITEMS {
            let successor = node.get_successor(pos);
            node.items[pos as usize] = successor.clone();
            self.delete_recursive(&mut node.children[pos as usize + 1], successor.key)
        } else {
            node.merge_children(pos);
            self.delete_recursive(&mut node.children[pos as usize], key)
        }
    }

    fn delete_from_subtree(&mut self, node: &mut Node, pos: i32, key: i32) -> Result<()> {
        let child_has_min = node.children[pos as usize].num_items == MIN_ITEMS;

        if child_has_min {
            self.fill_child(node, pos)?;
        }

        if pos >= node.num_children {
            self.delete_recursive(&mut node.children[pos as usize - 1], key)
        } else {
            let child = &node.children[pos as usize];
            if child.num_items == 0 {
                self.delete_recursive(&mut node.children[pos as usize + 1], key)
            } else {
                self.delete_recursive(&mut node.children[pos as usize], key)
            }
        }
    }

    fn fill_child(&mut self, node: &mut Node, pos: i32) -> Result<()> {
        if pos > 0 && node.children[pos as usize - 1].num_items > MIN_ITEMS {
            node.borrow_from_prev(pos)
        } else if pos < node.num_children - 1
            && node.children[pos as usize + 1].num_items > MIN_ITEMS
        {
            node.borrow_from_next(pos)
        } else if pos > 0 {
            node.merge_children(pos - 1)
        } else {
            node.merge_children(pos)
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests;
