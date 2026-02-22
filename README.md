# AbhinEngine 1.0

A UCI-compatible chess engine written in **Rust** by Abhin.

## Features

- **Alpha-Beta search** with iterative deepening
- **Principal Variation Search (PVS)**
- **Quiescence search** to avoid horizon effect
- **Late Move Reduction (LMR)** for faster search
- **Transposition Table** with Zobrist hashing (1M entries)
- **Move ordering** — TT move, MVV-LVA captures, killer moves, history heuristic
- **Repetition detection** and 50-move rule
- **Phase-aware evaluation** — opening, middlegame, endgame blending
- **Piece-Square Tables** for all pieces
- **King safety** evaluation
- **Pawn structure** — doubled and isolated pawn penalties
- **Bishop pair bonus**
- **Rook bonuses** — open file, semi-open file, 7th rank
- **Mobility scoring**
- **Full UCI protocol** support

## Building

Requires [Rust](https://rustup.rs/) (stable).

```bash
git clone https://github.com/abhineetabhaykulkarni-cloud/AbhinEngine
cd AbhinEngine
cargo build --release
```

The binary will be at `target/release/AbhinEngine.exe` (Windows) or `target/release/AbhinEngine` (Linux/Mac).

## Usage

AbhinEngine uses the **UCI protocol**. You can use it with any UCI-compatible GUI:

- [Arena](http://www.playwitharena.de/)
- [Cutechess](https://cutechess.com/)
- [Banksia GUI](https://banksiagui.com/)
- [Lucas Chess](https://lucaschess.pythonanywhere.com/)

### UCI Commands

```
uci          → engine info
isready      → readiness check
ucinewgame   → reset for new game
position ... → set board position
go depth N   → search to depth N
quit         → exit
```

### Example

```
uci
isready
position startpos moves e2e4 e7e5
go depth 7
```

## Strength

- Plays solid opening theory
- Searches to depth 7 by default
- Estimated ~1600-1800 ELO (CCRL testing pending)

## Author

**Abhin** — Student developer, India 🇮🇳  
Classical musician (Hindustani vocal) and AI/tech enthusiast.

## License

MIT
