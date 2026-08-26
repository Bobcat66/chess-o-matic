// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

use crate::engine::Evaluator;
use crate::game::{Color,Board,BoardAnal,ChessMove};
use std::cmp;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchResult {
    best_move: ChessMove,
    best_score: i32
}

impl SearchResult {
    pub fn new(best_move: ChessMove, best_score: i32) -> SearchResult {
        SearchResult { best_move: best_move, best_score: best_score }
    }
}
// This is an implementation of the negamax algorithm with alpha-beta pruning
pub fn negamax<E: Evaluator>(board: Board, depth: usize, threads: usize) -> SearchResult {
    SearchResult::new(ChessMove::new((0,0),(0,0),None),0) // placeholder
}

fn negamax_impl<E: Evaluator>(board: &Board, alpha: i32, beta: i32, depth: usize) -> i32 {
    return 0 // placeholder
}

fn quiescence<E: Evaluator>(board: &Board, alpha: i32, beta: i32) -> i32 {
    let anal = board.anal();
    let stand_pat = E::eval(board,&anal);
    if stand_pat >= beta {
        return beta;
    }
    alpha = cmp::max(alpha, stand_pat);
    
}