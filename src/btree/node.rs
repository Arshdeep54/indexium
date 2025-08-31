use std::{fmt, io::Result};

use super::{
    Item, MAX_ITEMS, MIN_ITEMS,
    paging::{Page, PageID, Pager},
};

#[derive(Clone, Debug)]
pub struct Node {
    pub id: PageID,
    pub items: Vec<Item>,
    #[allow(clippy::vec_box)]
    pub children: Vec<Box<Node>>,
    pub num_items: i32,
    num_children: i32,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

impl Node {
    pub fn new(id: PageID) -> Self {
        Node {
            id,
            items: Vec::new(),
            children: Vec::new(),
            num_items: 0,
            num_children: 0,
        }
    }

    pub fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        for _ in 0..indent {
            write!(f, "  ")?;
        }
        write!(f, "[")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{item}")?;
        }
        write!(f, "]")?;

        for child in &self.children {
            child.fmt_with_indent(f, indent + 1)?;
        }

        Ok(())
    }
    pub fn is_leaf(&self) -> bool {
        self.num_children == 0
    }
    pub fn search(&self, key: i32) -> (i32, bool) {
        let mut low: i32 = 0;
        let mut high: i32 = self.num_items;
        let mut mid;
        while low < high {
            mid = low + (high - low) / 2;
            if self.items[mid as usize].key == key {
                return (mid, true);
            } else if self.items[mid as usize].key < key {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        (low, false)
    }

    pub fn insert_item_at(&mut self, pos: i32, item: Item) {
        if pos > self.num_items || pos < 0 {
            return;
        }
        let mut insert_pos = pos as usize;
        while insert_pos < self.items.len() && self.items[insert_pos].key < item.key {
            insert_pos += 1;
        }

        self.items.insert(insert_pos, item);
        self.num_items += 1;
    }

    pub fn insert_child_at(&mut self, pos: i32, node: Node) {
        if pos > self.num_items || pos < 0 {
            return;
        }
        self.children.insert(pos as usize, Box::new(node));
        self.num_children += 1;
    }

    pub fn split(&mut self, pager: &mut Pager) -> Result<(Item, Node)> {
        let new_id = pager.allocate_page()?;
        let mut new_node = Node::new(new_id);

        let mid = MIN_ITEMS;
        let mid_item = self.items[mid as usize].clone();

        new_node.items = self.items[mid as usize..].to_vec();
        new_node.num_items = self.num_items - mid;

        if !self.is_leaf() {
            new_node.children = self.children[mid as usize + 1..].to_vec();
            new_node.num_children = new_node.num_items + 1;
        }

        self.items.truncate(mid as usize);
        self.num_items = mid;

        if !self.is_leaf() {
            self.children.truncate(mid as usize + 1);
            self.num_children = self.num_items + 1;
        }

        Ok((mid_item, new_node))
    }

    pub fn insert(&mut self, item: Item, pager: &mut Pager) {
        let (mut pos, found) = self.search(item.key);
        if found {
            println!("Key already exist");
            return;
        }

        if self.is_leaf() {
            self.insert_item_at(pos, item);
            return;
        }

        if self.children[pos as usize].num_items >= MAX_ITEMS {
            let (mid_item, new_node) = self.children[pos as usize].split(pager).unwrap();

            if !self.is_leaf() {
                let nav_item = Item {
                    key: mid_item.key,
                    val: String::new(),
                };
                self.insert_item_at(pos, nav_item);
            } else {
                self.insert_item_at(pos, mid_item);
            }

            self.insert_child_at(pos + 1, new_node);

            let condition = item.key - self.items[pos as usize].key;
            if condition < 0 {
            } else if condition > 0 {
                pos += 1;
            } else {
                return;
            }
        }
        self.children[pos as usize].insert(item, pager);
    }
}

impl Node {
    pub fn to_page(&self) -> Page {
        if self.is_leaf() {
            Page::Leaf {
                id: self.id,
                items: self.items.clone(),
            }
        } else {
            let keys = self.items.iter().map(|it| it.key).collect();
            let children = self.children.iter().map(|c| c.id).collect();
            Page::Internal {
                id: self.id,
                keys,
                children,
            }
        }
    }

    pub fn from_page(page: &Page) -> Self {
        match page {
            Page::Leaf { id, items } => {
                let num_items = items.len() as i32;
                Node {
                    id: *id,
                    items: items.clone(),
                    children: vec![],
                    num_items,
                    num_children: 0,
                }
            }
            Page::Internal {
                id,
                keys,
                children: child_ids,
            } => {
                let items: Vec<Item> = keys
                    .iter()
                    .map(|k| Item {
                        key: *k,
                        val: String::new(),
                    })
                    .collect();
                let num_items = items.len() as i32;
                let num_children = child_ids.len() as i32;

                Node {
                    id: *id,
                    items,
                    children: Vec::new(),
                    num_items,
                    num_children,
                }
            }
        }
    }
}
