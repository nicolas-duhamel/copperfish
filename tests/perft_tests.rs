use chess::*;
use std::str::FromStr;

pub fn perft(game: &Game, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let movegen = MoveGen::new_legal(&game.current_position());
    for mv in movegen {
        let mut child = game.clone();
        child.make_move(mv);
        nodes += perft(&child, depth - 1);
    }

    nodes
}

#[test]
fn perft_position_1() {
    let game: Game = Game::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .expect("Valid FEN");
    assert_eq!(perft(&game, 1), 20);
    assert_eq!(perft(&game, 2), 400);
    assert_eq!(perft(&game, 3), 8902);
    assert_eq!(perft(&game, 4), 197281);
}

#[test]
fn perft_position_2() {
    let game: Game =
        Game::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .expect("Valid FEN");
    assert_eq!(perft(&game, 1), 48);
    assert_eq!(perft(&game, 2), 2039);
    assert_eq!(perft(&game, 3), 97862);
    // assert_eq!(perft(&game, 4), 4085603);
}

#[test]
fn perft_position_3() {
    let game: Game =
        Game::from_str("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").expect("Valid FEN");
    assert_eq!(perft(&game, 1), 14);
    assert_eq!(perft(&game, 2), 191);
    assert_eq!(perft(&game, 3), 2812);
    assert_eq!(perft(&game, 4), 43238);
}

#[test]
fn perft_position_4() {
    let game: Game =
        Game::from_str("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
            .expect("Valid FEN");
    assert_eq!(perft(&game, 1), 6);
    assert_eq!(perft(&game, 2), 264);
    assert_eq!(perft(&game, 3), 9467);
}

#[test]
fn perft_position_5() {
    let game: Game = Game::from_str("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
        .expect("Valid FEN");
    assert_eq!(perft(&game, 1), 44);
    assert_eq!(perft(&game, 2), 1486);
    assert_eq!(perft(&game, 3), 62379);
}

#[test]
fn perft_position_6() {
    let game: Game =
        Game::from_str("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")
            .expect("Valid FEN");
    assert_eq!(perft(&game, 1), 46);
    assert_eq!(perft(&game, 2), 2079);
    assert_eq!(perft(&game, 3), 89890);
}
