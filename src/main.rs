// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

mod game;

use game::{Board, Piece, Color, PieceType, ChessMove};
fn main() {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    println!("{}", board.render_ascii());
    board.try_move(ChessMove::from_uci("e2e4").unwrap()).unwrap();
    println!("{}", board.render_ascii());
}
