// board.rs — Chess board with bugfixes:
// 1. make_uci_move now validates moves properly (fixes illegal move bug)
// 2. Repetition detection added

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color { White, Black }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece { Pawn, Knight, Bishop, Rook, Queen, King }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColoredPiece {
    pub piece: Piece,
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub promotion: Option<Piece>,
    pub captured: Option<Piece>,
    pub is_ep: bool,
    pub is_castle: bool,
}

impl Move {
    pub fn null() -> Self {
        Move { from: 0, to: 0, promotion: None, captured: None, is_ep: false, is_castle: false }
    }

    pub fn to_uci(&self) -> String {
        if self.from == 0 && self.to == 0 { return "0000".to_string(); }
        let files = "abcdefgh";
        let from_f = files.chars().nth((self.from % 8) as usize).unwrap();
        let from_r = (self.from / 8) + 1;
        let to_f   = files.chars().nth((self.to % 8) as usize).unwrap();
        let to_r   = (self.to / 8) + 1;
        let promo  = match self.promotion {
            Some(Piece::Queen)  => "q",
            Some(Piece::Rook)   => "r",
            Some(Piece::Bishop) => "b",
            Some(Piece::Knight) => "n",
            _ => "",
        };
        format!("{}{}{}{}{}", from_f, from_r, to_f, to_r, promo)
    }
}

#[derive(Clone)]
pub struct Board {
    pub squares: [Option<ColoredPiece>; 64],
    pub side: Color,
    pub castling: u8,
    pub ep_square: Option<u8>,
    pub halfmove: u32,
    pub hash: u64,
    history: Vec<HistoryEntry>,
    pub position_hashes: Vec<u64>, // for repetition detection
}

#[derive(Clone)]
struct HistoryEntry {
    mv: Move,
    castling: u8,
    ep_square: Option<u8>,
    halfmove: u32,
    hash: u64,
}

impl Board {
    pub fn start_pos() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn from_fen(fen: &str) -> Self {
        let mut board = Board {
            squares: [None; 64],
            side: Color::White,
            castling: 0b1111,
            ep_square: None,
            halfmove: 0,
            hash: 0,
            history: Vec::new(),
            position_hashes: Vec::new(),
        };

        let parts: Vec<&str> = fen.split(' ').collect();

        let mut rank = 7i32;
        let mut file = 0i32;
        for ch in parts[0].chars() {
            match ch {
                '/' => { rank -= 1; file = 0; }
                '1'..='8' => { file += ch as i32 - '0' as i32; }
                _ => {
                    let color = if ch.is_uppercase() { Color::White } else { Color::Black };
                    let piece = match ch.to_ascii_lowercase() {
                        'p' => Piece::Pawn, 'n' => Piece::Knight, 'b' => Piece::Bishop,
                        'r' => Piece::Rook, 'q' => Piece::Queen,  'k' => Piece::King,
                        _ => { file += 1; continue; }
                    };
                    let sq = (rank * 8 + file) as usize;
                    if sq < 64 {
                        board.squares[sq] = Some(ColoredPiece { piece, color });
                    }
                    file += 1;
                }
            }
        }

        if parts.len() > 1 {
            board.side = if parts[1] == "b" { Color::Black } else { Color::White };
        }

        board.castling = 0;
        if parts.len() > 2 {
            let c = parts[2];
            if c.contains('K') { board.castling |= 0b0001; }
            if c.contains('Q') { board.castling |= 0b0010; }
            if c.contains('k') { board.castling |= 0b0100; }
            if c.contains('q') { board.castling |= 0b1000; }
        }

        if parts.len() > 3 && parts[3] != "-" {
            board.ep_square = sq_from_str(parts[3]);
        }

        board
    }

    pub fn piece_at(&self, sq: u8) -> Option<ColoredPiece> {
        self.squares[sq as usize]
    }

    pub fn make_move(&mut self, mv: Move) {
        // Store hash for repetition detection
        self.position_hashes.push(self.hash);

        self.history.push(HistoryEntry {
            mv,
            castling: self.castling,
            ep_square: self.ep_square,
            halfmove: self.halfmove,
            hash: self.hash,
        });

        let moving = match self.squares[mv.from as usize] {
            Some(p) => p,
            None => { self.side = opposite(self.side); return; }
        };

        if mv.is_castle {
            self.squares[mv.to as usize] = Some(moving);
            self.squares[mv.from as usize] = None;
            let (rook_from, rook_to) = if mv.to > mv.from {
                (mv.from + 3, mv.from + 1)
            } else {
                (mv.from - 4, mv.from - 1)
            };
            if rook_from < 64 && rook_to < 64 {
                let rook = self.squares[rook_from as usize];
                self.squares[rook_to as usize] = rook;
                self.squares[rook_from as usize] = None;
            }
        } else {
            if mv.is_ep {
                let ep_pawn_sq = if self.side == Color::White {
                    mv.to.wrapping_sub(8)
                } else {
                    mv.to + 8
                };
                if ep_pawn_sq < 64 {
                    self.squares[ep_pawn_sq as usize] = None;
                }
            }

            self.squares[mv.to as usize] = if let Some(promo) = mv.promotion {
                Some(ColoredPiece { piece: promo, color: moving.color })
            } else {
                Some(moving)
            };
            self.squares[mv.from as usize] = None;
        }

        if matches!(moving.piece, Piece::King) {
            match moving.color {
                Color::White => self.castling &= !0b0011,
                Color::Black => self.castling &= !0b1100,
            }
        }
        match mv.from {
            0  => self.castling &= !0b0010,
            7  => self.castling &= !0b0001,
            56 => self.castling &= !0b1000,
            63 => self.castling &= !0b0100,
            _  => {}
        }
        match mv.to {
            0  => self.castling &= !0b0010,
            7  => self.castling &= !0b0001,
            56 => self.castling &= !0b1000,
            63 => self.castling &= !0b0100,
            _  => {}
        }

        self.ep_square = if matches!(moving.piece, Piece::Pawn) {
            let diff = (mv.to as i32 - mv.from as i32).abs();
            if diff == 16 {
                Some((mv.from + mv.to) / 2)
            } else { None }
        } else { None };

        // Reset halfmove on pawn move or capture
        if matches!(moving.piece, Piece::Pawn) || mv.captured.is_some() || mv.is_ep {
            self.halfmove = 0;
        } else {
            self.halfmove += 1;
        }

        self.side = opposite(self.side);
    }

    pub fn unmake_move(&mut self) {
        let entry = match self.history.pop() {
            Some(e) => e,
            None => return,
        };
        self.position_hashes.pop();

        let mv = entry.mv;
        self.castling = entry.castling;
        self.ep_square = entry.ep_square;
        self.halfmove = entry.halfmove;
        self.hash = entry.hash;
        self.side = opposite(self.side);

        let moved = self.squares[mv.to as usize];

        if mv.is_castle {
            self.squares[mv.from as usize] = moved;
            self.squares[mv.to as usize] = None;
            let (rook_from, rook_to) = if mv.to > mv.from {
                (mv.from + 3, mv.from + 1)
            } else {
                (mv.from - 4, mv.from - 1)
            };
            if rook_from < 64 && rook_to < 64 {
                let rook = self.squares[rook_to as usize];
                self.squares[rook_from as usize] = rook;
                self.squares[rook_to as usize] = None;
            }
        } else {
            let original_piece = if mv.promotion.is_some() {
                Some(ColoredPiece { piece: Piece::Pawn, color: self.side })
            } else {
                moved
            };
            self.squares[mv.from as usize] = original_piece;

            if mv.is_ep {
                self.squares[mv.to as usize] = None;
                let ep_sq = if self.side == Color::White {
                    mv.to.wrapping_sub(8)
                } else {
                    mv.to + 8
                };
                if ep_sq < 64 {
                    self.squares[ep_sq as usize] = Some(ColoredPiece {
                        piece: Piece::Pawn,
                        color: opposite(self.side),
                    });
                }
            } else {
                self.squares[mv.to as usize] = mv.captured.map(|p| ColoredPiece {
                    piece: p,
                    color: opposite(self.side),
                });
            }
        }
    }

    /// Make a move from UCI string — returns false if move is illegal
    pub fn make_uci_move(&mut self, uci: &str) -> bool {
        let moves = crate::movegen::generate_moves(self);
        for mv in moves {
            if mv.to_uci() == uci {
                self.make_move(mv);
                return true;
            }
        }
        eprintln!("info string WARNING: illegal UCI move attempted: {}", uci);
        false
    }

    pub fn in_check(&self) -> bool {
        let king_sq = self.find_king(self.side);
        king_sq.map(|sq| self.is_attacked(sq, opposite(self.side))).unwrap_or(false)
    }

    pub fn find_king(&self, color: Color) -> Option<u8> {
        for sq in 0u8..64 {
            if let Some(cp) = self.squares[sq as usize] {
                if cp.piece == Piece::King && cp.color == color {
                    return Some(sq);
                }
            }
        }
        None
    }

    pub fn is_attacked(&self, sq: u8, by: Color) -> bool {
        for from in 0u8..64 {
            if let Some(cp) = self.squares[from as usize] {
                if cp.color == by && self.piece_attacks(from, sq, cp.piece) {
                    return true;
                }
            }
        }
        false
    }

    fn piece_attacks(&self, from: u8, to: u8, piece: Piece) -> bool {
        let fr = (from / 8) as i32;
        let ff = (from % 8) as i32;
        let tr = (to / 8) as i32;
        let tf = (to % 8) as i32;
        let dr = tr - fr;
        let df = tf - ff;

        match piece {
            Piece::Pawn => {
                let dir = if self.squares[from as usize].unwrap().color == Color::White { 1 } else { -1 };
                dr == dir && df.abs() == 1
            }
            Piece::Knight => {
                (dr.abs() == 2 && df.abs() == 1) || (dr.abs() == 1 && df.abs() == 2)
            }
            Piece::Bishop => {
                dr.abs() == df.abs() && dr != 0 && self.path_clear(from, to)
            }
            Piece::Rook => {
                (dr == 0 || df == 0) && !(dr == 0 && df == 0) && self.path_clear(from, to)
            }
            Piece::Queen => {
                ((dr.abs() == df.abs()) || dr == 0 || df == 0) && !(dr == 0 && df == 0) && self.path_clear(from, to)
            }
            Piece::King => {
                dr.abs() <= 1 && df.abs() <= 1 && !(dr == 0 && df == 0)
            }
        }
    }

    fn path_clear(&self, from: u8, to: u8) -> bool {
        let fr = (from / 8) as i32;
        let ff = (from % 8) as i32;
        let tr = (to / 8) as i32;
        let tf = (to % 8) as i32;
        let dr = (tr - fr).signum();
        let df = (tf - ff).signum();
        let mut r = fr + dr;
        let mut f = ff + df;
        while (r, f) != (tr, tf) {
            let sq = (r * 8 + f) as u8;
            if self.squares[sq as usize].is_some() {
                return false;
            }
            r += dr;
            f += df;
        }
        true
    }

    pub fn has_non_pawn_material(&self) -> bool {
        for sq in 0u8..64 {
            if let Some(cp) = self.squares[sq as usize] {
                if cp.color == self.side && !matches!(cp.piece, Piece::Pawn | Piece::King) {
                    return true;
                }
            }
        }
        false
    }

    /// Check for threefold repetition
    pub fn is_repetition(&self) -> bool {
        let current = self.hash;
        let count = self.position_hashes.iter().filter(|&&h| h == current).count();
        count >= 2
    }

    /// Check for 50-move rule
    pub fn is_fifty_move_rule(&self) -> bool {
        self.halfmove >= 100
    }
}

pub fn opposite(c: Color) -> Color {
    match c { Color::White => Color::Black, Color::Black => Color::White }
}

pub fn sq_from_str(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();
    if bytes.len() < 2 { return None; }
    let file = bytes[0].wrapping_sub(b'a');
    let rank = bytes[1].wrapping_sub(b'1');
    if file < 8 && rank < 8 { Some(rank * 8 + file) } else { None }
}

pub fn piece_value(p: Piece) -> i32 {
    match p {
        Piece::Pawn   => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook   => 500,
        Piece::Queen  => 900,
        Piece::King   => 20000,
    }
}