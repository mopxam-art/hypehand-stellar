use config::{Config, Environment};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Testnet,
    Futurenet,
    Mainnet,
}

impl core::fmt::Display for Network {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Network::Testnet => write!(f, "testnet"),
            Network::Futurenet => write!(f, "futurenet"),
            Network::Mainnet => write!(f, "mainnet"),
        }
    }
}

impl std::str::FromStr for Network {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "testnet" => Ok(Network::Testnet),
            "futurenet" => Ok(Network::Futurenet),
            "mainnet" => Ok(Network::Mainnet),
            _ => Err(ConfigError::InvalidNetwork(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub network: Network,
    pub rpc_url: String,
    pub contract_id: Option<String>,
    pub admin_secret_key: Option<String>,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: Network::Testnet,
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            contract_id: None,
            admin_secret_key: None,
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    InvalidNetwork(String),
    InvalidRpcUrl(String),
    InvalidContractId(String),
    InvalidSecretKey(String),
    LoadError(String),
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::InvalidNetwork(net) => {
                write!(f, "Invalid network '{}', expected 'testnet', 'futurenet', or 'mainnet'", net)
            }
            ConfigError::InvalidRpcUrl(url) => {
                write!(f, "Invalid RPC URL '{}': must start with http:// or https://", url)
            }
            ConfigError::InvalidContractId(cid) => {
                write!(f, "Invalid Stellar contract ID '{}': must start with 'C' and be 56 characters long", cid)
            }
            ConfigError::InvalidSecretKey(key) => {
                write!(f, "Invalid Stellar admin secret key: must start with 'S' and be 56 characters long (got length {})", key.len())
            }
            ConfigError::LoadError(err) => {
                write!(f, "Failed to load configuration: {}", err)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl AppConfig {
    /// Cascading configuration loader:
    /// 1. Built-in sane defaults (Network::Testnet, official RPC)
    /// 2. Optional .env file (if present)
    /// 3. Environment variables prefixed with HYPEHAND_ (e.g. HYPEHAND_RPC_URL)
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_with_env_file(None)
    }

    /// Load configuration with an explicit optional .env file path
    pub fn load_with_env_file(env_path: Option<&Path>) -> Result<Self, ConfigError> {
        if let Some(path) = env_path {
            let _ = dotenvy::from_path(path);
        } else {
            let _ = dotenvy::dotenv();
        }

        let defaults = AppConfig::default();

        let builder = Config::builder()
            .set_default("network", defaults.network.to_string())
            .map_err(|e| ConfigError::LoadError(e.to_string()))?
            .set_default("rpc_url", defaults.rpc_url)
            .map_err(|e| ConfigError::LoadError(e.to_string()))?
            .set_default("log_level", defaults.log_level)
            .map_err(|e| ConfigError::LoadError(e.to_string()))?
            .add_source(
                Environment::with_prefix("HYPEHAND")
                    .separator("__")
                    .prefix_separator("_"),
            );

        let config: AppConfig = builder
            .build()
            .map_err(|e| ConfigError::LoadError(e.to_string()))?
            .try_deserialize()
            .map_err(|e| ConfigError::LoadError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Validates all fields against domain integrity rules
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate RPC URL format
        if !self.rpc_url.starts_with("http://") && !self.rpc_url.starts_with("https://") {
            return Err(ConfigError::InvalidRpcUrl(self.rpc_url.clone()));
        }

        // Validate Contract ID format if provided
        if let Some(ref cid) = self.contract_id {
            if cid.len() != 56 || !cid.starts_with('C') || !cid.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(ConfigError::InvalidContractId(cid.clone()));
            }
        }

        // Validate Admin Secret Key format if provided
        if let Some(ref sec) = self.admin_secret_key {
            if sec.len() != 56 || !sec.starts_with('S') || !sec.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(ConfigError::InvalidSecretKey(sec.clone()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Mutex for serializing tests that manipulate global environment variables
    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_default_config_loading() {
        let _guard = ENV_MUTEX.lock().unwrap();
        // Clear env vars that might interfere
        std::env::remove_var("HYPEHAND_NETWORK");
        std::env::remove_var("HYPEHAND_RPC_URL");
        std::env::remove_var("HYPEHAND_CONTRACT_ID");
        std::env::remove_var("HYPEHAND_ADMIN_SECRET_KEY");
        std::env::remove_var("HYPEHAND_LOG_LEVEL");

        let cfg = AppConfig::load_with_env_file(None).expect("Failed to load default config");
        assert_eq!(cfg.network, Network::Testnet);
        assert_eq!(cfg.rpc_url, "https://soroban-testnet.stellar.org");
        assert_eq!(cfg.contract_id, None);
        assert_eq!(cfg.admin_secret_key, None);
        assert_eq!(cfg.log_level, "info");
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_override_via_environment_variables() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("HYPEHAND_NETWORK", "futurenet");
        std::env::set_var("HYPEHAND_RPC_URL", "https://rpc-futurenet.stellar.org");
        std::env::set_var(
            "HYPEHAND_CONTRACT_ID",
            "CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5555555555555555555555",
        );
        std::env::set_var(
            "HYPEHAND_ADMIN_SECRET_KEY",
            "SA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5555555555555555555555",
        );
        std::env::set_var("HYPEHAND_LOG_LEVEL", "debug");

        let cfg = AppConfig::load_with_env_file(None).expect("Failed to load overridden config");
        assert_eq!(cfg.network, Network::Futurenet);
        assert_eq!(cfg.rpc_url, "https://rpc-futurenet.stellar.org");
        assert_eq!(
            cfg.contract_id,
            Some("CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5555555555555555555555".to_string())
        );
        assert_eq!(
            cfg.admin_secret_key,
            Some("SA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5555555555555555555555".to_string())
        );
        assert_eq!(cfg.log_level, "debug");

        // Clean up
        std::env::remove_var("HYPEHAND_NETWORK");
        std::env::remove_var("HYPEHAND_RPC_URL");
        std::env::remove_var("HYPEHAND_CONTRACT_ID");
        std::env::remove_var("HYPEHAND_ADMIN_SECRET_KEY");
        std::env::remove_var("HYPEHAND_LOG_LEVEL");
    }

    #[test]
    fn test_override_via_custom_env_file() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir = std::env::temp_dir();
        let env_file_path = temp_dir.join("hypehand_test.env");

        let content = "HYPEHAND_NETWORK=mainnet\nHYPEHAND_LOG_LEVEL=warn\n";
        fs::write(&env_file_path, content).expect("Failed to write temporary .env");

        let cfg = AppConfig::load_with_env_file(Some(&env_file_path)).expect("Load custom .env");
        assert_eq!(cfg.network, Network::Mainnet);
        assert_eq!(cfg.log_level, "warn");

        let _ = fs::remove_file(&env_file_path);
        std::env::remove_var("HYPEHAND_NETWORK");
        std::env::remove_var("HYPEHAND_LOG_LEVEL");
    }

    #[test]
    fn test_validation_errors() {
        // Invalid RPC URL
        let cfg_bad_rpc = AppConfig {
            rpc_url: "ftp://invalid-rpc-address.org".to_string(),
            ..Default::default()
        };
        assert!(matches!(cfg_bad_rpc.validate(), Err(ConfigError::InvalidRpcUrl(_))));

        // Valid RPC URL, invalid contract ID (too short)
        let cfg_short_cid = AppConfig {
            contract_id: Some("CTOOSHORT".to_string()),
            ..Default::default()
        };
        assert!(matches!(cfg_short_cid.validate(), Err(ConfigError::InvalidContractId(_))));

        // Invalid contract ID prefix (must start with C)
        let cfg_bad_prefix = AppConfig {
            contract_id: Some("GA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5555555555555555555555".to_string()),
            ..Default::default()
        };
        assert!(matches!(cfg_bad_prefix.validate(), Err(ConfigError::InvalidContractId(_))));

        // Valid contract ID, invalid secret key (must start with S)
        let cfg_bad_sec = AppConfig {
            contract_id: Some("CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5555555555555555555555".to_string()),
            admin_secret_key: Some("GA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5555555555555555555555".to_string()),
            ..Default::default()
        };
        assert!(matches!(cfg_bad_sec.validate(), Err(ConfigError::InvalidSecretKey(_))));

        // Valid configuration
        let cfg_valid = AppConfig {
            contract_id: Some("CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5555555555555555555555".to_string()),
            admin_secret_key: Some("SA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5555555555555555555555".to_string()),
            ..Default::default()
        };
        assert!(cfg_valid.validate().is_ok());
    }

    #[test]
    fn test_network_from_str() {
        assert_eq!("testnet".parse::<Network>().unwrap(), Network::Testnet);
        assert_eq!("futurenet".parse::<Network>().unwrap(), Network::Futurenet);
        assert_eq!("mainnet".parse::<Network>().unwrap(), Network::Mainnet);
        assert!("invalid".parse::<Network>().is_err());
    }
}
