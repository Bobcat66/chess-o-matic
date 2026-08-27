// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

// this is a simple evaluator that solely checks material. Its called GreedEval because its greedy and likes to hoard material

use crate::engine::evaluation::Evaluator;
use crate::game::BoardStatus::Checkmate;
use crate::game::{PieceType,BoardAnal,BoardStatus};

// piece value in centipawns
fn piece_table(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 100,
        PieceType::Bishop => 300,
        PieceType::Knight => 300,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 10000
    }
}
pub struct GreedEval {}

impl Evaluator for GreedEval {
    fn eval(anal: &BoardAnal) -> i32 {
        let mut score = 0;
        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = anal.board.get((file,rank)) {
                    score += piece_table(piece.kind) * if piece.color == anal.board.to_move { 1 } else { -1 };
                }
            }
        }
        // Checkmate
        if anal.board_status() == BoardStatus::Checkmate {
            score = -10000
        }
        score
    }
}