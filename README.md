# Copperfish

![Elo badge](https://img.shields.io/badge/Elo~1550-blue?style=flat-square)

**Copperfish** is a chess engine written in **Rust**.  
It implements modern search techniques, efficient move ordering, and communicates via the **UCI protocol**, making it compatible with popular chess GUIs such as Arena, CuteChess, and Lichess bots.

You can play against Copperfish on [Lichess](https://lichess.org/@/crabfish-bot).

---

## Features

- **Search**
  - Minimax with **alpha-beta pruning**
  - **Iterative deepening** in a separate thread with time control
  - **Aspiration window** as the main search driver
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

Copperfish has been tested against Stockfish (restricted to ~1600 Elo).  

Results of Copperfish vs Stockfish (0+3, NULL - 1t, NULL - 16MB, 8moves_v3.pgn):
Elo: -11.59 +/- 56.66, nElo: -12.83 +/- 62.16
LOS: 34.29 %, DrawRatio: 41.67 %, PairsRatio: 0.94
Games: 120, Wins: 51, Losses: 55, Draws: 14, Points: 58.0 (48.33 %)
Ptnml(0-2): [12, 6, 25, 8, 9], WL/DD Ratio: inf

This suggests Copperfish performs at **~1550 Elo**.  

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
