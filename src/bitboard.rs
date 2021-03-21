use crate::board::{BoardMailbox, Color, Piece, ASCII_PIECES, UNICODE_PIECES};
use crate::interface::{algebraic_to_index, index_to_algebraic, print_board};
use core::fmt;
use rayon::prelude::*;
use std::fmt::{Debug, Formatter};
use std::num::{NonZeroU64, NonZeroU8};
use std::str::from_utf8;
use std::{cmp::Ordering, convert::TryFrom};

// Board
const A_FILE: u64 = 0x0101010101010101;
const H_FILE: u64 = 0x8080808080808080;

const NOT_A_FILE: u64 = 0xfefefefefefefefe;
const NOT_B_FILE: u64 = 0xfdfdfdfdfdfdfdfd;
const NOT_G_FILE: u64 = 0xbfbfbfbfbfbfbfbf;
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

const NOT_AB_FILE: u64 = NOT_A_FILE & NOT_B_FILE;
const NOT_GH_FILE: u64 = NOT_G_FILE & NOT_H_FILE;

const RANK1: u64 = 0x00000000000000ff;
const RANK2: u64 = 0x000000000000ff00;
const RANK3: u64 = 0x0000000000ff0000;
const RANK4: u64 = 0x00000000ff000000;
const RANK5: u64 = 0x000000ff00000000;
const RANK6: u64 = 0x0000ff0000000000;
const RANK7: u64 = 0x00ff000000000000;
const RANK8: u64 = 0xff00000000000000;

const DIAGONAL: u64 = 0x8040201008040201;
const ANTIDIAGONAL: u64 = 0x0102040810204080;

const LIGHT_SQUARES: u64 = 0x55aa55aa55aa55aa;
const DARK_SQUARES: u64 = 0xaa55aa55aa55aa55;

// Move flags
const QUITE_MOVE: u16 = 0b0000;
const DOUBLE_PAWN_PUSH: u16 = 0b0001;
const KING_CASTLE: u16 = 0b0010;
const QUEEN_CASTLE: u16 = 0b0011;
const CAPTURE: u16 = 0b0100;
const EP_CAPTURE: u16 = 0b0101;

pub const KNIGHT_PROMOTION: u16 = 0b1000;
pub const BISHOP_PROMOTION: u16 = 0b1001;
pub const ROOK_PROMOTION: u16 = 0b1010;
pub const QUEEN_PROMOTION: u16 = 0b1011;

pub const KNIGHT_PROMOTION_CAPTURE: u16 = 0b1100;
pub const BISHOP_PROMOTION_CAPTURE: u16 = 0b1101;
pub const ROOK_PROMOTION_CAPTURE: u16 = 0b1110;
pub const QUEEN_PROMOTION_CAPTURE: u16 = 0b1111;

// Castling State
const WHITE_A_ROOK: u8 = 0b000001;
const WHITE_H_ROOK: u8 = 0b000010;
const WHITE_KING: u8 = 0b000100;
const BLACK_A_ROOK: u8 = 0b001000;
const BLACK_H_ROOK: u8 = 0b010000;
const BLACK_KING: u8 = 0b100000;
//const ALL: u8 = 0b111111;

const CASTLE_WHITE_KING: u8 = WHITE_KING | WHITE_H_ROOK;
const CASTLE_WHITE_QUEEEN: u8 = WHITE_KING | WHITE_A_ROOK;
const CASTLE_BLACK_KING: u8 = BLACK_KING | BLACK_H_ROOK;
const CASTLE_BLACK_QUEEN: u8 = BLACK_KING | BLACK_A_ROOK;

#[derive(Clone, Copy, Debug)]
enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
    NorthEast = 4,
    NorthWest = 5,
    SouthEast = 6,
    SouthWest = 7,
    NorthNorthEast = 8,
    NorthNorthWest = 9,
    SouthSouthEast = 10,
    SouthSouthWest = 11,
    NorthWestWest = 12,
    NorthEastEast = 13,
    SouthWestWest = 14,
    SouthEastEast = 15,
    None,
}

impl From<usize> for Direction {
    fn from(value: usize) -> Self {
        match value {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            3 => Direction::West,
            4 => Direction::NorthEast,
            5 => Direction::NorthWest,
            6 => Direction::SouthEast,
            7 => Direction::SouthWest,
            8 => Direction::NorthNorthEast,
            9 => Direction::NorthNorthWest,
            10 => Direction::SouthSouthEast,
            11 => Direction::SouthSouthWest,
            12 => Direction::NorthWestWest,
            13 => Direction::NorthEastEast,
            14 => Direction::SouthWestWest,
            15 => Direction::SouthEastEast,
            _ => Direction::None,
        }
    }
}

// Look up arrays
const KING_ATTACKS: [u64; 64] = {
    let mut attacks = [0; 64];
    let mut i = 0;
    let mut king = 1;
    while i < 64 {
        attacks[i] = knight_attacks2(king);
        i += 1;
        king = king << 1;
    }
    attacks
};

const KNIGHT_ATTACKS: [u64; 64] = {
    let mut attacks = [0; 64];
    let mut i = 0;
    let mut knight = 1;
    while i < 64 {
        attacks[i] = knight_attacks2(knight);
        i += 1;
        knight = knight << 1;
    }
    attacks
};

const RAY_ATTACKS: [[u64; 65]; 8] = {
    let mut attacks = [[0; 65]; 8];
    let mut i = 0;
    while i < 64 {
        attacks[Direction::North as usize][i] = fill_north(1 << i, !1) & !(1 << i);
        attacks[Direction::South as usize][i] = fill_south(1 << i, !1) & !(1 << i);
        attacks[Direction::East as usize][i] = fill_east(1 << i, !1) & !(1 << i);
        attacks[Direction::West as usize][i] = fill_west(1 << i, !1) & !(1 << i);
        attacks[Direction::NorthEast as usize][i] = fill_north_east(1 << i, !1) & !(1 << i);
        attacks[Direction::NorthWest as usize][i] = fill_north_west(1 << i, !1) & !(1 << i);
        attacks[Direction::SouthEast as usize][i] = fill_south_east(1 << i, !1) & !(1 << i);
        attacks[Direction::SouthWest as usize][i] = fill_south_west(1 << i, !1) & !(1 << i);
        i += 1;
    }
    attacks
};

// Structs
#[derive(Clone, Debug)]
pub struct BitBoard(pub [u64; 12]);

impl BitBoard {
    pub(crate) const fn new() -> Self {
        Self([0; 12])
    }

    const fn empty_squares(&self) -> u64 {
        !self.0[0]
            & !self.0[1]
            & !self.0[2]
            & !self.0[3]
            & !self.0[4]
            & !self.0[5]
            & !self.0[6]
            & !self.0[7]
            & !self.0[8]
            & !self.0[9]
            & !self.0[10]
            & !self.0[11]
    }

    const fn occupied_squares(&self) -> u64 {
        self.0[0]
            | self.0[1]
            | self.0[2]
            | self.0[3]
            | self.0[4]
            | self.0[5]
            | self.0[6]
            | self.0[7]
            | self.0[8]
            | self.0[9]
            | self.0[10]
            | self.0[11]
    }

    const fn white_pieces(&self) -> u64 {
        self.0[0] | self.0[1] | self.0[2] | self.0[3] | self.0[4] | self.0[5]
    }

    const fn black_pieces(&self) -> u64 {
        self.0[6] | self.0[7] | self.0[8] | self.0[9] | self.0[10] | self.0[11]
    }

    const fn color_pieces(&self, color: Color) -> u64 {
        self.0[color as usize * 6]
            | self.0[color as usize * 6 + 1]
            | self.0[color as usize * 6 + 2]
            | self.0[color as usize * 6 + 3]
            | self.0[color as usize * 6 + 4]
            | self.0[color as usize * 6 + 5]
    }

    const fn opposite_color_pieces(&self, color: Color) -> u64 {
        let c = !(color as usize) & 1;

        self.0[color as usize * 6]
            | self.0[c * 6 + 1]
            | self.0[c * 6 + 2]
            | self.0[c * 6 + 3]
            | self.0[c * 6 + 4]
            | self.0[c * 6 + 5]
    }

    fn set_piece(&mut self, index: usize, color: Color, piece: Piece) {
        self.0[color as usize * 6 + piece as usize] |= 1 << index;
    }

    fn clear_piece(&mut self, index: usize, color: Color, piece: Piece) {
        self.0[color as usize * 6 + piece as usize] &= !(1 << index);
    }

    pub const fn get_set(&self, color: Color, piece: Piece) -> u64 {
        self.0[color as usize * 6 + piece as usize]
    }

    pub const fn get_piece(&self, index: usize) -> Option<(Color, Piece)> {
        let mask = 1 << index;

        if (self.get_set(Color::Black, Piece::Pawn) & mask) != 0 {
            return Some((Color::Black, Piece::Pawn));
        }

        if (self.get_set(Color::Black, Piece::Rook) & mask) != 0 {
            return Some((Color::Black, Piece::Rook));
        }

        if (self.get_set(Color::Black, Piece::Bishop) & mask) != 0 {
            return Some((Color::Black, Piece::Bishop));
        }

        if (self.get_set(Color::Black, Piece::Knight) & mask) != 0 {
            return Some((Color::Black, Piece::Knight));
        }

        if (self.get_set(Color::Black, Piece::Queen) & mask) != 0 {
            return Some((Color::Black, Piece::Queen));
        }

        if (self.get_set(Color::Black, Piece::King) & mask) != 0 {
            return Some((Color::Black, Piece::King));
        }

        if (self.get_set(Color::White, Piece::Pawn) & mask) != 0 {
            return Some((Color::White, Piece::Pawn));
        }

        if (self.get_set(Color::White, Piece::Rook) & mask) != 0 {
            return Some((Color::White, Piece::Rook));
        }

        if (self.get_set(Color::White, Piece::Bishop) & mask) != 0 {
            return Some((Color::White, Piece::Bishop));
        }

        if (self.get_set(Color::White, Piece::Knight) & mask) != 0 {
            return Some((Color::White, Piece::Knight));
        }

        if (self.get_set(Color::White, Piece::Queen) & mask) != 0 {
            return Some((Color::White, Piece::Queen));
        }

        if (self.get_set(Color::White, Piece::King) & mask) != 0 {
            return Some((Color::White, Piece::King));
        }

        None
    }

    pub fn flip_board(&mut self) {
        for i in &mut self.0 {
            *i = i.swap_bytes();
        }
        self.0.swap(0, 6);
        self.0.swap(1, 7);
        self.0.swap(2, 8);
        self.0.swap(3, 9);
        self.0.swap(4, 10);
        self.0.swap(5, 11);
    }

    pub fn print_board(&self, index: Option<usize>, moves: Option<&Vec<BitBoardMove>>) {
        let mailbox = BoardMailbox::from(self.clone());
        for rank in (0..8).rev() {
            print!(" {} ", rank + 1);
            for file in 0..8 {
                let light_square = (file + rank) % 2 != 0;

                match index {
                    Some(i) if i == rank * 8 + file => {
                        print!("\x1B[48;2;239;80;80m");
                    }
                    Some(i) => match &moves {
                        Some(moves_vec) => {
                            if moves_vec.contains(&BitBoardMove::new(
                                i as u16,
                                (rank * 8 + file) as u16,
                                0,
                            )) {
                                print!("\x1B[48;2;239;80;80m");
                            } else {
                                if light_square {
                                    print!("\x1B[48;2;255;206;158m");
                                } else {
                                    print!("\x1B[48;2;209;139;71m");
                                }
                            }
                        }
                        None => {
                            if light_square {
                                print!("\x1B[48;2;255;206;158m");
                            } else {
                                print!("\x1B[48;2;209;139;71m");
                            }
                        }
                    },
                    None => {
                        if light_square {
                            print!("\x1B[48;2;255;206;158m");
                        } else {
                            print!("\x1B[48;2;209;139;71m");
                        }
                    }
                }

                // Set foreground to black
                print!("\x1B[38;2;0;0;0m");

                match mailbox.0[rank * 8 + file] {
                    Some((color, piece)) => {
                        print!(" {} ", UNICODE_PIECES[color as usize][piece as usize])
                    }
                    None => print!("   "),
                }
            }
            print!("\x1B[0m\n");
        }
        print!("  ");
        for i in 0..8 {
            print!("  {}", (b'a' + i) as char);
        }
        println!("\n");
    }
}

impl From<BoardMailbox> for BitBoard {
    fn from(mailbox: BoardMailbox) -> Self {
        let mut bitboard = BitBoard::new();
        mailbox.0.iter().enumerate().for_each(|(i, p)| {
            if let Some((color, piece)) = p {
                bitboard.set_piece(i, *color, *piece)
            }
        });
        bitboard
    }
}

#[derive(Clone, Debug)]
pub struct BitBoardState {
    pub bitboard: BitBoard,
    pub active_color: Color,
    pub castling: u8,
    pub en_passant: u8,
    pub half_moves: u8,
    pub full_moves: u16,
}

impl BitBoardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_fen<S: AsRef<str>>(s: S) -> Result<Self, String> {
        let mut bitboard = BitBoard::new();

        let mut file = 0;
        let mut rank = 7;

        let mut fen_board = s.as_ref().split_ascii_whitespace();

        for c in fen_board
            .next()
            .ok_or(String::from("FEN is empty"))?
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

                    bitboard.set_piece((rank * 8 + file) as usize, color, piece);
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

        let mut castling = 0;
        let castling_str = fen_board.next().unwrap_or("KQkq");
        if castling_str.contains("K") {
            castling |= CASTLE_WHITE_KING;
        }
        if castling_str.contains("Q") {
            castling |= CASTLE_WHITE_QUEEEN;
        }
        if castling_str.contains("k") {
            castling |= CASTLE_BLACK_KING;
        }
        if castling_str.contains("q") {
            castling |= CASTLE_BLACK_QUEEN;
        }

        let en_passant_str = fen_board.next().unwrap_or("-");
        let en_passant = algebraic_to_index(en_passant_str.as_bytes()).unwrap_or(64) as u8;

        let half_moves = fen_board.next().unwrap_or("0").parse::<u8>().unwrap();
        let full_moves = fen_board.next().unwrap_or("1").parse::<u16>().unwrap();

        Ok(BitBoardState {
            bitboard,
            active_color,
            castling,
            en_passant,
            half_moves,
            full_moves,
        })
    }

    pub fn mirror_board(&mut self) {
        self.bitboard.flip_board();
        let ep = 1u64.overflowing_shl(self.en_passant.into()).0;
        self.en_passant = (((ep & RANK3) << 24) | ((ep & RANK6) >> 24)).trailing_zeros() as u8;
        let lower = self.castling & 0b111;
        let upper = (self.castling & 0b111000) >> 3;
        self.castling = (lower << 3) | upper
    }

    pub fn change_side(&mut self) {
        self.active_color = match self.active_color {
            Color::White => Color::Black,
            Color::Black => {
                self.full_moves += 1;
                Color::White
            }
        };
        self.half_moves += 1;
    }

    pub fn apply_move(&mut self, m: &BitBoardMove) {
        let from_type = self.bitboard.get_piece(m.get_from() as usize);
        let to_type = self.bitboard.get_piece(m.get_to() as usize);

        if to_type == Some((Color::Black, Piece::Rook)) {
            if (1 << m.get_from()) & A_FILE != 0 {
                self.castling &= !BLACK_A_ROOK;
            }
            if (1 << m.get_from()) & H_FILE != 0 {
                self.castling &= !BLACK_H_ROOK;
            }
        }

        if let Some((color, piece)) = to_type {
            self.bitboard.clear_piece(m.get_to() as usize, color, piece);
        }

        self.en_passant = 64;

        if let Some((color, piece)) = from_type {
            match piece {
                Piece::King => {
                    self.castling &= !WHITE_KING;
                }
                Piece::Rook => {
                    if (1 << m.get_from()) & A_FILE != 0 {
                        self.castling &= !WHITE_A_ROOK;
                    }
                    if (1 << m.get_from()) & H_FILE != 0 {
                        self.castling &= !WHITE_H_ROOK;
                    }
                }
                _ => {}
            }

            let mut p = piece;
            match m.get_flags() & !CAPTURE {
                QUEEN_PROMOTION => p = Piece::Queen,
                KNIGHT_PROMOTION => p = Piece::Knight,
                ROOK_PROMOTION => p = Piece::Rook,
                BISHOP_PROMOTION => p = Piece::Bishop,
                DOUBLE_PAWN_PUSH => self.en_passant = (m.get_from() + 8) as u8,
                EP_CAPTURE => {
                    self.bitboard
                        .clear_piece((m.get_to() - 8).into(), Color::Black, Piece::Pawn)
                }
                QUEEN_CASTLE => {
                    self.bitboard.clear_piece(0, Color::White, Piece::Rook);
                    self.bitboard.set_piece(3, Color::White, Piece::Rook);
                }
                KING_CASTLE => {
                    self.bitboard.clear_piece(7, Color::White, Piece::Rook);
                    self.bitboard.set_piece(5, Color::White, Piece::Rook);
                }
                _ => {}
            }

            self.bitboard.set_piece(m.get_to() as usize, color, p);
            self.bitboard
                .clear_piece(m.get_from() as usize, color, piece);
        }
    }
}

impl Default for BitBoardState {
    fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

#[derive(Copy, Clone, Debug, Eq)]
pub struct BitBoardMove(u16);

impl BitBoardMove {
    pub const fn new(from: u16, to: u16, flags: u16) -> Self {
        BitBoardMove((to & 0x3f) | ((from & 0x3f) << 6) | ((flags & 0xf) << 12))
    }

    pub const fn from_long_algebraic(from: &[u8]) -> Result<Self, ()> {
        let (start, end) = if from.len() >= 4 {
            (
                match algebraic_to_index(&[from[0], from[1]]) {
                    Ok(s) => s as u16,
                    Err(_) => return Err(()),
                },
                match algebraic_to_index(&[from[2], from[3]]) {
                    Ok(e) => e as u16,
                    Err(_) => return Err(()),
                },
            )
        } else {
            return Err(());
        };

        let flags = if from.len() == 5 {
            match from[4] {
                b'q' | b'Q' => QUEEN_PROMOTION,
                b'n' | b'N' => KNIGHT_PROMOTION,
                b'b' | b'B' => BISHOP_PROMOTION,
                b'r' | b'R' => ROOK_PROMOTION,
                _ => 0,
            }
        } else {
            0
        };

        Ok(BitBoardMove::new(start, end, flags))
    }

    pub fn to_long_algebraic(&self) -> Result<String, String> {
        let mut algebric = String::with_capacity(5);

        let from = index_to_algebraic(self.get_from() as usize);
        algebric.push_str(from_utf8(&from).unwrap());

        let to = index_to_algebraic(self.get_to() as usize);
        algebric.push_str(from_utf8(&to).unwrap());

        match self.get_flags() {
            QUEEN_PROMOTION | QUEEN_PROMOTION_CAPTURE => algebric.push('q'),
            BISHOP_PROMOTION | BISHOP_PROMOTION_CAPTURE => algebric.push('b'),
            KNIGHT_PROMOTION | KNIGHT_PROMOTION_CAPTURE => algebric.push('n'),
            ROOK_PROMOTION | ROOK_PROMOTION_CAPTURE => algebric.push('r'),
            _ => {}
        };

        Ok(algebric)
    }

    pub const fn get_to(&self) -> u16 {
        self.0 & 0x003f
    }

    pub const fn get_from(&self) -> u16 {
        (self.0 >> 6) & 0x003f
    }

    pub const fn get_flags(&self) -> u16 {
        (self.0 >> 12) & 0x000f
    }

    pub fn set_to(&mut self, to: u16) {
        self.0 &= !0x3f;
        self.0 |= to & 0x3f;
    }

    pub fn set_from(&mut self, from: u16) {
        self.0 &= !(0x3f << 6);
        self.0 |= (from & 0x3f) << 6;
    }

    pub fn set_flags(&mut self, flags: u16) {
        self.0 &= !(0xf << 12);
        self.0 |= (flags & 0xf) << 12;
    }
}

impl PartialEq for BitBoardMove {
    fn eq(&self, other: &Self) -> bool {
        (self.0 & 0x0fff) == (other.0 & 0x0fff)
    }
}

pub struct BitBoardMoves {
    moves: [BitBoardMove; 256],
    head: usize,
    index: usize,
}

impl BitBoardMoves {
    fn new() -> BitBoardMoves {
        BitBoardMoves {
            moves: [BitBoardMove(0); 256],
            head: 0,
            index: 0,
        }
    }

    fn push(&mut self, m: BitBoardMove) {
        self.moves[self.head] = m;
        self.head += 1;
    }

    fn pop(&mut self) -> BitBoardMove {
        self.head -= 1;
        self.moves[self.head]
    }
}

impl Iterator for BitBoardMoves {
    type Item = BitBoardMove;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.head {
            let r = self.moves[self.index];
            self.index += 1;
            Some(r)
        } else {
            self.index = 0;
            None
        }
    }
}

pub fn generate_moves_bitboard(bitboard: &BitBoard) {
    let _empty = bitboard.empty_squares();
    let _white = bitboard.white_pieces();
    let _blacks = bitboard.black_pieces();

    let _king_attacks = king_attacks(bitboard.get_set(Color::White, Piece::King));
}

pub fn print_bitboard(board: u64) {
    let board = board.reverse_bits();
    for i in 0..8 {
        println!(
            "{}",
            format!("{:08b}", ((board >> (i * 8)) & 0xff)).replace("0", ".")
        );
    }
}

pub fn pop_lsb(b: &mut u64) -> Option<u32> {
    let trailing_zeros = b.trailing_zeros();
    let index = if trailing_zeros == 64 {
        None
    } else {
        Some(trailing_zeros)
    };

    *b = *b & !(1u64.overflowing_shl(trailing_zeros).0);

    index
}

pub const fn is_empty(b: u64) -> u64 {
    (((b as i64).overflowing_sub(1).0) >> 63) as u64
}

const fn south_one(b: u64) -> u64 {
    b.overflowing_shr(8).0
}

const fn north_one(b: u64) -> u64 {
    b.overflowing_shl(8).0
}

const fn east_one(b: u64) -> u64 {
    (b << 1) & NOT_A_FILE
}

const fn west_one(b: u64) -> u64 {
    (b >> 1) & NOT_H_FILE
}

const fn north_east_one(b: u64) -> u64 {
    (b << 9) & NOT_A_FILE
}

const fn north_west_one(b: u64) -> u64 {
    (b << 7) & NOT_H_FILE
}

const fn south_east_one(b: u64) -> u64 {
    (b >> 7) & NOT_A_FILE
}

const fn south_west_one(b: u64) -> u64 {
    (b >> 9) & NOT_H_FILE
}

const fn north_north_east(b: u64) -> u64 {
    (b << 17) & NOT_A_FILE
}

const fn north_east_east(b: u64) -> u64 {
    (b << 10) & NOT_AB_FILE
}

const fn south_east_east(b: u64) -> u64 {
    (b >> 6) & NOT_AB_FILE
}

const fn south_south_east(b: u64) -> u64 {
    (b >> 15) & NOT_A_FILE
}

const fn north_north_west(b: u64) -> u64 {
    (b << 15) & NOT_H_FILE
}

const fn north_west_west(b: u64) -> u64 {
    (b << 6) & NOT_GH_FILE
}

const fn south_west_west(b: u64) -> u64 {
    (b >> 10) & NOT_GH_FILE
}

const fn south_south_west(b: u64) -> u64 {
    (b >> 17) & NOT_H_FILE
}

// The fill algorithms are the Kogge-Stone Algorithm
//https://www.chessprogramming.org/Kogge-Stone_Algorithm

const fn fill_north(rook: u64, empty: u64) -> u64 {
    let mut rook = rook;
    let mut empty = empty;

    rook |= empty & (rook << 8);
    empty &= empty << 8;
    rook |= empty & (rook << 16);
    empty &= empty << 16;
    rook |= empty & (rook << 32);
    return rook;
}

const fn fill_south(rook: u64, empty: u64) -> u64 {
    let mut empty = empty;
    let mut rook = rook;

    rook |= empty & (rook >> 8);
    empty &= empty >> 8;
    rook |= empty & (rook >> 16);
    empty &= empty >> 16;
    rook |= empty & (rook >> 32);
    return rook;
}

const fn fill_east(rook: u64, empty: u64) -> u64 {
    let mut rook = rook;
    let mut empty = empty & NOT_A_FILE;

    rook |= empty & (rook << 1);
    empty &= empty << 1;
    rook |= empty & (rook << 2);
    empty &= empty << 2;
    rook |= empty & (rook << 4);
    return rook;
}

const fn fill_west(rook: u64, empty: u64) -> u64 {
    let mut rook = rook;
    let mut empty = empty & NOT_H_FILE;

    rook |= empty & (rook >> 1);
    empty &= empty >> 1;
    rook |= empty & (rook >> 2);
    empty &= empty >> 2;
    rook |= empty & (rook >> 4);
    return rook;
}

const fn fill_north_east(bishop: u64, empty: u64) -> u64 {
    let mut bishop = bishop;
    let mut empty = empty & NOT_A_FILE;

    bishop |= empty & (bishop << 9);
    empty &= empty << 9;
    bishop |= empty & (bishop << 18);
    empty &= empty << 18;
    bishop |= empty & (bishop << 36);
    return bishop;
}

const fn fill_south_east(bishop: u64, empty: u64) -> u64 {
    let mut bishop = bishop;
    let mut empty = empty & NOT_A_FILE;

    bishop |= empty & (bishop >> 7);
    empty &= empty >> 7;
    bishop |= empty & (bishop >> 14);
    empty &= empty >> 14;
    bishop |= empty & (bishop >> 28);
    return bishop;
}

const fn fill_north_west(bishop: u64, empty: u64) -> u64 {
    let mut bishop = bishop;
    let mut empty = empty & NOT_H_FILE;

    bishop |= empty & (bishop << 7);
    empty &= empty << 7;
    bishop |= empty & (bishop << 14);
    empty &= empty << 14;
    bishop |= empty & (bishop << 28);
    return bishop;
}

const fn fill_south_west(bishop: u64, empty: u64) -> u64 {
    let mut bishop = bishop;
    let mut empty = empty & NOT_H_FILE;

    bishop |= empty & (bishop >> 9);
    empty &= empty >> 9;
    bishop |= empty & (bishop >> 18);
    empty &= empty >> 18;
    bishop |= empty & (bishop >> 36);
    return bishop;
}

const fn attack_north(rook: u64, empty: u64) -> u64 {
    north_one(fill_north(rook, empty))
}

const fn attack_south(rook: u64, empty: u64) -> u64 {
    south_one(fill_south(rook, empty))
}

const fn attack_east(rook: u64, empty: u64) -> u64 {
    east_one(fill_east(rook, empty))
}

const fn attack_west(rook: u64, empty: u64) -> u64 {
    west_one(fill_west(rook, empty))
}

const fn attack_north_east(bishop: u64, empty: u64) -> u64 {
    north_east_one(fill_north_east(bishop, empty))
}

const fn attack_north_west(bishop: u64, empty: u64) -> u64 {
    north_west_one(fill_north_west(bishop, empty))
}
const fn attack_south_east(bishop: u64, empty: u64) -> u64 {
    south_east_one(fill_south_east(bishop, empty))
}
const fn attack_south_west(bishop: u64, empty: u64) -> u64 {
    south_west_one(fill_south_west(bishop, empty))
}

const fn get_ray_attacks(index: usize, occupied: u64, direction: Direction) -> u64 {
    let attacks = RAY_ATTACKS[direction as usize][index];
    let blockers = attacks & occupied;
    let lowest_bit = blockers.trailing_zeros();
    attacks ^ RAY_ATTACKS[direction as usize][lowest_bit as usize]
}

const fn get_negative_ray_attacks(index: usize, occupied: u64, direction: Direction) -> u64 {
    let attacks = RAY_ATTACKS[direction as usize][index];
    let blockers = attacks & occupied;
    let highest_bit = match 63u32.overflowing_sub(blockers.leading_zeros()).0 {
        i if i < 64 => i,
        _ => 64,
    };
    attacks ^ RAY_ATTACKS[direction as usize][highest_bit as usize]
}

const fn white_single_push_targets(pawns: u64, empty: u64) -> u64 {
    north_one(pawns) & empty
}

const fn black_single_push_targets(pawns: u64, empty: u64) -> u64 {
    south_one(pawns) & empty
}

const fn white_double_push_targets(pawns: u64, empty: u64) -> u64 {
    let single_push = white_single_push_targets(pawns, empty);
    north_one(single_push) & empty & RANK4
}

const fn black_double_push_targets(pawns: u64, empty: u64) -> u64 {
    let single_push = black_single_push_targets(pawns, empty);
    south_one(single_push) & empty & RANK5
}

const fn white_pawns_able_push(pawns: u64, empty: u64) -> u64 {
    south_one(empty) & pawns
}

const fn white_pawns_able_double_push(pawns: u64, empty: u64) -> u64 {
    let empty_rank_3 = south_one(empty & RANK4) & empty;
    white_pawns_able_double_push(pawns, empty_rank_3)
}

const fn black_pawns_able_push(pawns: u64, empty: u64) -> u64 {
    north_one(empty) & pawns
}

const fn black_pawns_able_double_push(pawns: u64, empty: u64) -> u64 {
    let empty_rank_4 = south_one(empty & RANK4) & empty;
    black_pawns_able_double_push(pawns, empty_rank_4)
}

const fn white_pawn_east_attacks(pawns: u64, blacks: u64) -> u64 {
    north_east_one(pawns) & blacks
}

const fn white_pawn_west_attacks(pawns: u64, blacks: u64) -> u64 {
    north_west_one(pawns) & blacks
}

const fn black_pawn_east_attacks(pawns: u64, whites: u64) -> u64 {
    south_east_one(pawns) & whites
}

const fn black_pawn_west_attacks(pawns: u64, whites: u64) -> u64 {
    south_west_one(pawns) & whites
}

const fn king_attacks(king: u64) -> u64 {
    let attacks = east_one(king) | west_one(king);
    let row = attacks | king;
    attacks | north_one(row) | south_one(row)
}

const fn knight_attacks(knight: u64) -> u64 {
    north_north_east(knight)
        | north_east_east(knight)
        | south_east_east(knight)
        | south_south_east(knight)
        | north_north_west(knight)
        | north_west_west(knight)
        | south_west_west(knight)
        | south_south_west(knight)
}

const fn knight_attacks2(knights: u64) -> u64 {
    let l1 = (knights >> 1) & NOT_H_FILE;
    let l2 = (knights >> 2) & NOT_GH_FILE;
    let r1 = (knights << 1) & NOT_A_FILE;
    let r2 = (knights << 2) & NOT_AB_FILE;
    let h1 = l1 | r1;
    let h2 = l2 | r2;
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
}

fn move_targets(state: &BitBoardState, color: Color) -> [u64; 16] {
    let opposite_color = match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    let mut any_attacks = 0;

    let mut hor_inbetween = 0;
    let mut ver_inbetween = 0;
    let mut dia_inbetween = 0;
    let mut ant_inbetween = 0;

    let mut hor_inbetween_ep = 0;
    let mut dia_inbetween_ep = 0;
    let mut ant_inbetween_ep = 0;

    let mut w_ksuper_attacks_orth = 0;
    let mut w_ksuper_attacks_dia = 0;

    let empty = state.bitboard.empty_squares();
    let occupied = !empty;
    let our_king = state.bitboard.get_set(color, Piece::King);
    let our_king_index = our_king.trailing_zeros() as usize;
    let empty_and_our_king = !(occupied ^ our_king);
    let empty_and_our_king_and_ep = !(occupied
        ^ our_king
        ^ ((1u64.overflowing_shl(state.en_passant as u32).0 & (RANK6 | RANK3)) >> 8));

    let our_pieces = state.bitboard.color_pieces(color);
    let their_pieces = state.bitboard.color_pieces(opposite_color);

    let orthogonal_set = state.bitboard.get_set(opposite_color, Piece::Rook)
        | state.bitboard.get_set(opposite_color, Piece::Queen);
    let diagonal_set = state.bitboard.get_set(opposite_color, Piece::Bishop)
        | state.bitboard.get_set(opposite_color, Piece::Queen);

    // black rooks and queens west
    let attacks = attack_west(orthogonal_set, empty_and_our_king);
    let ep_attacks = attack_west(orthogonal_set, empty_and_our_king_and_ep);
    any_attacks |= attacks;
    let super_attacks = get_ray_attacks(our_king_index, occupied, Direction::East);
    w_ksuper_attacks_orth |= super_attacks;
    hor_inbetween |= attacks & super_attacks;
    hor_inbetween_ep |= ep_attacks & super_attacks;

    // black rooks and queens east
    let attacks = attack_east(orthogonal_set, empty_and_our_king);
    let ep_attacks = attack_east(orthogonal_set, empty_and_our_king_and_ep);
    any_attacks |= attacks;
    let super_attacks = get_negative_ray_attacks(our_king_index, occupied, Direction::West);
    w_ksuper_attacks_orth |= super_attacks;
    hor_inbetween |= attacks & super_attacks;
    hor_inbetween_ep |= ep_attacks & super_attacks;

    // black rooks and queens north
    let attacks = attack_north(orthogonal_set, empty_and_our_king);
    any_attacks |= attacks;
    let super_attacks = get_negative_ray_attacks(our_king_index, occupied, Direction::South);
    w_ksuper_attacks_orth |= super_attacks;
    ver_inbetween |= attacks & super_attacks;

    // black rooks and queens south
    let attacks = attack_south(orthogonal_set, empty_and_our_king);
    any_attacks |= attacks;
    let super_attacks = get_ray_attacks(our_king_index, occupied, Direction::North);
    w_ksuper_attacks_orth |= super_attacks;
    ver_inbetween |= attacks & super_attacks;

    // black bishops and queens north east
    let attacks = attack_north_east(diagonal_set, empty_and_our_king);
    let ep_attacks = attack_north_east(diagonal_set, empty_and_our_king_and_ep);
    any_attacks |= attacks;
    let super_attacks = get_negative_ray_attacks(our_king_index, occupied, Direction::SouthWest);
    w_ksuper_attacks_dia |= super_attacks;
    dia_inbetween |= attacks & super_attacks;
    dia_inbetween_ep |= ep_attacks;

    // black bishops and queens south west
    let attacks = attack_south_west(diagonal_set, empty_and_our_king);
    let ep_attacks = attack_south_west(diagonal_set, empty_and_our_king_and_ep);
    any_attacks |= attacks;
    let super_attacks = get_ray_attacks(our_king_index, occupied, Direction::NorthEast);
    w_ksuper_attacks_dia |= super_attacks;
    dia_inbetween |= attacks & super_attacks;
    dia_inbetween_ep |= ep_attacks;

    // black bishops and queens north west
    let attacks = attack_north_west(diagonal_set, empty_and_our_king);
    let ep_attacks = attack_north_west(diagonal_set, empty_and_our_king_and_ep);
    any_attacks |= attacks;
    let super_attacks = get_negative_ray_attacks(our_king_index, occupied, Direction::SouthEast);
    w_ksuper_attacks_dia |= super_attacks;
    ant_inbetween |= attacks & super_attacks;
    ant_inbetween_ep |= ep_attacks;

    // black bishops and queens south east
    let attacks = attack_south_east(diagonal_set, empty_and_our_king);
    let ep_attacks = attack_south_east(diagonal_set, empty_and_our_king_and_ep);
    any_attacks |= attacks;
    let super_attacks = get_ray_attacks(our_king_index, occupied, Direction::NorthWest);
    w_ksuper_attacks_dia |= super_attacks;
    ant_inbetween |= attacks & super_attacks;
    ant_inbetween_ep |= ep_attacks;

    // black knights
    any_attacks |= knight_attacks(state.bitboard.get_set(opposite_color, Piece::Knight));
    // black pawns
    any_attacks |= south_east_one(state.bitboard.get_set(opposite_color, Piece::Pawn));
    any_attacks |= south_west_one(state.bitboard.get_set(opposite_color, Piece::Pawn));
    // black king
    any_attacks |= king_attacks(state.bitboard.get_set(opposite_color, Piece::King));

    let en_passant_pawn = (1u64.overflowing_shl(state.en_passant as u32).0 & RANK6) >> 8;
    let _en_passant_attacking_pawns = (east_one(en_passant_pawn) | west_one(en_passant_pawn))
        & state.bitboard.get_set(color, Piece::Pawn);

    // Check for check
    let all_inbetween = hor_inbetween | ver_inbetween | dia_inbetween | ant_inbetween;
    let all_inbetween_ep = hor_inbetween_ep | dia_inbetween_ep | ant_inbetween_ep;
    let blocks = all_inbetween & !occupied;
    let check_from = (w_ksuper_attacks_orth & orthogonal_set)
        | (w_ksuper_attacks_dia & diagonal_set)
        | (knight_attacks(our_king) & state.bitboard.get_set(opposite_color, Piece::Knight))
        | ((north_east_one(our_king) | north_west_one(our_king))
            & state.bitboard.get_set(opposite_color, Piece::Pawn));

    let null_if_check = is_empty(any_attacks & our_king); /* signed shifts */
    let null_if_dbl_check = is_empty(check_from & (check_from.overflowing_sub(1).0));

    let check_to = check_from | blocks | null_if_check;
    let target_mask = !our_pieces & check_to & null_if_dbl_check;

    //state.bitboard.print_board(None, None);
    //print_bitboard(state.bitboard.get_set(Color::Black, Piece::Bishop) & state.bitboard.color_pieces(Color::White));

    // Valid moves
    let mut move_targets = [0; 16];

    let orthogonal_set =
        state.bitboard.get_set(color, Piece::Rook) | state.bitboard.get_set(color, Piece::Queen);
    let diagonal_set =
        state.bitboard.get_set(color, Piece::Bishop) | state.bitboard.get_set(color, Piece::Queen);

    // horizontal rook and queen moves
    let sliders = orthogonal_set & !(all_inbetween ^ hor_inbetween);
    move_targets[Direction::East as usize] = attack_east(sliders, empty) & target_mask;
    move_targets[Direction::West as usize] = attack_west(sliders, empty) & target_mask;

    // horizontal rook and queen moves
    let sliders = orthogonal_set & !(all_inbetween ^ ver_inbetween);
    move_targets[Direction::North as usize] = attack_north(sliders, empty) & target_mask;
    move_targets[Direction::South as usize] = attack_south(sliders, empty) & target_mask;

    // diagonal bishop and queen moves
    let sliders = diagonal_set & !(all_inbetween ^ dia_inbetween);
    move_targets[Direction::NorthEast as usize] = attack_north_east(sliders, empty) & target_mask;
    move_targets[Direction::SouthWest as usize] = attack_south_west(sliders, empty) & target_mask;
    // antidiagonal bishop and queen moves
    let sliders = diagonal_set & !(all_inbetween ^ ant_inbetween);
    move_targets[Direction::NorthWest as usize] = attack_north_west(sliders, empty) & target_mask;
    move_targets[Direction::SouthEast as usize] = attack_south_east(sliders, empty) & target_mask;

    // knight moves
    let knights = state.bitboard.get_set(color, Piece::Knight) & !all_inbetween;
    move_targets[Direction::NorthNorthEast as usize] = north_north_east(knights) & target_mask;
    move_targets[Direction::NorthEastEast as usize] = north_east_east(knights) & target_mask;
    move_targets[Direction::SouthEastEast as usize] = south_east_east(knights) & target_mask;
    move_targets[Direction::SouthSouthEast as usize] = south_south_east(knights) & target_mask;
    move_targets[Direction::NorthNorthWest as usize] = north_north_west(knights) & target_mask;
    move_targets[Direction::NorthWestWest as usize] = north_west_west(knights) & target_mask;
    move_targets[Direction::SouthWestWest as usize] = south_west_west(knights) & target_mask;
    move_targets[Direction::SouthSouthWest as usize] = south_south_west(knights) & target_mask;

    // pawn captures and en passant
    let targets = their_pieces & target_mask;
    let pawns = state.bitboard.get_set(color, Piece::Pawn) & !(all_inbetween ^ dia_inbetween);
    move_targets[Direction::NorthEast as usize] |= north_east_one(pawns) & targets;
    let pawns = state.bitboard.get_set(color, Piece::Pawn) & !(all_inbetween ^ ant_inbetween);
    move_targets[Direction::NorthWest as usize] |= north_west_one(pawns) & targets;

    let ep_target = 1u64.overflowing_shl(state.en_passant as u32).0;

    let pawns = state.bitboard.get_set(color, Piece::Pawn)
        & !(all_inbetween ^ dia_inbetween)
        & !all_inbetween_ep;
    move_targets[Direction::NorthEast as usize] |= north_east_one(pawns) & ep_target;
    let pawns = state.bitboard.get_set(color, Piece::Pawn)
        & !(all_inbetween ^ ant_inbetween)
        & !all_inbetween_ep;
    move_targets[Direction::NorthWest as usize] |= north_west_one(pawns) & ep_target;

    // pawn pushes
    let pawns = state.bitboard.get_set(color, Piece::Pawn) & !(all_inbetween ^ ver_inbetween);
    let pawn_pushes = north_one(pawns) & !occupied;
    move_targets[Direction::North as usize] |= pawn_pushes & target_mask;
    // and double pushs
    move_targets[Direction::North as usize] |=
        north_one(pawn_pushes) & !occupied & target_mask & RANK4;

    /* king moves */
    let target_mask = !(our_pieces | any_attacks);
    move_targets[Direction::West as usize] |= west_one(our_king) & target_mask;
    move_targets[Direction::East as usize] |= east_one(our_king) & target_mask;
    move_targets[Direction::North as usize] |= north_one(our_king) & target_mask;
    move_targets[Direction::South as usize] |= south_one(our_king) & target_mask;
    move_targets[Direction::NorthEast as usize] |= north_east_one(our_king) & target_mask;
    move_targets[Direction::SouthWest as usize] |= south_west_one(our_king) & target_mask;
    move_targets[Direction::NorthWest as usize] |= north_west_one(our_king) & target_mask;
    move_targets[Direction::SouthEast as usize] |= south_east_one(our_king) & target_mask;

    // Left Castle
    let target_mask = !(occupied | any_attacks);
    let castling_rights = !((((state.castling & WHITE_KING) as i64 - 1) >> 63) as u64);
    let check_clear = !((((west_one(our_king) & target_mask) as i64 - 1) >> 63) as u64);
    let nothing_1 = !is_empty(west_one(west_one(our_king)) & target_mask);
    let nothing_2 = !is_empty(west_one(west_one(west_one(our_king))) & target_mask);
    move_targets[Direction::West as usize] |= (west_one(west_one(our_king)) & target_mask)
        & castling_rights
        & check_clear
        & nothing_1
        & nothing_2;

    // Right Castle
    let castling_rights = !((((state.castling & CASTLE_WHITE_KING) as i64 - 1) >> 63) as u64);
    let check_clear = !is_empty((east_one(our_king) & target_mask) as u64);
    let nothing = !is_empty(east_one(east_one(our_king)) & target_mask);
    move_targets[Direction::East as usize] |=
        (east_one(east_one(our_king)) & target_mask) & castling_rights & check_clear & nothing;

    move_targets
}

pub fn generate_moves(state: &BitBoardState) -> Vec<BitBoardMove> {
    let color = state.active_color;
    let state = match color {
        Color::White => state.clone(),
        Color::Black => {
            let mut s = state.clone();
            s.mirror_board();
            s.change_side();
            s
        }
    };

    let mut moves = Vec::with_capacity(256);

    let mut move_targets = move_targets(&state, Color::White);

    // for i in &move_targets {
    //     print_bitboard(*i);
    //     println!();
    // }
    let occupied = state.bitboard.occupied_squares();
    let pawns = state.bitboard.get_set(Color::White, Piece::Pawn);

    while move_targets[Direction::North as usize] != 0 {
        let mut target_square =
            63u32.saturating_sub(move_targets[Direction::North as usize].leading_zeros());
        let source_square = 63u32.saturating_sub(
            (RAY_ATTACKS[Direction::South as usize][target_square as usize]
                & occupied
                & !(1 << target_square))
                .leading_zeros(),
        );

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let pawn = (1 << source_square) & pawns;
        let double_pawn_push = !is_empty((pawn << 16) & (1 << target_square)) as u16;
        let promotion = !is_empty((pawn << 8) & RANK8) as u16;
        let flags = (capture & CAPTURE) | (double_pawn_push & DOUBLE_PAWN_PUSH);

        if promotion == 0 {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags,
            ));
        } else {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | QUEEN_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | ROOK_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | BISHOP_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | KNIGHT_PROMOTION,
            ));
        }

        move_targets[Direction::North as usize] &= !(1u64 << target_square);
        target_square -= 8;
        while target_square > source_square {
            let bit =
                (move_targets[Direction::North as usize] & (1 << target_square)) >> target_square;
            move_targets[Direction::North as usize] &= !(1u64 << target_square);
            if bit != 0 {
                let capture = !is_empty((1 << target_square) & occupied) as u16;
                let double_pawn_push = !is_empty((pawn << 16) & (1 << target_square)) as u16;
                let flags = (capture & CAPTURE) | (double_pawn_push & DOUBLE_PAWN_PUSH);

                moves.push(BitBoardMove::new(
                    source_square as u16,
                    target_square as u16,
                    flags,
                ));
            }
            target_square -= 8;
        }
    }

    while move_targets[Direction::South as usize] != 0 {
        let mut target_square = move_targets[Direction::South as usize].trailing_zeros();
        let source_square = (RAY_ATTACKS[Direction::North as usize][target_square as usize]
            & occupied
            & !(1 << target_square))
            .trailing_zeros();

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let pawn = (1 << source_square) & pawns;
        let double_pawn_push = !is_empty((pawn >> 16) & (1 << target_square)) as u16;
        let promotion = !is_empty((pawn >> 8) & RANK1) as u16;
        let flags = (capture & CAPTURE) | (double_pawn_push & DOUBLE_PAWN_PUSH);

        if promotion == 0 {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags,
            ));
        } else {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | QUEEN_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | ROOK_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | BISHOP_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | KNIGHT_PROMOTION,
            ));
        }

        move_targets[Direction::South as usize] &= !(1u64 << target_square);
        target_square += 8;
        while target_square < source_square {
            let bit =
                (move_targets[Direction::South as usize] & (1 << target_square)) >> target_square;
            move_targets[Direction::South as usize] &= !(1u64 << target_square);
            if bit != 0 {
                let capture = !is_empty((1 << target_square) & occupied) as u16;
                let flags = capture & CAPTURE;

                moves.push(BitBoardMove::new(
                    source_square as u16,
                    target_square as u16,
                    flags,
                ));
            }
            target_square += 8;
        }
    }

    while move_targets[Direction::East as usize] != 0 {
        let mut target_square =
            63u32.saturating_sub(move_targets[Direction::East as usize].leading_zeros());
        let source_square = 63u32.saturating_sub(
            (RAY_ATTACKS[Direction::West as usize][target_square as usize]
                & occupied
                & !(1 << target_square))
                .leading_zeros(),
        );

        //TODO: CASTLING
        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let mut flags = capture & CAPTURE;

        if let Some((_, Piece::King)) = state.bitboard.get_piece(source_square as usize) {
            flags |= QUEEN_CASTLE
        }

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::East as usize] &= !(1u64 << target_square);
        target_square -= 1;
        while target_square > source_square {
            let bit =
                (move_targets[Direction::East as usize] & (1 << target_square)) >> target_square;
            move_targets[Direction::East as usize] &= !(1u64 << target_square);

            let capture = !is_empty((1 << target_square) & occupied) as u16;
            let flags = capture & CAPTURE;

            if bit != 0 {
                moves.push(BitBoardMove::new(
                    source_square as u16,
                    target_square as u16,
                    flags,
                ));
            }
            target_square -= 1;
        }
    }

    while move_targets[Direction::West as usize] != 0 {
        let mut target_square = move_targets[Direction::West as usize].trailing_zeros();
        let source_square = (RAY_ATTACKS[Direction::East as usize][target_square as usize]
            & occupied
            & !(1 << target_square))
            .trailing_zeros();

        //TODO: Castling
        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::West as usize] &= !(1u64 << target_square);
        target_square += 1;
        while target_square < source_square {
            let bit =
                (move_targets[Direction::West as usize] & (1 << target_square)) >> target_square;
            move_targets[Direction::West as usize] &= !(1u64 << target_square);
            if bit != 0 {
                let capture = !is_empty((1 << target_square) & occupied) as u16;
                let mut flags = capture & CAPTURE;

                if let Some((_, Piece::King)) = state.bitboard.get_piece(source_square as usize) {
                    flags |= KING_CASTLE
                }

                moves.push(BitBoardMove::new(
                    source_square as u16,
                    target_square as u16,
                    flags,
                ));
            }
            target_square += 1;
        }
    }

    while move_targets[Direction::NorthEast as usize] != 0 {
        let mut target_square =
            63u32.saturating_sub(move_targets[Direction::NorthEast as usize].leading_zeros());
        let source_square = 63u32.saturating_sub(
            (RAY_ATTACKS[Direction::SouthWest as usize][target_square as usize]
                & occupied
                & !(1 << target_square))
                .leading_zeros(),
        );

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        let pawn = (1 << source_square) & pawns;
        let promotion = !is_empty((pawn << 8) & RANK8) as u16;

        if promotion == 0 {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags,
            ));
        } else {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | QUEEN_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | ROOK_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | BISHOP_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | KNIGHT_PROMOTION,
            ));
        }

        move_targets[Direction::NorthEast as usize] &= !(1u64 << target_square);
        target_square -= 9;
        while target_square > source_square {
            let bit = (move_targets[Direction::NorthEast as usize] & (1 << target_square))
                >> target_square;
            move_targets[Direction::NorthEast as usize] &= !(1u64 << target_square);
            if bit != 0 {
                let capture = !is_empty((1 << target_square) & occupied) as u16;
                let flags = capture & CAPTURE;

                moves.push(BitBoardMove::new(
                    source_square as u16,
                    target_square as u16,
                    flags,
                ));
            }
            target_square -= 9;
        }
    }

    while move_targets[Direction::NorthWest as usize] != 0 {
        let mut target_square =
            63u32.saturating_sub(move_targets[Direction::NorthWest as usize].leading_zeros());
        let source_square = 63u32.saturating_sub(
            (RAY_ATTACKS[Direction::SouthEast as usize][target_square as usize]
                & occupied
                & !(1 << target_square))
                .leading_zeros(),
        );

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        let pawn = (1 << source_square) & pawns;
        let promotion = !is_empty((pawn << 8) & RANK8) as u16;

        if promotion == 0 {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags,
            ));
        } else {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | QUEEN_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | ROOK_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | BISHOP_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | KNIGHT_PROMOTION,
            ));
        }

        move_targets[Direction::NorthWest as usize] &= !(1u64 << target_square);
        target_square -= 7;
        while target_square > source_square {
            let bit = (move_targets[Direction::NorthWest as usize] & (1 << target_square))
                >> target_square;
            move_targets[Direction::NorthWest as usize] &= !(1u64 << target_square);
            if bit != 0 {
                let capture = !is_empty((1 << target_square) & occupied) as u16;
                let flags = capture & CAPTURE;

                moves.push(BitBoardMove::new(
                    source_square as u16,
                    target_square as u16,
                    flags,
                ));
            }
            target_square -= 7;
        }
    }

    while move_targets[Direction::SouthWest as usize] != 0 {
        let mut target_square = move_targets[Direction::SouthWest as usize].trailing_zeros();
        let source_square = (RAY_ATTACKS[Direction::NorthEast as usize][target_square as usize]
            & occupied
            & !(1 << target_square))
            .trailing_zeros();

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        let pawn = (1 << source_square) & pawns;
        let promotion = !is_empty((pawn >> 8) & RANK1) as u16;

        if promotion == 0 {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags,
            ));
        } else {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | QUEEN_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | ROOK_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | BISHOP_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | KNIGHT_PROMOTION,
            ));
        }

        move_targets[Direction::SouthWest as usize] &= !(1u64 << target_square);
        target_square += 9;
        while target_square < source_square {
            let bit = (move_targets[Direction::SouthWest as usize] & (1 << target_square))
                >> target_square;
            move_targets[Direction::SouthWest as usize] &= !(1u64 << target_square);
            if bit != 0 {
                let capture = !is_empty((1 << target_square) & occupied) as u16;
                let flags = capture & CAPTURE;

                moves.push(BitBoardMove::new(
                    source_square as u16,
                    target_square as u16,
                    flags,
                ));
            }
            target_square += 9;
        }
    }

    while move_targets[Direction::SouthEast as usize] != 0 {
        let mut target_square = move_targets[Direction::SouthEast as usize].trailing_zeros();
        let source_square = (RAY_ATTACKS[Direction::NorthWest as usize][target_square as usize]
            & occupied
            & !(1 << target_square))
            .trailing_zeros();

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        let pawn = (1 << source_square) & pawns;
        let promotion = !is_empty((pawn >> 8) & RANK1) as u16;

        if promotion == 0 {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags,
            ));
        } else {
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | QUEEN_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | ROOK_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | BISHOP_PROMOTION,
            ));
            moves.push(BitBoardMove::new(
                source_square as u16,
                target_square as u16,
                flags | KNIGHT_PROMOTION,
            ));
        }

        move_targets[Direction::SouthEast as usize] &= !(1u64 << target_square);
        target_square += 7;
        while target_square < source_square {
            let bit = (move_targets[Direction::SouthEast as usize] & (1 << target_square))
                >> target_square;
            move_targets[Direction::SouthEast as usize] &= !(1u64 << target_square);
            if bit != 0 {
                let capture = !is_empty((1 << target_square) & occupied) as u16;
                let flags = capture & CAPTURE;

                moves.push(BitBoardMove::new(
                    source_square as u16,
                    target_square as u16,
                    flags,
                ));
            }
            target_square += 7;
        }
    }

    // Knights

    while move_targets[Direction::NorthNorthEast as usize] != 0 {
        let target_square =
            move_targets[Direction::NorthNorthEast as usize].trailing_zeros() as u16;
        let source_square = south_south_west(1 << target_square).trailing_zeros() as u16;

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::NorthNorthEast as usize] &= !(1u64 << target_square);
    }

    while move_targets[Direction::NorthEastEast as usize] != 0 {
        let target_square = move_targets[Direction::NorthEastEast as usize].trailing_zeros() as u16;
        let source_square = south_west_west(1 << target_square).trailing_zeros() as u16;

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::NorthEastEast as usize] &= !(1u64 << target_square);
    }

    while move_targets[Direction::SouthEastEast as usize] != 0 {
        let target_square = move_targets[Direction::SouthEastEast as usize].trailing_zeros() as u16;
        let source_square = north_west_west(1 << target_square).trailing_zeros() as u16;

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::SouthEastEast as usize] &= !(1u64 << target_square);
    }

    while move_targets[Direction::SouthSouthEast as usize] != 0 {
        let target_square =
            move_targets[Direction::SouthSouthEast as usize].trailing_zeros() as u16;
        let source_square = north_north_west(1 << target_square).trailing_zeros() as u16;

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::SouthSouthEast as usize] &= !(1u64 << target_square);
    }

    while move_targets[Direction::NorthNorthWest as usize] != 0 {
        let target_square =
            move_targets[Direction::NorthNorthWest as usize].trailing_zeros() as u16;
        let source_square = south_south_east(1 << target_square).trailing_zeros() as u16;

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::NorthNorthWest as usize] &= !(1u64 << target_square);
    }

    while move_targets[Direction::NorthWestWest as usize] != 0 {
        let target_square = move_targets[Direction::NorthWestWest as usize].trailing_zeros() as u16;
        let source_square = south_east_east(1 << target_square).trailing_zeros() as u16;

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::NorthWestWest as usize] &= !(1u64 << target_square);
    }

    while move_targets[Direction::SouthWestWest as usize] != 0 {
        let target_square = move_targets[Direction::SouthWestWest as usize].trailing_zeros() as u16;
        let source_square = north_east_east(1 << target_square).trailing_zeros() as u16;

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::SouthWestWest as usize] &= !(1u64 << target_square);
    }

    while move_targets[Direction::SouthSouthWest as usize] != 0 {
        let target_square =
            move_targets[Direction::SouthSouthWest as usize].trailing_zeros() as u16;
        let source_square = north_north_east(1 << target_square).trailing_zeros() as u16;

        let capture = !is_empty((1 << target_square) & occupied) as u16;
        let flags = capture & CAPTURE;

        moves.push(BitBoardMove::new(
            source_square as u16,
            target_square as u16,
            flags,
        ));

        move_targets[Direction::SouthSouthWest as usize] &= !(1u64 << target_square);
    }

    if color == Color::Black {
        for m in &mut moves {
            m.set_to((1u64 << m.get_to()).swap_bytes().trailing_zeros() as u16);
            m.set_from((1u64 << m.get_from()).swap_bytes().trailing_zeros() as u16);
        }
    }

    moves
}

// fn genetate_knight_moves(targets: &mut [u64; 16], direction: Direction, moves: &mut Vec<BitBoardMove>) {

// }

fn count_moves(s: &[u64]) -> u32 {
    let mut count = 0;
    for i in s {
        count += i.count_ones();
    }
    count
}

pub fn perft(board: &BitBoardState, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let moves = generate_moves(board);

    if depth == 1 {
        return moves.len();
    }

    let nodes = moves
        .into_par_iter()
        .map(|m| {
            let mut board_copy = board.clone();
            board_copy.apply_move(&m);
            board_copy.change_side();
            let moves = perft(&board_copy, depth - 1);
            moves
        })
        .sum();

    return nodes;
}

pub fn perft_report(board: &BitBoardState, depth: usize) -> String {
    let mut report = String::new();

    let mut moves = generate_moves(board);
    moves.sort_by(|a, b| match a.get_from().cmp(&b.get_from()) {
        Ordering::Less => Ordering::Less,
        Ordering::Equal => a.get_to().cmp(&b.get_to()),
        Ordering::Greater => Ordering::Greater,
    });

    let mut total_nodes = 0;

    for m in &moves {
        let mut board_copy = board.clone();
        board_copy.apply_move(&m);
        board_copy.mirror_board();
        board_copy.change_side();
        let nodes = perft(&board_copy, depth - 1);
        total_nodes += nodes;

        let from = index_to_algebraic(m.get_from() as usize);
        let to = index_to_algebraic(m.get_to() as usize);

        report.push_str(&format!(
            "{}{}: {}\n",
            from_utf8(&from).unwrap(),
            from_utf8(&to).unwrap(),
            nodes
        ));
    }

    report.push_str(&format!("\nNodes searched: {}\n", total_nodes));

    report
}

#[cfg(test)]
mod test {
    use crate::interface::index_to_algebraic;
    use crate::{bitboard::*, board};
    use std::str::from_utf8;

    #[test]
    fn test_bitboard_move() {
        let e1 = BitBoardMove::from_long_algebraic(b"e2e4").unwrap();
        println!("{} {}", e1.get_from(), e1.get_to());
        let e2 = e1.to_long_algebraic().unwrap();
        println!("{}", e2);
        assert_eq!(e2, "e2e4");
    }

    #[test]
    fn flood_functions() {}

    #[test]
    fn test_move_targets() {
        let board = BitBoardState::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -  ",
        )
        .unwrap();

        let moves = generate_moves(&board);
        for m in moves {
            let from_i = index_to_algebraic(m.get_from() as usize);
            let from = from_utf8(&from_i).unwrap();
            let to_i = index_to_algebraic(m.get_to() as usize);
            let to = from_utf8(&to_i).unwrap();

            println!("from {} to {} : flags {:04b}", from, to, m.get_flags());
        }
    }

    #[test]
    fn position_1() {
        let mut board =
            BitBoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        assert_eq!(perft(&mut board, 0), 1);
        assert_eq!(perft(&mut board, 1), 20);
        assert_eq!(perft(&mut board, 2), 400);
        assert_eq!(perft(&mut board, 3), 8_902);
        assert_eq!(perft(&mut board, 4), 197_281);
        assert_eq!(perft(&mut board, 5), 4_865_609);
        //assert_eq!(perft(&mut board, 6), 119_060_324);
    }

    #[test]
    fn position_2() {
        let mut board = BitBoardState::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        )
        .unwrap();

        assert_eq!(perft(&mut board, 1), 48);
        assert_eq!(perft(&mut board, 2), 2_039);
        assert_eq!(perft(&mut board, 3), 97_862);
        //assert_eq!(perft(&mut board, 4), 4_085_603);
        //assert_eq!(perft(&mut board, 4), 197_281);
    }

    #[test]
    fn position_3() {
        let mut board = BitBoardState::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap();

        assert_eq!(perft(&mut board, 1), 14);
        assert_eq!(perft(&mut board, 2), 191);
        assert_eq!(perft(&mut board, 3), 2_812);
        assert_eq!(perft(&mut board, 4), 43_238);
        //assert_eq!(perft(&mut board, 5), 674_624);
    }

    #[test]
    fn position_4() {
        let mut board = BitBoardState::from_fen(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        )
        .unwrap();

        assert_eq!(perft(&mut board, 1), 6);
        assert_eq!(perft(&mut board, 2), 264);
        assert_eq!(perft(&mut board, 3), 9_467);
        assert_eq!(perft(&mut board, 4), 422_333);
        //assert_eq!(perft(&mut board, 5), 15_833_292);
    }

    #[test]
    fn position_5() {
        let mut board =
            BitBoardState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap();

        assert_eq!(perft(&mut board, 1), 44);
        assert_eq!(perft(&mut board, 2), 1_486);
        assert_eq!(perft(&mut board, 3), 62_379);
        assert_eq!(perft(&mut board, 4), 2_103_487);
        //assert_eq!(perft(&mut board, 5), 89_941_194);
    }

    #[test]
    fn position_6() {
        let mut board = BitBoardState::from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();

        assert_eq!(perft(&mut board, 1), 46);
        assert_eq!(perft(&mut board, 2), 2_079);
        assert_eq!(perft(&mut board, 3), 89_890);
        assert_eq!(perft(&mut board, 4), 3_894_594);
    }

    #[test]
    fn test_blah() {
        let mut board =
            BitBoardState::from_fen("rnQqkbn1/8/8/pp1ppp2/PP1PPPpr/2P3Pp/7P/RNB1KBNR b KQq - 0 12")
                .unwrap();
        println!("{}", perft_report(&board, 1));

        assert!(false);
    }
}
