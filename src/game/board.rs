// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

use crate::game::{Color, Movable, Piece, PieceType, move_flags};
use regex::Regex;
use std::fmt;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessMove {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub promotion: Option<PieceType>,
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
}

// board analysis
pub struct BoardAnal {
    pub pieces: HashMap<(usize,usize),Vec<(usize,usize)>>, // hashmap of pieces and their legal squares
    pub checking_pieces: HashMap<(usize,usize),Vec<(usize,usize)>> // hashmap of checking pieces and squares that would block their checks
}

impl BoardAnal {
    pub fn new() -> BoardAnal {
        BoardAnal {
            pieces: HashMap::new(),
            checking_pieces: HashMap::new()
        }
    }

    // returns all pieces attacking a square
    pub fn attackers(&self, (file, rank): (usize, usize)) -> Vec<(usize,usize)> {
        let mut attackers: Vec<(usize,usize)> = Vec::new();
        for piece in self.pieces.iter() {
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

    // returns all legal squares for a given piece, and a list of all squares that could block check if this piece is giving check
    pub fn get_piece_squares(&self,(file, rank): (usize, usize)) -> (Option<Vec<(usize,usize)>>, Option<Vec<(usize,usize)>>) {
        let piece_opt = self.get((file, rank));
        if piece_opt.is_none() { 
            return (None,None);
        }
        let piece = piece_opt.unwrap();
        let pattern = piece.kind.move_pattern();
        let mut squares: Vec<(usize,usize)> = Vec::new();
        let mut blocking_squares: Option<Vec<(usize,usize)>> = None;
        for &(df, dr, flags) in pattern {
            let mut f = file as i8;
            let mut r = rank as i8;
            // Handle special flags here
            if flags & move_flags::EN_PASSANT != 0 {
                if let Some((ep_file, ep_rank)) = self.en_passant_target {
                    if (f + df) as usize == ep_file && (r + dr) as usize == ep_rank {
                        squares.push((ep_file, ep_rank));
                    }
                }
                continue;
            }
            if flags & move_flags::BLACK_ONLY != 0 && piece.color != Color::Black {
                continue;
            }
            if flags & move_flags::WHITE_ONLY != 0 && piece.color != Color::White {
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
            let mut blocking_squares_temp: Vec<(usize,usize)> = Vec::new();
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
                            blocking_squares = Some(blocking_squares_temp);
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
                blocking_squares_temp.push((f as usize, r as usize));
                if flags & move_flags::NON_REPEATABLE != 0 {
                    break; // Stop if the move is non-repeatable
                }
            }
        }
        (Some(squares),blocking_squares)
    }

    // short for "analysis"
    pub fn anal(&self) -> BoardAnal {
        let mut anal: BoardAnal = BoardAnal::new();
        for (rindex, rank) in self.squares.iter().enumerate() {
            for (findex, piece) in rank.iter().enumerate() {
                if piece.is_some() {
                    let piece_square_result = self.get_piece_squares((findex,rindex));
                    if let Some(piece_squares) = piece_square_result.0 {
                        anal.pieces.insert((rindex,findex), piece_squares);
                    }
                    if let Some(checking_squares) = piece_square_result.1 {
                        anal.checking_pieces.insert((rindex, findex), checking_squares);
                    }
                }
            }
        }
        anal
    }

    pub fn try_move(&mut self, chess_move: ChessMove) -> Result<MoveStatus, String> {
        let anal = self.anal();
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

        // For simplicity, we will just move the piece without checking legality
        self.squares[to_rank][to_file] = Some(piece);
        self.squares[from_rank][from_file] = None;

        // Update the turn
        self.to_move = match self.to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        Ok(MoveStatus::Ok)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_fen());
    }
}