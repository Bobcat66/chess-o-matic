// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project


use crate::{engine::{evaluation::Evaluator, search::TranspositionTable}, game::{Board, BoardAnal, MoveStatus, ChessMove, Color, ZobristKeys}};
use std::marker::PhantomData;
use std::sync::Arc;
pub mod search;
pub mod evaluation;

pub struct Engine<E: Evaluator> {
    tt: Arc<TranspositionTable>,
    pub board: Board,
    _marker: PhantomData<E>,
}

impl<E: Evaluator> Engine<E> {
    pub fn new(keys: ZobristKeys, board: Board) -> Engine<E> {
        Engine::<E> {
            tt: Arc::new(TranspositionTable::new(keys)),
            board: board,
            _marker: PhantomData
        }
    }

    pub fn submit_move(&mut self, mv: ChessMove) -> Result<(),MoveStatus> {
        let anal = self.board.anal();
        self.board.try_move(&anal, mv)
    }

    pub fn next(&mut self, depth: usize, helper_threads: usize) -> Option<ChessMove> {
        let tt = self.tt.clone();
        let res = search::negamax_search::<E>(tt, self.board, depth, helper_threads)?;
        let anal = self.board.anal();
        // TODO: Error if move fails?
        self.board.try_move(&anal, res.best_move);
        return Some(res.best_move);
    }
}
