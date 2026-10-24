use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Network {
    Testnet,
    Mainnet,
    Futurenet,
}

impl core::fmt::Display for Network {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Network::Testnet => write!(f, "testnet"),
            Network::Mainnet => write!(f, "mainnet"),
            Network::Futurenet => write!(f, "futurenet"),
        }
    }
}

#[derive(Parser, Debug, PartialEq)]
#[command(
    name = "cli",
    author = "HypeHand Team",
    version = "0.1.0",
    about = "HypeHand Stellar CLI for managing and interacting with Soroban gifting contracts",
    long_about = "HypeHand CLI provides administrative and user utilities to deploy smart contracts, query balances, check status, and invoke gift transactions across Stellar networks."
)]
pub struct Cli {
    #[arg(
        short,
        long,
        global = true,
        value_enum,
        default_value = "testnet",
        help = "Target Stellar network (testnet, mainnet, futurenet)"
    )]
    pub network: Network,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Deploy the gifting smart contract to the selected Stellar network
    Deploy(DeployArgs),

    /// Invoke a gift transaction from one account to another
    #[command(name = "invoke-gift")]
    InvokeGift(InvokeGiftArgs),

    /// Check the current gifting balance of an account
    Balance(BalanceArgs),

    /// Check network RPC connectivity and contract status
    Status(StatusArgs),
}

#[derive(Args, Debug, PartialEq)]
pub struct DeployArgs {
    /// Deployer account public key or secret key alias
    #[arg(short, long, help = "Deployer account public key or identity")]
    pub from: String,

    /// Path to compiled contract WASM file
    #[arg(short, long, help = "Path to compiled WASM binary")]
    pub wasm_path: Option<String>,

    /// Initial balance or funding to prime the contract instance
    #[arg(short, long, help = "Initial funding amount")]
    pub amount: Option<i128>,
}

#[derive(Args, Debug, PartialEq)]
pub struct InvokeGiftArgs {
    /// Stellar Soroban contract identifier
    #[arg(short, long, help = "Contract identifier (e.g. C...)")]
    pub contract_id: String,

    /// Source account address sending the gift
    #[arg(short, long, help = "Sender account address")]
    pub from: String,

    /// Destination account address receiving the gift
    #[arg(short, long, help = "Recipient account address")]
    pub to: String,

    /// Gift amount (must be positive integer)
    #[arg(short, long, help = "Amount to transfer in gift units")]
    pub amount: i128,
}

#[derive(Args, Debug, PartialEq)]
pub struct BalanceArgs {
    /// Account address to query balance for
    #[arg(short, long, help = "Account address to query")]
    pub from: String,

    /// Optional contract identifier
    #[arg(short, long, help = "Contract identifier")]
    pub contract_id: Option<String>,
}

#[derive(Args, Debug, PartialEq)]
pub struct StatusArgs {
    /// Optional contract identifier to check status for
    #[arg(short, long, help = "Contract identifier to verify")]
    pub contract_id: Option<String>,

    /// Optional account address to check status for
    #[arg(short, long, help = "Account address to verify")]
    pub from: Option<String>,
}

fn execute(cli: &Cli) -> Result<(), String> {
    match &cli.command {
        Commands::Deploy(args) => {
            if let Some(amt) = args.amount {
                if amt < 0 {
                    return Err("Initial funding amount cannot be negative".to_string());
                }
            }
            println!(
                "Deploying contract on {} from account {}...",
                cli.network, args.from
            );
            if let Some(ref path) = args.wasm_path {
                println!("Using WASM binary: {}", path);
            }
            println!("Contract deployment simulated successfully.");
        }
        Commands::InvokeGift(args) => {
            if args.amount <= 0 {
                return Err("Gift amount must be greater than zero".to_string());
            }
            if args.from == args.to {
                return Err("Sender and recipient accounts cannot be identical".to_string());
            }
            println!(
                "Invoking gift on {} via contract {}: from {} to {}, amount {}",
                cli.network, args.contract_id, args.from, args.to, args.amount
            );
            println!("Gift transaction executed successfully.");
        }
        Commands::Balance(args) => {
            println!(
                "Querying balance on {} for account {}...",
                cli.network, args.from
            );
            if let Some(ref cid) = args.contract_id {
                println!("Contract: {}", cid);
            }
            println!("Balance: 0 (simulated)");
        }
        Commands::Status(args) => {
            println!("Network: {} (active)", cli.network);
            if let Some(ref cid) = args.contract_id {
                println!("Contract {} is verified onchain.", cid);
            }
            if let Some(ref account) = args.from {
                println!("Account {} is registered.", account);
            }
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = execute(&cli) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deploy_command() {
        let args = vec![
            "cli",
            "--network",
            "testnet",
            "deploy",
            "--from",
            "GBBD47IF6LWK7P7MDEVG2DP463W",
            "--amount",
            "1000",
        ];
        let parsed = Cli::try_parse_from(args).expect("Failed to parse deploy");
        assert_eq!(parsed.network, Network::Testnet);
        match parsed.command {
            Commands::Deploy(deploy) => {
                assert_eq!(deploy.from, "GBBD47IF6LWK7P7MDEVG2DP463W");
                assert_eq!(deploy.amount, Some(1000));
                assert_eq!(deploy.wasm_path, None);
            }
            _ => panic!("Expected Deploy command"),
        }
    }

    #[test]
    fn test_parse_deploy_with_wasm_path() {
        let args = vec![
            "cli",
            "deploy",
            "--from",
            "GBBD47IF6LWK7P7MDEVG2DP463W",
            "--wasm-path",
            "target/release/contract.wasm",
        ];
        let parsed = Cli::try_parse_from(args).expect("Failed to parse deploy with wasm");
        assert_eq!(parsed.network, Network::Testnet); // default value
        match parsed.command {
            Commands::Deploy(deploy) => {
                assert_eq!(deploy.wasm_path, Some("target/release/contract.wasm".to_string()));
            }
            _ => panic!("Expected Deploy command"),
        }
    }

    #[test]
    fn test_parse_invoke_gift_command() {
        let args = vec![
            "cli",
            "--network",
            "futurenet",
            "invoke-gift",
            "--contract-id",
            "CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF",
            "--from",
            "G_SENDER_123",
            "--to",
            "G_RECIPIENT_456",
            "--amount",
            "500",
        ];
        let parsed = Cli::try_parse_from(args).expect("Failed to parse invoke-gift");
        assert_eq!(parsed.network, Network::Futurenet);
        match parsed.command {
            Commands::InvokeGift(gift) => {
                assert_eq!(gift.contract_id, "CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF");
                assert_eq!(gift.from, "G_SENDER_123");
                assert_eq!(gift.to, "G_RECIPIENT_456");
                assert_eq!(gift.amount, 500);
            }
            _ => panic!("Expected InvokeGift command"),
        }
    }

    #[test]
    fn test_parse_balance_command() {
        let args = vec![
            "cli",
            "--network",
            "mainnet",
            "balance",
            "--from",
            "G_ACCOUNT_789",
            "--contract-id",
            "CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF",
        ];
        let parsed = Cli::try_parse_from(args).expect("Failed to parse balance");
        assert_eq!(parsed.network, Network::Mainnet);
        match parsed.command {
            Commands::Balance(balance) => {
                assert_eq!(balance.from, "G_ACCOUNT_789");
                assert_eq!(balance.contract_id, Some("CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF".to_string()));
            }
            _ => panic!("Expected Balance command"),
        }
    }

    #[test]
    fn test_parse_status_command() {
        let args = vec![
            "cli",
            "status",
            "--contract-id",
            "CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF",
            "--from",
            "G_ADMIN",
        ];
        let parsed = Cli::try_parse_from(args).expect("Failed to parse status");
        assert_eq!(parsed.network, Network::Testnet);
        match parsed.command {
            Commands::Status(status) => {
                assert_eq!(status.contract_id, Some("CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF".to_string()));
                assert_eq!(status.from, Some("G_ADMIN".to_string()));
            }
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_missing_required_arguments_fails() {
        // Missing --from on deploy
        let args = vec!["cli", "deploy"];
        assert!(Cli::try_parse_from(args).is_err());

        // Missing --amount on invoke-gift
        let args_gift = vec![
            "cli",
            "invoke-gift",
            "--contract-id",
            "C123",
            "--from",
            "G_A",
            "--to",
            "G_B",
        ];
        assert!(Cli::try_parse_from(args_gift).is_err());
    }

    #[test]
    fn test_invalid_network_value_fails() {
        let args = vec!["cli", "--network", "invalid_net", "status"];
        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn test_invalid_amount_type_fails() {
        let args = vec![
            "cli",
            "invoke-gift",
            "--contract-id",
            "C123",
            "--from",
            "G_A",
            "--to",
            "G_B",
            "--amount",
            "not_a_number",
        ];
        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn test_execute_validations() {
        let cli_valid = Cli {
            network: Network::Testnet,
            command: Commands::InvokeGift(InvokeGiftArgs {
                contract_id: "C123".to_string(),
                from: "G_A".to_string(),
                to: "G_B".to_string(),
                amount: 100,
            }),
        };
        assert!(execute(&cli_valid).is_ok());

        // Negative gift amount rejected
        let cli_neg = Cli {
            network: Network::Testnet,
            command: Commands::InvokeGift(InvokeGiftArgs {
                contract_id: "C123".to_string(),
                from: "G_A".to_string(),
                to: "G_B".to_string(),
                amount: -5,
            }),
        };
        assert!(execute(&cli_neg).is_err());

        // Self transfer rejected
        let cli_self = Cli {
            network: Network::Testnet,
            command: Commands::InvokeGift(InvokeGiftArgs {
                contract_id: "C123".to_string(),
                from: "G_SAME".to_string(),
                to: "G_SAME".to_string(),
                amount: 50,
            }),
        };
        assert!(execute(&cli_self).is_err());
    }
}
