// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

use std::fmt;

// MOVE FLAGS
pub mod move_flags {
    pub const NONE: u16 =           0b0000000000000000; // No special flags
    pub const NON_REPEATABLE: u16 = 0b0000000000000001; // If a move pattern is not repeatable (e.g. a knight's or king's move)
    pub const R8_PROMOTION: u16 =   0b0000000000000010; // If a move pattern can result in a promotion on rank 8 (e.g. a pawn's forward move). Unused for now
    pub const R1_PROMOTION: u16 =   0b0000000000000100; // If a move pattern can result in a promotion on rank 1 (e.g. a pawn's forward move). Unused for now
    pub const DOUBLEPAWN: u16 =     0b0000000000001000; // If a move pattern is a double pawn move
    pub const MOVE_ONLY: u16 =      0b0000000000010000; // If a move pattern can only be used to move to an empty square (e.g. a pawn's forward move)
    pub const CAPTURE_ONLY: u16 =   0b0000000000100000; // If a move pattern can only be used to capture an enemy piece (e.g. a pawn's diagonal capture)
    pub const KINGS_CASTLE: u16 =   0b0000000001000000; // If a move pattern is a king's castle
    pub const QUEENS_CASTLE: u16 =  0b0000000010000000; // If a move pattern is a queen's castle
    pub const EN_PASSANT: u16 =     0b0000000100000000; // If a move pattern is an en passant capture (e.g. a pawn's en passant capture)
    pub const WHITE_ONLY: u16 =     0b0000001000000000; // If a move pattern can only be used by white pieces (e.g. a pawn's forward move)
    pub const BLACK_ONLY: u16 =     0b0000010000000000; // If a move pattern can only be used by black pieces (e.g. a pawn's forward move)
}

pub mod move_patterns {
    use super::move_flags::*;
    pub const PAWN_MOVES: [(i8,i8,u16); 12] = [
        ( 0,  1, MOVE_ONLY | R8_PROMOTION | NON_REPEATABLE | WHITE_ONLY),
        ( 0, -1, MOVE_ONLY | R1_PROMOTION | NON_REPEATABLE | BLACK_ONLY),
        ( 1,  1, CAPTURE_ONLY | R8_PROMOTION | NON_REPEATABLE | WHITE_ONLY),
        (-1, -1, CAPTURE_ONLY | R1_PROMOTION | NON_REPEATABLE | BLACK_ONLY),
        ( 0,  2, MOVE_ONLY | WHITE_ONLY | DOUBLEPAWN | NON_REPEATABLE),
        ( 0, -2, MOVE_ONLY | BLACK_ONLY | DOUBLEPAWN | NON_REPEATABLE),
        (-1,  1, CAPTURE_ONLY | WHITE_ONLY | R8_PROMOTION | NON_REPEATABLE),
        ( 1, -1, CAPTURE_ONLY | BLACK_ONLY | R1_PROMOTION | NON_REPEATABLE),
        ( 1,  1, EN_PASSANT | WHITE_ONLY | R8_PROMOTION | NON_REPEATABLE),
        (-1,  1, EN_PASSANT | WHITE_ONLY | R8_PROMOTION | NON_REPEATABLE),
        ( 1, -1, EN_PASSANT | BLACK_ONLY | R1_PROMOTION | NON_REPEATABLE),
        (-1, -1, EN_PASSANT | BLACK_ONLY | R1_PROMOTION | NON_REPEATABLE)
    ];

    pub const ROOK_MOVES: [(i8,i8,u16); 4] = [
        ( 0,  1, NONE),
        ( 0, -1, NONE),
        ( 1,  0, NONE),
        (-1,  0, NONE)
    ];

    pub const KNIGHT_MOVES: [(i8,i8,u16); 8] = [
        ( 2,  1, NON_REPEATABLE),
        ( 2, -1, NON_REPEATABLE),
        (-2,  1, NON_REPEATABLE),
        (-2, -1, NON_REPEATABLE),
        ( 1,  2, NON_REPEATABLE),
        ( 1, -2, NON_REPEATABLE),
        (-1,  2, NON_REPEATABLE),
        (-1, -2, NON_REPEATABLE)
    ];

    pub const BISHOP_MOVES: [(i8,i8,u16); 4] = [
        ( 1,  1, NONE),
        ( 1, -1, NONE),
        (-1,  1, NONE),
        (-1, -1, NONE)
    ];

    pub const QUEEN_MOVES: [(i8,i8,u16); 8] = [
        ( 1,  0, NONE),
        (-1,  0, NONE),
        ( 0,  1, NONE),
        ( 0, -1, NONE),
        ( 1,  1, NONE),
        ( 1, -1, NONE),
        (-1,  1, NONE),
        (-1, -1, NONE)
    ];

    pub const KING_MOVES: [(i8,i8,u16); 10] = [
        ( 1,  0, NON_REPEATABLE),
        (-1,  0, NON_REPEATABLE),
        ( 0,  1, NON_REPEATABLE),
        ( 0, -1, NON_REPEATABLE),
        ( 1,  1, NON_REPEATABLE),
        ( 1, -1, NON_REPEATABLE),
        (-1,  1, NON_REPEATABLE),
        (-1, -1, NON_REPEATABLE),
        ( 2,  0, KINGS_CASTLE | NON_REPEATABLE | MOVE_ONLY),
        (-2,  0, QUEENS_CASTLE | NON_REPEATABLE | MOVE_ONLY)
    ];
}
// A move is represented as a tuple of (rank_delta, file_delta, flags). The rank and file deltas are relative to the piece's current position. The flags are a bitfield that describes the move's properties.
pub trait Movable {
    fn move_pattern(&self) -> &'static [(i8, i8, u16)];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King
}

impl Movable for PieceType {
    fn move_pattern(&self) -> &'static [(i8, i8, u16)] {
        match self {
            PieceType::Pawn => &move_patterns::PAWN_MOVES,
            PieceType::Rook => &move_patterns::ROOK_MOVES,
            PieceType::Knight => &move_patterns::KNIGHT_MOVES,
            PieceType::Bishop => &move_patterns::BISHOP_MOVES,
            PieceType::Queen => &move_patterns::QUEEN_MOVES,
            PieceType::King => &move_patterns::KING_MOVES
        }
    }
}

impl PieceType {
    pub fn what(&self) -> &str {
        match self {
            PieceType::Pawn => "Pawn",
            PieceType::Rook => "Rook",
            PieceType::Knight => "Knight",
            PieceType::Bishop => "Bishop",
            PieceType::Queen => "Queen",
            PieceType::King => "King",
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.what());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
    pub fn what(&self) -> &str {
        match self {
            Color::White => "White",
            Color::Black => "Black"
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.what());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceType
}

impl Piece {
    pub fn new(color: Color, kind: PieceType) -> Piece {
        Piece {
            color: color,
            kind: kind
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{} {}", self.color, self.kind);
    }
}

