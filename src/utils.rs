use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::cube::{Cube, Move};
use crate::models::Block;

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
    let mut cube = Cube::default();
    for m in scramble {
        cube.apply_move(m);
    }
    for m in solution {
        cube.apply_move(m);
    }
    cube.is_solved()
}

pub struct BranchBlock {
    pub branch: String,
    pub block: Block,
}

/// Generate an ASCII graph line for a list of blocks.
///
/// Will generate a graph like this:
/// ```
/// *    // New hash
/// *    // Hash is known
/// | *  // New hash
/// */   // Hash is known
/// *    // Both branches originate from this hash
/// ```
/// Bottom up. Start with single root and for each node check how many notes have that block as parent.
/// The root is the last block in the list.
pub fn generate_branch_display(blocks: Vec<Block>) -> Vec<BranchBlock> {
    let mut hash_to_block = HashMap::new();
    let mut parent_to_children: HashMap<String, Vec<String>> = HashMap::new();

    for block in &blocks {
        hash_to_block.insert(block.hash.clone(), block.clone());
        if let Some(parent_hash) = &block.parent_hash {
            parent_to_children
                .entry(parent_hash.clone())
                .or_default()
                .push(block.hash.clone());
        }
    }

    let mut roots = vec![];
    for block in &blocks {
        if block.parent_hash.is_none() {
            roots.push(block.hash.clone());
        }
    }

    let mut result = Vec::new();

    fn dfs(
        hash: &str,
        prefix: String,
        parent_to_children: &HashMap<String, Vec<String>>,
        hash_to_block: &HashMap<String, Block>,
        result: &mut Vec<BranchBlock>,
    ) {
        let block = hash_to_block.get(hash).unwrap();
        result.push(BranchBlock {
            branch: format!("{}* ", prefix),
            block: block.clone(),
        });

        let children = parent_to_children.get(hash).cloned().unwrap_or_default();

        if children.len() > 1 {
            result.push(BranchBlock {
                branch: format!("{}|/", prefix),
                block: block.clone(),
            });
        }

        for (i, child) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            let child_prefix = format!("{}{}", prefix, if is_last { "  " } else { "| " });
            dfs(
                child,
                child_prefix,
                parent_to_children,
                hash_to_block,
                result,
            );
        }
    }

    for root in roots {
        dfs(
            &root,
            String::new(),
            &parent_to_children,
            &hash_to_block,
            &mut result,
        );
    }

    result.reverse();
    result
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
