// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

use crate::game::PieceType::Pawn;
use crate::game::board::MoveStatus::Illegal;
use crate::game::{Color, Movable, Piece, PieceType, move_flags};
use regex::Regex;
use std::fmt;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessMove {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub promotion: Option<PieceType>
}

impl ChessMove {
    pub fn new(from: (usize, usize), to: (usize, usize), promotion: Option<PieceType>) -> ChessMove {
        ChessMove {
            from,
            to,
            promotion,
        }
    }

    pub fn from_uci(uci: &str) -> Result<ChessMove, String> {
        if uci.len() < 4 || uci.len() > 5 {
            return Err("Invalid UCI string length".into());
        }
        let from_file = uci.chars().nth(0).unwrap();
        let from_rank = uci.chars().nth(1).unwrap();
        let to_file = uci.chars().nth(2).unwrap();
        let to_rank = uci.chars().nth(3).unwrap();

        let from = (
            (from_file as u8 - b'a') as usize,
            (from_rank as u8 - b'1') as usize,
        );
        let to = (
            (to_file as u8 - b'a') as usize,
            (to_rank as u8 - b'1') as usize,
        );

        let promotion = if uci.len() == 5 {
            match uci.chars().nth(4).unwrap() {
                'q' => Some(PieceType::Queen),
                'r' => Some(PieceType::Rook),
                'b' => Some(PieceType::Bishop),
                'n' => Some(PieceType::Knight),
                _ => return Err("Invalid promotion piece".into()),
            }
        } else {
            None
        };

        Ok(ChessMove::new(from, to, promotion))
    }

    pub fn to_uci(&self) -> String {
        let mut uci: String = String::new();
        uci.push((self.from.0 as u8 + b'a') as char);
        uci.push((self.from.1 as u8 + b'1') as char);
        uci.push((self.to.0 as u8 + b'a') as char);
        uci.push((self.to.1 as u8 + b'1') as char);
        if self.promotion.is_some() {
            match self.promotion.unwrap() {
                PieceType::Queen => uci.push('q'),
                PieceType::Rook => uci.push('r'),
                PieceType::Bishop => uci.push('b'),
                PieceType::Knight => uci.push('n'),
                // Theoretically these should never be called, should I have a failure case anyways just in case?
                PieceType::King => {},
                PieceType::Pawn => {}

            }
        }
        uci
    }

    pub fn delta(&self) -> (i8,i8) {
        (
            self.to.0 as i8 - self.from.0 as i8,
            self.to.1 as i8 - self.from.1 as i8
        )
    }
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_uci());
    }
}

// board analysis
pub struct BoardAnal {
    white_pieces: HashMap<(usize,usize),Vec<(usize,usize)>>, // hashmap of white pieces and their legal squares
    black_pieces: HashMap<(usize,usize),Vec<(usize,usize)>>, // hashmap of white pieces and their legal squares
    white_checking: bool, // whether or not white is giving check
    black_checking: bool, // whether or not black is giving check
}

impl BoardAnal {
    pub fn new(white_pieces: HashMap<(usize,usize),Vec<(usize,usize)>>, black_pieces: HashMap<(usize,usize),Vec<(usize,usize)>>, white_checking: bool, black_checking: bool) -> BoardAnal {
        BoardAnal {
            white_pieces: white_pieces,
            black_pieces: black_pieces,
            white_checking: white_checking,
            black_checking: black_checking
        }
    }

    pub fn pieces(&self, color: Color) -> &HashMap<(usize,usize),Vec<(usize,usize)>> {
        match color {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces
        }
    }

    // if the given color is in check
    pub fn in_check(&self, color: Color) -> bool {
        match color {
            Color::White => self.black_checking,
            Color::Black => self.white_checking
        }
        // reversed because black_checking is if black is *giving check*, and vice versa for white_checking
    }

    // returns all pieces of a given color attacking a square
    pub fn attackers(&self, (file, rank): (usize, usize), color: Color) -> Vec<(usize,usize)> {
        let mut attackers: Vec<(usize,usize)> = Vec::new();
        for piece in self.pieces(color) {
            if piece.1.contains(&(file,rank)) { attackers.push(piece.0.clone()); }
        }
        attackers
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveStatus {
    Ok,
    Illegal,
    Check,
    Checkmate,
    Stalemate
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub squares: [[Option<Piece>; 8]; 8],
    pub to_move: Color,
    pub en_passant_target: Option<(usize, usize)>,
    pub castle_rights: (bool, bool, bool, bool), // (white_kingside, white_queenside, black_kingside, black_queenside)
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Board {
    pub fn new(squares: [[Option<Piece>; 8]; 8], to_move: Color, en_passant_target: Option<(usize, usize)>, castle_rights: (bool, bool, bool, bool), halfmove_clock: u32, fullmove_number: u32) -> Board {
        Board {
            squares: squares,
            to_move: to_move,
            en_passant_target: en_passant_target,
            castle_rights: castle_rights,
            halfmove_clock: halfmove_clock,
            fullmove_number: fullmove_number
        }
    }

    pub fn from_fen(fen: &str) -> Result<Board, String> {
        // Implementation for creating a board from FEN string
        let re = Regex::new(r"^([pnbrqkPNBRQK1-8]+)(/[pnbrqkPNBRQK1-8]+){7} (w|b) ([KQkq]{1,4}|-) ([a-h][36]|-) (\d+) (\d+)$").unwrap();
        if !re.is_match(fen) {
            return Err("Invalid FEN string".into());
        }
        let fields: Vec<&str> = fen.split(' ').collect();
        let mut squares: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];
        let rows: Vec<&str> = fields[0].split('/').rev().collect(); // Reverse the rows to match the board representation, as fen starts from row 8 to row 1
        for (rank, row) in rows.iter().enumerate() {
            let mut file = 0;
            for c in row.chars() {
                if c.is_digit(10) {
                    file += c.to_digit(10).unwrap() as usize;
                } else {
                    let color = if c.is_uppercase() { Color::White } else { Color::Black };
                    let kind = match c.to_ascii_lowercase() {
                        'p' => PieceType::Pawn,
                        'r' => PieceType::Rook,
                        'n' => PieceType::Knight,
                        'b' => PieceType::Bishop,
                        'q' => PieceType::Queen,
                        'k' => PieceType::King,
                        _ => return Err("Invalid piece character".into()),
                    };
                    squares[rank][file] = Some(Piece::new(color, kind));
                    file += 1;
                }
            }
        }
        let to_move = match fields[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid color to move".into()),
        };
        let castle_rights = (
            fields[2].contains('K'),
            fields[2].contains('Q'),
            fields[2].contains('k'),
            fields[2].contains('q'),
        );
        let en_passant_target = if fields[3] == "-" {
            None
        } else {
            let file = fields[3].chars().nth(0).unwrap();
            let rank = fields[3].chars().nth(1).unwrap();
            let file_index = (file as u8 - b'a') as usize;
            let rank_index = (rank as u8 - b'1') as usize;
            Some((file_index, rank_index))
        };
        let halfmove_clock: u32 = fields[4].parse().unwrap();
        let fullmove_number: u32 = fields[5].parse().unwrap();
        Ok(Board::new(squares, to_move, en_passant_target, castle_rights, halfmove_clock, fullmove_number))
    }

    pub fn render_ascii(&self) -> String {
        let mut board_str = String::new();
        for rank in (0..8).rev() {
            board_str.push_str(&format!("{} ", rank + 1));
            for file in 0..8 {
                match self.squares[rank][file] {
                    Some(piece) => {
                        let piece_char = match piece.kind {
                            PieceType::Pawn => 'p',
                            PieceType::Rook => 'r',
                            PieceType::Knight => 'n',
                            PieceType::Bishop => 'b',
                            PieceType::Queen => 'q',
                            PieceType::King => 'k',
                        };
                        let display_char = if piece.color == Color::White {
                            piece_char.to_ascii_uppercase()
                        } else {
                            piece_char
                        };
                        board_str.push(display_char);
                    }
                    None => board_str.push('.'),
                }
                board_str.push(' ');
            }
            board_str.push('\n');
        }
        board_str.push_str("  a b c d e f g h\n");
        board_str
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                match self.squares[rank][file] {
                    Some(piece) => {
                        if empty_count > 0 {
                            fen.push_str(&empty_count.to_string());
                            empty_count = 0;
                        }
                        let piece_char = match piece.kind {
                            PieceType::Pawn => 'p',
                            PieceType::Rook => 'r',
                            PieceType::Knight => 'n',
                            PieceType::Bishop => 'b',
                            PieceType::Queen => 'q',
                            PieceType::King => 'k',
                        };
                        let display_char = if piece.color == Color::White {
                            piece_char.to_ascii_uppercase()
                        } else {
                            piece_char
                        };
                        fen.push(display_char);
                    }
                    None => empty_count += 1,
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }
        fen.push(' ');
        fen.push(if self.to_move == Color::White { 'w' } else { 'b' });
        fen.push(' ');
        let castle_rights_str = format!(
            "{}{}{}{}",
            if self.castle_rights.0 { "K" } else { "" },
            if self.castle_rights.1 { "Q" } else { "" },
            if self.castle_rights.2 { "k" } else { "" },
            if self.castle_rights.3 { "q" } else { "" },
        );
        fen.push_str(if castle_rights_str.is_empty() { "-" } else { &castle_rights_str });
        fen.push(' ');
        if let Some((file, rank)) = self.en_passant_target {
            let file_char = (file as u8 + b'a') as char;
            let rank_char = (rank as u8 + b'1') as char;
            fen.push(file_char);
            fen.push(rank_char);
        } else {
            fen.push('-');
        }
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());
        fen
    }

    pub fn get(&self, (file, rank): (usize, usize)) -> Option<Piece> {
        if file >= 8 || rank >= 8 {
            return None;
        }
        self.squares[rank][file]
    }

    pub fn edit_board(&mut self, square: (usize,usize), piece: Option<Piece>) {
        self.squares[square.1][square.0] = piece;
    }

    // Copies the board and applies a move to the copy, then returns the copy, allows "looking ahead" without editing the board itself. 
    pub fn lookahead(&self, chess_move: ChessMove, update_metadata: bool) -> Result<Board,String> {
        let mut fwd = self.clone();
        if let Some(piece) = fwd.get(chess_move.from) {
            if update_metadata {
                let reset_halfmove = piece.kind == PieceType::Pawn || fwd.get(chess_move.to).is_some();
                fwd.update_metadata(reset_halfmove);
            }
            fwd.edit_board(chess_move.from,None);
            fwd.edit_board(
                chess_move.to, 
                if let Some(ptype) = chess_move.promotion {
                    Some(Piece::new(piece.color,ptype))
                } else {
                    Some(piece)
                });
            Ok(fwd)
        } else {
            Err("No piece at from square".into())
        }
    }

    // returns all legal squares for a given piece, and whether or not this piece is giving check
    fn get_piece_squares(&self,(file, rank): (usize, usize)) -> (Option<Vec<(usize,usize)>>, bool) {
        let piece_opt = self.get((file, rank));
        if piece_opt.is_none() { 
            return (None,false);
        }
        let piece = piece_opt.unwrap();
        let pattern = piece.kind.move_pattern();
        let mut squares: Vec<(usize,usize)> = Vec::new();
        let mut checking: bool = false;
        for &(df, dr, flags) in pattern {
            let mut f = file as i8;
            let mut r = rank as i8;
            // Handle special flags here
            if flags & move_flags::BLACK_ONLY != 0 && piece.color != Color::Black {
                continue;
            }
            if flags & move_flags::WHITE_ONLY != 0 && piece.color != Color::White {
                continue;
            }
            if flags & move_flags::EN_PASSANT != 0 {
                if let Some((ep_file, ep_rank)) = self.en_passant_target {
                    if (f + df) as usize == ep_file && (r + dr) as usize == ep_rank {
                        squares.push((ep_file, ep_rank));
                    }
                }
                continue;
            }
            if flags & move_flags::DOUBLEPAWN != 0 {
                // Check if the pawn has moved before. For simplicity, we will assume that if the pawn is on its starting rank, it hasn't moved.
                let starting_rank = match piece.color {
                    Color::White => 1,
                    Color::Black => 6,
                };
                if rank != starting_rank {
                    continue;
                }
            }
            if flags & move_flags::KINGS_CASTLE != 0 {
                match piece.color {
                    Color::White => if !self.castle_rights.0 { continue; }
                    Color::Black => if !self.castle_rights.2 { continue; }
                }
                if self.get((f as usize + 1, r as usize)).is_some() { continue; } // we only check one square because the other will be checked in the loop
                // This is really chopped lol
            }
            if flags & move_flags::QUEENS_CASTLE != 0 {
                match piece.color {
                    Color::White => if !self.castle_rights.1 { continue; }
                    Color::Black => if !self.castle_rights.3 { continue; }
                }
                if self.get((f as usize - 1, r as usize)).is_some() { continue; } // we only check one square because the other will be checked in the loop
                // This is really chopped lol. We can subtract 1 from usize safely and trust our check in get because we know it'll overflow
            }
            loop {
                f += df;
                r += dr;
                if f < 0 || f >= 8 || r < 0 || r >= 8 {
                    break;
                }
                let target_square = self.get((f as usize, r as usize));
                if let Some(target_piece) = target_square {
                    if target_piece.color != piece.color && flags & move_flags::MOVE_ONLY == 0 {
                        if target_piece.kind == PieceType::King {
                            checking = true;
                        }
                        squares.push((f as usize, r as usize));
                    }
                    break; // Stop if we hit a piece
                } else {
                    if flags & move_flags::CAPTURE_ONLY != 0 {
                        break; // Stop if the move is capture-only and the square is empty
                    }
                    squares.push((f as usize, r as usize));
                }
                if flags & move_flags::NON_REPEATABLE != 0 {
                    break; // Stop if the move is non-repeatable
                }
            }
        }
        (Some(squares),checking)
    }

    // short for "analysis"
    pub fn anal(&self) -> BoardAnal {
        let mut white_pieces: HashMap<(usize,usize),Vec<(usize,usize)>> = HashMap::new();
        let mut black_pieces: HashMap<(usize,usize),Vec<(usize,usize)>> = HashMap::new();
        let mut white_checking: bool = false;
        let mut black_checking: bool = false;
        for (rindex, rank) in self.squares.iter().enumerate() {
            for (findex, piece_opt) in rank.iter().enumerate() {
                if let Some(piece) = piece_opt {
                    let piece_square_result = self.get_piece_squares((findex,rindex));
                    if let Some(piece_squares) = piece_square_result.0 {
                        match piece.color {
                            Color::White => { &mut white_pieces },
                            Color::Black => { &mut black_pieces }
                        }.insert((rindex,findex), piece_squares);
                    }
                    if piece_square_result.1 {
                        match piece.color {
                            Color::White => { white_checking = true },
                            Color::Black => { black_checking = true }
                        }
                    }
                }
            }
        }
        BoardAnal::new(white_pieces,black_pieces,white_checking,black_checking)
    }

    // Applies a transform to a piece on the board. Returns if a piece was captured For internal use only
    fn apply_transform(&mut self, from: (usize,usize), to: (usize,usize)) -> bool {
        let capture = self.get(to).is_some();
        self.squares[to.1][to.0] = self.get(from);
        self.squares[from.1][from.0] = None;
        capture
    }

    // updates metadata for a move (updates halfmove clock, move number, and side to move)
    pub fn update_metadata(&mut self, reset_halfmove: bool) {
        if reset_halfmove {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        if self.to_move == Color::Black { self.fullmove_number += 1 }
        self.to_move = self.to_move.opposite();
    }

    // TODO: Add check logic. This is the main 
    pub fn try_move(&mut self, chess_move: ChessMove) -> Result<MoveStatus, String> {
        let (from_file, from_rank) = chess_move.from;
        let (to_file, to_rank) = chess_move.to;

        if from_file >= 8 || from_rank >= 8 || to_file >= 8 || to_rank >= 8 {
            return Err("Move out of bounds".into());
        }

        let piece_option = self.squares[from_rank][from_file];
        if piece_option.is_none() {
            return Err("No piece at the source square".into());
        }

        let piece = piece_option.unwrap();
        if piece.color != self.to_move {
            return Err("It's not this piece's turn to move".into());
        }

        let anal = self.anal();
        // check if we are in check now
        
        let valid_squares = anal.pieces(self.to_move).get(&chess_move.from).unwrap(); // This should be guaranteed as we already checked that there is a piece on the square

        if !valid_squares.contains(&chess_move.to) {
            return Ok(MoveStatus::Illegal)
        }

        let fwd = self.lookahead(chess_move,true)?;
        let fwdanal = fwd.anal();
        if fwdanal.in_check(self.to_move) {
            return Ok(MoveStatus::Illegal);
        }
        

        // special cases
        match piece.kind {
            PieceType::King => {
                // Check if move is castle
                if chess_move.delta().0.abs() == 2 {
                    let delta = chess_move.delta();
                    let final_attackers = anal.attackers(chess_move.to,self.to_move.opposite());
                    let transit_attackers = anal.attackers(((to_file as i8 + (1 * (delta.0 / 2))) as usize,to_rank),self.to_move.opposite());
                    if final_attackers.len() > 0 || transit_attackers.len() > 0 {
                        return Ok(MoveStatus::Illegal)
                    }
                    match delta.0 {
                        // kingside
                        2 => {
                            self.apply_transform(chess_move.from,chess_move.to);
                            self.apply_transform((chess_move.from.0 + 3, chess_move.from.1),(chess_move.to.0 - 1, chess_move.to.1));
                            match self.to_move {
                                Color::White => { self.castle_rights.0 = false; }
                                Color::Black => { self.castle_rights.2 = false; }
                            }
                            return Ok(MoveStatus::Ok);
                        },
                        // queenside
                        -2 => {
                            self.apply_transform(chess_move.from,chess_move.to);
                            self.apply_transform((chess_move.from.0 - 4, chess_move.from.1),(chess_move.to.0 + 1, chess_move.to.1));
                            match self.to_move {
                                Color::White => { self.castle_rights.1 = false; }
                                Color::Black => { self.castle_rights.3 = false; }
                            }
                            return Ok(MoveStatus::Ok);
                        }
                        _ => {
                            return Err("How does your castle have your king moving not two files bro".into()); // Again, this is impossible to reach
                        }
                    }
                }
                // normal case
                self.apply_transform(chess_move.from, chess_move.to);
            },
            PieceType::Pawn => {
                // En passant
                if let Some(target) = self.en_passant_target {
                    if chess_move.to.eq(&target) {

                    }
                }
            }, 
            _ => {
            }
        }

        // Normal case
        // Ensure move doesn't put king in check 
        self.apply_transform(chess_move.from, chess_move.to);

        Ok(MoveStatus::Ok)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_fen());
    }
}