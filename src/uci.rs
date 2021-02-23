use crate::bitboard::{BitBoard, BitBoardState};

const APPLICATION_VERSION: &'static str = "0.0.1";
const APPLICATION_NAME: &'static str = "Grants's AI";
const APPLICATION_AUTHOR: &'static str = "Grant";

pub enum ResponseType {
    Print(String),
    Nothing,
    Quit,
}

pub struct Options {
    hash_table_size: usize,
}

pub struct UCIDriver {
    debug: bool,
    board: BitBoardState,
}

impl UCIDriver {
    pub fn new() -> Self {
        Self {
            debug: false,
            board: BitBoardState::new(),
        }
    }

    pub fn parse_command(&mut self, command: &str) -> ResponseType {
        let command_vec: Vec<&str> = command.trim().split_whitespace().collect();

        match *command_vec {
            ["uci"] => {
                let mut response = String::new();
                response.push_str(&self.id());
                response.push_str(&self.options());
                response.push_str("uciok");
                ResponseType::Print(response)
            }
            ["quit"] => ResponseType::Quit,
            ["isready"] => ResponseType::Print(String::from("readyok")),
            ["ucinewgame"] => ResponseType::Nothing,
            ["debug", "on"] => {
                self.debug = true;
                ResponseType::Nothing
            }
            ["debug", "off"] => {
                self.debug = false;
                ResponseType::Nothing
            }
            ["position", "fen", fen, "moves", ref moves @ ..] => {
                self.board = match BitBoardState::from_fen(fen) {
                    Ok(board) => board,
                    Err(e) => {
                        return ResponseType::Print(format!(
                            "Unable to construct board from FEN: {}",
                            e
                        ))
                    }
                };
                ResponseType::Print(format!("{:?}\n", moves))
            }
            ["position", "startpositon", "moves", ref _moves @ ..] => ResponseType::Nothing,
            ["setoption", "name", _id, "value", _x] => ResponseType::Nothing,
            ["go", ..] => ResponseType::Print(format!("bestmove e7e5")),
            _ => ResponseType::Print(format!("Unknown command: {}", command)),
        }
    }

    pub fn id(&self) -> String {
        format!(
            "id name {} {}\nid author {}\n",
            APPLICATION_NAME, APPLICATION_VERSION, APPLICATION_NAME
        )
    }

    pub fn options(&self) -> String {
        String::from("")
    }
}
