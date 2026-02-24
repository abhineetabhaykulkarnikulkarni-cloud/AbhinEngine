# AbhinEngine 1.0

A UCI-compatible chess engine written in **Rust** by Abhin.

---

## Download

1. Go to the [Releases](../../releases) page
2. Download `AbhinEngine.exe` (Windows)
3. Done — no installation needed!

---

## How to Use

AbhinEngine uses the **UCI protocol**. You need a chess GUI to play against it.

### Recommended GUIs (free)

| GUI | Download |
|-----|----------|
| Arena | http://www.playwitharena.de/ |
| Banksia | https://banksiagui.com/ |
| Lucas Chess | https://lucaschess.pythonanywhere.com/ |
| Cutechess | https://cutechess.com/ |

---

## Setup in Arena (Step by Step)

1. Open **Arena**
2. Go to **Engines → Manage**
3. Click **Add**
4. Browse to `AbhinEngine.exe` and select it
5. Set engine type to **UCI**
6. Click **OK**
7. Go to **Engines → Load Engine** and select AbhinEngine
8. Start a game!

---

## Setup in Banksia GUI

1. Open **Banksia GUI**
2. Go to **Engines → Add Engine**
3. Browse to `AbhinEngine.exe`
4. Protocol: **UCI**
5. Click **OK**

---

## Setup in Lucas Chess

1. Open **Lucas Chess**
2. Go to **Tools → External Engines**
3. Click **New**
4. Browse to `AbhinEngine.exe`
5. Protocol: **UCI**
6. Save and close

---

## Playing Against AbhinEngine

Once set up in any GUI:
- Set AbhinEngine as one of the players
- Set time control or fixed depth (recommended: **depth 7**)
- Start the game!

---

## Engine Options

| Option | Default | Description |
|--------|---------|-------------|
| Hash | 64 MB | Transposition table size |
| Ponder | false | Think on opponent's time |

---

## Features

- Alpha-Beta search with iterative deepening
- Principal Variation Search (PVS)
- Quiescence search
- Late Move Reduction (LMR)
- Transposition Table with Zobrist hashing
- Killer moves + history heuristic move ordering
- Phase-aware evaluation (opening / middlegame / endgame)
- Piece-Square Tables for all pieces
- King safety evaluation
- Pawn structure (doubled/isolated penalties)
- Bishop pair bonus
- Rook bonuses (open file, 7th rank)
- Mobility scoring
- Repetition detection and 50-move rule
- Full UCI protocol

---

## Strength

- Estimated **~1600-1800 ELO**
- CCRL official testing pending

---

## System Requirements

- Windows 10 or later (64-bit)
- No installation required

---

## Author

**Abhin** — Student developer, India 🇮🇳  

---

## License

MIT — free to use, modify and distribute.
