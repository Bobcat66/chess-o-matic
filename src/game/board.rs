// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

// TODO: Rewrite board to be immutable

use crate::game::{Color, Movable, Piece, PieceType, move_flags};
use regex::Regex;
use std::fmt;
use std::collections::{HashMap,HashSet};

// file, rank
pub type Square = (usize,usize);
pub type RawMove = (Square,Square); // Todo, switch apply_transform to this???
// stands for Raw Analysis and nothing else
type RawAnal = (HashMap<Square,Vec<Square>>,HashMap<Square,Vec<Square>>); // (white moves, black moves)


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceType>
}

impl ChessMove {
    pub fn new(from: Square, to: Square, promotion: Option<PieceType>) -> ChessMove {
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

fn raw_pieces(raw: &RawAnal, color: Color) -> &HashMap<Square,Vec<Square>> {
    match color {
        Color::White => &raw.0,
        Color::Black => &raw.1,
    }
}

// board analysis
pub struct BoardAnal {
    pub raw: RawAnal,
    pub legal_moves: Vec<ChessMove>, // legal moves
    pub board: Board
}

impl BoardAnal {
    pub fn new(raw: RawAnal, legal_moves: Vec<ChessMove>, board: Board) -> BoardAnal {
        BoardAnal {
            raw: raw,
            legal_moves: legal_moves,
            board: board
        }
    }

    pub fn pieces(&self, color: Color) -> &HashMap<Square,Vec<Square>> { raw_pieces(&self.raw,color) }

    pub fn is_legal(&self, chess_move: ChessMove) -> Result<(),MoveStatus> {
        let legal = self.legal_moves.contains(&chess_move);
        if legal {
            return Ok(());
        }
        move_validate(&self.board, chess_move)?;
        move_status(&self.board, &self.raw, chess_move)
    }

    pub fn board_status(&self) -> BoardStatus {
        let check = self.board.checking_pieces(self.board.to_move).len() > 0;
        let mate = self.legal_moves.len() == 0;
        if check && mate { return BoardStatus::Checkmate; }
        if check && !mate { return BoardStatus::Check; }
        if !check && mate { return BoardStatus::Stalemate; }
        return BoardStatus::Ok
    }

    pub fn lookahead(&self, chess_move: ChessMove) -> Result<BoardAnal,MoveStatus> {
        let board = self.board.lookahead(chess_move)?;
        return Ok(board.anal());
    }

    pub fn captures(&self, color: Color) -> Vec<ChessMove> {
        let mut captures: Vec<ChessMove> = Vec::new();
        let mut enemy_positions: HashSet<Square> = self.pieces(color.opposite()).keys().copied().collect();
        if let Some(square) = self.board.en_passant_target {
            enemy_positions.insert(square);
        }
        for (start_square,move_list) in self.pieces(color).iter() {
            let move_set: HashSet<Square> = move_list.into_iter().copied().collect();
            for capture_square in move_set.intersection(&enemy_positions) {
                if let Some(piece) = self.board.get(*start_square) {
                    if piece.kind == PieceType::Pawn && (capture_square.1 == match piece.color {
                        Color::White => 7,
                        Color::Black => 0
                    }) {
                        // check legality 
                        if self.is_legal(ChessMove::new(*start_square,*capture_square,Some(PieceType::Queen))).is_ok() {
                            captures.push(ChessMove::new(*start_square,*capture_square,Some(PieceType::Queen)));
                            captures.push(ChessMove::new(*start_square,*capture_square,Some(PieceType::Rook)));
                            captures.push(ChessMove::new(*start_square,*capture_square,Some(PieceType::Bishop)));
                            captures.push(ChessMove::new(*start_square,*capture_square,Some(PieceType::Knight)));
                        }
                    } else {
                        let mv = ChessMove::new(*start_square,*capture_square,None);
                        if self.is_legal(mv).is_ok() {
                            captures.push(ChessMove::new(*start_square,*capture_square,None));
                        }
                    }
                }
            }
        }
        captures
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveStatus {
    Illegal,
    IllegalInCheck,
    IllegalCastleBlocked,
    IllegalNoPromotionNominated,
    Invalid,
    InvalidEmptySquare,
    InvalidWrongTurn,
    InvalidOutOfBounds
}

impl MoveStatus {
    pub fn what(&self) -> &str {
        match self {
            MoveStatus::Illegal => "Illegal",
            MoveStatus::IllegalInCheck => "Illegal: In Check",
            MoveStatus::IllegalCastleBlocked => "Illegal: Castle Blocked",
            MoveStatus::IllegalNoPromotionNominated => "Illegal: No Promotion Nominated",
            MoveStatus::Invalid => "Invalid",
            MoveStatus::InvalidEmptySquare => "Invalid: Attempted to move from empty square",
            MoveStatus::InvalidWrongTurn => "Invalid: Not this piece's turn",
            MoveStatus::InvalidOutOfBounds => "Invalid: Attempted to move out of bounds"
        }
    }
}

impl fmt::Display for MoveStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.what());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardStatus {
    Ok,
    Check,
    Checkmate,
    Stalemate
}

impl BoardStatus {
    pub fn what(&self) -> &str {
        match self {
            BoardStatus::Ok => "Ok",
            BoardStatus::Check => "Check",
            BoardStatus::Checkmate => "Checkmate",
            BoardStatus::Stalemate => "Stalemate"
        }
    }
    pub fn terminal(&self) -> bool {
        match self {
            BoardStatus::Ok => false,
            BoardStatus::Check => false,
            BoardStatus::Checkmate => true,
            BoardStatus::Stalemate => true
        }
    }
}

impl fmt::Display for BoardStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.what());
    }
}


// Claude slop
pub struct ZobristKeys {
    piece_square: [[[u64; 64]; 6]; 2],  // [color][piece_type][square_index]
    black_to_move: u64,
    castle_rights: [u64; 4],            // one per right: WK, WQ, BK, BQ
    en_passant_file: [u64; 8],          // one per file — only file matters, not rank
}

impl ZobristKeys {
    pub fn new() -> Self {
        // seeded RNG so keys are reproducible across runs — important if you ever
        // want two instances of your engine to agree on hash values (e.g. for testing)
        use rand::{SeedableRng, Rng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xC0FFEE);
        
        let mut piece_square = [[[0u64; 64]; 6]; 2];
        for color in 0..2 {
            for piece in 0..6 {
                for sq in 0..64 {
                    piece_square[color][piece][sq] = rng.next_u64();
                }
            }
        }
        
        ZobristKeys {
            piece_square,
            black_to_move: rng.next_u64(),
            castle_rights: [rng.next_u64(), rng.next_u64(), rng.next_u64(), rng.next_u64()],
            en_passant_file: std::array::from_fn(|_| rng.next_u64()),
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub squares: [[Option<Piece>; 8]; 8],
    pub to_move: Color,
    pub en_passant_target: Option<Square>,
    pub castle_rights: (bool, bool, bool, bool), // (white_kingside, white_queenside, black_kingside, black_queenside)
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

fn move_validate(board: &Board, chess_move: ChessMove) -> Result<(),MoveStatus> {
    let (from_file, from_rank) = chess_move.from;
    let (to_file, to_rank) = chess_move.to;

    if from_file >= 8 || from_rank >= 8 || to_file >= 8 || to_rank >= 8 {
        return Err(MoveStatus::InvalidOutOfBounds);
    }

    let piece_option = board.squares[from_rank][from_file];
    if piece_option.is_none() {
        return Err(MoveStatus::InvalidEmptySquare);
    }

    let piece = piece_option.unwrap();
    if piece.color != board.to_move {
        return Err(MoveStatus::InvalidWrongTurn);
    }
    return Ok(());
}

fn move_status(board: &Board, anal: &RawAnal, chess_move: ChessMove) -> Result<(), MoveStatus> {

    // check if we are in check now
    let valid_squares = raw_pieces(anal,board.to_move).get(&chess_move.from).unwrap(); // This should be guaranteed as we already checked that there is a piece on the square

    let piece_opt= board.get(chess_move.from);
    if piece_opt.is_none() {
        return Err(MoveStatus::InvalidEmptySquare);
    }
    let piece = piece_opt.unwrap();

    if !valid_squares.contains(&chess_move.to) {
        return Err(MoveStatus::Illegal);
    }

    let fwd = board.lookahead(chess_move)?;
    if fwd.checking_pieces(board.to_move).len() > 0 {
        return Err(MoveStatus::IllegalInCheck);
    }
    // Castling logic because castling is a special boy that needs its own logic
    if chess_move.delta().0.abs() == 2 && piece.kind == PieceType::King {
        if board.checking_pieces(board.to_move).len() > 0 {
            return Err(MoveStatus::IllegalInCheck);
        }
        let delta = chess_move.delta();
        let final_attackers = board.attacking_pieces(chess_move.to,board.to_move.opposite());
        let transit_attackers = board.attacking_pieces(((chess_move.to.0 as i8 + (1 * (delta.0 / 2))) as usize,chess_move.to.1),board.to_move.opposite());
        if final_attackers.len() > 0 || transit_attackers.len() > 0 {
            return Err(MoveStatus::IllegalCastleBlocked)
        }
    }
    // Promotion logic
    // Behold the boolean statement of doom and despair
    if piece.kind == PieceType::Pawn && (chess_move.to.1 == match piece.color {
        Color::White => 7,
        Color::Black => 0
    }) && chess_move.promotion.is_none() {
        return Err(MoveStatus::IllegalNoPromotionNominated);
    }
    
    return Ok(());
}

// legal moves for the color to move
fn legal_moves(board: &Board, anal: &RawAnal) -> Vec<ChessMove> {
    let mut legal_moves: Vec<ChessMove> = Vec::new();
    for (pos,move_list) in raw_pieces(anal,board.to_move).iter() {
        let piece_opt = board.get(*pos);
        if piece_opt.is_none() {
            continue;
        }
        let piece = piece_opt.unwrap();
        for end_pos in move_list {
            if piece.kind == PieceType::Pawn && (end_pos.1 == match piece.color {
                Color::White => 7,
                Color::Black => 0
            }) {
                for piece_type in [PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen] {
                    let chess_move = ChessMove::new(*pos,*end_pos,Some(piece_type));
                    if move_status(board, anal, chess_move).is_ok() {
                        legal_moves.push(chess_move);
                    }
                } 
            } else {
                let chess_move = ChessMove::new(*pos,*end_pos,None);
                if move_status(board, anal, chess_move).is_ok() {
                    legal_moves.push(chess_move);
                }
            }
        }
    }
    return legal_moves;
}

impl Board {
    pub fn new(squares: [[Option<Piece>; 8]; 8], to_move: Color, en_passant_target: Option<Square>, castle_rights: (bool, bool, bool, bool), halfmove_clock: u32, fullmove_number: u32) -> Board {
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

    pub fn get(&self, (file, rank): Square) -> Option<Piece> {
        if file >= 8 || rank >= 8 {
            return None;
        }
        self.squares[rank][file]
    }

    pub fn edit_board(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square.1][square.0] = piece;
    }

    // Copies the board and applies a move to the copy, then returns the copy, allows "looking ahead" without editing the board itself. 
    pub fn lookahead(&self, chess_move: ChessMove) -> Result<Board,MoveStatus> {
        let mut fwd = self.clone();
        if fwd.get(chess_move.from).is_some() {
            fwd.apply_move(chess_move)?;
            Ok(fwd)
        } else {
            Err(MoveStatus::InvalidEmptySquare)
        }
    }

    // calculates pieces of a given color attacking a given square
    pub fn attacking_pieces(&self,(file,rank): Square, color: Color) -> Vec<Square> {
        // Project rays out to find attackers
        let mut attackers: Vec<Square> = Vec::new();
        for (df,dr) in [(1,0),(0,1),(1,1),(-1,0),(0,-1),(-1,-1),(-1,1),(1,-1)] {
            let mut f = file as i8;
            let mut r = rank as i8;
            let mut distance = 0;
            loop {
                f += df;
                r += dr;
                distance += 1;
                if f < 0 || f >= 8 || r < 0 || r >= 8 { break; }
                if let Some(piece) = self.get((f as usize, r as usize)) {
                    if piece.color == color {
                        let is_diagonal = df != 0 && dr != 0;
                        let attacks = match piece.kind {
                            PieceType::Queen => true,
                            PieceType::Rook => !is_diagonal,
                            PieceType::Bishop => is_diagonal,
                            PieceType::Pawn => {
                                // pawns only attack one square diagonally, forward relative to their own color
                                distance == 1 && is_diagonal &&
                                ((piece.color == Color::White && dr == -1) || (piece.color == Color::Black && dr == 1))
                            }
                            PieceType::King => distance == 1, // adjacent king "attacks" for check-detection purposes
                            _ => false,
                        };
                        if attacks { attackers.push((f as usize, r as usize)); }
                    }
                    break;
                }
            }
        }
        // Check for knights
        for (df,dr) in [(2, 1),(2,-1),(-2,1),(-2,-1),(1,2),(1,-2),(-1,2),(-1,-2)] {
            let f = file as i8 + df;
            let r = rank as i8 + dr;
            if f < 0 || f >= 8 || r < 0 || r >= 8 {
                continue;
            }
            if let Some(piece) = self.get((f as usize,r as usize)) {
                if piece.color == color && piece.kind == PieceType::Knight {
                    attackers.push((f as usize, r as usize));
                }
            }
        }
        attackers
    }

    // Calculates pieces that are giving check to the given color
    pub fn checking_pieces(&self,color: Color) -> Vec<Square> {
        let mut file: usize = 0;
        let mut rank: usize = 0;
        let mut found: bool = false;
        // Search for the king
        'search: for rindex in 0..8 {
            for findex in 0..8 {
                if let Some(piece) = self.get((findex,rindex)) {
                    if piece.kind == PieceType::King && piece.color == color {
                        file = findex;
                        rank = rindex;
                        found = true;
                        break 'search;
                    }
                }
            }
        }
        // No king found, therefore no pieces can be giving check. New chess strategy dropped?
        if !found { return Vec::new(); }
        self.attacking_pieces((file,rank), color.opposite())
    }

    // returns all legal squares for a given piece (ignoring checks), and whether or not this piece is giving check
    fn get_piece_squares(&self,(file, rank): Square) -> Option<Vec<Square>> {
        let piece_opt = self.get((file, rank));
        if piece_opt.is_none() { 
            return None;
        }
        let piece = piece_opt.unwrap();
        let pattern = piece.kind.move_pattern();
        let mut squares: Vec<Square> = Vec::new();
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
        Some(squares)
    }

    // short for "analysis"
    pub fn anal(&self) -> BoardAnal {
        let mut raw: RawAnal = (HashMap::new(),HashMap::new());
        for (rindex, rank) in self.squares.iter().enumerate() {
            for (findex, piece_opt) in rank.iter().enumerate() {
                if let Some(piece) = piece_opt {
                    let piece_square_result = self.get_piece_squares((findex,rindex));
                    if let Some(piece_squares) = piece_square_result {
                        match piece.color {
                            Color::White => { &mut raw.0 },
                            Color::Black => { &mut raw.1 }
                        }.insert((findex,rindex), piece_squares);
                    }
                }
            }
        }
        let legal = legal_moves(self, &raw);
        BoardAnal::new(raw,legal,self.clone())
    }

    fn apply_transform_impl(&mut self, from: Square, to: Square, piece: Option<Piece>) -> bool {
        let capture = self.get(to).is_some();
        self.squares[to.1][to.0] = piece;
        self.squares[from.1][from.0] = None;
        capture
    }

    fn apply_transform(&mut self, from: Square, to: Square) -> bool {
        self.apply_transform_impl(from, to, self.get(from))
    }

    fn apply_move_impl(&mut self, chess_move: ChessMove) -> bool {
        // Verifies castling rights
        // yes it checks every move shut up
        match chess_move.from {
            (7,0) => { self.castle_rights.0 = false; }, // revoke white kingside castling
            (0,0) => { self.castle_rights.1 = false; }, // revoke white queenside castling
            (7,7) => { self.castle_rights.2 = false; }, // revoke black kingside castling
            (0,7) => { self.castle_rights.3 = false; }, // revoke black queenside castling
            _ => {}
        }
        match chess_move.to {
            (7,0) => { self.castle_rights.0 = false; }, // revoke white kingside castling
            (0,0) => { self.castle_rights.1 = false; }, // revoke white queenside castling
            (7,7) => { self.castle_rights.2 = false; }, // revoke black kingside castling
            (0,7) => { self.castle_rights.3 = false; }, // revoke black queenside castling
            _ => {}
        }
        self.apply_transform_impl(
            chess_move.from, 
            chess_move.to, 
            if let Some(ptype) = chess_move.promotion {
                Some(Piece::new(self.to_move,ptype))
            } else {
                self.get(chess_move.from)
            }
        )
    }

    // updates metadata for a move (updates halfmove clock, move number, and side to move)
    pub fn update_metadata(&mut self, reset_halfmove: bool, en_passant_target: Option<Square>) {
        if reset_halfmove {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        if self.to_move == Color::Black { self.fullmove_number += 1 }
        self.to_move = self.to_move.opposite();
        self.en_passant_target = en_passant_target;
    }

    pub fn apply_move(&mut self, chess_move: ChessMove) -> Result<(),MoveStatus> {
        // duplication ew
        let piece_option = self.get(chess_move.from);
        if piece_option.is_none() {
            return Err(MoveStatus::InvalidEmptySquare);
        }
        let piece = piece_option.unwrap();

        match piece.kind {
            PieceType::King => {
                let delta = chess_move.delta();
                // Check if move is castle
                if delta.0.abs() == 2 {
                    // TODO: move this into a variable so it isn't uselessly recalcula
                    match delta.0 {
                        // kingside
                        2 => {
                            self.apply_move_impl(chess_move);
                            self.apply_transform((chess_move.from.0 + 3, chess_move.from.1),(chess_move.to.0 - 1, chess_move.to.1));
                            match self.to_move {
                                Color::White => { self.castle_rights.0 = false; self.castle_rights.1 = false}
                                Color::Black => { self.castle_rights.2 = false; self.castle_rights.3 = false}
                            }
                            self.update_metadata(false, None);
                            return Ok(());
                        },
                        // queenside
                        -2 => {
                            self.apply_move_impl(chess_move);
                            self.apply_transform((chess_move.from.0 - 4, chess_move.from.1),(chess_move.to.0 + 1, chess_move.to.1));
                            match self.to_move {
                                Color::White => { self.castle_rights.0 = false; self.castle_rights.1 = false}
                                Color::Black => { self.castle_rights.2 = false; self.castle_rights.3 = false}
                            }
                            self.update_metadata(false, None);
                            return Ok(());
                        }
                        _ => {
                            return Err(MoveStatus::Invalid); // Again, this is impossible to reach
                        }
                    }
                }
                // normal case
                let captured = self.apply_move_impl(chess_move);
                // revoke castling rights
                match self.to_move {
                    Color::White => { self.castle_rights.0 = false; self.castle_rights.1 = false}
                    Color::Black => { self.castle_rights.2 = false; self.castle_rights.3 = false}
                }
                self.update_metadata(captured, None);
                return Ok(())
            },
            PieceType::Pawn => {
                // En passant
                if let Some(target) = self.en_passant_target {
                    if chess_move.to.eq(&target) {
                        self.edit_board((target.0,(target.1 as i8 + match self.to_move {
                            Color::White => -1,
                            Color::Black => 1
                        }) as usize), None);
                        self.apply_move_impl(chess_move);
                        self.update_metadata(true, None);
                        return Ok(());
                    }
                }
                // Normal case
                self.apply_move_impl(chess_move);
                // handle en passant
                if chess_move.delta().1.abs() == 2 {
                    self.update_metadata(true,Some((chess_move.to.0,(chess_move.to.1 as i8 + match self.to_move {
                            Color::White => -1,
                            Color::Black => 1
                        }) as usize)));
                } else {
                    self.update_metadata(true,None);
                }
                return Ok(());
            }, 
            PieceType::Rook => {
                let captured = self.apply_move_impl(chess_move);
                self.update_metadata(captured, None);
                return Ok(());
            }
            _ => {
                let captured = self.apply_move_impl(chess_move);
                self.update_metadata(captured, None);
                return Ok(());
            }
        }
    }

    pub fn try_move(&mut self, anal: &BoardAnal, chess_move: ChessMove) -> Result<(), MoveStatus> {
        anal.is_legal(chess_move)?;
        self.apply_move(chess_move)
    }

    // Claude slop
    pub fn zobrist_hash(&self, keys: &ZobristKeys) -> u64 {
        let mut hash: u64 = 0;

        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = self.squares[rank][file] {
                    let color_idx = match piece.color {
                        Color::White => 0,
                        Color::Black => 1,
                    };
                    let piece_idx = match piece.kind {
                        PieceType::Pawn => 0,
                        PieceType::Knight => 1,
                        PieceType::Bishop => 2,
                        PieceType::Rook => 3,
                        PieceType::Queen => 4,
                        PieceType::King => 5,
                    };
                    let sq_idx = rank * 8 + file;
                    hash ^= keys.piece_square[color_idx][piece_idx][sq_idx];
                }
            }
        }

        if self.to_move == Color::Black {
            hash ^= keys.black_to_move;
        }

        if self.castle_rights.0 { hash ^= keys.castle_rights[0]; }
        if self.castle_rights.1 { hash ^= keys.castle_rights[1]; }
        if self.castle_rights.2 { hash ^= keys.castle_rights[2]; }
        if self.castle_rights.3 { hash ^= keys.castle_rights[3]; }

        if let Some((file, _rank)) = self.en_passant_target {
            hash ^= keys.en_passant_file[file];
        }

        hash
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_fen());
    }
}