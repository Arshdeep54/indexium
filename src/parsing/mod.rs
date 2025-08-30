use crate::{IndexSession, btree::Item};

struct Command<KeyType, ValType> {
    index_type: String,
    index_function: String,
    key: KeyType,
    value: Option<ValType>,
}

impl<KeyType, ValType> Command<KeyType, ValType>
where
    KeyType: std::str::FromStr,
    ValType: std::str::FromStr,
{
    fn new(command: &str) -> Self {
        let tokens: Vec<&str> = command.split(" ").collect();
        let value = if tokens.len() > 3 {
            Some(tokens[3].parse::<ValType>().ok().unwrap())
        } else {
            None
        };
        Command {
            index_type: tokens[0].to_string(),
            index_function: tokens[1].to_string(),
            key: tokens[2].parse().ok().unwrap(),
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
                index_session.btree.insert(Item {
                    key: cmd.key,
                    val: cmd.value.clone().unwrap(),
                });
                println!("{:?}", index_session.btree);
            }
            "SEARCH" | "search" => match index_session.btree.search(cmd.key) {
                Ok(val) => println!("Value {val}"),
                Err(_) => println!("Key not found"),
            },
            "DELETE" | "delete" => {
                index_session.btree.delete(cmd.key);
            }
            _ => {}
        }
    }
}
