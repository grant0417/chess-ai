use crate::bitboard::{BitBoard, generate_moves, BitBoardState};
use crate::board::{Board, Color, Piece};

fn evaluate_board(board: &Board, evaluate_color: Color) -> i64 {
    let mut value = 0;
    for location in board.pieces.iter() {
        if let Some((piece_color, piece_type)) = location {
            if evaluate_color == *piece_color {
                value += piece_value(*piece_type);
            } else {
                value -= piece_value(*piece_type);
            }
        }
    }
    value
}

pub fn evaluate_bitboard(bitboard: &BitBoardState, evaluate_color: Color) -> i64 {
    let mut bitboard = bitboard.clone();
    let opposite_color = match evaluate_color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    let mut value = 0;
    value += bitboard.bitboard.get_set(evaluate_color, Piece::Pawn).count_ones() as i64;
    value += bitboard.bitboard.get_set(evaluate_color, Piece::Rook).count_ones() as i64
        * piece_value(Piece::Rook);
    value += bitboard.bitboard.get_set(evaluate_color, Piece::Bishop).count_ones() as i64
        * piece_value(Piece::Bishop);
    value += bitboard.bitboard.get_set(evaluate_color, Piece::Knight).count_ones() as i64
        * piece_value(Piece::Knight);
    value += bitboard.bitboard.get_set(evaluate_color, Piece::Queen).count_ones() as i64
        * piece_value(Piece::Queen);

    value -= bitboard.bitboard.get_set(opposite_color, Piece::Pawn).count_ones() as i64;
    value -= bitboard.bitboard.get_set(opposite_color, Piece::Rook).count_ones() as i64
        * piece_value(Piece::Rook);
    value -= bitboard.bitboard.get_set(opposite_color, Piece::Bishop).count_ones() as i64
        * piece_value(Piece::Bishop);
    value -= bitboard.bitboard.get_set(opposite_color, Piece::Knight).count_ones() as i64
        * piece_value(Piece::Knight);
    value -= bitboard.bitboard.get_set(opposite_color, Piece::Queen).count_ones() as i64
        * piece_value(Piece::Queen);

    bitboard.active_color = evaluate_color;
    let our_moves = generate_moves(&bitboard).len();
    bitboard.active_color = opposite_color;
    let their_moves = generate_moves(&bitboard).len();

    let move_val = 10 * (our_moves - their_moves) as i64;
    value += move_val;

    value
}

fn piece_value(piece: Piece) -> i64 {
    match piece {
        Piece::King => 0,
        Piece::Queen => 900,
        Piece::Rook => 500,
        Piece::Bishop => 300,
        Piece::Knight => 300,
        Piece::Pawn => 100,
    }
}
