# Copperfish

![Elo badge](https://img.shields.io/badge/Elo~1600-blue?style=flat-square)

**Copperfish** is a chess engine written in **Rust**.  
It implements modern search techniques, efficient move ordering, and communicates via the **UCI protocol**, making it compatible with popular chess GUIs such as Arena, CuteChess, and Lichess bots.

You can play against Copperfish on [Lichess](https://lichess.org/@/crabfish-bot).

---

## Features

- **Search**
  - Minimax with **alpha-beta pruning**
  - **Iterative deepening** in a separate thread with time control
  - **MTD(f)** as the main search driver
  - **Quiescence search** to reduce horizon effect
  - **Move ordering** using:
    - **MVV-LVA** (Most Valuable Victim - Least Valuable Attacker)
    - **Killer moves** heuristic
    - **Transposition table** with Zobrist hashing

- **Evaluation**
  - **Material balance**
  - **Piece-square tables**
  - **Rook bonuses** for open and semi-open files
  - Bonus for rooks on the 7th rank
  - Stalemate and checkmate detection

- **Protocol**
  - Full **UCI** support for easy integration with other chess GUIs

---

## Strength

Copperfish has been tested against Stockfish (restricted to ~1320 Elo).  

Results of Copperfish vs Stockfish (10+5, NULL - 1t, NULL - 16MB, openings.pgn):
Elo: 190.85 +/- 229.86, nElo: 194.22 +/- 152.27
LOS: 99.38 %, DrawRatio: 20.00 %, PairsRatio: 7.00
Games: 20, Wins: 14, Losses: 4, Draws: 2, Points: 15.0 (75.00 %)
Ptnml(0-2): [1, 0, 2, 2, 5], WL/DD Ratio: inf

This suggests Copperfish performs at **~1600 Elo**.  

---

## Build & Run

You need [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
# Clone the repository
git clone https://github.com/nicolas-duhamel/copperfish.git
cd copperfish

# Build
cargo build --release

# Run in UCI mode
cargo run --release
```