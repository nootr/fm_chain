use rubiks_moves::moves::Algorithm;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::cube::Move;
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

pub fn verify_solution(raw_scramble: &[Move], raw_solution: &[Move]) -> bool {
    let scramble = Algorithm::from(&format_moves(raw_scramble)).unwrap();
    let solution = Algorithm::from(&format_moves(raw_solution)).unwrap();
    solution.solves(&scramble)
}

#[derive(Debug)]
pub struct BranchBlock {
    pub branch: String,
    pub block: Block,
}

fn build_tree(blocks: &[Block]) -> HashMap<String, Vec<Block>> {
    let mut tree: HashMap<String, Vec<Block>> = HashMap::new();
    for block in blocks {
        if let Some(parent_hash) = &block.parent_hash {
            tree.entry(parent_hash.clone())
                .or_default()
                .push(block.clone());
        }
    }

    // Sort children for consistent output
    for children in tree.values_mut() {
        children.sort_by_key(|b| b.hash.clone());
    }

    tree
}

fn find_root(blocks: &[Block]) -> Option<&Block> {
    blocks.iter().find(|b| b.parent_hash.is_none())
}

fn collect_branch_blocks_reverse(
    tree: &HashMap<String, Vec<Block>>,
    block_map: &HashMap<String, Block>,
    hash: &str,
    prefix: String,
    last: bool,
    out: &mut Vec<BranchBlock>,
    is_root: bool,
) {
    let block = block_map.get(hash).expect("Block not found");

    let connector = if is_root {
        ""
    } else if last {
        "┌─ "
    } else {
        "├─ "
    };

    let line = format!("{}{}", prefix, connector);
    out.push(BranchBlock {
        branch: line,
        block: block.clone(),
    });

    let child_prefix = if is_root {
        "".to_string()
    } else if last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    if let Some(children) = tree.get(hash) {
        for (i, child) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            collect_branch_blocks_reverse(
                tree,
                block_map,
                &child.hash,
                child_prefix.clone(),
                is_last,
                out,
                false,
            );
        }
    }
}

pub fn generate_branch_display(blocks: &[Block]) -> Vec<BranchBlock> {
    let block_map: HashMap<String, Block> =
        blocks.iter().map(|b| (b.hash.clone(), b.clone())).collect();

    let tree = build_tree(blocks);
    let root = find_root(blocks).expect("No root block found");

    let mut result = Vec::new();
    collect_branch_blocks_reverse(
        &tree,
        &block_map,
        &root.hash,
        String::new(),
        true,
        &mut result,
        true,
    );
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
