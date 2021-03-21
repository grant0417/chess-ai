use std::collections::HashMap;

use crate::bitboard::{BitBoard, BitBoardState, perft_report};
use crate::{APPLICATION_AUTHOR, APPLICATION_NAME, APPLICATION_VERSION};

pub enum ResponseType {
    Print(String),
    Nothing,
    Quit,
}

pub struct Options {
    hash: usize,
    log_file: Option<String>,
}

impl Options {
    fn new() -> Self {
        Self::default()
    }

    fn get_options(&self) -> String {
        String::from(
            "option name Hash type spin default 16 min 1 max 33554432\n\
             option name LogFile type string default \n",
        )
    }

    fn set_option<S: AsRef<str>>(&mut self, option: S, value: S) {
        match option.as_ref().trim().to_lowercase().as_str() {
            "hash" => {
                self.hash = value.as_ref().parse().unwrap();
            }
            "logfile" => {
                let file = value.as_ref().trim();
                self.log_file = if file.len() == 0 {
                    None
                } else {
                    Some(String::from(file))
                };
            }
            _ => {}
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            hash: 16,
            log_file: None,
        }
    }
}

pub struct UCIDriver {
    debug: bool,
    board: BitBoardState,
    options: Options,
}

impl UCIDriver {
    pub fn new() -> Self {
        Self {
            debug: false,
            board: BitBoardState::new(),
            options: Options::new(),
        }
    }

    pub fn parse_command(&mut self, command: &str) -> ResponseType {
        let command_vec: Vec<&str> = command.trim().split_whitespace().collect();

        match *command_vec {
            ["uci"] => {
                let mut response = String::new();
                response.push_str(&self.id());
                response.push_str(&self.options.get_options());
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
            ["position", "fen", fen] => {
                self.board = match BitBoardState::from_fen(fen) {
                    Ok(board) => board,
                    Err(e) => {
                        return ResponseType::Print(format!(
                            "Unable to construct board from FEN: {}",
                            e
                        ))
                    }
                };
                ResponseType::Nothing
            }
            ["position", "startpositon", "moves", ref _moves @ ..] => ResponseType::Nothing,
            ["setoption", "name", option, "value", value] => {
                self.options.set_option(option, value);
                ResponseType::Nothing
            }
            ["setoption", "name", option, "value"] => {
                self.options.set_option(option, "");
                ResponseType::Nothing
            }
            ["setoption", "name", option, value] => {
                self.options.set_option(option, value);
                ResponseType::Nothing
            }
            ["setoption", "name", option] => {
                self.options.set_option(option, "");
                ResponseType::Nothing
            }
            ["go", "perft", depth] => {
                ResponseType::Print(perft_report(&self.board, depth.parse().unwrap()))
            }
            ["go", ..] => ResponseType::Print(format!("bestmove e7e5")),
            _ => ResponseType::Print(format!("Unknown command: {}", command)),
        }
    }

    pub fn id(&self) -> String {
        format!(
            "id name {} {}\nid author {}\n",
            APPLICATION_NAME, APPLICATION_VERSION, APPLICATION_AUTHOR
        )
    }
}
