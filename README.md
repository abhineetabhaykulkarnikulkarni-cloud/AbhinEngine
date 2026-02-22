# AbhinEngine 1.0

A UCI-compatible chess engine written in **Rust** by Abhin.

## Features

- Alpha-Beta search with iterative deepening
- Principal Variation Search (PVS)
- Quiescence search
- Late Move Reduction (LMR)
- Transposition Table with Zobrist hashing
- Move ordering — TT move, MVV-LVA, killer moves, history heuristic
- Repetition detection and 50-move rule
- Phase-aware evaluation (opening / middlegame / endgame)
- Piece-Square Tables for all pieces
- King safety, pawn structure, bishop pair, rook bonuses
- Mobility scoring
- Full UCI protocol

## Download

Grab the latest `.exe` from [Releases](../../releases).

## Usage

AbhinEngine uses the **UCI protocol**. Use it with any UCI-compatible GUI:

- [Arena](http://www.playwitharena.de/)
- [Cutechess](https://cutechess.com/)
- [Banksia GUI](https://banksiagui.com/)
- [Lucas Chess](https://lucaschess.pythonanywhere.com/)

Add the `.exe` as a UCI engine in your GUI of choice and play!

### Example UCI commands

```
position startpos moves e2e4 e7e5
go depth 7
```

## Strength

- Estimated ~1600-1800 ELO
- CCRL testing pending

## Author

**Abhin** — Student developer, India 🇮🇳

## License

MIT
