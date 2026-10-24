use serde::{Deserialize, Serialize};

/// Off-chain user metadata entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub address: String,
    pub created_at: i64,
    pub alias: Option<String>,
    pub tags: Vec<String>,
}

impl User {
    pub fn new(address: impl Into<String>, created_at: i64) -> Self {
        Self {
            address: address.into(),
            created_at,
            alias: None,
            tags: Vec::new(),
        }
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// DTO for recording a newly confirmed gift transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewGiftRecord {
    pub tx_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub timestamp: i64,
}

/// Persisted gift transaction entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GiftRecord {
    pub id: i64,
    pub tx_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub timestamp: i64,
}
