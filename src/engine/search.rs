// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

// TODO: Move ordering

use crate::engine::Evaluator;
use crate::game::{Color,Board,BoardAnal,ChessMove,BoardStatus,ZobristKeys};
use std::cmp;
use std::thread;
use dashmap::DashMap;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::Arc;

// Claude Slop
#[derive(Clone, Copy, Debug, PartialEq)]
struct TTEntry {
    depth: usize,             // how deep this result was searched to
    score: i32,             // the evaluated/searched score
    best_move: Option<ChessMove>,  // best move found, useful for move ordering
    node_type: NodeType,    // see below — was this exact, a lower bound, or upper bound?
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NodeType {
    Exact,      // score is the true minimax value
    LowerBound, // score is at least this good (search was cut off by a beta cutoff)
    UpperBound, // score is at most this good (search never raised alpha)
}

// Me Slop
struct TranspositionTable {
    keys: ZobristKeys,
    table: DashMap<u64,TTEntry>
}

impl TranspositionTable {
    pub fn new(keys: ZobristKeys) -> TranspositionTable {
        TranspositionTable { keys, table: DashMap::new() }
    }

    pub fn get(&self,board: &Board) -> Option<TTEntry> {
        let hash = board.zobrist_hash(&self.keys);
        return self.table.get(&hash).and_then(|x| Some(*x))
    }

    pub fn insert(&self, board: &Board, depth: usize, score: i32, best_move: Option<ChessMove>, node_type: NodeType) -> Option<TTEntry> {
        let hash = board.zobrist_hash(&self.keys);
        let entry = TTEntry {
            depth: depth,
            score: score,
            best_move: best_move,
            node_type: node_type
        };
        self.table.insert(hash, entry)
    }
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
// This is an implementation of the negamax algorithm with alpha-beta pruning. Partial claude slop
pub fn negamax_search<E: Evaluator>(board: Board, depth: usize, threads: usize) -> SearchResult {
    let keys = ZobristKeys::new();
    let tt = Arc::new(TranspositionTable::new(keys)); // tt: TranspositionTable

    let mut handles = Vec::new();
    for thread_id in 0..threads {
        let tt = Arc::clone(&tt); // cheap — bumps a reference count
        let board = board.clone();
        let start_depth = 1 + (thread_id % 3);
        handles.push(thread::spawn(move || {
            // this thread has its own Arc<TranspositionTable> pointing at the same underlying table
            negamax::<E>(&tt, &(board.anal()), start_depth.max(depth), i32::MIN, i32::MAX)
        }));
    }
    for h in handles { h.join().unwrap(); } // Results are discarded as we only care about TT contributions

    // Main negamax thread
    let mut alpha = i32::MIN;
    let beta = i32::MAX;
    let anal = &board.anal();

    // TT probe
    if let Some(entry) = tt.get(&board) {
        if entry.depth >= depth {
            match entry.node_type {
                NodeType::Exact if entry.best_move.is_some() => return SearchResult { best_move: entry.best_move.unwrap(), best_score: entry.score },
                NodeType::LowerBound if entry.score >= beta && entry.best_move.is_some() => return SearchResult { best_move: entry.best_move.unwrap(), best_score: entry.score },
                NodeType::UpperBound if entry.score <= alpha && entry.best_move.is_some() => return SearchResult { best_move: entry.best_move.unwrap(), best_score: entry.score },
                _ => {}
            }
        }
    }

    // Degenerate case, Realistically this will basically never get called.
    if depth == 0 || anal.board_status().terminal() {
        return SearchResult{ best_move: *anal.legal_moves.get(0).unwrap(), best_score: E::eval(anal) };
    }

    let mut best = i32::MIN;
    let mut best_move: Option<ChessMove> = None;
    let moves = anal.legal_moves.clone();

    for chess_move in &moves {
        let child = anal.lookahead(*chess_move).unwrap();
        let score = -negamax::<E>(&tt, &child, depth - 1, -beta, -alpha);
        if score > best {
            best = score;
            best_move = Some(*chess_move);
        }
        alpha = cmp::max(alpha, score);
    }

    tt.insert(&anal.board, depth,best, best_move, NodeType::Exact);
    return SearchResult { best_move: best_move.unwrap(), best_score: best };
}

fn negamax<E: Evaluator>(tt: &Arc<TranspositionTable>, anal: &BoardAnal, depth: usize, alpha: i32, beta: i32) -> i32 {
    let mut alpha = alpha;
    let original_alpha = alpha;

    // TT probe
    if let Some(entry) = tt.get(&anal.board) {
        if entry.depth >= depth {
            match entry.node_type {
                NodeType::Exact => return entry.score,
                NodeType::LowerBound if entry.score >= beta => return entry.score,
                NodeType::UpperBound if entry.score <= alpha => return entry.score,
                _ => {}
            }
        }
    }

    if depth == 0 || anal.board_status().terminal() {
        return quiescence::<E>(tt, anal, alpha, beta);
    }

    let mut best = i32::MIN;
    let mut best_move: Option<ChessMove> = None;
    let moves = anal.legal_moves.clone();

    for chess_move in &moves {
        let child = anal.lookahead(*chess_move).unwrap();
        let score = -negamax::<E>(tt, &child, depth - 1, -beta, -alpha);
        if score > best {
            best = score;
            best_move = Some(*chess_move);
        }
        alpha = cmp::max(alpha, score);
        if alpha >= beta {
            break;
        }
    }

    // TT store
    let node_type = if best <= original_alpha {
        NodeType::UpperBound
    } else if best >= beta {
        NodeType::LowerBound
    } else {
        NodeType::Exact
    };
    tt.insert(&anal.board, depth,best, best_move, node_type);

    best
}

fn quiescence<E: Evaluator>(tt: &Arc<TranspositionTable>, anal: &BoardAnal, alpha: i32, beta: i32) -> i32 {
    let original_alpha = alpha;
    let mut alpha = alpha;

    if let Some(entry) = tt.get(&anal.board) {
        if entry.depth >= 0 { // always true for usize, but conceptually: any stored entry qualifies at depth 0
            match entry.node_type {
                NodeType::Exact => return entry.score,
                NodeType::LowerBound if entry.score >= beta => return entry.score,
                NodeType::UpperBound if entry.score <= alpha => return entry.score,
                _ => {}
            }
        }
    }

    let stand_pat = E::eval(&anal);
    if stand_pat >= beta {
        tt.insert(&anal.board, 0, stand_pat, None, NodeType::LowerBound);
        return beta;
    }
    alpha = cmp::max(alpha, stand_pat);

    let captures = anal.captures(anal.board.to_move);
    if captures.is_empty() {
        tt.insert(&anal.board, 0, alpha, None, NodeType::Exact);
        return alpha;
    }

    let mut best_move: Option<ChessMove> = None;
    for capture in captures {
        let child = anal.lookahead(capture).unwrap();
        let score = -quiescence::<E>(tt, &child, -beta, -alpha);
        if score >= beta {
            tt.insert(&anal.board, 0, score, Some(capture), NodeType::LowerBound);
            return beta;
        }
        if score > alpha {
            alpha = score;
            best_move = Some(capture);
        }
    }

    let node_type = if alpha <= original_alpha { NodeType::UpperBound } else { NodeType::Exact };
    tt.insert(&anal.board, 0, alpha, best_move, node_type);
    alpha
}

