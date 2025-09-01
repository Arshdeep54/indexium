use crate::{
    IndexSession,
    btree::{Item, utils::Visualizer},
};
use std::path::PathBuf;

struct Command<KeyType, ValType> {
    index_type: String,
    index_function: String,
    key: Option<KeyType>,
    value: Option<ValType>,
}

impl<KeyType, ValType> Command<KeyType, ValType>
where
    KeyType: std::str::FromStr,
    ValType: std::str::FromStr,
{
    fn new(command: &str) -> Option<Self> {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        if tokens.is_empty() {
            return None;
        }

        let index_type = tokens[0].to_string();
        if tokens.len() < 2 {
            return None;
        }

        let index_function = tokens[1].to_string();
        let key = if tokens.len() > 2 {
            tokens[2].parse::<KeyType>().ok()
        } else {
            None
        };
        let value = if tokens.len() > 3 {
            tokens[3].parse::<ValType>().ok()
        } else {
            None
        };

        Some(Command {
            index_type,
            index_function,
            key,
            value,
        })
    }
}

pub fn parse_command(index_session: &mut IndexSession, command: &str) {
    let trimmed_command = command.trim();
    if trimmed_command.is_empty() {
        return;
    }
    println!("Command: {trimmed_command}");

    let mut viz_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    viz_path.push("src");
    viz_path.push("btree");
    viz_path.push("tests");
    viz_path.push("visualizer.md");
    let visualizer = Visualizer::new(viz_path.to_str().unwrap());

    let cmd = match Command::<i32, String>::new(trimmed_command) {
        Some(cmd) => cmd,
        None => return,
    };

    if cmd.index_type.as_str() == "BTREE" || cmd.index_type.as_str() == "btree" {
        match cmd.index_function.as_str() {
            "INSERT" | "insert" => {
                let key = match cmd.key {
                    Some(k) => k,
                    None => {
                        eprintln!("Error: Missing key for INSERT");
                        return;
                    }
                };
                let val = match cmd.value.clone() {
                    Some(v) => v,
                    None => {
                        eprintln!(
                            "Error: Missing value for INSERT (key = {})",
                            cmd.key.unwrap()
                        );
                        return;
                    }
                };

                index_session.btree.insert(Item { key, val });

                if let Err(e) = visualizer.update(&index_session.btree) {
                    eprintln!("Failed to update visualization: {e}");
                }
            }
            "SEARCH" | "search" => {
                let key = match cmd.key {
                    Some(k) => k,
                    None => {
                        eprintln!("Error: Missing key for INSERT");
                        return;
                    }
                };

                match index_session.btree.search(key) {
                    Ok(val) => println!("Value {val}"),
                    Err(_) => println!("Key not found"),
                }
            }
            "DELETE" | "delete" => {
                let key = match cmd.key {
                    Some(k) => k,
                    None => {
                        eprintln!("Error: Missing key for DELETE");
                        return;
                    }
                };
                match index_session.btree.delete(key) {
                    Ok(_) => {
                        println!("Successfully deleted key {key}");

                        if let Err(e) = visualizer.update(&index_session.btree) {
                            eprintln!("Failed to update visualization: {e}");
                        }
                    }
                    Err(e) => println!("Failed to delete key {key}: {e}"),
                }
            }
            "SNAPSHOT" | "snapshot" => {
                index_session.btree.snapshot().expect("Failed to snapshot");
            }
            _ => {}
        }
    }
}
