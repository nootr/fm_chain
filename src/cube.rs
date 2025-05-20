use std::array;
use std::fmt::{Display, Formatter};

const URF: usize = 0;
const ULF: usize = 1;
const ULB: usize = 2;
const UBR: usize = 3;
const DRF: usize = 4;
const DLF: usize = 5;
const DLB: usize = 6;
const DBR: usize = 7;
const UF: usize = 0;
const UR: usize = 1;
const UL: usize = 2;
const UB: usize = 3;
const DF: usize = 4;
const DR: usize = 5;
const DL: usize = 6;
const DB: usize = 7;
const FR: usize = 8;
const FL: usize = 9;
const BR: usize = 10;
const BL: usize = 11;

#[derive(Debug, Clone)]
pub struct Piece {
    original_permutation: u8,
    orientation: u8,
}

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

#[derive(Debug, Clone)]
pub struct Cube {
    pub corners: [Piece; 8],
    pub edges: [Piece; 12],
}

impl Default for Cube {
    fn default() -> Self {
        let corners = array::from_fn(|i| Piece {
            original_permutation: i as u8,
            orientation: 0,
        });

        let edges = array::from_fn(|i| Piece {
            original_permutation: i as u8,
            orientation: 0,
        });

        Self { corners, edges }
    }
}

impl Cube {
    fn cycle<T: Clone>(arr: &mut [T], indices: &[usize], turns: u8) {
        for _ in 0..turns {
            let temp = arr[indices[0]].clone();
            for i in 0..indices.len() - 1 {
                arr[indices[i]] = arr[indices[i + 1]].clone();
            }
            arr[indices[indices.len() - 1]] = temp;
        }
    }

    pub fn apply_move(&mut self, m: &Move) {
        match m {
            Move::U(n) => {
                Self::cycle(&mut self.corners, &[URF, ULF, ULB, UBR], *n);
                Self::cycle(&mut self.edges, &[UF, UL, UB, UR], *n);
            }
            Move::D(n) => {
                Self::cycle(&mut self.corners, &[DRF, DBR, DLB, DLF], *n);
                Self::cycle(&mut self.edges, &[DF, DR, DB, DL], *n);
            }
            Move::L(n) => {
                Self::cycle(&mut self.corners, &[ULF, DLF, DLB, ULB], *n);
                Self::cycle(&mut self.edges, &[UL, FL, DL, BL], *n);
            }
            Move::R(n) => {
                Self::cycle(&mut self.corners, &[URF, UBR, DBR, DRF], *n);
                Self::cycle(&mut self.edges, &[UR, BR, DR, FR], *n);
            }
            Move::F(n) => {
                Self::cycle(&mut self.corners, &[ULF, URF, DRF, DLF], *n);
                Self::cycle(&mut self.edges, &[UF, FR, DF, FL], *n);
            }
            Move::B(n) => {
                Self::cycle(&mut self.corners, &[UBR, ULB, DLB, DBR], *n);
                Self::cycle(&mut self.edges, &[UB, BL, DB, BR], *n);
            }
        }
    }

    fn corners_solved(&self) -> bool {
        self.corners
            .iter()
            .enumerate()
            .all(|(i, piece)| piece.original_permutation == i as u8 && piece.orientation == 0)
    }

    fn edges_solved(&self) -> bool {
        self.edges
            .iter()
            .enumerate()
            .all(|(i, piece)| piece.original_permutation == i as u8 && piece.orientation == 0)
    }

    pub fn is_solved(&self) -> bool {
        self.corners_solved() && self.edges_solved()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    #[test]
    fn test_cube_initialization() {
        let cube = Cube::default();
        assert_eq!(cube.corners.len(), 8);
        assert_eq!(cube.edges.len(), 12);
        assert!(cube.is_solved());
    }

    #[test]
    fn test_cube_apply_move() {
        let mut cube = Cube::default();
        let m = Move::U(1);
        cube.apply_move(&m);
        assert!(!cube.is_solved());
    }

    #[test]
    fn test_inverse_move() {
        let mut tests_done = 0;

        // Generate all possible moves with all possible counts
        for count in 1..=3 {
            for m in [
                Move::U(count),
                Move::D(count),
                Move::L(count),
                Move::R(count),
                Move::F(count),
                Move::B(count),
            ] {
                tests_done += 1;
                let inverse = m.inverse();
                let mut cube = Cube::default();
                cube.apply_move(&m);
                cube.apply_move(&inverse);
                assert!(
                    cube.is_solved(),
                    "Cube should be solved after applying {} and its inverse",
                    format!("{:?}", m)
                );
            }
        }

        assert_eq!(tests_done, 18, "All moves should be tested");
    }

    #[test]
    fn test_cube_multiple_moves_inverse() {
        let mut rng = rand::rng();
        let mut cube = Cube::default();

        let move_variants = [
            |n| Move::U(n),
            |n| Move::D(n),
            |n| Move::L(n),
            |n| Move::R(n),
            |n| Move::F(n),
            |n| Move::B(n),
        ];

        let mut moves = Vec::new();
        for _ in 0..1000 {
            let mv_fn = move_variants[rng.random_range(0..6)];
            let amount = rng.random_range(1..=3);
            moves.push(mv_fn(amount));
        }

        for (i, m) in moves.iter().enumerate() {
            cube.apply_move(m);
            assert!(
                i < 10 || !cube.is_solved(),
                "Cube should be not solved after applying moves"
            );
        }

        for (i, m) in moves.iter().rev().enumerate() {
            assert!(
                i > (1000 - 10) || !cube.is_solved(),
                "Cube should be not solved before applying inverses"
            );
            cube.apply_move(&m.inverse());
        }

        assert!(
            cube.is_solved(),
            "Cube should be solved after applying moves and their inverses"
        );
    }

    #[test]
    fn test_parse_moves() {
        let input = " U2 D'  L\nR2 F' B";
        let expected = vec![
            Move::U(2),
            Move::D(3),
            Move::L(1),
            Move::R(2),
            Move::F(3),
            Move::B(1),
        ];
        let result = parse_moves(input);
        assert_eq!(result, expected, "Parsed moves should match expected");
    }
}
