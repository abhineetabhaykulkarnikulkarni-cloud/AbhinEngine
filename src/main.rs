// main.rs — UCI interface for AbhinEngine

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
                println!("id name AbhinEngine 1.0");
                println!("id author Abhin");
                println!("option name Hash type spin default 64 min 1 max 512");
                println!("option name Ponder type check default false");
                println!("uciok");
            }
            "isready"    => println!("readyok"),
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
                let depth = pick_depth(line, &board);
                engine.tt.clear();
                let (best_move, score) = engine.search(&mut board, depth);

                println!("info depth {} score cp {} nodes {}",
                    depth, score, engine.nodes);
                println!("bestmove {}", best_move.to_uci());
            }
            _ => {}
        }
    }
}

fn pick_depth(line: &str, board: &Board) -> u8 {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Explicit depth
    for i in 0..parts.len() {
        if parts[i] == "depth" {
            if let Some(d) = parts.get(i+1).and_then(|s| s.parse::<u8>().ok()) {
                return d.min(10);
            }
        }
    }

    if line.contains("infinite") { return 8; }



    7
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
            let fen_end = parts[i..].iter().position(|&p| p == "moves").unwrap_or(parts.len() - i);
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