// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

mod game;
mod engine;

use game::{Board, ChessMove};
fn main() {
    // TODO: Move this into an actual test suite
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    println!("Fried Liver Attack");
    board.try_move(&(board.anal()),ChessMove::from_uci("e2e4").unwrap()).unwrap();
    board.try_move(&(board.anal()),ChessMove::from_uci("e7e5").unwrap()).unwrap();
    println!("1. e4 e5\n");
    println!("{}", board.render_ascii());

    board.try_move(&(board.anal()),ChessMove::from_uci("g1f3").unwrap()).unwrap();
    board.try_move(&(board.anal()),ChessMove::from_uci("b8c6").unwrap()).unwrap();
    println!("2. Nf3 Nc6\n");
    println!("{}", board.render_ascii());

    board.try_move(&(board.anal()),ChessMove::from_uci("f1c4").unwrap()).unwrap();
    board.try_move(&(board.anal()),ChessMove::from_uci("g8f6").unwrap()).unwrap();
    println!("3. Bc4 Nf6\n");
    println!("{}", board.render_ascii());

    board.try_move(&(board.anal()),ChessMove::from_uci("f3g5").unwrap()).unwrap();
    board.try_move(&(board.anal()),ChessMove::from_uci("d7d5").unwrap()).unwrap();
    println!("4. Ng5 d5\n");
    println!("{}", board.render_ascii());

    board.try_move(&(board.anal()),ChessMove::from_uci("e4d5").unwrap()).unwrap();
    board.try_move(&(board.anal()),ChessMove::from_uci("f6d5").unwrap()).unwrap();
    println!("5. exd5 Nxd5\n");
    println!("{}", board.render_ascii());

    board.try_move(&(board.anal()),ChessMove::from_uci("g5f7").unwrap()).unwrap();
    board.try_move(&(board.anal()),ChessMove::from_uci("e8f7").unwrap()).unwrap();
    println!("6. Nxf7 Kxf7\n");
    println!("{}", board.render_ascii());

    board.try_move(&(board.anal()),ChessMove::from_uci("d1f3").unwrap()).unwrap();
    board.try_move(&(board.anal()),ChessMove::from_uci("f7e6").unwrap()).unwrap();
    println!("7. Qf3+ Ke6\n");
    println!("{}", board.render_ascii());

    let mut evergreen_board = Board::from_fen("r1bqk1nr/pppp1ppp/2n5/b7/2BpP3/2P2N2/P4PPP/RNBQK2R w KQkq - 0 7").unwrap();
    println!("The Evergreen Game, Adolf Anderssen versus Jean Dufresne, 1852 (Move 7)");
    println!("{}", evergreen_board.render_ascii());
    // Castling smoketest
    evergreen_board.try_move(&(evergreen_board.anal()),ChessMove::from_uci("e1g1").unwrap()).unwrap();
    println!("{}", evergreen_board.render_ascii());

    // simple unfinished CLI game
    let mut game_board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    loop {
        let anal = board.anal();
        if anal.board_status().terminal() { break; }
        println!("{}", game_board.render_ascii());
    }
    
}
