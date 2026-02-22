// eval.rs — Phase-aware evaluation (opening / middlegame / endgame)
//
// Key improvements:
//  • Game phase blending — PSTs smoothly shift opening→endgame
//  • Queen penalised for early development
//  • Knights don't rush out before centre established
//  • King safety — penalise exposed king in middlegame
//  • Pawn structure — doubled/isolated penalties
//  • Bishop pair bonus
//  • Rook on open file / 7th rank bonus
//  • Mobility bonus

use crate::board::{Board, Color, Piece, opposite};

const VAL_PAWN:   i32 = 100;
const VAL_KNIGHT: i32 = 320;
const VAL_BISHOP: i32 = 340;
const VAL_ROOK:   i32 = 500;
const VAL_QUEEN:  i32 = 950;

// ── PSTs: opening and endgame per piece ─────────────────────────────────────
// Rank 0 = rank 1 for white. Black mirrors: sq ^ 56

const PAWN_OP: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    -5, -5, -5, -5, -5, -5, -5, -5,
    -2, -2,  0,  5,  5,  0, -2, -2,
    -2, -2,  2, 22, 22,  2, -2, -2,
    -2, -2,  4, 24, 24,  4, -2, -2,
     3,  3,  6,  8,  8,  6,  3,  3,
    45, 45, 40, 30, 30, 40, 45, 45,
     0,  0,  0,  0,  0,  0,  0,  0,
];
const PAWN_EG: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    10, 10, 10, 10, 10, 10, 10, 10,
     5,  5,  5,  5,  5,  5,  5,  5,
     2,  2,  5, 10, 10,  5,  2,  2,
     0,  0,  5, 12, 12,  5,  0,  0,
    -2, -2,  0,  5,  5,  0, -2, -2,
    -5, -5, -5, -5, -5, -5, -5, -5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

// Knights: don't rush, reward central squares
const KNIGHT_OP: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-25, -5, -5, -5, -5,-25,-40,
    -30, -5,  8, 12, 12,  8, -5,-30,
    -30, -5, 12, 18, 18, 12, -5,-30,
    -30, -5, 12, 18, 18, 12, -5,-30,
    -30, -5,  8, 12, 12,  8, -5,-30,
    -40,-25, -5, -5, -5, -5,-25,-40,
    -50,-40,-35,-30,-30,-35,-40,-50,
];
const KNIGHT_EG: [i32; 64] = [
    -60,-40,-30,-30,-30,-30,-40,-60,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 20, 28, 28, 20,  5,-30,
    -30,  5, 20, 28, 28, 20,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -60,-40,-30,-30,-30,-30,-40,-60,
];

const BISHOP_OP: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -10, 10, 12, 12, 12, 12, 10,-10,
    -10,  5, 12, 15, 15, 12,  5,-10,
    -10,  5, 12, 15, 15, 12,  5,-10,
    -10, 10, 12, 12, 12, 12, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];
const BISHOP_EG: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  8, 10, 10,  8,  0,-10,
    -10,  0, 10, 12, 12, 10,  0,-10,
    -10,  0, 10, 12, 12, 10,  0,-10,
    -10,  0,  8, 10, 10,  8,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

const ROOK_OP: [i32; 64] = [
     0,  0,  0,  5,  5,  0,  0,  0,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     5, 10, 10, 10, 10, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];
const ROOK_EG: [i32; 64] = [
     5,  5,  5,  5,  5,  5,  5,  5,
    10, 10, 10, 10, 10, 10, 10, 10,
     0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,
];

// Queen: heavy penalty for early sorties
const QUEEN_OP: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-15,-10, -5, -5,-10,-15,-20,
];
const QUEEN_EG: [i32; 64] = [
    -30,-20,-10,  0,  0,-10,-20,-30,
    -20,-10,  0,  5,  5,  0,-10,-20,
    -10,  0, 10, 10, 10, 10,  0,-10,
      0,  5, 10, 15, 15, 10,  5,  0,
      0,  5, 10, 15, 15, 10,  5,  0,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -20,-10,  0,  5,  5,  0,-10,-20,
    -30,-20,-10,  0,  0,-10,-20,-30,
];

// King: hide + castle in opening, centralise in endgame
const KING_OP: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-30,-30,-20,-20,-10,
     15, 20,-10,-15,-15,-10, 20, 15,
     20, 30, 10,-10,-10, 10, 30, 20,
];
const KING_EG: [i32; 64] = [
    -50,-30,-20,-10,-10,-20,-30,-50,
    -30,-10,  5, 10, 10,  5,-10,-30,
    -20,  5, 15, 20, 20, 15,  5,-20,
    -10, 10, 20, 25, 25, 20, 10,-10,
    -10, 10, 20, 25, 25, 20, 10,-10,
    -20,  5, 15, 20, 20, 15,  5,-20,
    -30,-10,  5, 10, 10,  5,-10,-30,
    -50,-30,-20,-10,-10,-20,-30,-50,
];

// ── Phase (256=opening, 0=endgame) ──────────────────────────────────────────

fn game_phase(board: &Board) -> i32 {
    let mut mat = 0;
    for sq in 0u8..64 {
        if let Some(cp) = board.squares[sq as usize] {
            mat += match cp.piece {
                Piece::Knight | Piece::Bishop => 1,
                Piece::Rook  => 2,
                Piece::Queen => 4,
                _ => 0,
            };
        }
    }
    ((mat * 256) / 28).min(256)
}

fn pst_blend(sq: u8, color: Color, op: &[i32;64], eg: &[i32;64], phase: i32) -> i32 {
    let idx = if color == Color::White { sq as usize } else { (sq ^ 56) as usize };
    (op[idx] * phase + eg[idx] * (256 - phase)) / 256
}

// ── Pawn structure ───────────────────────────────────────────────────────────

fn pawn_structure(board: &Board, color: Color) -> i32 {
    let mut file_cnt = [0u8; 8];
    for sq in 0u8..64 {
        if let Some(cp) = board.squares[sq as usize] {
            if cp.piece == Piece::Pawn && cp.color == color {
                file_cnt[(sq % 8) as usize] += 1;
            }
        }
    }
    let mut score = 0;
    for f in 0..8usize {
        if file_cnt[f] == 0 { continue; }
        if file_cnt[f] > 1 { score -= 20 * (file_cnt[f]-1) as i32; } // doubled
        let isolated = (f == 0 || file_cnt[f-1] == 0) && (f == 7 || file_cnt[f+1] == 0);
        if isolated { score -= 15; }
    }
    score
}

// ── King safety ──────────────────────────────────────────────────────────────

fn king_safety(board: &Board, color: Color, phase: i32) -> i32 {
    if phase < 60 { return 0; }
    let king_sq = match board.find_king(color) { Some(s) => s, None => return 0 };
    let kf = (king_sq % 8) as i32;
    let mut score = 0;
    // Open files near king
    for df in -1i32..=1 {
        let f = kf + df;
        if f < 0 || f >= 8 { continue; }
        let has_pawn = (0u8..8).any(|r| {
            board.squares[(r*8+f as u8) as usize]
                .map_or(false, |cp| cp.piece == Piece::Pawn && cp.color == color)
        });
        if !has_pawn { score -= 18 * phase / 256; }
    }
    // King in centre penalty
    if kf >= 2 && kf <= 5 { score -= 22 * phase / 256; }
    score
}

// ── Bishop pair ──────────────────────────────────────────────────────────────

fn bishop_pair(board: &Board, color: Color) -> i32 {
    let n = board.squares.iter()
        .filter_map(|s| *s)
        .filter(|cp| cp.color == color && cp.piece == Piece::Bishop)
        .count();
    if n >= 2 { 30 } else { 0 }
}

// ── Rook bonuses ─────────────────────────────────────────────────────────────

fn rook_bonus(board: &Board, color: Color) -> i32 {
    let mut score = 0;
    let seventh = if color == Color::White { 6u8 } else { 1u8 };
    for sq in 0u8..64 {
        let Some(cp) = board.squares[sq as usize] else { continue };
        if cp.color != color || cp.piece != Piece::Rook { continue; }
        let file = sq % 8;
        let friendly = (0u8..8).any(|r| board.squares[(r*8+file) as usize]
            .map_or(false, |p| p.piece == Piece::Pawn && p.color == color));
        let enemy = (0u8..8).any(|r| board.squares[(r*8+file) as usize]
            .map_or(false, |p| p.piece == Piece::Pawn && p.color != color));
        if !friendly && !enemy { score += 20; }
        else if !friendly      { score += 10; }
        if sq / 8 == seventh   { score += 25; }
    }
    score
}

// ── Mobility ─────────────────────────────────────────────────────────────────

fn mobility(board: &Board, color: Color) -> i32 {
    let mut count = 0i32;
    for from in 0u8..64 {
        let Some(cp) = board.squares[from as usize] else { continue };
        if cp.color != color { continue; }
        let (fr,ff) = ((from/8) as i32, (from%8) as i32);
        match cp.piece {
            Piece::Knight => {
                for (dr,df) in [(-2,-1),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)] {
                    let (tr,tf)=(fr+dr,ff+df);
                    if tr>=0&&tr<8&&tf>=0&&tf<8 {
                        let to=(tr*8+tf) as u8;
                        if board.squares[to as usize].map_or(true,|c|c.color!=color){count+=1;}
                    }
                }
            }
            Piece::Bishop => count += slider_mob(board,from,color,&[(-1,-1),(-1,1),(1,-1),(1,1)]),
            Piece::Rook   => count += slider_mob(board,from,color,&[(-1,0),(1,0),(0,-1),(0,1)]),
            Piece::Queen  => {
                count += slider_mob(board,from,color,&[(-1,-1),(-1,1),(1,-1),(1,1)]);
                count += slider_mob(board,from,color,&[(-1,0),(1,0),(0,-1),(0,1)]);
            }
            _ => {}
        }
    }
    count
}

fn slider_mob(board: &Board, from: u8, color: Color, dirs: &[(i32,i32)]) -> i32 {
    let mut n = 0;
    let (fr,ff) = ((from/8) as i32, (from%8) as i32);
    for &(dr,df) in dirs {
        let (mut tr,mut tf) = (fr+dr,ff+df);
        while tr>=0&&tr<8&&tf>=0&&tf<8 {
            let to=(tr*8+tf) as u8;
            if let Some(cp)=board.squares[to as usize] { if cp.color!=color{n+=1;} break; }
            n+=1; tr+=dr; tf+=df;
        }
    }
    n
}

// ── Main entry ───────────────────────────────────────────────────────────────

pub fn evaluate(board: &Board) -> i32 {
    let phase = game_phase(board);
    let mut score = 0i32;

    for sq in 0u8..64 {
        let Some(cp) = board.squares[sq as usize] else { continue };
        let (op, eg, mat) = match cp.piece {
            Piece::Pawn   => (&PAWN_OP,   &PAWN_EG,   VAL_PAWN),
            Piece::Knight => (&KNIGHT_OP, &KNIGHT_EG, VAL_KNIGHT),
            Piece::Bishop => (&BISHOP_OP, &BISHOP_EG, VAL_BISHOP),
            Piece::Rook   => (&ROOK_OP,   &ROOK_EG,   VAL_ROOK),
            Piece::Queen  => (&QUEEN_OP,  &QUEEN_EG,  VAL_QUEEN),
            Piece::King   => (&KING_OP,   &KING_EG,   0),
        };
        let val = mat + pst_blend(sq, cp.color, op, eg, phase);
        if cp.color == Color::White { score += val; } else { score -= val; }
    }

    score += pawn_structure(board, Color::White) - pawn_structure(board, Color::Black);
    score += king_safety(board, Color::White, phase) - king_safety(board, Color::Black, phase);
    score += bishop_pair(board, Color::White) - bishop_pair(board, Color::Black);
    score += rook_bonus(board, Color::White)  - rook_bonus(board, Color::Black);
    score += (mobility(board, Color::White)   - mobility(board, Color::Black)) * 3;

    if board.side == Color::White { score } else { -score }
}