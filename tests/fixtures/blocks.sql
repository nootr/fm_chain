-- Main Chain: Genesis Block (Height 0)
INSERT INTO blocks (
    hash,
    parent_hash,
    height,
    name,
    message,
    solution,
    solution_moves,
    solution_description,
    created_at
) VALUES (
    'genesis_block_hash_001',
    NULL,
    0,
    'Alice',
    'The very first block!',
    'U',
    1,
    'Initial solution',
    '2025-01-01 10:00:00'
);

-- Main Chain: Block 1 (Child of Genesis) (Height 1)
INSERT INTO blocks (
    hash,
    parent_hash,
    height,
    name,
    message,
    solution,
    solution_moves,
    solution_description,
    created_at
) VALUES (
    'main_chain_block_002',
    'genesis_block_hash_001',
    1,
    'Bob',
    'Building on top!',
    'D',
    1,
    'Simple follow-up',
    '2025-01-01 10:05:00'
);

-- Main Chain: Block 2 (Child of Block 1) (Height 2)
INSERT INTO blocks (
    hash,
    parent_hash,
    height,
    name,
    message,
    solution,
    solution_moves,
    solution_description,
    created_at
) VALUES (
    'main_chain_block_003',
    'main_chain_block_002',
    2,
    'Charlie',
    'Continuing the main path.',
    'L',
    1,
    'Keeping it simple',
    '2025-01-01 10:10:00'
);

-- Main Chain: Block 3 (Child of Block 2) (Height 3)
INSERT INTO blocks (
    hash,
    parent_hash,
    height,
    name,
    message,
    solution,
    solution_moves,
    solution_description,
    created_at
) VALUES (
    'main_chain_block_004',
    'main_chain_block_003',
    3,
    'David',
    'The longest chain wins!',
    'R',
    1,
    'Going for the win',
    '2025-01-01 10:15:00'
);

-- Fork Chain 1: Branching from Block 1 (Height 2, same height as main_chain_block_003)
-- This block will have a higher solution_moves than 'main_chain_block_003' but same height,
-- to test the tie-breaking rule (lower solution_moves wins for head selection).
-- So, for this to not be the main chain head, its solution_moves should be higher than
-- the current main chain's best at the same height.
INSERT INTO blocks (
    hash,
    parent_hash,
    height,
    name,
    message,
    solution,
    solution_moves,
    solution_description,
    created_at
) VALUES (
    'fork_chain_block_A_001',
    'main_chain_block_002',
    2, -- Same height as main_chain_block_003
    'Eve',
    'A brave new branch!',
    'F2 B2 U2',
    5, -- Higher solution_moves than main_chain_block_003 (which has 1)
    'Trying a new path',
    '2025-01-01 10:11:00'
);

-- Fork Chain 1: Child of Fork Block A (Height 3)
-- This block keeps the fork chain shorter in height than the main chain's head,
-- or has a higher solution_moves count to ensure it's not selected as main.
INSERT INTO blocks (
    hash,
    parent_hash,
    height,
    name,
    message,
    solution,
    solution_moves,
    solution_description,
    created_at
) VALUES (
    'fork_chain_block_A_002',
    'fork_chain_block_A_001',
    3,
    'Frank',
    'Another block in the fork!',
    'U2',
    2,
    'Still on the fork',
    '2025-01-01 10:16:00'
);

-- Fork Chain 2: Branching from Genesis Block (Height 1, an even shorter fork)
INSERT INTO blocks (
    hash,
    parent_hash,
    height,
    name,
    message,
    solution,
    solution_moves,
    solution_description,
    created_at
) VALUES (
    'fork_chain_block_B_001',
    'genesis_block_hash_001',
    1,
    'Grace',
    'Short and sweet fork.',
    'B',
    10, -- A very high solution_moves to definitely not be main
    'Just an alternative',
    '2025-01-01 10:06:00'
);
