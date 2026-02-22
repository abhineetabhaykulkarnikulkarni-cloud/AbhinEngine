// search.rs — Alpha-beta search with proper time management

use crate::board::{Board, Move, Color, Piece};
use crate::movegen::{generate_moves, generate_captures};
use crate::eval::evaluate;
use std::time::Instant;

const INF: i32 = 1_000_000;
const MATE: i32 = 900_000;

// ── Zobrist hashing ───────────────────────────────────────────────────────────

pub struct Zobrist {
    pieces:  [[[u64; 64]; 6]; 2],
    side:    u64,
    ep:      [u64; 64],
    castle:  [u64; 16],
}

impl Zobrist {
    pub fn new() -> Self {
        let mut s: u64 = 0x123456789abcdef0;
        let mut r = move || -> u64 {
            s ^= s << 13; s ^= s >> 7; s ^= s << 17; s
        };
        let mut z = Zobrist {
            pieces:  [[[0u64;64];6];2],
            side:    r(),
            ep:      [0u64;64],
            castle:  [0u64;16],
        };
        for c in 0..2 { for p in 0..6 { for sq in 0..64 { z.pieces[c][p][sq] = r(); }}}
        for i in 0..64 { z.ep[i] = r(); }
        for i in 0..16 { z.castle[i] = r(); }
        z
    }

    pub fn hash(&self, board: &Board) -> u64 {
        let mut h = 0u64;
        for sq in 0usize..64 {
            if let Some(cp) = board.squares[sq] {
                let pi = match cp.piece {
                    Piece::Pawn=>0, Piece::Knight=>1, Piece::Bishop=>2,
                    Piece::Rook=>3, Piece::Queen=>4, Piece::King=>5,
                };
                h ^= self.pieces[cp.color as usize][pi][sq];
            }
        }
        if board.side == Color::Black { h ^= self.side; }
        h ^= self.castle[(board.castling & 15) as usize];
        if let Some(ep) = board.ep_square { h ^= self.ep[ep as usize]; }
        h
    }
}

// ── Transposition table ───────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct TTEntry {
    hash:  u64,
    depth: u8,
    score: i32,
    flag:  u8, // 0=exact 1=lower 2=upper
    mv:    Move,
}

pub struct TT {
    data: Vec<TTEntry>,
    mask: usize,
}

impl TT {
    pub fn new() -> Self {
        let sz = 1 << 20;
        TT {
            data: vec![TTEntry { hash:0, depth:0, score:0, flag:0, mv: Move::null() }; sz],
            mask: sz - 1,
        }
    }
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let e = &self.data[hash as usize & self.mask];
        if e.hash == hash && e.depth > 0 { Some(e) } else { None }
    }
    pub fn store(&mut self, hash: u64, depth: u8, score: i32, flag: u8, mv: Move) {
        let idx = hash as usize & self.mask;
        let e = &mut self.data[idx];
        if e.hash != hash || depth >= e.depth {
            *e = TTEntry { hash, depth, score, flag, mv };
        }
    }
    pub fn clear(&mut self) {
        for e in &mut self.data { e.depth = 0; }
    }
}

// ── Search engine ─────────────────────────────────────────────────────────────

pub struct SearchEngine {
    pub tt:      TT,
    pub zob:     Zobrist,
    pub nodes:   u64,
    killer:      [[Option<Move>; 2]; 128],
    history:     [[i32; 64]; 64],
    rep_table:   Vec<u64>,
    // Time management
    start:       Option<Instant>,
    time_limit:  u64, // milliseconds
    stopped:     bool,
}

impl SearchEngine {
    pub fn new() -> Self {
        SearchEngine {
            tt:         TT::new(),
            zob:        Zobrist::new(),
            nodes:      0,
            killer:     [[None; 2]; 128],
            history:    [[0; 64]; 64],
            rep_table:  Vec::with_capacity(512),
            start:      None,
            time_limit: 5000,
            stopped:    false,
        }
    }

    pub fn clear(&mut self) {
        self.tt.clear();
        self.nodes = 0;
        self.killer = [[None; 2]; 128];
        self.history = [[0; 64]; 64];
        self.rep_table.clear();
        self.stopped = false;
    }

    pub fn push_position(&mut self, board: &Board) {
        self.rep_table.push(self.zob.hash(board));
    }

    fn elapsed_ms(&self) -> u64 {
        self.start.map(|s| s.elapsed().as_millis() as u64).unwrap_or(0)
    }

    fn check_time(&mut self) {
        if self.elapsed_ms() >= self.time_limit {
            self.stopped = true;
        }
    }

    pub fn search(
        &mut self,
        board: &mut Board,
        max_depth: u8,
        time_limit_ms: u64,
    ) -> (Move, i32) {
        self.nodes = 0;
        self.stopped = false;
        self.start = Some(Instant::now());
        self.time_limit = time_limit_ms;

        let mut best = Move::null();
        let mut best_score = 0;

        for depth in 1..=max_depth {
            let score = self.pvs(board, depth, -INF, INF, 0);

            // If stopped mid-search, don't use partial result
            if self.stopped { break; }

            best_score = score;

            let hash = self.zob.hash(board);
            if let Some(e) = self.tt.probe(hash) {
                if e.mv.from != e.mv.to { best = e.mv; }
            }

            println!("info depth {} score cp {} nodes {} time {} pv {}",
                depth, score, self.nodes, self.elapsed_ms(), best.to_uci());

            if score.abs() > MATE - 1000 { break; }

            // Stop if we've used more than half our time — next depth won't finish
            if self.elapsed_ms() >= self.time_limit / 2 { break; }
        }

        // Fallback
        if best.from == best.to {
            let moves = generate_moves(board);
            if let Some(&m) = moves.first() { best = m; }
        }

        (best, best_score)
    }

    fn is_draw(&self, hash: u64, halfmove: u32) -> bool {
        if halfmove >= 100 { return true; }
        self.rep_table.iter().filter(|&&h| h == hash).count() >= 2
    }

    fn pvs(&mut self, board: &mut Board, depth: u8,
           mut alpha: i32, beta: i32, ply: usize) -> i32 {
        self.nodes += 1;

        // Check time every 2048 nodes
        if self.nodes & 2047 == 0 { self.check_time(); }
        if self.stopped { return 0; }

        let hash = self.zob.hash(board);

        if ply > 0 && self.is_draw(hash, board.halfmove) { return 0; }

        // TT lookup
        if let Some(e) = self.tt.probe(hash) {
            if e.depth >= depth {
                match e.flag {
                    0 => return e.score,
                    1 => if e.score >= beta  { return e.score; }
                    2 => if e.score <= alpha { return e.score; }
                    _ => {}
                }
            }
        }

        if depth == 0 {
            return self.qsearch(board, alpha, beta);
        }

        let moves = generate_moves(board);
        if moves.is_empty() {
            return if board.in_check() { -MATE + ply as i32 } else { 0 };
        }

        let ordered = self.order(moves, hash, ply);
        let mut best_mv = ordered[0];
        let mut raised_alpha = false;

        self.rep_table.push(hash);

        for (i, &mv) in ordered.iter().enumerate() {
            board.make_move(mv);

            let score = if i == 0 {
                -self.pvs(board, depth - 1, -beta, -alpha, ply + 1)
            } else {
                let r: u8 = if i >= 3 && depth >= 3
                    && mv.captured.is_none()
                    && mv.promotion.is_none()
                    && !board.in_check()
                { 1 } else { 0 };

                let mut s = -self.pvs(board, depth - 1 - r, -alpha - 1, -alpha, ply + 1);
                if s > alpha {
                    s = -self.pvs(board, depth - 1, -beta, -alpha, ply + 1);
                }
                s
            };

            board.unmake_move();

            if self.stopped { self.rep_table.pop(); return 0; }

            if score > alpha {
                alpha = score;
                best_mv = mv;
                raised_alpha = true;

                if score >= beta {
                    if mv.captured.is_none() && ply < 128 {
                        self.killer[ply][1] = self.killer[ply][0];
                        self.killer[ply][0] = Some(mv);
                        let h = &mut self.history[mv.from as usize][mv.to as usize];
                        *h = (*h + depth as i32 * depth as i32).min(50_000);
                    }
                    self.rep_table.pop();
                    self.tt.store(hash, depth, beta, 1, mv);
                    return beta;
                }
            }
        }

        self.rep_table.pop();
        let flag = if !raised_alpha { 2 } else { 0 };
        self.tt.store(hash, depth, alpha, flag, best_mv);
        alpha
    }

    fn qsearch(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.stopped { return 0; }

        let stand_pat = evaluate(board);
        if stand_pat >= beta { return beta; }
        if stand_pat > alpha { alpha = stand_pat; }

        for mv in generate_captures(board) {
            let gain = mv.captured.map(|p| crate::board::piece_value(p)).unwrap_or(0);
            if stand_pat + gain + 200 < alpha { continue; }
            board.make_move(mv);
            let s = -self.qsearch(board, -beta, -alpha);
            board.unmake_move();
            if s >= beta { return beta; }
            if s > alpha { alpha = s; }
        }
        alpha
    }

    fn order(&self, mut moves: Vec<Move>, hash: u64, ply: usize) -> Vec<Move> {
        let tt_mv = self.tt.probe(hash).map(|e| e.mv);
        moves.sort_by_cached_key(|mv| {
            let mut s = 0i32;
            if Some(*mv) == tt_mv { s += 2_000_000; }
            if let Some(cap) = mv.captured {
                s += 1_000_000 + crate::board::piece_value(cap) * 10 - 100;
            }
            if mv.promotion == Some(Piece::Queen) { s += 900_000; }
            if ply < 128 {
                if self.killer[ply][0] == Some(*mv) { s += 800_000; }
                if self.killer[ply][1] == Some(*mv) { s += 700_000; }
            }
            s += self.history[mv.from as usize][mv.to as usize].min(600_000);
            -s
        });
        moves
    }
}
