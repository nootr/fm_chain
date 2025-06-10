use chrono::NaiveDateTime;
use sqlx::{FromRow, SqlitePool};

use crate::utils;

#[derive(Debug, Clone, FromRow)]
pub struct Block {
    pub hash: String,
    pub parent_hash: Option<String>,
    pub height: i64,
    pub name: String,
    pub message: String,
    pub solution: String,
    pub solution_moves: u8,
    pub solution_description: String,
    pub created_at: Option<NaiveDateTime>,
}

impl Block {
    // Get scramble for this block
    pub fn scramble(&self) -> String {
        let scramble = utils::scramble_from_hash(&self.hash);
        utils::format_moves(&scramble)
    }

    pub async fn get_main_chain_hashes(db: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
        Ok(sqlx::query_scalar!(
            r#"
            WITH RECURSIVE main_chain AS (
              SELECT hash, height, solution_moves
              FROM blocks
              WHERE (height, solution_moves) = (
                SELECT height, solution_moves
                FROM blocks
                ORDER BY height DESC, solution_moves DESC
                LIMIT 1
              )

              UNION ALL

              SELECT b.hash, b.height, b.solution_moves
              FROM blocks b
              INNER JOIN main_chain mc ON b.hash = (
                SELECT parent_hash FROM blocks WHERE hash = mc.hash
              )
            )
            SELECT hash FROM main_chain
            "#
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .filter_map(|h| h)
        .collect())
    }

    // Fetch all blocks
    pub async fn find_all(
        db: &SqlitePool,
        main_chain_only: bool,
        page_size: Option<u32>,
        page_offset: Option<u32>,
    ) -> Result<Vec<Block>, sqlx::Error> {
        let mut query_str = String::from(
            "SELECT hash, parent_hash, height, name, message, solution, solution_moves, solution_description, created_at
             FROM blocks"
        );

        if main_chain_only {
            let hashes = Self::get_main_chain_hashes(db).await?;
            if hashes.is_empty() {
                return Ok(Vec::new());
            }
            query_str.push_str(" WHERE hash IN (");
            query_str.push_str(
                &hashes
                    .iter()
                    .map(|h| format!("'{}'", h))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            query_str.push(')');
        }

        query_str.push_str(" ORDER BY height DESC, solution_moves DESC");

        // Conditionally add LIMIT and OFFSET clauses
        if page_size.is_some() {
            query_str.push_str(" LIMIT ?");
        }
        if page_offset.is_some() {
            query_str.push_str(" OFFSET ?");
        }

        let mut query = sqlx::query_as::<_, Block>(&query_str);

        if let Some(size) = page_size {
            query = query.bind(size);
        }
        if let Some(offset) = page_offset {
            query = query.bind(offset);
        }

        query.fetch_all(db).await
    }

    // Fetch a block by hash
    pub async fn find_by_hash(db: &SqlitePool, hash: &str) -> Result<Block, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            "SELECT hash, parent_hash, height, name, message, solution, solution_moves, solution_description, created_at
             FROM blocks
             WHERE hash = ?",
        )
        .bind(hash)
        .fetch_one(db)
        .await
    }

    // Create a genesis block
    pub async fn create_genesis(
        db: &SqlitePool,
        hash: &str,
        name: &str,
        message: &str,
        solution: &str,
        solution_moves: u8,
        solution_description: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            "INSERT INTO blocks (
                hash, height, name, message, solution, solution_moves, solution_description
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING hash, parent_hash, height, name, message, solution, solution_moves, solution_description, created_at",
        )
        .bind(hash)
        .bind(0)
        .bind(name)
        .bind(message)
        .bind(solution)
        .bind(solution_moves)
        .bind(solution_description)
        .fetch_one(db)
        .await
    }

    // Create a child block
    #[allow(clippy::too_many_arguments)]
    pub async fn create_child(
        &self,
        db: &SqlitePool,
        hash: &str,
        name: &str,
        message: &str,
        solution: &str,
        solution_moves: u8,
        solution_description: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            "INSERT INTO blocks (
                hash, parent_hash, height, name, message, solution, solution_moves, solution_description
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING hash, parent_hash, height, name, message, solution, solution_moves, solution_description, created_at",
        )
        .bind(hash)
        .bind(&self.hash)
        .bind(self.height + 1)
        .bind(name)
        .bind(message)
        .bind(solution)
        .bind(solution_moves)
        .bind(solution_description)
        .fetch_one(db)
        .await
    }

    pub fn short_hash(&self) -> String {
        self.hash.chars().take(8).collect()
    }

    pub async fn find_main_chain_head(db: &SqlitePool) -> Result<Block, sqlx::Error> {
        Ok(Self::find_all(db, true, None, None).await?[0].clone())
    }
}
