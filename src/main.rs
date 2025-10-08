mod eval;
mod moves;
mod search;
mod uci;
mod zobrist;

use crabchess::prelude::*;
use eval::*;
use search::*;
use std::io::{self, BufRead, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};
use uci::*;
use zobrist::Zobrist;

fn main() {
    let mut zobrist = Arc::new(Mutex::new(Zobrist::new()));
    let mut tt = Arc::new(Mutex::new(TranspositionTable::new()));
    let mut position = ChessPosition::new();
    let mut turn = Color::White;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("uci") => {
                println!("id name Copperfish");
                println!("id author Nicolas Duhamel");
                println!("uciok");
            }
            Some("isready") => {
                println!("readyok");
            }
            Some("ucinewgame") => {
                zobrist = Arc::new(Mutex::new(Zobrist::new()));
                tt = Arc::new(Mutex::new(TranspositionTable::new()));
                position = ChessPosition::new();
                turn = Color::White;
            }
            Some("position") => {
                while let Some(token) = parts.next() {
                    if token == "startpos" {
                        position = ChessPosition::new();
                        turn = Color::White;
                    } else if token == "moves" {
                        while let Some(mv_str) = parts.next() {
                            if let Some(mv) = move_from_uci(&position, mv_str) {
                                position.apply_move(mv).unwrap();
                                turn = turn.other();
                            }
                        }
                    }
                }
            }
            Some("go") => {
                let best_move = search_with_time(
                    position.clone(),
                    turn,
                    Duration::from_millis(2990),
                    Arc::clone(&zobrist),
                    Arc::clone(&tt),
                );
                if let Some(best_move) = best_move {
                    println!("bestmove {}", best_move.uci());
                } else {
                    let hash = zobrist.lock().unwrap().hash_position(&position, turn);
                    if let Some(entry) = tt.lock().unwrap().get(&hash) {
                        println!("bestmove {}", entry.best_move.unwrap().uci());
                    } else {
                        println!("info string No legal moves found");
                    };
                }
            }
            Some("quit") => {
                break;
            }
            _ => {}
        }
        stdout.flush().unwrap();
    }
}

fn search_with_time(
    position: ChessPosition,
    turn: Color,
    max_time: Duration,
    zob: Arc<Mutex<Zobrist>>,
    tt: Arc<Mutex<TranspositionTable>>,
) -> Option<Move> {
    let tt_clone = Arc::clone(&tt);
    let zob_clone = Arc::clone(&zob);
    let best_move = Arc::new(Mutex::new(None));
    let stop_flag = Arc::new(AtomicBool::new(false));

    let best_move_clone = Arc::clone(&best_move);
    let stop_flag_clone = Arc::clone(&stop_flag);

    // Spawn search thread
    thread::spawn(move || {
        let mut tt = tt_clone.lock().unwrap();
        let mut zob = zob_clone.lock().unwrap();
        let hash = zob.hash_position(&position, turn);
        let (mut guess, depth_start) = if let Some(entry) = tt.get(&hash) {
            (entry.value, entry.depth.max(3) - 2)
        } else {
            (evaluate(&position), 1)
        };

        for depth in (depth_start..MAX_DEPTH).step_by(2) {
            let (mv, score) = aspiration_search(
                &position,
                turn,
                guess,
                depth,
                25,
                &mut tt,
                &mut zob,
                &stop_flag_clone,
            );

            if stop_flag_clone.load(Ordering::Relaxed) {
                break; // exit immediately if time is up
            }

            *best_move_clone.lock().unwrap() = Some(mv);
            guess = score;
            let sign = if turn == Color::White { 1 } else { -1 };
            if score.abs() > WHITE_MATE - MAX_DEPTH as i32 {
                println!(
                    "info depth {} score mate {}",
                    depth,
                    sign * (WHITE_MATE - score.abs() + 1)
                );
                stop_flag_clone.store(true, Ordering::Relaxed); // forced mate found, stop search
                break;
            }
            println!("info depth {} score cp {}", depth, sign * score);
        }
    });

    // Main thread: monitor time
    let start = Instant::now();
    while start.elapsed() < max_time && !stop_flag.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(10));
    }
    stop_flag.store(true, Ordering::Relaxed); // signal thread to stop

    // Return best move
    best_move.lock().unwrap().clone()
}
