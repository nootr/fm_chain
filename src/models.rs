use chrono::NaiveDateTime;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, FromRow)]
pub struct Block {
    pub hash: String,
    pub parent_hash: Option<String>,
    pub height: i64,
    pub message: String,
    pub solution: String,
    pub solution_moves: u8,
    pub solution_description: String,
    pub created_at: Option<NaiveDateTime>,
}

impl Block {
    // Fetch all blocks
    pub async fn find_all(db: &SqlitePool) -> Result<Vec<Block>, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            "SELECT hash, parent_hash, height, message, solution, solution_moves, solution_description, created_at
             FROM blocks
             ORDER BY
                height DESC,
                solution_moves DESC",
        )
        .fetch_all(db)
        .await
    }

    // Fetch a block by hash
    pub async fn find_by_hash(db: &SqlitePool, hash: &str) -> Result<Block, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            "SELECT hash, parent_hash, height, message, solution, solution_moves, solution_description, created_at
             FROM blocks
             WHERE hash = ?",
        )
        .bind(hash)
        .fetch_one(db)
        .await
    }

    // Create a child block
    pub async fn create_child(
        &self,
        db: &SqlitePool,
        hash: &str,
        message: &str,
        solution: &str,
        solution_moves: u8,
        solution_description: &str,
    ) -> Result<Self, sqlx::Error> {
        let block = sqlx::query_as::<_, Block>(
            "INSERT INTO blocks (
                hash, parent_hash, height, message, solution, solution_moves, solution_description
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING hash, parent_hash, height, message, solution, solution_moves, solution_description, created_at",
        )
        .bind(hash)
        .bind(&self.hash)
        .bind(self.height + 1)
        .bind(message)
        .bind(solution)
        .bind(solution_moves)
        .bind(solution_description)
        .fetch_one(db)
        .await?;

        Ok(block)
    }
}
