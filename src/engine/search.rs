// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

use crate::engine::Evaluator;
use crate::game::{Color,Board,BoardAnal,ChessMove,BoardStatus};
use std::cmp;

// Claude Slop
struct TTEntry {
    zobrist_key: u64,       // for collision detection (see below)
    depth: usize,             // how deep this result was searched to
    score: i32,             // the evaluated/searched score
    best_move: Option<ChessMove>,  // best move found, useful for move ordering
    node_type: NodeType,    // see below — was this exact, a lower bound, or upper bound?
}

enum NodeType {
    Exact,      // score is the true minimax value
    LowerBound, // score is at least this good (search was cut off by a beta cutoff)
    UpperBound, // score is at most this good (search never raised alpha)
}

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

fn negamax_impl<E: Evaluator>(anal: &BoardAnal, depth: usize, alpha: i32, beta: i32) -> i32 {
    if depth == 0 || anal.board_status().terminal() {
        return quiescence::<E>(anal, alpha, beta);
    }
    let mut best = std::i32::MIN;
    let mut alpha = alpha;
    let moves = &anal.legal_moves;
    for chess_move in moves {
        let child = anal.lookahead(*chess_move).unwrap();
        let score = -negamax_impl::<E>(&child,depth-1,-beta,-alpha);
        best = cmp::max(best,score);
        alpha = cmp::max(alpha,score);
        if alpha >= beta {
            break;
        }
    }
    best // placeholder
}

fn quiescence<E: Evaluator>(anal: &BoardAnal, alpha: i32, beta: i32) -> i32 {
    let stand_pat = E::eval(&anal);
    if stand_pat >= beta {
        return beta;
    }
    let mut alpha = cmp::max(alpha, stand_pat);
    
    let captures = anal.captures(anal.board.to_move);
    for capture in captures {
        let child = anal.lookahead(capture).unwrap();
        let score= -quiescence::<E>(&child, -beta, -alpha);
        if score >= beta { return beta; }
        alpha = cmp::max(alpha,score);
    }

    return alpha;

}