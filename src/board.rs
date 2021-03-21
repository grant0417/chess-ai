use crate::bitboard::{pop_lsb, BitBoard};
use crate::interface::algebraic_to_index;
use crate::move_gen::{generate_legal_moves, Move, MoveFlag, DIRECTION_OFFSETS};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::num::NonZeroU8;

pub const UNICODE_PIECES: &[&[char]] = &[
    &['♔', '♕', '♖', '♗', '♘', '♙'],
    &['♚', '♛', '♜', '♝', '♞', '♟'],
];

pub const ASCII_PIECES: &[&[char]] = &[
    &['K', 'Q', 'R', 'B', 'N', 'P'],
    &['k', 'q', 'r', 'b', 'n', 'p'],
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl TryFrom<usize> for Color {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Color::White),
            1 => Ok(Color::Black),
            _ => Err(String::from("Not possible")),
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => {
                write!(f, "W")
            }
            Color::Black => {
                write!(f, "B")
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Piece {
    King = 0b000,
    Queen = 0b001,
    Rook = 0b010,
    Bishop = 0b011,
    Knight = 0b100,
    Pawn = 0b101,
}

impl TryFrom<usize> for Piece {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Piece::King),
            1 => Ok(Piece::Queen),
            2 => Ok(Piece::Rook),
            3 => Ok(Piece::Bishop),
            4 => Ok(Piece::Knight),
            5 => Ok(Piece::Pawn),
            _ => Err(format!("Piece index out of range: {}", value)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoardMailbox(pub [Option<(Color, Piece)>; 64]);

impl From<BitBoard> for BoardMailbox {
    fn from(bitboard: BitBoard) -> Self {
        let mut bitboard = bitboard;
        let mut mailbox = BoardMailbox([None; 64]);
        for color in 0..2 {
            for piece in 0..6 {
                while let Some(i) = pop_lsb(&mut bitboard.0[color * 6 + piece]) {
                    mailbox.0[i as usize] = Some((
                        Color::try_from(color).unwrap(),
                        Piece::try_from(piece).unwrap(),
                    ))
                }
            }
        }
        mailbox
    }
}

#[derive(Clone, Debug)]
pub struct MoveHistory {
    piece_taken: Option<(Color, Piece)>,
    starting_location: u8,
    ending_location: u8,
    castling: u8,
    half_moves: u8,
    en_passant: Option<NonZeroU8>,
}

#[derive(Clone, Debug)]
pub struct Board {
    pub history: Vec<MoveHistory>,
    pub pieces: [Option<(Color, Piece)>; 64],
    pub bit_board: BitBoard,
    pub active_color: Color,
    pub castling: u8,
    pub en_passant: Option<NonZeroU8>,
    pub half_moves: usize,
    pub full_moves: usize,
}

impl Board {
    /// Reads a Forsyth–Edwards Notation string and outputs a board
    /// https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
    pub fn from_fen<S: AsRef<str>>(s: S) -> Result<Self, String> {
        let mut bit_board = [0; 12];
        let mut populated_board = [None; 64];
        let mut file = 0;
        let mut rank = 7;

        let mut fen_board = s.as_ref().split_ascii_whitespace();

        for c in fen_board
            .next()
            .ok_or(String::from("Unable to find board"))?
            .chars()
        {
            match c {
                '/' => {
                    file = 0;
                    rank -= 1;
                }
                c if c.is_ascii_digit() => {
                    file += c
                        .to_digit(10)
                        .ok_or(format!("Unable to convert digit: {}", c))?;
                }
                c if ASCII_PIECES[0].contains(&c.to_ascii_uppercase()) => {
                    let color = if c.is_ascii_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };

                    let piece = Piece::try_from(
                        ASCII_PIECES[0]
                            .iter()
                            .position(|&v| v == c.to_ascii_uppercase())
                            .unwrap(),
                    )?;

                    populated_board[(rank * 8 + file) as usize] = Some((color, piece));
                    bit_board[color as usize * 6 + piece as usize] |= 1 << (rank * 8 + file);
                    file += 1;
                }
                c => return Err(format!("Unexpected character: {}", c)),
            }
        }

        let active_color = match fen_board.next().unwrap_or("w").as_bytes() {
            b"w" => Color::White,
            b"b" => Color::Black,
            _ => Color::White,
        };

        // let mut castling = Castling::NONE;
        let _castling_str = fen_board.next().unwrap_or("KQkq");
        // if castling_str.contains("K") {
        //     castling.insert(Castling::WHITE_KNIGHT);
        // }
        // if castling_str.contains("Q") {
        //     castling.insert(Castling::WHITE_QUEEN);
        // }
        // if castling_str.contains("k") {
        //     castling.insert(Castling::BLACK_KNIGHT);
        // }
        // if castling_str.contains("q") {
        //     castling.insert(Castling::BLACK_QUEEN);
        // }

        let en_passant_str = fen_board.next().unwrap_or("-");
        let en_passant = algebraic_to_index(en_passant_str.as_bytes())
            .ok()
            .map(|v| NonZeroU8::new(v as u8))
            .flatten();

        let half_moves = fen_board.next().unwrap_or("0").parse::<usize>().unwrap();
        let full_moves = fen_board.next().unwrap_or("1").parse::<usize>().unwrap();

        Ok(Self {
            history: Vec::new(),
            pieces: populated_board,
            bit_board: BitBoard(bit_board),
            active_color,
            castling: 0,
            en_passant,
            half_moves,
            full_moves,
        })
    }

    pub fn move_piece(&mut self, m: &Move) {
        self.history.push(MoveHistory {
            piece_taken: self.pieces[m.end_index as usize],
            starting_location: m.start_index,
            ending_location: m.end_index,
            castling: self.castling,
            half_moves: self.half_moves as u8,
            en_passant: self.en_passant,
        });

        self.half_moves += 1;
        if let Some(_) = self.pieces[m.end_index as usize] {
            self.half_moves = 0;
        }

        let mut new_piece = self.pieces[m.start_index as usize];

        if let Some((color, Piece::Pawn)) = self.pieces[m.start_index as usize] {
            // En passant
            if let Some(en_passant) = self.en_passant {
                if en_passant.get() as usize == m.end_index as usize {
                    match color {
                        Color::White => self.pieces[m.end_index as usize - 8] = None,
                        Color::Black => self.pieces[m.end_index as usize + 8] = None,
                    }
                }
            }
            self.en_passant = None;
            if m.start_index as usize / 8 == 1 && m.end_index as usize / 8 == 3
                || m.start_index as usize / 8 == 6 && m.end_index as usize / 8 == 4
            {
                self.en_passant =
                    NonZeroU8::new((m.start_index as usize + m.end_index as usize) as u8 / 2);
            }

            // Promotion
            match m.flag {
                Some(MoveFlag::PromoteQueen) => {
                    new_piece = Some((color, Piece::Queen));
                }
                Some(MoveFlag::PromoteKnight) => {
                    new_piece = Some((color, Piece::Knight));
                }
                Some(MoveFlag::PromoteBishop) => {
                    new_piece = Some((color, Piece::Bishop));
                }
                Some(MoveFlag::PromoteRook) => {
                    new_piece = Some((color, Piece::Rook));
                }
                _ => {}
            }

            // Half move rule
            self.half_moves = 0;
        } else {
            self.en_passant = None;
        }

        if let Some((_king_color, Piece::King)) = self.pieces[m.start_index as usize] {
            // match m.flag {
            //     Some(MoveFlag::CastlingKnight) => {
            //         self.pieces[m.start_index as usize + 1] =
            //             self.pieces[m.start_index as usize + 3];
            //         self.pieces[m.start_index as usize + 3] = None;
            //         match king_color {
            //             Color::White => self.castling.remove(Castling::WHITE_KING),
            //             Color::Black => self.castling.remove(Castling::BLACK_KING),
            //         }
            //     }
            //     Some(MoveFlag::CastlingQueen) => {
            //         self.pieces[m.start_index as usize - 1] =
            //             self.pieces[m.start_index as usize - 4];
            //         self.pieces[m.start_index as usize - 4] = None;
            //         match king_color {
            //             Color::White => self.castling.remove(Castling::WHITE_KING),
            //             Color::Black => self.castling.remove(Castling::BLACK_KING),
            //         }
            //     }
            //     Some(MoveFlag::InitialMove) => match king_color {
            //         Color::White => self.castling.remove(Castling::WHITE_KING),
            //         Color::Black => self.castling.remove(Castling::BLACK_KING),
            //     },
            //     _ => {}
            // }
        }

        // Track castling legality
        // if let Some((Color::White, Piece::Rook)) = self.pieces[m.start_index as usize] {
        //     if m.start_index as usize == 0 {
        //         self.castling.remove(Castling::WHITE_QUEEN);
        //     }
        //     if m.start_index as usize == 7 {
        //         self.castling.remove(Castling::WHITE_KNIGHT);
        //     }
        // }
        // if let Some((Color::Black, Piece::Rook)) = self.pieces[m.start_index as usize] {
        //     if m.start_index as usize == 56 {
        //         self.castling.remove(Castling::BLACK_QUEEN);
        //     }
        //     if m.start_index as usize == 63 {
        //         self.castling.remove(Castling::BLACK_KNIGHT);
        //     }
        // }
        // if let Some((Color::White, Piece::King)) = self.pieces[m.start_index as usize] {
        //     self.castling.remove(Castling::WHITE_QUEEN);
        //     self.castling.remove(Castling::WHITE_KNIGHT);
        // }
        // if let Some((Color::Black, Piece::King)) = self.pieces[m.start_index as usize] {
        //     self.castling.remove(Castling::BLACK_QUEEN);
        //     self.castling.remove(Castling::BLACK_KNIGHT);
        // }

        self.pieces[m.end_index as usize] = new_piece;
        self.pieces[m.start_index as usize] = None;

        self.active_color = match self.active_color {
            Color::White => Color::Black,
            Color::Black => {
                self.full_moves += 1;
                Color::White
            }
        };
    }

    pub fn revert_last_move(&mut self, m: &Move) {
        match self.history.pop() {
            None => {}
            Some(past_state) => {
                let opponent_color = self.active_color;
                self.active_color = match self.active_color {
                    Color::White => {
                        self.full_moves -= 1;
                        Color::Black
                    }
                    Color::Black => Color::White,
                };

                self.en_passant = past_state.en_passant;
                self.castling = past_state.castling;
                self.half_moves = past_state.half_moves as usize;

                self.pieces[past_state.starting_location as usize] =
                    self.pieces[past_state.ending_location as usize];
                self.pieces[past_state.ending_location as usize] = past_state.piece_taken;

                match m.flag {
                    Some(MoveFlag::PromoteRook)
                    | Some(MoveFlag::PromoteBishop)
                    | Some(MoveFlag::PromoteKnight)
                    | Some(MoveFlag::PromoteQueen) => {
                        self.pieces[past_state.starting_location as usize] = self.pieces
                            [past_state.starting_location as usize]
                            .map(|(c, _)| (c, Piece::Pawn));
                    }
                    Some(MoveFlag::EnPassantCapture) => {
                        if let Some(pawn_location) = past_state.en_passant {
                            self.pieces[(pawn_location.get() as i32
                                + DIRECTION_OFFSETS[opponent_color as usize])
                                as usize] = Some((opponent_color, Piece::Pawn));
                        }
                    }
                    Some(MoveFlag::CastlingQueen) => {
                        self.pieces[m.start_index as usize - 4] =
                            self.pieces[m.start_index as usize - 1];
                        self.pieces[m.start_index as usize - 1] = None;
                    }
                    Some(MoveFlag::CastlingKnight) => {
                        self.pieces[m.start_index as usize + 3] =
                            self.pieces[m.start_index as usize + 1];
                        self.pieces[m.start_index as usize + 1] = None;
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::board::{Board, Color};
    use std::num::NonZeroU8;

    #[test]
    fn board_fen() {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
        let board1 =
            Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2")
                .unwrap();
        let board2 =
            Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2")
                .unwrap();
        let _board3 =
            Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w Kq c5 4 11")
                .unwrap();
        assert_eq!(board1.en_passant, NonZeroU8::new(42));
        assert_eq!(board1.half_moves, 0);
        assert_eq!(board1.active_color, Color::White);
        assert_eq!(board2.en_passant, None);
        assert_eq!(board2.half_moves, 1);
        assert_eq!(board2.full_moves, 2);
        assert_eq!(board2.active_color, Color::Black);
    }
}
