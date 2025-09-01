use std::fs::{self, write};
use std::io;
use std::path::Path;

use super::{Btree, Node};

pub struct Visualizer {
    output_path: String,
}

impl Visualizer {
    pub fn new(path: &str) -> Self {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create visualization directory");
            }
        }

        Visualizer {
            output_path: path.to_string(),
        }
    }

    pub fn update(&self, btree: &Btree) -> io::Result<()> {
        let mermaid = self.generate_mermaid(btree);
        let content = format!(
            "# B-tree Visualization\n\n\
            Current state of the B-tree:\n\n\
            ```mermaid\n\
            graph TD\n\
            {mermaid}\n\
            ```\n"
        );
        write(&self.output_path, content)
    }

    fn generate_mermaid(&self, btree: &Btree) -> String {
        match &btree.root {
            None => String::from("empty[Empty Tree]"),
            Some(root) => {
                let mut nodes = Vec::new();
                let mut edges = Vec::new();
                self.generate_node_diagram(&mut nodes, &mut edges, root, None);
                format!("{}\n{}", nodes.join("\n"), edges.join("\n"))
            }
        }
    }

    fn generate_node_diagram(
        &self,
        nodes: &mut Vec<String>,
        edges: &mut Vec<String>,
        node: &Node,
        parent_id: Option<u32>,
    ) {
        let items: Vec<String> = node
            .items
            .iter()
            .map(|item| format!("{}:{}", item.key, self.truncate_value(&item.val, 5)))
            .collect();

        nodes.push(format!(
            "    n{}[\"Node {}<br>{}\"]",
            node.id,
            node.id,
            items.join(" | ")
        ));

        if let Some(parent) = parent_id {
            edges.push(format!("    n{} --> n{}", parent, node.id));
        }

        for child in &node.children {
            self.generate_node_diagram(nodes, edges, child, Some(node.id));
        }
    }

    fn truncate_value(&self, value: &str, max_len: usize) -> String {
        if value.len() <= max_len {
            value.to_string()
        } else {
            format!("{}...", &value[..max_len])
        }
    }
}
