use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    U(u8),
    D(u8),
    L(u8),
    R(u8),
    F(u8),
    B(u8),
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let modifier = |&n| match n {
            1 => "",
            2 => "2",
            3 => "'",
            _ => unreachable!(),
        };
        write!(
            f,
            "{}",
            match self {
                Move::U(n) => format!("U{}", modifier(n)),
                Move::D(n) => format!("D{}", modifier(n)),
                Move::L(n) => format!("L{}", modifier(n)),
                Move::R(n) => format!("R{}", modifier(n)),
                Move::F(n) => format!("F{}", modifier(n)),
                Move::B(n) => format!("B{}", modifier(n)),
            }
        )
    }
}

impl Move {
    pub fn inverse(&self) -> Self {
        match self {
            Move::U(n) => Move::U(4 - n),
            Move::D(n) => Move::D(4 - n),
            Move::L(n) => Move::L(4 - n),
            Move::R(n) => Move::R(4 - n),
            Move::F(n) => Move::F(4 - n),
            Move::B(n) => Move::B(4 - n),
        }
    }

    pub fn combine(&self, other: &Self) -> Option<Self> {
        match (self, other) {
            (Move::U(n1), Move::U(n2)) => Some(Move::U((n1 + n2) % 4)),
            (Move::D(n1), Move::D(n2)) => Some(Move::D((n1 + n2) % 4)),
            (Move::L(n1), Move::L(n2)) => Some(Move::L((n1 + n2) % 4)),
            (Move::R(n1), Move::R(n2)) => Some(Move::R((n1 + n2) % 4)),
            (Move::F(n1), Move::F(n2)) => Some(Move::F((n1 + n2) % 4)),
            (Move::B(n1), Move::B(n2)) => Some(Move::B((n1 + n2) % 4)),
            _ => None,
        }
    }
}

pub fn parse_moves(s: &str) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        let count = match chars.peek() {
            Some('2') => {
                chars.next();
                2
            }
            Some('\'') => {
                chars.next();
                3
            }
            _ => 1,
        };

        let m = match c {
            'U' => Move::U(count),
            'D' => Move::D(count),
            'L' => Move::L(count),
            'R' => Move::R(count),
            'F' => Move::F(count),
            'B' => Move::B(count),
            _ => continue,
        };
        moves.push(m);
    }

    moves
}
