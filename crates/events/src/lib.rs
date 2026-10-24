use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCreatedEvent {
    pub creator: String,
    pub initial_balance: i128,
    pub timestamp: u64,
}

impl AccountCreatedEvent {
    pub fn new(creator: impl Into<String>, initial_balance: i128, timestamp: u64) -> Self {
        Self { creator: creator.into(), initial_balance, timestamp }
    }

    pub fn validate(&self) -> Result<(), EventValidationError> {
        if self.creator.trim().is_empty() {
            return Err(EventValidationError::EmptyAddress("creator".to_string()));
        }
        if self.initial_balance <= 0 {
            return Err(EventValidationError::InvalidAmount(self.initial_balance));
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GiftTransferredEvent {
    pub from: String,
    pub to: String,
    pub amount: i128,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

impl GiftTransferredEvent {
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: i128,
        timestamp: u64,
        memo: Option<String>,
    ) -> Self {
        Self { from: from.into(), to: to.into(), amount, timestamp, memo }
    }

    pub fn validate(&self) -> Result<(), EventValidationError> {
        if self.from.trim().is_empty() {
            return Err(EventValidationError::EmptyAddress("from".to_string()));
        }
        if self.to.trim().is_empty() {
            return Err(EventValidationError::EmptyAddress("to".to_string()));
        }
        if self.from == self.to {
            return Err(EventValidationError::SelfTransferNotAllowed);
        }
        if self.amount <= 0 {
            return Err(EventValidationError::InvalidAmount(self.amount));
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BalanceUpdatedEvent {
    pub account: String,
    pub old_balance: i128,
    pub new_balance: i128,
    pub timestamp: u64,
    pub reason: String,
}

impl BalanceUpdatedEvent {
    pub fn new(
        account: impl Into<String>,
        old_balance: i128,
        new_balance: i128,
        timestamp: u64,
        reason: impl Into<String>,
    ) -> Self {
        Self { account: account.into(), old_balance, new_balance, timestamp, reason: reason.into() }
    }

    pub fn validate(&self) -> Result<(), EventValidationError> {
        if self.account.trim().is_empty() {
            return Err(EventValidationError::EmptyAddress("account".to_string()));
        }
        if self.old_balance < 0 {
            return Err(EventValidationError::NegativeBalance(
                "old_balance".to_string(),
                self.old_balance,
            ));
        }
        if self.new_balance < 0 {
            return Err(EventValidationError::NegativeBalance(
                "new_balance".to_string(),
                self.new_balance,
            ));
        }
        if self.reason.trim().is_empty() {
            return Err(EventValidationError::EmptyField("reason".to_string()));
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Unified polymorphic event container for contract indexing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data", rename_all = "snake_case")]
pub enum ContractEvent {
    AccountCreated(AccountCreatedEvent),
    GiftTransferred(GiftTransferredEvent),
    BalanceUpdated(BalanceUpdatedEvent),
}

impl ContractEvent {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn validate(&self) -> Result<(), EventValidationError> {
        match self {
            ContractEvent::AccountCreated(e) => e.validate(),
            ContractEvent::GiftTransferred(e) => e.validate(),
            ContractEvent::BalanceUpdated(e) => e.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventValidationError {
    EmptyAddress(String),
    EmptyField(String),
    InvalidAmount(i128),
    NegativeBalance(String, i128),
    SelfTransferNotAllowed,
}

impl core::fmt::Display for EventValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EventValidationError::EmptyAddress(fld) => {
                write!(f, "Address field '{}' cannot be empty", fld)
            }
            EventValidationError::EmptyField(fld) => write!(f, "Field '{}' cannot be empty", fld),
            EventValidationError::InvalidAmount(amt) => {
                write!(f, "Amount must be strictly positive, got {}", amt)
            }
            EventValidationError::NegativeBalance(fld, bal) => {
                write!(f, "Balance '{}' cannot be negative, got {}", fld, bal)
            }
            EventValidationError::SelfTransferNotAllowed => {
                write!(f, "Sender and recipient accounts cannot be identical")
            }
        }
    }
}

impl std::error::Error for EventValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_created_roundtrip() {
        let event = AccountCreatedEvent::new("GBBD47IF6LWK7P7MDEVG2DP463W", 10_000, 1729780000);
        assert!(event.validate().is_ok());

        let json = event.to_json().expect("Serialization failed");
        let deserialized = AccountCreatedEvent::from_json(&json).expect("Deserialization failed");
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_account_created_validation() {
        // Empty creator
        let empty_creator = AccountCreatedEvent::new("", 10_000, 1729780000);
        assert_eq!(
            empty_creator.validate(),
            Err(EventValidationError::EmptyAddress("creator".to_string()))
        );

        // Zero or negative initial balance
        let zero_bal = AccountCreatedEvent::new("GBBD47IF6LWK7P7MDEVG2DP463W", 0, 1729780000);
        assert_eq!(zero_bal.validate(), Err(EventValidationError::InvalidAmount(0)));

        let neg_bal = AccountCreatedEvent::new("GBBD47IF6LWK7P7MDEVG2DP463W", -50, 1729780000);
        assert_eq!(neg_bal.validate(), Err(EventValidationError::InvalidAmount(-50)));
    }

    #[test]
    fn test_gift_transferred_roundtrip_with_and_without_memo() {
        // With memo
        let event_with_memo = GiftTransferredEvent::new(
            "G_SENDER_111",
            "G_RECIPIENT_222",
            2_500,
            1729780100,
            Some("Happy Birthday!".to_string()),
        );
        assert!(event_with_memo.validate().is_ok());

        let json = event_with_memo.to_json().expect("Serialization failed");
        assert!(json.contains("Happy Birthday!"));
        let deserialized = GiftTransferredEvent::from_json(&json).expect("Deserialization failed");
        assert_eq!(event_with_memo, deserialized);

        // Without memo
        let event_no_memo =
            GiftTransferredEvent::new("G_SENDER_111", "G_RECIPIENT_222", 1_000, 1729780200, None);
        assert!(event_no_memo.validate().is_ok());

        let json_no_memo = event_no_memo.to_json().expect("Serialization failed");
        assert!(!json_no_memo.contains("memo"));
        let deserialized_no_memo =
            GiftTransferredEvent::from_json(&json_no_memo).expect("Deserialization failed");
        assert_eq!(event_no_memo, deserialized_no_memo);
    }

    #[test]
    fn test_gift_transferred_validation() {
        // Self transfer
        let self_transfer = GiftTransferredEvent::new("G_SAME", "G_SAME", 500, 1729780000, None);
        assert_eq!(self_transfer.validate(), Err(EventValidationError::SelfTransferNotAllowed));

        // Invalid amount
        let invalid_amt = GiftTransferredEvent::new("G_SENDER", "G_RECIPIENT", 0, 1729780000, None);
        assert_eq!(invalid_amt.validate(), Err(EventValidationError::InvalidAmount(0)));

        // Empty sender or recipient
        let empty_from = GiftTransferredEvent::new("", "G_RECIPIENT", 500, 1729780000, None);
        assert_eq!(
            empty_from.validate(),
            Err(EventValidationError::EmptyAddress("from".to_string()))
        );
    }

    #[test]
    fn test_balance_updated_roundtrip_and_validation() {
        let event =
            BalanceUpdatedEvent::new("G_USER_333", 5_000, 7_500, 1729780300, "deposit_reward");
        assert!(event.validate().is_ok());

        let json = event.to_json().expect("Serialization failed");
        let deserialized = BalanceUpdatedEvent::from_json(&json).expect("Deserialization failed");
        assert_eq!(event, deserialized);

        // Negative balance validation
        let neg_old = BalanceUpdatedEvent::new("G_USER_333", -1, 100, 1729780300, "deposit");
        assert_eq!(
            neg_old.validate(),
            Err(EventValidationError::NegativeBalance("old_balance".to_string(), -1))
        );

        let empty_reason = BalanceUpdatedEvent::new("G_USER_333", 100, 200, 1729780300, "");
        assert_eq!(
            empty_reason.validate(),
            Err(EventValidationError::EmptyField("reason".to_string()))
        );
    }

    #[test]
    fn test_contract_event_polymorphic_enum() {
        let e1 = ContractEvent::AccountCreated(AccountCreatedEvent::new("G_CREATOR", 1000, 100));
        let e2 =
            ContractEvent::GiftTransferred(GiftTransferredEvent::new("G_A", "G_B", 200, 200, None));
        let e3 = ContractEvent::BalanceUpdated(BalanceUpdatedEvent::new(
            "G_A",
            1000,
            800,
            200,
            "gift_sent",
        ));

        for event in [&e1, &e2, &e3] {
            assert!(event.validate().is_ok());
            let json = event.to_json().expect("Enum serialization failed");
            let deserialized =
                ContractEvent::from_json(&json).expect("Enum deserialization failed");
            assert_eq!(event, &deserialized);
        }
    }

    #[test]
    fn test_invalid_json_payload_resilience() {
        // Malformed JSON syntax
        assert!(AccountCreatedEvent::from_json("{ not valid json }").is_err());
        assert!(GiftTransferredEvent::from_json("").is_err());

        // Missing required fields
        let missing_amount = r#"{"from": "G_A", "to": "G_B", "timestamp": 12345}"#;
        assert!(GiftTransferredEvent::from_json(missing_amount).is_err());

        // Field type mismatch (amount as string instead of integer)
        let type_mismatch =
            r#"{"creator": "G_A", "initial_balance": "lots_of_coins", "timestamp": 12345}"#;
        assert!(AccountCreatedEvent::from_json(type_mismatch).is_err());

        // Invalid polymorphic tag
        let invalid_tag = r#"{"event_type": "unknown_event", "data": {}}"#;
        assert!(ContractEvent::from_json(invalid_tag).is_err());
    }
}
