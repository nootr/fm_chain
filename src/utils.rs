use sha2::{Digest, Sha256};

use crate::cube::{Cube, Move};

pub fn format_data(parent_hash: String, message: String) -> Vec<u8> {
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

        moves.push(move_defs[index].clone());
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

pub fn format_scramble(scramble: &[Move]) -> String {
    let mut formatted = String::new();
    for m in scramble {
        if !formatted.is_empty() {
            formatted.push(' ');
        }
        formatted.push_str(&m.to_string());
    }
    formatted
}

pub fn verify_solution(scramble: &[Move], solution: &[Move]) -> bool {
    let mut cube = Cube::new();
    for m in scramble {
        cube.apply_move(m);
    }
    for m in solution {
        cube.apply_move(m);
    }
    cube.is_solved()
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
}
