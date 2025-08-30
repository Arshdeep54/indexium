use crate::{IndexSession, btree::Item};

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
    fn new(command: &str) -> Self {
        let tokens: Vec<&str> = command.split(" ").collect();
        let key = if tokens.len() > 2 {
            Some(tokens[2].parse::<KeyType>().ok().unwrap())
        } else {
            None
        };
        let value = if tokens.len() > 3 {
            Some(tokens[3].parse::<ValType>().ok().unwrap())
        } else {
            None
        };
        Command {
            index_type: tokens[0].to_string(),
            index_function: tokens[1].to_string(),
            key,
            value,
        }
    }
}
pub fn parse_command(index_session: &mut IndexSession, command: &str) {
    let trimmed_command = command.trim();
    println!("Command: {trimmed_command}");
    let cmd: Command<i32, String> = Command::new(command);
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
                println!("{:?}", index_session.btree);
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
                        eprintln!("Error: Missing key for INSERT");
                        return;
                    }
                };
                index_session.btree.delete(key);
            }
            "SNAPSHOT" | "snapshot" => {
                index_session.btree.snapshot().expect("Failed to snapshot");
            }
            _ => {}
        }
    }
}
