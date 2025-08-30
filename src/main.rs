use std::{fs, path::PathBuf};

use btree::Btree;
use input_handler::InputHandler;
use parsing::parse_command;
mod btree;
mod parsing;
pub struct IndexSession {
    btree: Btree,
}
impl IndexSession {
    fn new() -> Self {
        let filename = "data/btree.snap";
        let page_size = 4096;

        let btree =
            if PathBuf::from(filename).exists() && Btree::is_valid_snapshot(filename, page_size) {
                match Btree::load_snapshot(filename, page_size) {
                    Ok(bt) => bt,
                    Err(e) => {
                        eprintln!("Failed to load snapshot: {e}. Creating new B-tree.");
                        Btree::new(filename, page_size).expect("Failed to create Btree")
                    }
                }
            } else {
                Btree::new(filename, page_size).expect("Failed to create Btree")
            };
        IndexSession { btree }
    }
}
fn main() {
    let mut index_session = IndexSession::new();

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
        parse_command(&mut index_session, &line);
    }
}
