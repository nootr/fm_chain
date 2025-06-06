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
