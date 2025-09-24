use crate::eval::SquareIdx;
use crabchess::prelude::*;
use rand::prelude::*;
use rand::rng;

pub struct Zobrist {
    table: [[[u64; 64]; 2]; 6], // piece_type × color × square
    side_to_move: u64,          // random 64-bit number for side to move
}

impl Zobrist {
    pub fn new() -> Self {
        let mut rng = rng();
        let mut table = [[[0u64; 64]; 2]; 6];

        for pt_idx in 0..6 {
            for color_idx in 0..2 {
                for sq in 0..64 {
                    table[pt_idx][color_idx][sq] = rng.random::<u64>();
                }
            }
        }

        let side_to_move = rng.random::<u64>();

        Zobrist {
            table,
            side_to_move,
        }
    }

    pub fn hash_position(&self, pos: &ChessPosition, turn: Color) -> u64 {
        let mut h = 0u64;

        for &file in File::all().iter() {
            for &rank in Rank::all().iter() {
                let sq = Square(file, rank);
                if let Some(piece) = pos.get(sq) {
                    let pt_idx = match piece.piece_type {
                        Type::Pawn => 0,
                        Type::Knight => 1,
                        Type::Bishop => 2,
                        Type::Rook => 3,
                        Type::Queen => 4,
                        Type::King => 5,
                    };
                    let color_idx = if piece.color == Color::White { 0 } else { 1 };
                    h ^= self.table[pt_idx][color_idx][sq.to_index()];
                }
            }
        }

        // XOR side_to_move only if it's White's turn
        if turn == Color::White {
            h ^= self.side_to_move;
        }

        h
    }
}
