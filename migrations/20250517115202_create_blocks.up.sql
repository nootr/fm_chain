CREATE TABLE blocks (
    hash TEXT PRIMARY KEY,
    parent_hash TEXT,
    height INTEGER NOT NULL,
    message TEXT NOT NULL,
    solution TEXT NOT NULL,
    solution_moves INTEGER NOT NULL,
    solution_description TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY(parent_hash) REFERENCES blocks(hash) ON DELETE CASCADE
);

-- Genesis block
INSERT INTO blocks (hash, parent_hash, height, message, solution, solution_moves, solution_description)
VALUES (
    '65144AGB2624GB7A4D9D1C3C7777H30',
    NULL,
    0,
    'Hi! - Nootr',
    '',
    0,
    'Genesis block - no solution'
);
