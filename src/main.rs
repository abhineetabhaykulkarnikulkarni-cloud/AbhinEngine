// main.rs — UCI interface for AbhinEngine with proper time management

use std::io::{self, BufRead};

mod board;
mod search;
mod eval;
mod movegen;
mod book;

use board::Board;
use search::SearchEngine;

fn main() {
    let stdin = io::stdin();
    let mut engine = SearchEngine::new();
    let mut board = Board::start_pos();

    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        let line = line.trim();

        match line {
            "uci" => {
                println!("id name AbhinEngine 1.0.1");
                println!("id author Abhin");
                println!("option name Hash type spin default 64 min 1 max 512");
                println!("option name Ponder type check default false");
                println!("uciok");
            }
            "isready"    => println!("readyok"),
            _ if line.starts_with("setoption name Hash value") => {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(mb) = parts.last().and_then(|s| s.parse::<usize>().ok()) {
                    engine.tt.resize(mb);
                }
            }
            "ucinewgame" => {
                board = Board::start_pos();
                engine.clear();
            }
            "quit" => break,
            _ if line.starts_with("position") => {
                board = parse_position(line);
                engine.push_position(&board);
            }
            _ if line.starts_with("go") => {
                let (max_depth, time_ms) = pick_time(line, &board);
                engine.tt.clear();
                let (best_move, _score) = engine.search(&mut board, max_depth, time_ms);
                println!("bestmove {}", best_move.to_uci());
            }
            _ => {}
        }
    }
}

/// Returns (max_depth, time_limit_ms)
fn pick_time(line: &str, board: &Board) -> (u8, u64) {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Explicit depth — give plenty of time
    for i in 0..parts.len() {
        if parts[i] == "depth" {
            if let Some(d) = parts.get(i+1).and_then(|s| s.parse::<u8>().ok()) {
                return (d.min(12), 300_000);
            }
        }
    }

    // Infinite — search deep with lots of time
    if line.contains("infinite") {
        return (12, 300_000);
    }

    // Movetime — use exactly that much time
    if let Some(mt) = get_val(&parts, "movetime") {
        return (12, mt.saturating_sub(50).max(50));
    }

    // Clock-based time management
    let time_key = if board.side == board::Color::White { "wtime" } else { "btime" };
    let inc_key  = if board.side == board::Color::White { "winc"  } else { "binc"  };
    let movestogo_key = "movestogo";

    let clock_ms = get_val(&parts, time_key).unwrap_or(10_000);
    let inc_ms   = get_val(&parts, inc_key).unwrap_or(0);
    let movestogo = get_val(&parts, movestogo_key).unwrap_or(25);

    // How much time to spend this move:
    // Use clock/movestogo + a fraction of increment
    let alloc = (clock_ms / movestogo.max(1)) + inc_ms * 3 / 4;

    // Never use more than 1/3 of remaining clock
    let alloc = alloc.min(clock_ms / 3);

    // Safety margin
    let alloc = alloc.saturating_sub(50).max(50);

    (12, alloc)
}

fn get_val(parts: &[&str], key: &str) -> Option<u64> {
    parts.iter().position(|&p| p == key)
        .and_then(|i| parts.get(i+1))
        .and_then(|s| s.parse().ok())
}

fn parse_position(line: &str) -> Board {
    let mut board = Board::start_pos();
    let parts: Vec<&str> = line.split_whitespace().collect();
    let mut i = 1;

    if i < parts.len() {
        if parts[i] == "startpos" {
            board = Board::start_pos();
            i += 1;
        } else if parts[i] == "fen" {
            i += 1;
            let fen_end = parts[i..].iter()
                .position(|&p| p == "moves")
                .unwrap_or(parts.len() - i);
            let fen = parts[i..i+fen_end].join(" ");
            board = Board::from_fen(&fen);
            i += fen_end;
        }
    }

    if i < parts.len() && parts[i] == "moves" {
        i += 1;
        while i < parts.len() {
            board.make_uci_move(parts[i]);
            i += 1;
        }
    }

    board
}
