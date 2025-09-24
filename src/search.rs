use crate::eval::evaluate;
use crate::moves::generate_legal_moves;
use crate::zobrist::Zobrist;
use crabchess::prelude::*;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

enum Bound {
    Exact,
    Lower,
    Upper,
}

pub struct TTEntry {
    best_move: Option<Move>,
    depth: usize,
    value: i32,
    bound: Bound,
}

pub type TranspositionTable = HashMap<u64, TTEntry>;

pub const MAX_DEPTH: usize = 10;
pub fn mtdf(
    position: &ChessPosition,
    turn: Color,
    first_guess: i32,
    depth: usize,
    tt: &mut TranspositionTable,
    zob: &mut Zobrist,
    stop_flag: &Arc<AtomicBool>,
) -> (Move, i32) {
    let mut guess = first_guess;
    let mut upper_bound = i32::MAX;
    let mut lower_bound = i32::MIN;
    let mut killer_moves: [[Option<Move>; 2]; MAX_DEPTH] = [[None; 2]; MAX_DEPTH];

    while lower_bound < upper_bound {
        if stop_flag.load(Ordering::Relaxed) {
            break; // exit immediately if time is up
        }
        let beta = guess.max(lower_bound + 1);
        let (_, eval) = minimax(
            position,
            turn,
            depth,
            beta - 1,
            beta,
            turn == Color::White,
            tt,
            zob,
            &mut killer_moves,
            stop_flag,
        );
        guess = eval;
        if guess < beta {
            upper_bound = guess;
        } else {
            lower_bound = guess;
        }
    }

    // after convergence, lookup root move from TT
    let hash = zob.hash_position(position, turn);
    let best_move = tt.get(&hash).unwrap().best_move.unwrap();

    (best_move, guess)
}

const WHITE_MATE: i32 = i32::MAX;
const BLACK_MATE: i32 = i32::MIN;

fn minimax(
    position: &ChessPosition,
    turn: Color,
    depth: usize,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
    tt: &mut TranspositionTable,
    zob: &mut Zobrist,
    killer_moves: &mut [[Option<Move>; 2]; MAX_DEPTH],
    stop_flag: &Arc<AtomicBool>,
) -> (Option<Move>, i32) {
    if position.threefold_repetition() {
        return (None, 0);
    }

    let hash = zob.hash_position(position, turn);
    if let Some(entry) = tt.get(&hash) {
        if entry.depth >= depth {
            match entry.bound {
                Bound::Exact => return (entry.best_move, entry.value),
                Bound::Lower if entry.value >= beta => return (entry.best_move, entry.value),
                Bound::Upper if entry.value <= alpha => return (entry.best_move, entry.value),
                _ => {}
            }
        }
    }

    match position.status() {
        PositionStatus::Stalemate
        | PositionStatus::InsufficientMaterial
        | PositionStatus::FiftyMoveRule => return (None, 0),
        PositionStatus::Checkmate(_) => {
            return if maximizing {
                (None, BLACK_MATE)
            } else {
                (None, WHITE_MATE)
            };
        }
        _ => {}
    }

    if depth == 0 {
        let eval = evaluate(position);
        tt.insert(
            hash,
            TTEntry {
                best_move: None,
                depth: 0,
                value: eval,
                bound: Bound::Exact,
            },
        );
        return (None, eval);
    }

    let moves = generate_legal_moves(&position, turn, &killer_moves[depth as usize]);

    if maximizing {
        let mut max_eval = i32::MIN;
        let mut best_move = None;
        let alpha_orig = alpha;

        for mv in moves {
            if stop_flag.load(Ordering::Relaxed) {
                return (best_move, max_eval); // exit immediately if time is up
            }
            let mut child = position.clone();
            if let Err(_) = child.apply_move(mv) {
                continue;
            }

            let (_, mut eval) = minimax(
                &child,
                turn.other(),
                depth - 1,
                alpha,
                beta,
                false,
                tt,
                zob,
                killer_moves,
                stop_flag,
            );

            if eval > WHITE_MATE - MAX_DEPTH as i32 {
                eval -= 1; // handle mate in n moves
            }

            if eval > max_eval {
                max_eval = eval;
                best_move = Some(mv);
            }

            alpha = alpha.max(eval);
            if max_eval >= beta {
                killer_moves[depth as usize].rotate_right(1);
                killer_moves[depth as usize][0] = Some(mv);
                break; // beta cutoff
            }
        }

        // Store in TT
        let bound = if max_eval <= alpha_orig {
            Bound::Upper
        } else if max_eval >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        tt.insert(
            hash,
            TTEntry {
                best_move,
                depth,
                value: max_eval,
                bound,
            },
        );
        (best_move, max_eval)
    } else {
        let mut min_eval = i32::MAX;
        let mut best_move = None;
        let beta_orig = beta;

        for mv in moves {
            if stop_flag.load(Ordering::Relaxed) {
                return (best_move, min_eval); // exit immediately if time is up
            }
            let mut child = position.clone();
            if let Err(_) = child.apply_move(mv) {
                continue;
            }

            let (_, mut eval) = minimax(
                &child,
                turn.other(),
                depth - 1,
                alpha,
                beta,
                true,
                tt,
                zob,
                killer_moves,
                stop_flag,
            );

            if eval < BLACK_MATE + MAX_DEPTH as i32 {
                eval += 1; // handle mate in n moves
            }

            if eval < min_eval {
                min_eval = eval;
                best_move = Some(mv);
            }

            beta = beta.min(eval);
            if min_eval <= alpha {
                killer_moves[depth as usize].rotate_right(1);
                killer_moves[depth as usize][0] = Some(mv);
                break; // alpha cutoff
            }
        }

        // Store in TT
        let bound = if min_eval <= alpha {
            Bound::Upper
        } else if min_eval >= beta_orig {
            Bound::Lower
        } else {
            Bound::Exact
        };

        tt.insert(
            hash,
            TTEntry {
                best_move,
                depth,
                value: min_eval,
                bound,
            },
        );
        (best_move, min_eval)
    }
}
