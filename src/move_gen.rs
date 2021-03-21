use crate::board::{Board, Color, Piece};
use rayon::prelude::*;

pub const DIRECTION_OFFSETS: [i32; 8] = [8, -8, -1, 1, 7, -7, 9, -9];
pub const NUM_SQUARE_TO_EDGE: [[usize; 8]; 64] = {
    let mut arr = [[0; 8]; 64];

    let mut file = 0;
    while file < 8 {
        let mut rank = 0;
        while rank < 8 {
            let num_north = 7 - rank;
            let num_south = rank;
            let num_west = file;
            let num_east = 7 - file;

            let index = rank * 8 + file;

            let num_nw = if num_north <= num_west {
                num_north
            } else {
                num_west
            };

            let num_se = if num_south <= num_east {
                num_south
            } else {
                num_east
            };

            let num_ne = if num_north <= num_east {
                num_north
            } else {
                num_east
            };

            let num_sw = if num_south <= num_west {
                num_south
            } else {
                num_west
            };

            arr[index] = [
                num_north, num_south, num_west, num_east, num_nw, num_se, num_ne, num_sw,
            ];
            rank += 1;
        }
        file += 1;
    }

    arr
};
const KNIGHT_MOVES: [i32; 8] = [15, 17, -17, -15, 6, -10, 10, -6];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MoveFlag {
    EnPassantCapture,
    CastlingQueen,
    CastlingKnight,
    PromoteQueen,
    PromoteKnight,
    PromoteBishop,
    PromoteRook,
    InitialMove,
}

#[derive(Copy, Clone, Debug, Eq)]
pub struct Move {
    pub start_index: u8,
    pub end_index: u8,
    pub flag: Option<MoveFlag>,
}

impl Move {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start_index: start as u8,
            end_index: end as u8,
            flag: None,
        }
    }

    pub fn new_flag(start: usize, end: usize, flag: MoveFlag) -> Self {
        Self {
            start_index: start as u8,
            end_index: end as u8,
            flag: Some(flag),
        }
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.start_index == other.start_index && self.end_index == other.end_index
    }
}

pub fn generate_legal_moves(board: &mut Board) -> Vec<Move> {
    let mut legal_moves = Vec::with_capacity(256);
    'verify_loop: for m in generate_pseudo_legal_moves(board) {
        let current_color = board.active_color;

        board.move_piece(&m);

        // Find King
        let mut king_location = None;
        for location in 0..64 {
            if Some((current_color, Piece::King)) == board.pieces[location] {
                king_location = Some(location);
                break;
            }
        }

        let opponent_responses = generate_pseudo_legal_moves(board);

        for response in opponent_responses {
            if Some(response.end_index as usize) == king_location {
                board.revert_last_move(&m);
                continue 'verify_loop;
            }
        }

        board.revert_last_move(&m);

        legal_moves.push(m);
    }
    legal_moves
}

pub fn generate_pseudo_legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(256);

    // (0..64).into_par_iter().filter_map(|i| {
    //     if let Some((color, piece)) = board.pieces[i] {
    //         if color == board.active_color {
    //             match piece {
    //                 Piece::Bishop | Piece::Rook | Piece::Queen => {
    //                     gen_sliding_moves(&mut moves, &board, i)
    //                 }
    //                 Piece::Knight => gen_knight_moves(&mut moves, &board, i),
    //                 Piece::King => gen_king_moves(&mut moves, &board, i),
    //                 Piece::Pawn => gen_pawn_moves(&mut moves, &board, i),
    //             };
    //         }
    //     }
    //     Some('a')
    // });

    for start in 0..64 {
        if let Some((color, piece)) = board.pieces[start] {
            if color == board.active_color {
                match piece {
                    Piece::Bishop | Piece::Rook | Piece::Queen => {
                        gen_sliding_moves(&mut moves, &board, start)
                    }
                    Piece::Knight => gen_knight_moves(&mut moves, &board, start),
                    Piece::King => gen_king_moves(&mut moves, &board, start),
                    Piece::Pawn => gen_pawn_moves(&mut moves, &board, start),
                };
            }
        }
    }
    moves
}

pub fn gen_sliding_moves(moves: &mut Vec<Move>, board: &Board, start: usize) {
    if let Some((_, piece)) = board.pieces[start] {
        let start_dir = if let Piece::Bishop = piece { 4 } else { 0 };
        let end_dir = if let Piece::Rook = piece { 4 } else { 8 };

        for dir_index in start_dir..end_dir {
            for n in 0..NUM_SQUARE_TO_EDGE[start][dir_index] {
                let target_square =
                    (start as i32 + DIRECTION_OFFSETS[dir_index] * (n as i32 + 1)) as usize;

                if target_square >= 64 {
                    continue;
                }

                match board.pieces[target_square] {
                    Some((color, _)) if color == board.active_color => {
                        break;
                    }
                    Some((color, _)) if color != board.active_color => {
                        moves.push(Move::new(start, target_square));
                        break;
                    }
                    None => {
                        moves.push(Move::new(start, target_square));
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
    }
}

fn gen_pawn_moves(moves: &mut Vec<Move>, board: &Board, start: usize) {
    if let Some((color, _)) = board.pieces[start] {
        let direction = color as usize;

        // Single advance
        let target_square = (start as i32 + DIRECTION_OFFSETS[direction]) as usize;
        if target_square < 64 {
            if None == board.pieces[target_square] {
                if target_square / 8 == 0 || target_square / 8 == 7 {
                    moves.push(Move::new_flag(start, target_square, MoveFlag::PromoteQueen))
                } else {
                    moves.push(Move::new(start, target_square));
                }
                // Initial double advance
                if color == Color::White && start / 8 == 1
                    || color == Color::Black && start / 8 == 6
                {
                    let target_square = (start as i32 + DIRECTION_OFFSETS[direction] * 2) as usize;
                    if None == board.pieces[target_square] {
                        moves.push(Move::new_flag(start, target_square, MoveFlag::InitialMove));
                    }
                }
            }
        }

        // Captures

        // Right
        let target_square = (start as i32 + DIRECTION_OFFSETS[6 - direction]) as usize;
        if target_square < 64 && target_square % 8 != 0 {
            if let Some((color, _)) = board.pieces[target_square] {
                if color != board.active_color {
                    if target_square / 8 == 0 || target_square / 8 == 7 {
                        moves.push(Move::new_flag(start, target_square, MoveFlag::PromoteQueen))
                    } else {
                        moves.push(Move::new(start, target_square));
                    }
                }
            }
            if let Some(pawn_en_passant) = board.en_passant {
                if pawn_en_passant.get() as usize == target_square {
                    if let Some((color, Piece::Pawn)) = board.pieces[start + 1] {
                        if color != board.active_color {
                            moves.push(Move::new_flag(
                                start,
                                target_square,
                                MoveFlag::EnPassantCapture,
                            ));
                        }
                    }
                }
            }
        }

        // Left
        let target_square = (start as i32 + DIRECTION_OFFSETS[4 + direction * 3]) as usize;
        if target_square < 64 && target_square % 8 != 7 {
            if let Some((color, _)) = board.pieces[target_square] {
                if color != board.active_color {
                    if target_square / 8 == 0 || target_square / 8 == 7 {
                        moves.push(Move::new_flag(start, target_square, MoveFlag::PromoteQueen))
                    } else {
                        moves.push(Move::new(start, target_square));
                    }
                }
            }
            if let Some(pawn_en_passant) = board.en_passant {
                if pawn_en_passant.get() as usize == target_square {
                    if let Some((color, Piece::Pawn)) = board.pieces[start - 1] {
                        if color != board.active_color {
                            moves.push(Move::new_flag(
                                start,
                                target_square,
                                MoveFlag::EnPassantCapture,
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn gen_king_moves(moves: &mut Vec<Move>, board: &Board, start: usize) {
    for dir_index in 0..8 {
        if NUM_SQUARE_TO_EDGE[start][dir_index] != 0 {
            let target_square = (start as i32 + DIRECTION_OFFSETS[dir_index]) as usize;
            match board.pieces[target_square] {
                Some((color, _)) if color != board.active_color => {
                    moves.push(Move::new(start, target_square));
                }
                None => {
                    moves.push(Move::new(start, target_square));
                }
                _ => {}
            }
        }
    }

    // if board.active_color == Color::White && board.castling.contains(Castling::WHITE_QUEEN)
    //     || board.active_color == Color::Black && board.castling.contains(Castling::BLACK_QUEEN)
    // {
    //     if None == board.pieces[start - 1] {
    //         if None == board.pieces[start - 2] {
    //             if None == board.pieces[start - 3] {
    //                 if Some((board.active_color, Piece::Rook)) == board.pieces[start - 4] {
    //                     moves.push(Move::new_flag(start, start - 2, MoveFlag::CastlingQueen));
    //                 }
    //             }
    //         }
    //     }
    // }

    // if board.active_color == Color::White && board.castling.contains(Castling::WHITE_KNIGHT)
    //     || board.active_color == Color::Black && board.castling.contains(Castling::BLACK_KNIGHT)
    // {
    //     if None == board.pieces[start + 1] {
    //         if None == board.pieces[start + 2] {
    //             if Some((board.active_color, Piece::Rook)) == board.pieces[start + 3] {
    //                 moves.push(Move::new_flag(start, start + 2, MoveFlag::CastlingKnight));
    //             }
    //         }
    //     }
    // }
}

fn gen_knight_moves(moves: &mut Vec<Move>, board: &Board, start: usize) {
    for dir_index in 0..4 {
        if NUM_SQUARE_TO_EDGE[start][dir_index] >= 2 {
            for i in 0..2 {
                if dir_index / 2 == 0 && NUM_SQUARE_TO_EDGE[start][i + 2] >= 1
                    || dir_index / 2 == 1 && NUM_SQUARE_TO_EDGE[start][i] >= 1
                {
                    let target_square = (start as i32 + KNIGHT_MOVES[dir_index * 2 + i]) as usize;

                    match board.pieces[target_square] {
                        Some((color, _)) if color != board.active_color => {
                            moves.push(Move::new(start, target_square));
                        }
                        None => {
                            moves.push(Move::new(start, target_square));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn perft(board: &mut Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(board);

    let mut nodes = 0;

    if depth == 1 {
        return moves.len();
    }

    for m in &moves {
        board.move_piece(m);
        nodes += perft(board, depth - 1);
        board.revert_last_move(m);
    }
    return nodes;
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::move_gen::perft;

    // Based on test positions from the chess programming wiki:
    // https://www.chessprogramming.org/Perft_Results

    // #[test]
    // fn position_1() {
    //     let mut board =
    //         Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    //     assert_eq!(perft(&mut board, 0), 1);
    //     assert_eq!(perft(&mut board, 1), 20);
    //     assert_eq!(perft(&mut board, 2), 400);
    //     assert_eq!(perft(&mut board, 3), 8_902);
    //     //assert_eq!(perft(&mut board, 4), 197_281);
    //     //assert_eq!(perft(&mut board, 5), 4_865_609);
    //     //assert_eq!(perft(&mut board, 6), 119_060_324);
    // }

    // #[test]
    // fn position_2() {
    //     let mut board =
    //         Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
    //             .unwrap();

    //     assert_eq!(perft(&mut board, 1), 48);
    //     assert_eq!(perft(&mut board, 2), 2_039);
    //     assert_eq!(perft(&mut board, 3), 97_862);
    //     //assert_eq!(perft(&mut board, 4), 4_085_603);
    //     //assert_eq!(perft(&mut board, 4), 197_281);
    // }

    // #[test]
    // fn position_3() {
    //     let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap();

    //     assert_eq!(perft(&mut board, 1), 14);
    //     assert_eq!(perft(&mut board, 2), 191);
    //     assert_eq!(perft(&mut board, 3), 2_812);
    //     assert_eq!(perft(&mut board, 4), 43_238);
    //     //assert_eq!(perft(&mut board, 5), 674_624);
    // }

    // #[test]
    // fn position_4() {
    //     let mut board =
    //         Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
    //             .unwrap();

    //     assert_eq!(perft(&mut board, 1), 6);
    //     assert_eq!(perft(&mut board, 2), 264);
    //     assert_eq!(perft(&mut board, 3), 9_467);
    //     assert_eq!(perft(&mut board, 4), 422_333);
    //     //assert_eq!(perft(&mut board, 5), 15_833_292);
    // }

    // #[test]
    // fn position_5() {
    //     let mut board =
    //         Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    //     assert_eq!(perft(&mut board, 1), 44);
    //     assert_eq!(perft(&mut board, 2), 1_486);
    //     assert_eq!(perft(&mut board, 3), 62_379);
    //     assert_eq!(perft(&mut board, 4), 2_103_487);
    //     //assert_eq!(perft(&mut board, 5), 89_941_194);
    // }

    // #[test]
    // fn position_6() {
    //     let mut board = Board::from_fen(
    //         "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    //     )
    //     .unwrap();

    //     assert_eq!(perft(&mut board, 1), 1);
    //     assert_eq!(perft(&mut board, 2), 46);
    //     assert_eq!(perft(&mut board, 3), 2_079);
    //     assert_eq!(perft(&mut board, 4), 89_890);
    //     assert_eq!(perft(&mut board, 5), 3_894_594);
    // }
}
