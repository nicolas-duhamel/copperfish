use crabchess::prelude::*;

pub trait UciFormat {
    fn uci(&self) -> String;
}

impl UciFormat for Move {
    fn uci(&self) -> String {
        match self {
            Move::Standard {
                initial_square,
                final_square,
                ..
            } => {
                format!(
                    "{}{}",
                    square_to_uci(*initial_square),
                    square_to_uci(*final_square)
                )
            }
            Move::EnPassant {
                initial_square,
                final_square,
                ..
            } => {
                format!(
                    "{}{}",
                    square_to_uci(*initial_square),
                    square_to_uci(*final_square)
                )
            }
            Move::Castle { color, side, .. } => match (color, side) {
                (Color::White, Side::Kingside) => "e1g1".to_string(),
                (Color::White, Side::Queenside) => "e1c1".to_string(),
                (Color::Black, Side::Kingside) => "e8g8".to_string(),
                (Color::Black, Side::Queenside) => "e8c8".to_string(),
            },
            Move::PawnPromotion {
                initial_square,
                final_square,
                new_type,
                ..
            } => {
                let promo_char = match new_type {
                    PromotedType::Queen => "q",
                    PromotedType::Rook => "r",
                    PromotedType::Bishop => "b",
                    PromotedType::Knight => "n",
                };
                format!(
                    "{}{}{}",
                    square_to_uci(*initial_square),
                    square_to_uci(*final_square),
                    promo_char
                )
            }
        }
    }
}

pub fn square_to_uci(sq: Square) -> String {
    let file = (b'a' + sq.file() as u8) as char; // file: 0..7 → a..h
    let rank = (sq.rank() as u8 + 1).to_string(); // rank: 0..7 → 1..8
    format!("{}{}", file, rank)
}

pub fn move_from_uci(position: &ChessPosition, uci_move: &str) -> Option<Move> {
    let from_sq = square_from_uci(&uci_move[0..2])?;
    let to_sq = square_from_uci(&uci_move[2..4])?;

    let (piece_type, piece_color) = position.get(from_sq).map(|p| (p.piece_type, p.color))?;

    let is_capture = position.get(to_sq).is_some();

    // en passant: pawn moves diagonally to an empty square
    if !is_capture {
        if let Some(p) = position.get(from_sq) {
            if p.piece_type == Type::Pawn && from_sq.file() != to_sq.file() {
                return Some(Move::EnPassant {
                    initial_square: from_sq,
                    capture_square: Square(to_sq.file(), from_sq.rank()),
                    final_square: to_sq,
                    piece_color: p.color,
                    timer_update: None,
                });
            }
        }
    }

    // adhoc handling of castling
    match (uci_move, piece_type) {
        ("e1g1", Type::King) => {
            return Some(Move::Castle {
                color: Color::White,
                side: Side::Kingside,
                timer_update: None,
            })
        }
        ("e1c1", Type::King) => {
            return Some(Move::Castle {
                color: Color::White,
                side: Side::Queenside,
                timer_update: None,
            })
        }
        ("e8g8", Type::King) => {
            return Some(Move::Castle {
                color: Color::Black,
                side: Side::Kingside,
                timer_update: None,
            })
        }
        ("e8c8", Type::King) => {
            return Some(Move::Castle {
                color: Color::Black,
                side: Side::Queenside,
                timer_update: None,
            })
        }
        _ => {}
    };

    // Handle promotions
    let promotion = if uci_move.len() == 5 {
        match &uci_move[4..5] {
            "q" => Some(Type::Queen),
            "r" => Some(Type::Rook),
            "b" => Some(Type::Bishop),
            "n" => Some(Type::Knight),
            _ => None,
        }
    } else {
        None
    };

    if promotion.is_some() {
        Some(Move::PawnPromotion {
            initial_square: from_sq,
            piece_color,
            final_square: to_sq,
            is_capture,
            new_type: match promotion.unwrap() {
                Type::Queen => PromotedType::Queen,
                Type::Rook => PromotedType::Rook,
                Type::Bishop => PromotedType::Bishop,
                Type::Knight => PromotedType::Knight,
                _ => return None, // Invalid promotion type
            },
            timer_update: None,
        })
    } else {
        Some(Move::Standard {
            initial_square: from_sq,
            piece_type,
            piece_color,
            final_square: to_sq,
            is_capture,
            timer_update: None,
        })
    }
}

pub fn square_from_uci(s: &str) -> Option<Square> {
    if s.len() != 2 {
        return None;
    }
    let file = File::from_char(s.chars().nth(0).unwrap()).unwrap();
    let rank = Rank::from_char(&s.chars().nth(1).unwrap()).unwrap();
    Some(Square(file, rank))
}
