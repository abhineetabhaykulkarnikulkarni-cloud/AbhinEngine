// movegen.rs — Legal move generation

use crate::board::{Board, Color, Move, Piece, ColoredPiece, opposite};

pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = generate_pseudo_legal(board);
    // Filter out moves that leave king in check
    moves.retain(|&mv| {
        let mut b = board.clone();
        b.make_move(mv);
        let king_sq = b.find_king(board.side);
        let legal = king_sq.map(|sq| !b.is_attacked(sq, opposite(board.side))).unwrap_or(false);
        legal
    });
    moves
}

pub fn generate_captures(board: &Board) -> Vec<Move> {
    generate_moves(board).into_iter().filter(|m| m.captured.is_some() || m.is_ep).collect()
}

fn generate_pseudo_legal(board: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(50);

    for from in 0u8..64 {
        let Some(cp) = board.squares[from as usize] else { continue };
        if cp.color != board.side { continue; }

        match cp.piece {
            Piece::Pawn   => gen_pawn_moves(board, from, cp.color, &mut moves),
            Piece::Knight => gen_leaper_moves(board, from, cp.color, &KNIGHT_DELTAS, &mut moves),
            Piece::Bishop => gen_slider_moves(board, from, cp.color, &BISHOP_DIRS, &mut moves),
            Piece::Rook   => gen_slider_moves(board, from, cp.color, &ROOK_DIRS, &mut moves),
            Piece::Queen  => {
                gen_slider_moves(board, from, cp.color, &BISHOP_DIRS, &mut moves);
                gen_slider_moves(board, from, cp.color, &ROOK_DIRS, &mut moves);
            }
            Piece::King   => {
                gen_leaper_moves(board, from, cp.color, &KING_DELTAS, &mut moves);
                gen_castling(board, from, cp.color, &mut moves);
            }
        }
    }
    moves
}

const KNIGHT_DELTAS: [(i32,i32);8] = [(-2,-1),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)];
const KING_DELTAS:   [(i32,i32);8] = [(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)];
const BISHOP_DIRS:   [(i32,i32);4] = [(-1,-1),(-1,1),(1,-1),(1,1)];
const ROOK_DIRS:     [(i32,i32);4] = [(-1,0),(1,0),(0,-1),(0,1)];

fn gen_pawn_moves(board: &Board, from: u8, color: Color, moves: &mut Vec<Move>) {
    let dir: i32 = if color == Color::White { 1 } else { -1 };
    let start_rank = if color == Color::White { 1 } else { 6 };
    let promo_rank  = if color == Color::White { 7 } else { 0 };

    let fr = (from / 8) as i32;
    let ff = (from % 8) as i32;

    // Single push
    let tr = fr + dir;
    if tr >= 0 && tr < 8 {
        let to = (tr * 8 + ff) as u8;
        if board.squares[to as usize].is_none() {
            if tr == promo_rank {
                for &promo in &[Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                    moves.push(Move { from, to, promotion: Some(promo), captured: None, is_ep: false, is_castle: false });
                }
            } else {
                moves.push(Move { from, to, promotion: None, captured: None, is_ep: false, is_castle: false });
                // Double push
                if fr == start_rank {
                    let tr2 = fr + dir * 2;
                    let to2 = (tr2 * 8 + ff) as u8;
                    if board.squares[to2 as usize].is_none() {
                        moves.push(Move { from, to: to2, promotion: None, captured: None, is_ep: false, is_castle: false });
                    }
                }
            }
        }
    }

    // Captures
    for df in [-1i32, 1] {
        let tf = ff + df;
        let tr = fr + dir;
        if tf < 0 || tf >= 8 || tr < 0 || tr >= 8 { continue; }
        let to = (tr * 8 + tf) as u8;

        // Normal capture
        if let Some(target) = board.squares[to as usize] {
            if target.color != color {
                if tr == promo_rank {
                    for &promo in &[Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                        moves.push(Move { from, to, promotion: Some(promo), captured: Some(target.piece), is_ep: false, is_castle: false });
                    }
                } else {
                    moves.push(Move { from, to, promotion: None, captured: Some(target.piece), is_ep: false, is_castle: false });
                }
            }
        }
        // En passant
        if Some(to) == board.ep_square {
            moves.push(Move { from, to, promotion: None, captured: Some(Piece::Pawn), is_ep: true, is_castle: false });
        }
    }
}

fn gen_leaper_moves(board: &Board, from: u8, color: Color, deltas: &[(i32,i32)], moves: &mut Vec<Move>) {
    let fr = (from / 8) as i32;
    let ff = (from % 8) as i32;
    for &(dr, df) in deltas {
        let tr = fr + dr;
        let tf = ff + df;
        if tr < 0 || tr >= 8 || tf < 0 || tf >= 8 { continue; }
        let to = (tr * 8 + tf) as u8;
        let captured = board.squares[to as usize].and_then(|cp| {
            if cp.color != color { Some(cp.piece) } else { None }
        });
        if board.squares[to as usize].map_or(true, |cp| cp.color != color) {
            moves.push(Move { from, to, promotion: None, captured, is_ep: false, is_castle: false });
        }
    }
}

fn gen_slider_moves(board: &Board, from: u8, color: Color, dirs: &[(i32,i32)], moves: &mut Vec<Move>) {
    let fr = (from / 8) as i32;
    let ff = (from % 8) as i32;
    for &(dr, df) in dirs {
        let mut tr = fr + dr;
        let mut tf = ff + df;
        while tr >= 0 && tr < 8 && tf >= 0 && tf < 8 {
            let to = (tr * 8 + tf) as u8;
            if let Some(cp) = board.squares[to as usize] {
                if cp.color != color {
                    moves.push(Move { from, to, promotion: None, captured: Some(cp.piece), is_ep: false, is_castle: false });
                }
                break;
            }
            moves.push(Move { from, to, promotion: None, captured: None, is_ep: false, is_castle: false });
            tr += dr;
            tf += df;
        }
    }
}

fn gen_castling(board: &Board, from: u8, color: Color, moves: &mut Vec<Move>) {
    let (ks_bit, qs_bit, king_sq) = match color {
        Color::White => (0b0001u8, 0b0010u8, 4u8),
        Color::Black => (0b0100u8, 0b1000u8, 60u8),
    };
    if from != king_sq { return; }
    if board.is_attacked(king_sq, opposite(color)) { return; }

    // Kingside
    if board.castling & ks_bit != 0 {
        let sq1 = king_sq + 1;
        let sq2 = king_sq + 2;
        if board.squares[sq1 as usize].is_none()
            && board.squares[sq2 as usize].is_none()
            && !board.is_attacked(sq1, opposite(color))
            && !board.is_attacked(sq2, opposite(color))
        {
            moves.push(Move { from, to: sq2, promotion: None, captured: None, is_ep: false, is_castle: true });
        }
    }
    // Queenside
    if board.castling & qs_bit != 0 {
        let sq1 = king_sq - 1;
        let sq2 = king_sq - 2;
        let sq3 = king_sq - 3;
        if board.squares[sq1 as usize].is_none()
            && board.squares[sq2 as usize].is_none()
            && board.squares[sq3 as usize].is_none()
            && !board.is_attacked(sq1, opposite(color))
            && !board.is_attacked(sq2, opposite(color))
        {
            moves.push(Move { from, to: sq2, promotion: None, captured: None, is_ep: false, is_castle: true });
        }
    }
}
