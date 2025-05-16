use sha2::{Digest, Sha256};

pub fn format_data(previous_hash: String, message: String) -> Vec<u8> {
    format!("{}|{}", previous_hash, message).as_bytes().to_vec()
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

pub struct Move {
    pub name: String,
    pub count: u8,
}

pub fn scramble_from_hash(hash: &str) -> Vec<Move> {
    // Maps index to (face, count)
    let move_defs = [
        ("R", 1),
        ("R", 3),
        ("R", 2),
        ("U", 1),
        ("U", 3),
        ("U", 2),
        ("F", 1),
        ("F", 3),
        ("F", 2),
        ("L", 1),
        ("L", 3),
        ("L", 2),
        ("D", 1),
        ("D", 3),
        ("D", 2),
        ("B", 1),
        ("B", 3),
        ("B", 2),
    ];

    let mut moves = Vec::new();
    for ch in hash.chars() {
        let index = match ch {
            '0'..='9' => ch as usize - '0' as usize,
            'A'..='H' => 10 + (ch as usize - 'A' as usize),
            _ => continue, // skip invalid characters
        };

        let (face, count) = move_defs[index];
        moves.push(Move {
            name: face.to_string(),
            count,
        });
    }

    moves
}

pub fn cleanup_scramble(scramble: &mut Vec<Move>) {
    let mut cleaned: Vec<Move> = Vec::new();

    for m in scramble.drain(..) {
        if let Some(last) = cleaned.last_mut() {
            if last.name == m.name {
                last.count = (last.count + m.count) % 4;
                if last.count == 0 {
                    cleaned.pop(); // Cancel out completely
                }
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
        let modifier = match m.count {
            1 => "",
            2 => "2",
            3 => "'",
            _ => unreachable!(),
        };
        if !formatted.is_empty() {
            formatted.push(' ');
        }
        formatted.push_str(&format!("{}{}", m.name, modifier));
    }
    formatted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_scramble() {
        let mut scramble = vec![
            Move {
                name: "R".to_string(),
                count: 1,
            },
            Move {
                name: "U".to_string(),
                count: 1,
            },
            Move {
                name: "U".to_string(),
                count: 2,
            },
            Move {
                name: "F".to_string(),
                count: 2,
            },
            Move {
                name: "F".to_string(),
                count: 2,
            },
            Move {
                name: "L".to_string(),
                count: 1,
            },
            Move {
                name: "L".to_string(),
                count: 3,
            },
            Move {
                name: "B".to_string(),
                count: 2,
            },
            Move {
                name: "B".to_string(),
                count: 3,
            },
        ];

        cleanup_scramble(&mut scramble);

        let expected = vec![
            Move {
                name: "R".to_string(),
                count: 1,
            },
            Move {
                name: "U".to_string(),
                count: 3,
            },
            Move {
                name: "B".to_string(),
                count: 1,
            },
        ];

        assert_eq!(scramble.len(), expected.len());
        for (a, b) in scramble.iter().zip(expected.iter()) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.count, b.count);
        }
    }
}
