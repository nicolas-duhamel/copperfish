use crabchess::prelude::*;

// Piece values
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 270;
const BISHOP_VALUE: i32 = 300;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 0;

// Piece-square tables (white pieces only)
const PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 15, 20, 20, 15, 10, 5, 4, 8, 12, 16, 16, 12, 8, 4, 3, 6, 9, 12,
    12, 9, 6, 3, 2, 4, 6, 8, 8, 6, 4, 2, 1, 2, 3, -10, -10, 3, 2, 1, 0, 0, 0, -40, -40, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_PST: [i32; 64] = [
    -10, -10, -10, -10, -10, -10, -10, -10, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -10, 0, 5, 10, 10, 5, 0, -10, -10, 0, 5, 10, 10, 5, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0,
    0, 0, 0, 0, 0, -10, -10, -30, -10, -10, -10, -10, -30, -10,
];
const BISHOP_PST: [i32; 64] = [
    -10, -10, -10, -10, -10, -10, -10, -10, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -10, 0, 5, 10, 10, 5, 0, -10, -10, 0, 5, 10, 10, 5, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0,
    0, 0, 0, 0, 0, -10, -10, -10, -20, -10, -10, -20, -10, -10,
];
const KING_PST: [i32; 64] = [
    -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40,
    -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -40,
    -40, -40, -40, -40, -40, -40, -40, -40, -40, -40, -20, -20, -20, -20, -20, -20, -20, -20, 0,
    20, 40, -20, 0, -20, 40, 20,
];

// King activity in endgame (reward center)
const KING_PST_ENDGAME: [i32; 64] = [
    0, 10, 20, 30, 30, 20, 10, 0, 10, 20, 30, 40, 40, 30, 20, 10, 20, 30, 40, 50, 50, 40, 30, 20,
    30, 40, 50, 60, 60, 50, 40, 30, 30, 40, 50, 60, 60, 50, 40, 30, 20, 30, 40, 50, 50, 40, 30, 20,
    10, 20, 30, 40, 40, 30, 20, 10, 0, 10, 20, 30, 30, 20, 10, 0,
];

// Flip for dark pieces
const FLIP: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23,
    8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
];

pub trait SquareIdx {
    fn to_index(&self) -> usize;
}

impl SquareIdx for Square {
    // Convert a Square to a 0..63 index
    fn to_index(&self) -> usize {
        // Assuming Square has `file` and `rank` as chars 'A'..'H' and '1'..'8'
        let file_index = ((self.file().char() as u8) - b'A') as usize;
        let rank_index = self.rank().char().to_digit(10).unwrap() as usize;

        (8 - rank_index) * 8 + file_index
    }
}

// Full evaluation function: material + piece-square tables + rook bonus
pub fn evaluate(pos: &ChessPosition) -> i32 {
    let mut score_white = 0;
    let mut score_black = 0;

    for &file in File::all().iter() {
        for &rank in Rank::all().iter() {
            let sq = Square(file, rank);
            if let Some(piece) = pos.get(sq) {
                let pst_value = match piece.piece_type {
                    Type::Pawn => PAWN_PST[sq.to_index()],
                    Type::Knight => KNIGHT_PST[sq.to_index()],
                    Type::Bishop => BISHOP_PST[sq.to_index()],
                    Type::Rook => 0,
                    Type::Queen => 0,
                    Type::King => king_value(sq.to_index(), is_endgame(pos)),
                };

                let value = match piece.color {
                    Color::White => match piece.piece_type {
                        Type::Pawn => PAWN_VALUE + pst_value,
                        Type::Knight => KNIGHT_VALUE + pst_value,
                        Type::Bishop => BISHOP_VALUE + pst_value,
                        Type::Rook => ROOK_VALUE,
                        Type::Queen => QUEEN_VALUE,
                        Type::King => KING_VALUE + pst_value,
                    },
                    Color::Black => {
                        let pst = match piece.piece_type {
                            Type::Pawn => PAWN_PST[FLIP[sq.to_index()]],
                            Type::Knight => KNIGHT_PST[FLIP[sq.to_index()]],
                            Type::Bishop => BISHOP_PST[FLIP[sq.to_index()]],
                            Type::Rook => 0,
                            Type::Queen => 0,
                            Type::King => king_value(FLIP[sq.to_index()], is_endgame(pos)),
                        };
                        match piece.piece_type {
                            Type::Pawn => PAWN_VALUE + pst,
                            Type::Knight => KNIGHT_VALUE + pst,
                            Type::Bishop => BISHOP_VALUE + pst,
                            Type::Rook => ROOK_VALUE,
                            Type::Queen => QUEEN_VALUE,
                            Type::King => KING_VALUE + pst,
                        }
                    }
                };

                match piece.color {
                    Color::White => score_white += value,
                    Color::Black => score_black += value,
                }
            }
        }
    }

    score_white += rook_bonus(pos, Color::White);
    score_black += rook_bonus(pos, Color::Black);

    score_white - score_black
}

const ROOK_OPEN_FILE_BONUS: i32 = 15;
const ROOK_SEMI_OPEN_FILE_BONUS: i32 = 10;
const ROOK_ON_SEVENTH_BONUS: i32 = 20;

fn rook_bonus(pos: &ChessPosition, color: Color) -> i32 {
    let mut score = 0;

    for &file in File::all().iter() {
        for &rank in Rank::all().iter() {
            let sq = Square(file, rank);
            if let Some(piece) = pos.get(sq) {
                if piece.piece_type != Type::Rook || piece.color != color {
                    continue;
                }

                let file = sq.file();
                let rank = sq.rank().char().to_digit(10).unwrap() - 1;

                let mut friendly_pawn = false;
                let mut enemy_pawn = false;

                for r in 0..8 {
                    let sq_check = Square(file, Rank::from_idx(r).unwrap());
                    if let Some(p) = pos.get(sq_check) {
                        if p.piece_type == Type::Pawn {
                            if p.color == color {
                                friendly_pawn = true;
                            } else {
                                enemy_pawn = true;
                            }
                        }
                    }
                }

                if !friendly_pawn && !enemy_pawn {
                    score += ROOK_OPEN_FILE_BONUS;
                } else if !friendly_pawn && enemy_pawn {
                    score += ROOK_SEMI_OPEN_FILE_BONUS;
                }

                if (color == Color::White && rank == 6) || (color == Color::Black && rank == 1) {
                    score += ROOK_ON_SEVENTH_BONUS;
                }
            }
        }
    }

    score
}

fn king_value(square: usize, is_endgame: bool) -> i32 {
    if is_endgame {
        KING_PST_ENDGAME[square]
    } else {
        KING_PST[square]
    }
}

pub fn is_endgame(pos: &ChessPosition) -> bool {
    let mut material_score = 0;

    for &file in File::all().iter() {
        for &rank in Rank::all().iter() {
            let sq = Square(file, rank);
            if let Some(piece) = pos.get(sq) {
                match piece.piece_type {
                    Type::Pawn | Type::King => {}
                    Type::Knight => material_score += KNIGHT_VALUE,
                    Type::Bishop => material_score += BISHOP_VALUE,
                    Type::Rook => material_score += ROOK_VALUE,
                    Type::Queen => material_score += QUEEN_VALUE,
                }
            }
        }
    }

    material_score <= 2000
}
