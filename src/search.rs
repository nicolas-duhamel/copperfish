use crate::eval::evaluate;
use crate::moves::*;
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
    pub best_move: Option<Move>,
    pub value: i32,
    pub depth: usize,
    bound: Bound,
}

pub type TranspositionTable = HashMap<u64, TTEntry>;

pub const MAX_DEPTH: usize = 20;
pub const WHITE_MATE: i32 = 1_000_000;
pub const BLACK_MATE: i32 = -1_000_000;

pub fn aspiration_search(
    pos: &ChessPosition,
    turn: Color,
    guess: i32,
    depth: usize,
    mut window: i32,
    tt: &mut TranspositionTable,
    zob: &mut Zobrist,
    stop_flag: &Arc<AtomicBool>,
) -> (Move, i32) {
    let mut alpha = (guess - window).max(BLACK_MATE);
    let mut beta = (guess + window).min(WHITE_MATE);
    let mut best_move = None;
    let mut score = guess;
    let mut killer_moves: [[Option<Move>; 2]; MAX_DEPTH] = [[None; 2]; MAX_DEPTH];

    loop {
        if stop_flag.load(Ordering::Relaxed) {
            break; // exit immediately if time is up
        }

        let (mv, val) = minimax(
            pos,
            turn,
            turn,
            depth,
            depth,
            alpha,
            beta,
            turn == Color::White,
            tt,
            zob,
            &mut killer_moves,
            stop_flag,
        );
        best_move = mv;
        score = val;

        if turn == Color::White && score > WHITE_MATE - MAX_DEPTH as i32 {
            return (best_move.unwrap(), score); // forced mate found, stop search
        }
        if turn == Color::Black && score < BLACK_MATE + MAX_DEPTH as i32 {
            return (best_move.unwrap(), score); // forced mate found, stop search
        }

        if score <= alpha {
            // fail low → widen window downward
            beta = alpha;
            alpha = (score - window).max(BLACK_MATE);
        } else if score >= beta {
            // fail high → widen window upward
            beta = (score + window).min(WHITE_MATE);
        } else {
            break; // score is within [alpha, beta]
        }
        window += window / 2;
    }

    (best_move.unwrap(), score)
}

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
    let mut upper_bound = WHITE_MATE;
    let mut lower_bound = BLACK_MATE;
    let mut killer_moves: [[Option<Move>; 2]; MAX_DEPTH] = [[None; 2]; MAX_DEPTH];

    while lower_bound < upper_bound {
        if stop_flag.load(Ordering::Relaxed) {
            break; // exit immediately if time is up
        }
        let beta = guess.max(lower_bound + 1);
        let (_, eval) = minimax(
            position,
            turn,
            turn,
            depth,
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
            // fail low
            upper_bound = guess;
        } else {
            // fail high
            lower_bound = guess;
        }
    }

    // after convergence, lookup root move from TT
    let hash = zob.hash_position(position, turn);
    let best_move = tt.get(&hash).unwrap().best_move.unwrap();

    (best_move, guess)
}

fn minimax(
    position: &ChessPosition,
    turn: Color,
    side_to_move: Color,
    depth: usize,
    original_depth: usize,
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
    let mut tt_move = None;
    if let Some(entry) = tt.get(&hash) {
        tt_move = entry.best_move;
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
        // Experimental
        // tempo bonus to limit the even-odd instability
        /*let tempo_bonus = if turn == side_to_move && turn == Color::White {
            20 * (original_depth as i32 - 2)
        } else if turn == side_to_move && turn == Color::Black {
            -20 * original_depth as i32
        } else {
            0
        };
        let eval = evaluate(position) + tempo_bonus;*/
        /*let mut eval = quiesce(position, turn, alpha, beta, tt, zob);
        if turn == Color::Black {
            eval = -eval;
        }*/
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

    let moves = generate_legal_moves(&position, turn, tt_move, &killer_moves[depth as usize]);

    if maximizing {
        let mut max_eval = BLACK_MATE;
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
                side_to_move,
                depth - 1,
                original_depth,
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
        let mut min_eval = WHITE_MATE;
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
                side_to_move,
                depth - 1,
                original_depth,
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

fn quiesce(
    position: &ChessPosition,
    turn: Color,
    mut alpha: i32,
    beta: i32,
    tt: &mut TranspositionTable,
    zob: &mut Zobrist,
) -> i32 {
    let hash = zob.hash_position(position, turn);
    if let Some(entry) = tt.get(&hash) {
        match entry.bound {
            Bound::Exact => return entry.value,
            _ => {}
        }
    }

    let mut best_eval = evaluate(position);
    if turn == Color::Black {
        best_eval = -best_eval;
    }

    if best_eval >= beta {
        return best_eval;
    }
    if best_eval > alpha {
        alpha = best_eval;
    }

    let moves = generate_captures(position, turn);
    for mv in moves {
        let mut child = position.clone();
        if let Err(_) = child.apply_move(mv) {
            continue;
        }
        let mut score = -quiesce(&child, turn.other(), -beta, -alpha, tt, zob);
        if turn == Color::Black {
            score = -score;
        }
        if score > best_eval {
            best_eval = score;
            if score > alpha {
                if score < beta {
                    alpha = score;
                } else {
                    break; // fail high
                }
            }
        }
    }

    tt.insert(
        hash,
        TTEntry {
            best_move: None,
            depth: 0,
            value: best_eval,
            bound: Bound::Exact,
        },
    );
    best_eval
}
