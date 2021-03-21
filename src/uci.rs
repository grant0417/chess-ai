use std::collections::HashMap;

use crate::bitboard::{
    perft_report, BitBoard, BitBoardMove, BitBoardState, BISHOP_PROMOTION,
    BISHOP_PROMOTION_CAPTURE, KNIGHT_PROMOTION, KNIGHT_PROMOTION_CAPTURE, QUEEN_PROMOTION,
    QUEEN_PROMOTION_CAPTURE, ROOK_PROMOTION, ROOK_PROMOTION_CAPTURE
};
use crate::interface::index_to_algebraic;
use crate::search::best_move;
use crate::{APPLICATION_AUTHOR, APPLICATION_NAME, APPLICATION_VERSION};
use std::str::from_utf8;

pub enum ResponseType {
    Response(String),
    Log(String),
    ResponseLog(String, String),
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
                ResponseType::Response(response)
            }
            ["quit"] => ResponseType::Quit,
            ["isready"] => ResponseType::Response(String::from("readyok")),
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
                        return ResponseType::Response(format!(
                            "Unable to construct board from FEN: {}",
                            e
                        ))
                    }
                };
                ResponseType::Response(format!("{:?}\n", moves))
            }
            ["position", "fen", fen] => {
                self.board = match BitBoardState::from_fen(fen) {
                    Ok(board) => board,
                    Err(e) => {
                        return ResponseType::Response(format!(
                            "Unable to construct board from FEN: {}",
                            e
                        ))
                    }
                };
                ResponseType::Nothing
            }
            ["position", "startpos", "moves", ref moves @ ..] => {
                self.board = BitBoardState::new();
                for m in moves {
                    self.board
                        .apply_move(&BitBoardMove::from_long_algebraic(m.as_bytes()).unwrap());
                    self.board.change_side();
                }
                ResponseType::Nothing
            }
            ["position", "startpos"] => {
                self.board = BitBoardState::new();
                ResponseType::Nothing
            }
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
                ResponseType::Response(perft_report(&self.board, depth.parse().unwrap()))
            }
            ["go", ..] => {
                let best_move = best_move(&self.board, 1);
                let from = index_to_algebraic(best_move.get_from() as usize);
                let to = index_to_algebraic(best_move.get_to() as usize);
                let promotion = match best_move.get_flags() {
                    KNIGHT_PROMOTION | KNIGHT_PROMOTION_CAPTURE => String::from("n"),
                    QUEEN_PROMOTION | QUEEN_PROMOTION_CAPTURE => String::from("q"),
                    BISHOP_PROMOTION | BISHOP_PROMOTION_CAPTURE => String::from("b"),
                    ROOK_PROMOTION | ROOK_PROMOTION_CAPTURE => String::from("r"),
                    _ => String::from(" "),
                };
                match (from_utf8(&from), from_utf8(&to)) {
                    (Ok(f), Ok(t)) => {
                        ResponseType::Response(format!("bestmove {}{}{}", f, t, promotion))
                    }
                    (_, _) => ResponseType::Response(format!(
                        "Unable to construct index {:?} {:?}",
                        from, to
                    )),
                }
            }
            _ => ResponseType::Response(format!("Unknown command: {}", command)),
        }
    }

    pub fn id(&self) -> String {
        format!(
            "id name {} {}\nid author {}\n",
            APPLICATION_NAME, APPLICATION_VERSION, APPLICATION_AUTHOR
        )
    }
}
