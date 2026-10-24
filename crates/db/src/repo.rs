use std::path::Path;
use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::DbError;
use crate::models::{GiftRecord, NewGiftRecord, User};

/// Thread-safe SQLite repository for off-chain user metadata and gift transaction logs
#[derive(Clone)]
pub struct DatabaseRepository {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseRepository {
    /// Opens an in-memory SQLite database and applies schema migrations
    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        let repo = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        repo.init_schema()?;
        Ok(repo)
    }

    /// Opens or creates a file-backed SQLite database and applies migrations
    pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        let repo = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        repo.init_schema()?;
        Ok(repo)
    }

    /// Initializes schema tables and indexes
    pub fn init_schema(&self) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS users (
                address TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL,
                alias TEXT,
                tags TEXT NOT NULL DEFAULT '[]'
            );

            CREATE TABLE IF NOT EXISTS gift_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tx_hash TEXT NOT NULL,
                from_address TEXT NOT NULL,
                to_address TEXT NOT NULL,
                amount INTEGER NOT NULL,
                memo TEXT,
                timestamp INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_gift_from ON gift_records(from_address);
            CREATE INDEX IF NOT EXISTS idx_gift_to ON gift_records(to_address);
            CREATE INDEX IF NOT EXISTS idx_gift_tx ON gift_records(tx_hash);
            "#,
        )?;

        Ok(())
    }

    /// Inserts a new user or updates alias and tags if the user already exists (idempotent)
    pub fn upsert_user(&self, user: &User) -> Result<(), DbError> {
        if user.address.trim().is_empty() {
            return Err(DbError::Validation("User address cannot be empty".to_string()));
        }

        let tags_json = serde_json::to_string(&user.tags)?;
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO users (address, created_at, alias, tags)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(address) DO UPDATE SET
                alias = excluded.alias,
                tags = excluded.tags;
            "#,
            params![user.address, user.created_at, user.alias, tags_json],
        )?;

        Ok(())
    }

    /// Retrieves user metadata by Stellar address
    pub fn get_user(&self, address: &str) -> Result<Option<User>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT address, created_at, alias, tags FROM users WHERE address = ?1",
        )?;

        let user = stmt
            .query_row(params![address], |row| {
                let tags_raw: String = row.get(3)?;
                let tags: Vec<String> = serde_json::from_str(&tags_raw).unwrap_or_default();

                Ok(User {
                    address: row.get(0)?,
                    created_at: row.get(1)?,
                    alias: row.get(2)?,
                    tags,
                })
            })
            .optional()?;

        Ok(user)
    }

    /// Records a new confirmed gift transaction and returns the persisted record with ID
    pub fn record_gift(&self, gift: &NewGiftRecord) -> Result<GiftRecord, DbError> {
        if gift.tx_hash.trim().is_empty() {
            return Err(DbError::Validation("Transaction hash cannot be empty".to_string()));
        }
        if gift.from_address.trim().is_empty() || gift.to_address.trim().is_empty() {
            return Err(DbError::Validation("Addresses cannot be empty".to_string()));
        }

        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO gift_records (tx_hash, from_address, to_address, amount, memo, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                gift.tx_hash,
                gift.from_address,
                gift.to_address,
                gift.amount,
                gift.memo,
                gift.timestamp
            ],
        )?;

        let id = conn.last_insert_rowid();

        Ok(GiftRecord {
            id,
            tx_hash: gift.tx_hash.clone(),
            from_address: gift.from_address.clone(),
            to_address: gift.to_address.clone(),
            amount: gift.amount,
            memo: gift.memo.clone(),
            timestamp: gift.timestamp,
        })
    }

    /// Retrieves transaction history for an address (both sent and received gifts), ordered newest first
    pub fn get_user_history(&self, address: &str) -> Result<Vec<GiftRecord>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, tx_hash, from_address, to_address, amount, memo, timestamp
            FROM gift_records
            WHERE from_address = ?1 OR to_address = ?1
            ORDER BY timestamp DESC, id DESC
            "#,
        )?;

        let rows = stmt.query_map(params![address], |row| {
            Ok(GiftRecord {
                id: row.get(0)?,
                tx_hash: row.get(1)?,
                from_address: row.get(2)?,
                to_address: row.get(3)?,
                amount: row.get(4)?,
                memo: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for item in rows {
            results.push(item?);
        }

        Ok(results)
    }

    /// Retrieves gifts sent by an address
    pub fn get_sent_gifts(&self, address: &str) -> Result<Vec<GiftRecord>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, tx_hash, from_address, to_address, amount, memo, timestamp
            FROM gift_records
            WHERE from_address = ?1
            ORDER BY timestamp DESC, id DESC
            "#,
        )?;

        let rows = stmt.query_map(params![address], |row| {
            Ok(GiftRecord {
                id: row.get(0)?,
                tx_hash: row.get(1)?,
                from_address: row.get(2)?,
                to_address: row.get(3)?,
                amount: row.get(4)?,
                memo: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for item in rows {
            results.push(item?);
        }

        Ok(results)
    }

    /// Retrieves gifts received by an address
    pub fn get_received_gifts(&self, address: &str) -> Result<Vec<GiftRecord>, DbError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, tx_hash, from_address, to_address, amount, memo, timestamp
            FROM gift_records
            WHERE to_address = ?1
            ORDER BY timestamp DESC, id DESC
            "#,
        )?;

        let rows = stmt.query_map(params![address], |row| {
            Ok(GiftRecord {
                id: row.get(0)?,
                tx_hash: row.get(1)?,
                from_address: row.get(2)?,
                to_address: row.get(3)?,
                amount: row.get(4)?,
                memo: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for item in rows {
            results.push(item?);
        }

        Ok(results)
    }
}
