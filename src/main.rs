use std::{fs, path::PathBuf};

use btree::Btree;
// use btree::{Btree, Item};
use input_handler::InputHandler;
use parsing::parse_command;
mod btree;
mod parsing;
pub struct IndexSession {
    btree:Btree,
}
impl IndexSession {
    fn new()->Self{
        IndexSession { btree: Btree::new() }
    }
    
}
fn main() {
    let mut index_session=IndexSession::new();

    let data_dir = PathBuf::from("data");
    if !data_dir.exists() {
        fs::create_dir(&data_dir).expect("Failed to create data directory");
    }
    let history_file = data_dir.join("history.txt");

    let mut input_handler =
        InputHandler::with_history_file(history_file).expect("Failed to initialize input handler");

    while let Ok(line) = input_handler.readline("indexium> ") {
        if line.eq_ignore_ascii_case("exit") {
            break;
        }
        parse_command(&mut index_session,&line);
    }
    // let mut btree = Btree::new();
    // match args.index_kind.as_str() {
    //     "BTREE" => match args.index_func.as_str() {
    //         "INSERT" => {
    //             btree.insert(Item { key: args.key, val: args.value });
    //             println!("{:?}",btree);
    //         }
    //         _ => {}
    //     },
    //     _ => {}
    // }
    

}
