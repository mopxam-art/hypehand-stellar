//! Off-Chain Database Layer and Repository for HypeHand Stellar Services
//!
//! Provides SQLite-backed persistent storage for user metadata, aliases, tags,
//! and gift transaction logs.

pub mod error;
pub mod models;
pub mod repo;

pub use error::DbError;
pub use models::{GiftRecord, NewGiftRecord, User};
pub use repo::DatabaseRepository;

#[cfg(test)]
mod tests {
    use super::*;

    const ALICE_ADDR: &str = "GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ";
    const BOB_ADDR: &str = "GB72D2W7D2V6YV222YFUXUAK3Y424EFTK76P36PB7W2QG5Y67E33SXYY";
    const CHARLIE_ADDR: &str = "GC2M6H5GEXQO6W7K45XU2SJZK4QFYWUGI46D4B7P36K25Y7M6P43RXYZ";

    #[test]
    fn test_insert_and_get_user() {
        let repo = DatabaseRepository::open_in_memory().expect("Failed to open db");

        let user = User::new(ALICE_ADDR, 1700000000)
            .with_alias("alice_wonderland")
            .with_tags(vec!["vip".to_string(), "early_adopter".to_string()]);

        repo.upsert_user(&user).expect("Failed to insert user");

        let fetched = repo.get_user(ALICE_ADDR).expect("Failed to get user");
        assert!(fetched.is_some());
        let fetched_user = fetched.unwrap();
        assert_eq!(fetched_user.address, ALICE_ADDR);
        assert_eq!(fetched_user.alias, Some("alice_wonderland".to_string()));
        assert_eq!(fetched_user.created_at, 1700000000);
        assert_eq!(fetched_user.tags, vec!["vip", "early_adopter"]);

        // Non-existent user
        let missing = repo.get_user(BOB_ADDR).expect("Query should succeed");
        assert!(missing.is_none());
    }

    #[test]
    fn test_upsert_user_idempotency() {
        let repo = DatabaseRepository::open_in_memory().expect("Failed to open db");

        let user_v1 = User::new(BOB_ADDR, 1700001000)
            .with_alias("bob_initial")
            .with_tags(vec!["beta".to_string()]);

        repo.upsert_user(&user_v1).expect("First upsert should succeed");

        // Second upsert with updated alias and new tags, created_at is preserved
        let user_v2 = User::new(BOB_ADDR, 1700999999)
            .with_alias("bob_updated")
            .with_tags(vec!["beta".to_string(), "verified".to_string()]);

        repo.upsert_user(&user_v2).expect("Second upsert should succeed");

        let fetched = repo.get_user(BOB_ADDR).expect("Get user").unwrap();
        assert_eq!(fetched.alias, Some("bob_updated".to_string()));
        assert_eq!(fetched.tags, vec!["beta", "verified"]);
        // created_at should remain unchanged from first insertion
        assert_eq!(fetched.created_at, 1700001000);
    }

    #[test]
    fn test_record_and_get_gift_history() {
        let repo = DatabaseRepository::open_in_memory().expect("Failed to open db");

        let gift1 = NewGiftRecord {
            tx_hash: "0x1111111111111111".to_string(),
            from_address: ALICE_ADDR.to_string(),
            to_address: BOB_ADDR.to_string(),
            amount: 500_0000000,
            memo: Some("Happy Birthday!".to_string()),
            timestamp: 1700002000,
        };

        let gift2 = NewGiftRecord {
            tx_hash: "0x2222222222222222".to_string(),
            from_address: BOB_ADDR.to_string(),
            to_address: CHARLIE_ADDR.to_string(),
            amount: 200_0000000,
            memo: None,
            timestamp: 1700003000,
        };

        let gift3 = NewGiftRecord {
            tx_hash: "0x3333333333333333".to_string(),
            from_address: CHARLIE_ADDR.to_string(),
            to_address: ALICE_ADDR.to_string(),
            amount: 100_0000000,
            memo: Some("Thanks!".to_string()),
            timestamp: 1700004000,
        };

        let rec1 = repo.record_gift(&gift1).expect("Record gift 1");
        assert_eq!(rec1.id, 1);
        let rec2 = repo.record_gift(&gift2).expect("Record gift 2");
        assert_eq!(rec2.id, 2);
        let rec3 = repo.record_gift(&gift3).expect("Record gift 3");
        assert_eq!(rec3.id, 3);

        // Alice history (sent gift1 to Bob, received gift3 from Charlie)
        let alice_history = repo.get_user_history(ALICE_ADDR).expect("Get Alice history");
        assert_eq!(alice_history.len(), 2);
        // Ordered newest first
        assert_eq!(alice_history[0].tx_hash, "0x3333333333333333");
        assert_eq!(alice_history[1].tx_hash, "0x1111111111111111");

        // Bob history (received gift1, sent gift2)
        let bob_history = repo.get_user_history(BOB_ADDR).expect("Get Bob history");
        assert_eq!(bob_history.len(), 2);
        assert_eq!(bob_history[0].tx_hash, "0x2222222222222222");
        assert_eq!(bob_history[1].tx_hash, "0x1111111111111111");

        // Sent and received specific queries
        let bob_sent = repo.get_sent_gifts(BOB_ADDR).expect("Get Bob sent");
        assert_eq!(bob_sent.len(), 1);
        assert_eq!(bob_sent[0].to_address, CHARLIE_ADDR);

        let bob_received = repo.get_received_gifts(BOB_ADDR).expect("Get Bob received");
        assert_eq!(bob_received.len(), 1);
        assert_eq!(bob_received[0].from_address, ALICE_ADDR);
    }

    #[test]
    fn test_validation_errors() {
        let repo = DatabaseRepository::open_in_memory().expect("Failed to open db");

        // Empty user address
        let invalid_user = User::new("", 12345);
        assert!(matches!(
            repo.upsert_user(&invalid_user),
            Err(DbError::Validation(_))
        ));

        // Empty gift tx hash
        let invalid_gift = NewGiftRecord {
            tx_hash: "   ".to_string(),
            from_address: ALICE_ADDR.to_string(),
            to_address: BOB_ADDR.to_string(),
            amount: 100,
            memo: None,
            timestamp: 100,
        };
        assert!(matches!(
            repo.record_gift(&invalid_gift),
            Err(DbError::Validation(_))
        ));
    }
}
