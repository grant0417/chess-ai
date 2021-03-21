use crate::bitboard::{generate_moves, BitBoard, BitBoardMove, BitBoardState};
use crate::board::Color;
use std::collections::BinaryHeap;
use crate::evaluation::evaluate_bitboard;
use std::cmp::Ordering;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use crate::util::Rng;

pub struct SearchDriver {
    transposition_table: Vec<i64>,
    zobrist_table: [[u64; 64]; 12]
}

impl SearchDriver {
    pub fn new(size: usize) -> Self {
        let mut rng = Rng::unix_seed();
        let mut zobrist_table = [[0; 64]; 12];
        //TODO: Add en passant and castling

        for i in 0..64 {
            for j in 0..12 {
                zobrist_table[j][i] = rng.rand_u64();
            }
        }

        Self {
            transposition_table: Vec::with_capacity(2usize.pow(size as u32)),
            zobrist_table
        }
    }

    pub fn best_move(&mut self, bitboard: &BitBoardState, depth: usize) -> BitBoardMove {
        let mut moves = self.evaluate_moves(bitboard, depth);
        moves.pop().unwrap().m
    }

    pub fn alpha_beta_max(&mut self, bitboard: &BitBoardState, color: Color, mut alpha: i64, beta: i64, depth: usize) -> i64 {
        if depth == 0 {
            let eval = evaluate_bitboard(bitboard, color);
            let table_len = self.transposition_table.len();
            self.transposition_table[bitboard.zobrist_hash(self.zobrist_table) as usize % table_len] = eval;
            return eval;
        }

        if self.transposition_table[bitboard.zobrist_hash(self.zobrist_table) as usize % self.transposition_table.len()] != 0 {
            return self.transposition_table[bitboard.zobrist_hash(self.zobrist_table) as usize % self.transposition_table.len()]
        }

        for m in generate_moves(bitboard) {
            let mut b = bitboard.clone();
            b.apply_move(&m);
            b.change_side();
            let score = self.alpha_beta_min(&b, color, alpha, beta, depth - 1);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }

    pub fn alpha_beta_min(&mut self, bitboard: &BitBoardState, color: Color, alpha: i64, mut  beta: i64, depth: usize) -> i64 {
        if depth == 0 {
            let eval = -evaluate_bitboard(bitboard, color);
            let table_len = self.transposition_table.len();
            self.transposition_table[bitboard.zobrist_hash(self.zobrist_table) as usize % table_len] = eval;
            return eval;
        }

        if self.transposition_table[bitboard.zobrist_hash(self.zobrist_table) as usize % self.transposition_table.len()] != 0 {
            return self.transposition_table[bitboard.zobrist_hash(self.zobrist_table) as usize % self.transposition_table.len()]
        }

        for m in generate_moves(bitboard) {
            let mut b = bitboard.clone();
            b.apply_move(&m);
            b.change_side();
            let score = self.alpha_beta_max(&b, color, alpha, beta, depth - 1);
            if score <= alpha {
                return alpha;
            }
            if score < beta {
                beta = score;
            }
        }
        beta
    }


    fn evaluate_moves(&mut self, bitboard: &BitBoardState, depth: usize) -> BinaryHeap<MoveValue> {
        let mut b = bitboard.clone();
        let moves = generate_moves(&mut b);
        moves.par_iter().map(|m| {
            let mut b_c = b.clone();
            b_c.apply_move(&m);
            let value = self.alpha_beta_max(&b_c, b.active_color, i64::MIN, i64::MAX, depth);
            MoveValue{ value, m: *m }
        }).collect()
    }

}

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


#[cfg(test)]
mod tests {
    use crate::bitboard::{BitBoardState, generate_moves};
    use crate::evaluation::evaluate_bitboard;
    use crate::search::{best_move, MoveValue};
    use std::collections::BinaryHeap;

    #[test]
    fn test_best_move() {
        let mut board =
            BitBoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")
                .unwrap();

        let b = best_move(&board, 0);
        println!("{:?}", b);
    }
}
