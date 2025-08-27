use core::fmt;

pub const DEGREE: i32 = 3;
pub const MIN_ITEMS: i32 = DEGREE - 1;
pub const MAX_ITEMS: i32 = DEGREE * 2;

#[derive(Clone,Debug)]
pub struct Item {
    pub key: i32,
    pub val: String,
}
impl fmt::Display for Item {
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            write!(f,"{}-{}",self.key,self.val)
        }
}

#[derive(Clone,Debug)]
pub struct Node {
    items: Vec<Item>,
    children: Vec<Box<Node>>,
    pub num_items: i32,
    num_children: i32,
}

impl fmt::Display for Node {
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
             self.fmt_with_indent(f, 0)
        }
}
impl Node {
    pub fn new() -> Self {
        Node {
            items: Vec::new(),
            children: Vec::new(),
            num_items: 0,
            num_children: 0,
        }
    }

    fn fmt_with_indent(&self, f:&mut fmt::Formatter<'_>, indent:usize) -> fmt::Result{
        for _ in 0..indent{
            write!(f,"  ")?;
        }
        write!(f,"[")?;
        for(i,item) in self.items.iter().enumerate(){
            if i>0 {
                write!(f,", ")?;
            }
            write!(f,"{:?}",item)?;
        }
        write!(f, "]")?;

        for child in &self.children{
            child.fmt_with_indent(f,indent+1)?;
        }

        Ok(())
    }
    fn is_leaf(&self) -> bool {
        self.num_children == 0
    }
    fn search(&self, key: i32) -> (i32, bool) {
        let mut low: i32 = 0;
        let mut high: i32 = self.num_items;
        let mut mid;
        while low < high {
            mid = low + (high - low) / 2;
            if self.items[mid as usize].key == key {
                return (mid, true);
            } else if self.items[mid as usize].key > key {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        (low, false)
    }

    fn insert_item_at(&mut self, pos: i32, item: Item) {
        if pos > self.num_items || pos < 0 {
            return;
        }
        self.items.insert(pos as usize, item);
        self.num_items += 1;
    }

    fn insert_child_at(&mut self, pos: i32, node: Node) {
        if pos > self.num_items || pos < 0 {
            return;
        }
        self.children.insert(pos as usize, Box::new(node));
        self.num_children += 1;
    }

    fn split(&mut self) -> (Item, Node) {
        let mid = MIN_ITEMS;
        let mid_item = self.items[mid as usize].clone();

        let mut new_node = Node::new();
        new_node.items = self.items[mid as usize + 1..].to_vec();
        new_node.num_items = MIN_ITEMS;

        if !self.is_leaf() {
            new_node.children = self.children[mid as usize + 1..].to_vec();
            new_node.num_children = MIN_ITEMS + 1;
        }

        if mid < self.num_items {
            self.items.truncate(mid as usize);
            self.num_items = mid;

            if !self.is_leaf() {
                self.children.truncate(mid as usize + 1);
            }
        }
        (mid_item, new_node)
    }

    fn insert(&mut self, item: Item) {
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
            let (mid_item, new_node) = self.children[pos as usize].split();
            self.insert_item_at(pos, mid_item);
            self.insert_child_at(pos + 1, new_node);

            let condition = item.key - self.items[pos as usize].key;
            if condition < 0 {
            } else if condition > 0 {
                pos += 1;
            } else {
                self.items[pos as usize] = item;
                return;
            }
        }
        self.children[pos as usize].insert(item);
    }
}
#[derive(Debug)]
pub struct Btree {
    pub root: Option<Box<Node>>,
}

impl fmt::Display for Btree{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        match &self.root{
            Some(node) => write!(f,"{}" ,node),
            None=> write!(f, "<empty tree>"),
        }
    }
}
impl Btree {
    pub fn new() -> Self {
        Btree {
            root: None,
        }
    }

    pub fn insert(&mut self, item: Item) {
        println!("Inserting {:?}",item.key);
        if self.root.is_none() {
            self.root = Some(Box::new(Node::new()));
        }
        let root_is_full = if let Some(root_node) = self.root.as_ref() {
            root_node.num_items >= MAX_ITEMS
        } else {
            false
        };

        if root_is_full {
            println!("Root is full {:?}",self.root.clone().unwrap().num_items);
            self.split_root();
        }

        if let Some(root_node) = self.root.as_mut() {
            root_node.insert(item);
        }
    }

    fn split_root(&mut self) {
        let mut old_root = self
            .root
            .take()
            .expect("Called split_root on an empty tree.");

        let (mid_item, new_node) = old_root.split();
        let mut new_root = Node::new();
        new_root.insert_item_at(0, mid_item);
        new_root.insert_child_at(0, *old_root);
        new_root.insert_child_at(1, new_node);
        self.root = Some(Box::new(new_root));

        println!("Root split. New num_items {:?}",self.root.clone().unwrap().num_items);
    }

    pub fn search(&self, key: i32) -> Result<String, String> {
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

        Err("Key not found".to_string())
    }
}
