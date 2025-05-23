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
    pub fn apply_move(&mut self, mv: &Move) {
        let (corner_cycle, edge_cycle, corner_orients, edge_orients) = match mv {
            Move::U(_) => (
                [URF, ULF, ULB, UBR],
                [UF, UL, UB, UR],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
            ),
            Move::D(_) => (
                [DRF, DLF, DLB, DBR],
                [DF, DL, DB, DR],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
            ),
            Move::F(_) => (
                [URF, ULF, DLF, DRF],
                [UF, FL, DF, FR],
                [1, 2, 1, 2],
                [1, 0, 1, 0],
            ),
            Move::B(_) => (
                [ULB, UBR, DBR, DLB],
                [UB, BR, DB, BL],
                [1, 2, 1, 2],
                [1, 0, 1, 0],
            ),
            Move::L(_) => (
                [ULF, ULB, DLB, DLF],
                [UL, BL, DL, FL],
                [1, 2, 1, 2],
                [1, 0, 1, 0],
            ),
            Move::R(_) => (
                [UBR, URF, DRF, DBR],
                [UR, FR, DR, BR],
                [1, 2, 1, 2],
                [1, 0, 1, 0],
            ),
        };

        let times = match mv {
            Move::U(t) | Move::D(t) | Move::L(t) | Move::R(t) | Move::F(t) | Move::B(t) => *t,
        };

        for _ in 0..times {
            // Corners
            let original_corners = self.corners.clone();
            for i in 0..4 {
                let from = corner_cycle[i];
                let to = corner_cycle[(i + 1) % 4];
                let new_orientation = (original_corners[from].orientation + corner_orients[i]) % 3;
                self.corners[to] = Piece {
                    original_permutation: original_corners[from].original_permutation,
                    orientation: new_orientation,
                };
            }

            // Edges
            let original_edges = self.edges.clone();
            for i in 0..4 {
                let from = edge_cycle[i];
                let to = edge_cycle[(i + 1) % 4];
                let new_orientation = (original_edges[from].orientation + edge_orients[i]) % 2;
                self.edges[to] = Piece {
                    original_permutation: original_edges[from].original_permutation,
                    orientation: new_orientation,
                };
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

    #[test]
    fn test_regression_orientation() {
        // Test case for a bug where orientation is not followed correctly
        let corner_twist_alg = "R' D R F D F' U' F D' F' R' D' R U";
        let mut cube = Cube::default();
        let moves = parse_moves(corner_twist_alg);
        for m in moves {
            cube.apply_move(&m);
        }
        assert!(
            !cube.corners_solved(),
            "Corners should not be solved after applying corner twist algorithm"
        );
        assert!(
            cube.edges_solved(),
            "Edges should be solved after applying corner twist algorithm"
        );
        assert!(
            !cube.is_solved(),
            "Cube should not be solved after applying corner twist algorithm"
        );
    }

    #[test]
    fn test_full_solves() {
        // Data source: https://www.fewest-moves.info/archive/468
        let solves = [
            "D L' U F2 B2 L2 U2 B2 D' L2 F2 U2 B2 F' U' L2 F2 U B' U R D'",
            "F2 D2 F2 L R2 U2 L B2 U2 L B2 L' B2 U2 F' L R2 F2 D2 L F D' L' B2 U",
            "B D' R' D L' D' R F' D B2 D' F D' B L' B' L2 B R' U2 D2 R D' B2 L' U",
            "B' U' L R F D' R2 D L2 F2 U' B2 L R D2 L' R B2 U' R' U L2 U D' R2 D2 F2",
            "D2 F2 U D B2 U' D F2 B D2 R2 B D2 R2 F D2 L B R2 D2 F' L' F' D' L' B2 U",
            "B L' B2 D2 L F' L B L' F L' B' L B L2 R' U2 D2 R2 U2 R' D' R U2 R' B2 L' U",
            "L F' L' B' L F L' D2 L U R B L U' L' D2 L U D' L2 B2 R' B L B' R2 D R'",
            "L F2 L' D2 F2 L' B2 U2 R' U2 B2 L2 R2 U2 B2 F' R2 L U D' L2 U' D' L F D' L' B2 U",
            "B' L U2 R F D2 R U2 R2 U B2 R2 U' R2 U2 R2 U' R2 U' F2 U R' F2 R U2 D R2 B2 U2",
            "L' U2 D2 B' R D' R' F D F' R F2 R' D R B' F' U2 F D F' U2 F2 D' B D2 F' D2 F R' D R U'",
            "B' D2 U2 B' L2 B' U B U' D' R' U' R D U2 B2 L2 B R B' L2 B R F R F' R B L' B' L' U' R D",
        ];

        for solve in &solves {
            let scramble =
                "R' U' F L2 B2 L2 F2 D U L2 U F2 U' R D2 U' F' L D2 F D' U2 B2 U' R' U' F";
            let mut cube = Cube::default();
            let scramble_moves = parse_moves(scramble);
            for m in &scramble_moves {
                cube.apply_move(m);
                assert!(
                    !cube.is_solved(),
                    "Cube should not be solved while applying scramble moves"
                );
            }
            let solve_moves = parse_moves(solve);
            for m in &solve_moves {
                assert!(
                    !cube.is_solved(),
                    "[{}] Move {}, cube should not be solved while applying solve moves",
                    solve,
                    format!("{:?}", m)
                );
                cube.apply_move(m);
            }
            assert!(
                cube.is_solved(),
                "[{}] Cube should be solved after applying solve moves",
                solve,
            );
        }
    }

    #[test]
    fn test_default_cube_solved() {
        let cube = Cube::default();
        assert!(cube.is_solved(), "Default cube should be solved");
        assert!(cube.corners_solved(), "Corners should be solved");
        assert!(cube.edges_solved(), "Edges should be solved");
    }

    #[test]
    fn test_six_sexy_solved() {
        let mut cube = Cube::default();
        for i in 0..6 {
            cube.apply_move(&Move::R(1));
            cube.apply_move(&Move::U(1));
            cube.apply_move(&Move::R(3));
            cube.apply_move(&Move::U(3));
            if i < 5 {
                assert!(
                    !cube.is_solved(),
                    "Cube should not be solved after {} sexy moves",
                    i + 1
                );
            } else {
                assert!(
                    cube.is_solved(),
                    "Cube should be solved after six sexy moves"
                );
            }
        }
    }
}
