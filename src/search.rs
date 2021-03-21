use crate::bitboard::{generate_moves, BitBoard, BitBoardMove, BitBoardState};
use crate::board::Color;
use std::collections::BinaryHeap;
use crate::evaluation::evaluate_bitboard;
use std::cmp::Ordering;

#[derive(Debug)]
struct MoveValue {
    value: i64,
    m: BitBoardMove,
}

impl PartialEq for MoveValue {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl PartialOrd for MoveValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Eq for MoveValue {

}

impl Ord for MoveValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

pub fn best_move(bitboard: &BitBoardState, _depth: usize) -> BitBoardMove {

}

pub fn alpha_beta_max(bitboard: &BitBoardState, color: Color, mut alpha: i64, beta: i64, depth: usize) -> i64 {
    if depth == 0 {
        return evaluate_bitboard(bitboard, color);
    }

    let moves = generate_moves(bitboard);

    for m in moves {
        let score = alpha_beta_min(bitboard, color, alpha, beta, depth - 1);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    return alpha
}

pub fn alpha_beta_min(bitboard: &BitBoardState, color: Color, alpha: i64, mut  beta: i64, depth: usize) -> i64 {

    if depth == 0 {
        return evaluate_bitboard(bitboard, color);
    }

    let moves = generate_moves(bitboard);

    for m in moves {
        let score = alpha_beta_min(bitboard, color, alpha, beta, depth - 1);
        if score <= alpha {
            return alpha;
        }
        if score < beta {
            beta = score;
        }
    }

    beta
}


pub fn evaluate_moves(bitboard: &BitBoardState) -> BinaryHeap<MoveValue> {
    let mut b = bitboard.clone();
    let moves = generate_moves(&mut b);
    let mut heap = BinaryHeap::new();
    for m in moves {
        let mut b_c = b.clone();
        b_c.apply_move(&m);

        let value = evaluate_bitboard(&b_c, b.active_color);
        heap.push(MoveValue{ value, m })
    }
    heap
}

#[cfg(test)]
mod tests {
    use crate::bitboard::{BitBoardState, generate_moves};
    use crate::evaluation::evaluate_bitboard;
    use crate::search::{best_move, MoveValue};
    use std::collections::BinaryHeap;

    #[test]
    fn test_best_move() {
        let mut board =
            BitBoardState::from_fen("rnQqkbn1/8/8/pp1ppp2/PP1PPPpr/2P3Pp/7P/RNB1KBNR b KQq - 0 12")
                .unwrap();

        let b = best_move(&board, 0);
        println!("{:?}", b);
    }
}
