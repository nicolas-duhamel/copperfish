use crate::eval::SquareIdx;
use crabchess::prelude::*;

pub fn generate_captures(pos: &ChessPosition, turn: Color) -> Vec<Move> {
    let mut captures = Vec::new();

    for &file in File::all().iter() {
        for &rank in Rank::all().iter() {
            let sq = Square(file, rank);
            for mv in pos.pseudolegal_threats(sq, turn.other()) {
                if !mv.is_legal(pos).unwrap_or(false) {
                    continue;
                }
                match mv {
                    Move::Standard { final_square, .. } if pos.get(final_square).is_none() => {
                        continue
                    }
                    Move::PawnPromotion { final_square, .. } if pos.get(final_square).is_none() => {
                        continue
                    }
                    Move::Castle { .. } => continue,
                    _ => {}
                }
                captures.push(mv);
            }
        }
    }

    captures
}

pub fn generate_legal_moves(
    pos: &ChessPosition,
    turn: Color,
    tt_move: Option<Move>,
    killer: &[Option<Move>; 2],
) -> Vec<Move> {
    // gather pseudo-legal candidates
    let mut candidates: Vec<Move> = Vec::new();
    for &file in File::all().iter() {
        for &rank in Rank::all().iter() {
            let sq = Square(file, rank);

            // moves to an empty square
            for mv in pos.pseudolegally_navigable(sq, turn) {
                candidates.push(mv);
            }

            // threats / captures to this square
            for mv in pos.pseudolegal_threats(sq, turn.other()) {
                candidates.push(mv);
            }
        }
    }

    let binding = pos.fen();
    let parts: Vec<&str> = binding.split_whitespace().collect();

    let castling = parts[2];
    for c in castling.chars() {
        let (castle_color, castle_side) = match c {
            'K' => (Color::White, Side::Kingside),
            'Q' => (Color::White, Side::Queenside),
            'k' => (Color::Black, Side::Kingside),
            'q' => (Color::Black, Side::Queenside),
            _ => continue,
        };

        if castle_color == turn {
            candidates.push(Move::Castle {
                color: castle_color,
                side: castle_side,
                timer_update: None,
            });
        }
    }

    // filter to legal moves, expanding promotions
    let mut legal = Vec::new();
    for mv in candidates {
        if mv.needs_promotion() {
            if let Some(opts) = mv.promotion_options(pos) {
                for ptype in opts {
                    if let Ok(prom) = mv.to_promotion(ptype) {
                        if prom.is_legal(pos).unwrap_or(false) {
                            legal.push(prom);
                        }
                    }
                }
            }
        } else if mv.is_legal(pos).unwrap_or(false) {
            legal.push(mv);
        }
    }

    legal.sort_by_key(|mv| std::cmp::Reverse(move_score(pos, mv, tt_move, killer)));

    legal
}

// Time optimization to reduce the tree search
// Most Valuable Victim, Least Valuable Aggressor
// MVV-LVA[victim][attacker]
pub const MVV_LVA: [[u8; 6]; 6] = [
    [0, 0, 0, 0, 0, 0],       // victim K (not possible)
    [50, 51, 52, 53, 54, 55], // victim Q
    [40, 41, 42, 43, 44, 45], // victim R
    [30, 31, 32, 33, 34, 35], // victim B
    [20, 21, 22, 23, 24, 25], // victim N
    [10, 11, 12, 13, 14, 15], // victim P
];
// Optimization for quiet moves (non-captures)
pub const BONUS_CENTER: [u8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 2, 2, 2, 2, 1, 0, 0, 1, 2, 3, 3, 2, 1, 0,
    0, 1, 2, 3, 3, 2, 1, 0, 0, 1, 2, 2, 2, 2, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn piece_to_index(piece: Type) -> usize {
    match piece {
        Type::King => 0,
        Type::Queen => 1,
        Type::Rook => 2,
        Type::Bishop => 3,
        Type::Knight => 4,
        Type::Pawn => 5,
    }
}

fn move_score(
    pos: &ChessPosition,
    mv: &Move,
    tt_move: Option<Move>,
    killer: &[Option<Move>; 2],
) -> u8 {
    if tt_move == Some(*mv) {
        return 200; // highest priority to TT move
    }

    if killer[0] == Some(*mv) || killer[1] == Some(*mv) {
        return 100; // killer move bonus
    }

    match mv {
        Move::Standard {
            piece_type,
            final_square,
            is_capture,
            ..
        } => {
            if *is_capture {
                let victim_idx = piece_to_index(pos.get(*final_square).unwrap().piece_type);
                let attacker_idx = piece_to_index(*piece_type);
                MVV_LVA[victim_idx][attacker_idx]
            } else {
                BONUS_CENTER[final_square.to_index()]
            }
        }
        Move::EnPassant { .. } => {
            let attacker_idx = piece_to_index(Type::Pawn);
            let victim_idx = piece_to_index(Type::Pawn);
            MVV_LVA[victim_idx][attacker_idx]
        }
        Move::PawnPromotion { final_square, .. } => {
            if let Some(victim) = pos.get(*final_square) {
                let victim_idx = piece_to_index(victim.piece_type);
                let attacker_idx = piece_to_index(Type::Pawn);
                MVV_LVA[victim_idx][attacker_idx]
            } else {
                10 // quiet promotion bonus
            }
        }
        Move::Castle { .. } => 0,
    }
}
