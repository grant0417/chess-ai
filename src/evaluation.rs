use crate::bitboard::BitBoard;
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

fn evaluate_bitboard(bitboard: &BitBoard, evaluate_color: Color) -> i64 {
    let opposite_color = match evaluate_color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    let mut value = 0;
    value += bitboard.get_set(evaluate_color, Piece::Pawn).count_ones() as i64;
    value += bitboard.get_set(evaluate_color, Piece::Rook).count_ones() as i64
        * piece_value(Piece::Rook);
    value += bitboard.get_set(evaluate_color, Piece::Bishop).count_ones() as i64
        * piece_value(Piece::Bishop);
    value += bitboard.get_set(evaluate_color, Piece::Knight).count_ones() as i64
        * piece_value(Piece::Knight);
    value += bitboard.get_set(evaluate_color, Piece::Queen).count_ones() as i64
        * piece_value(Piece::Queen);

    value -= bitboard.get_set(opposite_color, Piece::Pawn).count_ones() as i64;
    value -= bitboard.get_set(opposite_color, Piece::Rook).count_ones() as i64
        * piece_value(Piece::Rook);
    value -= bitboard.get_set(opposite_color, Piece::Bishop).count_ones() as i64
        * piece_value(Piece::Bishop);
    value -= bitboard.get_set(opposite_color, Piece::Knight).count_ones() as i64
        * piece_value(Piece::Knight);
    value -= bitboard.get_set(opposite_color, Piece::Queen).count_ones() as i64
        * piece_value(Piece::Queen);

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
