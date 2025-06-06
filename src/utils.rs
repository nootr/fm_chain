use rubiks_moves::moves::Algorithm;
use sha2::{Digest, Sha256};

use crate::cube::Move;

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

pub fn format_data(parent_hash: &str, message: &str) -> Vec<u8> {
    format!("{}|{}", parent_hash, message).as_bytes().to_vec()
}

pub fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();

    let mut n = u128::from_be_bytes(result[0..16].try_into().unwrap());
    let alphabet = b"0123456789ABCDEFGH"; // base18
    let mut base18 = String::new();
    while n > 0 {
        base18.insert(0, alphabet[(n % 18) as usize] as char);
        n /= 18;
    }
    base18
}

pub fn scramble_from_hash(hash: &str) -> Vec<Move> {
    // Maps index to (face, count)
    let move_defs = [
        Move::U(1),
        Move::U(2),
        Move::U(3),
        Move::D(1),
        Move::D(2),
        Move::D(3),
        Move::L(1),
        Move::L(2),
        Move::L(3),
        Move::R(1),
        Move::R(2),
        Move::R(3),
        Move::F(1),
        Move::F(2),
        Move::F(3),
        Move::B(1),
        Move::B(2),
        Move::B(3),
    ];

    let mut moves = Vec::new();
    for ch in hash.chars() {
        let index = match ch {
            '0'..='9' => ch as usize - '0' as usize,
            'A'..='H' => 10 + (ch as usize - 'A' as usize),
            _ => continue, // skip invalid characters
        };

        moves.push(move_defs[index]);
    }

    moves
}

pub fn cleanup_scramble(scramble: &mut Vec<Move>) {
    let mut cleaned: Vec<Move> = Vec::new();

    for m in scramble.drain(..) {
        if let Some(last) = cleaned.last_mut() {
            if last.inverse() == m {
                cleaned.pop();
                continue;
            }
            if let Some(new_move) = last.combine(&m) {
                *last = new_move;
                continue;
            }
        }
        cleaned.push(m);
    }

    *scramble = cleaned;
}

pub fn format_moves(moves: &[Move]) -> String {
    let mut formatted = String::new();
    for m in moves {
        if !formatted.is_empty() {
            formatted.push(' ');
        }
        formatted.push_str(&format!("{}", m));
    }
    formatted
}

pub fn verify_solution(scramble: &[Move], solution: &[Move]) -> bool {
    let scramble = Algorithm::from(&format_moves(scramble)).unwrap();
    let solution = Algorithm::from(&format_moves(solution)).unwrap();
    solution.solves(&scramble)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_scramble() {
        let mut scramble = vec![
            Move::R(1),
            Move::U(1),
            Move::U(2),
            Move::F(2),
            Move::F(2),
            Move::L(1),
            Move::L(3),
            Move::B(2),
            Move::B(3),
        ];

        cleanup_scramble(&mut scramble);

        let expected = vec![Move::R(1), Move::U(3), Move::B(1)];

        assert_eq!(scramble, expected);
    }

    #[test]
    fn test_verify_solution_ok() {
        // Data source: https://www.fewest-moves.info/archive/468
        let scramble_raw =
            "R' U' F L2 B2 L2 F2 D U L2 U F2 U' R D2 U' F' L D2 F D' U2 B2 U' R' U' F";
        let scramble = parse_moves(scramble_raw);
        let solves_raw = [
            "D L' U F2 B2 L2 U2 B2 D' L2 F2 U2 B2 F' U' L2 F2 U B' U R D'",
            "F2 D2 F2 L R2 U2 L B2 U2 L B2 L' B2 U2 F' L R2 F2 D2 L F D' L' B2 U",
            "B D' R' D L' D' R F' D B2 D' F D' B L' B' L2 B R' U2 D2 R D' B2 L' U",
            "B' U' L R F D' R2 D L2 F2 U' B2 L R D2 L' R B2 U' R' U L2 U D' R2 D2 F2",
            "D2 F2 U D B2 U' D F2 B D2 R2 B D2 R2 F D2 L B R2 D2 F' L' F' D' L' B2 U",
            "B L' B2 D2 L F' L B L' F L' B' L B L2 R' U2 D2 R2 U2 R' D' R U2 R' B2 L' U",
            "L F' L' B' L F L' D2 L U R B L U' L' D2 L U D' L2 B2 R' B L B' R2 D R'",
            "L F2 L' D2 F2 L' B2 U2 R' U2 B2 L2 R2 U2 B2 F' R2 L U D' L2 U' D' L F D' L' B2 U",
            "B' L U2 R F D2 R U2 R2 U B2 R2 U' R2 U2 R2 U' R2 U' F2 U R' F2 R U2 D R2 B2 U2",
        ];

        for solve_raw in solves_raw {
            let mut solve = parse_moves(solve_raw);
            assert!(
                verify_solution(&scramble, &solve),
                "[{}] Moves should solve cube",
                solve_raw
            );

            solve.extend(vec![Move::U(1)]);
            assert!(
                !verify_solution(&scramble, &solve),
                "[{} U] Moves should not solve cube",
                solve_raw
            );
        }
    }
}
