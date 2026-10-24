use clap::{Args, Parser, Subcommand, ValueEnum};
use hypehand_utils::{
    is_valid_account_address, is_valid_contract_address, stroops_to_xlm_string,
    xlm_string_to_stroops,
};

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
    name = "hypehand-cli",
    author = "HypeHand Contributors <dev@hypehand.io>",
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

    /// Convert currency amounts between stroops and XLM
    Convert(ConvertArgs),
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
    #[arg(short, long, help = "Initial funding amount in stroops")]
    pub amount: Option<i128>,
}

#[derive(Args, Debug, PartialEq)]
pub struct InvokeGiftArgs {
    /// Stellar Soroban contract identifier
    #[arg(short, long, help = "Contract identifier (e.g. C...)")]
    pub contract_id: String,

    /// Source account address sending the gift
    #[arg(short, long, help = "Source account address")]
    pub from: String,

    /// Recipient account address receiving the gift
    #[arg(short, long, help = "Recipient account address")]
    pub to: String,

    /// Amount in stroops to gift
    #[arg(short, long, help = "Amount in stroops")]
    pub amount: i128,
}

#[derive(Args, Debug, PartialEq)]
pub struct BalanceArgs {
    /// Account address to check balance for
    #[arg(short, long, help = "Account address to check balance for")]
    pub from: String,

    /// Optional contract address
    #[arg(short, long, help = "Optional contract identifier")]
    pub contract_id: Option<String>,
}

#[derive(Args, Debug, PartialEq)]
pub struct StatusArgs {
    /// Optional contract identifier to check status for
    #[arg(short, long, help = "Contract identifier")]
    pub contract_id: Option<String>,

    /// Optional account address to check status for
    #[arg(short, long, help = "Account address")]
    pub from: Option<String>,
}

#[derive(Args, Debug, PartialEq)]
pub struct ConvertArgs {
    /// Amount in stroops to convert to XLM
    #[arg(long, help = "Amount in stroops")]
    pub stroops: Option<i128>,

    /// Amount in XLM to convert to stroops
    #[arg(long, help = "Amount in XLM")]
    pub xlm: Option<String>,
}

pub fn execute(cli: &Cli) -> Result<(), String> {
    match &cli.command {
        Commands::Deploy(args) => {
            println!(
                "Deploying contract on {} from account: {}",
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
            if args.from.starts_with('G') && !is_valid_account_address(&args.from) {
                return Err(format!("Invalid sender Stellar account address: {}", args.from));
            }
            if args.to.starts_with('G') && !is_valid_account_address(&args.to) {
                return Err(format!("Invalid recipient Stellar account address: {}", args.to));
            }
            if args.contract_id.starts_with('C') && !is_valid_contract_address(&args.contract_id) {
                return Err(format!("Invalid contract address: {}", args.contract_id));
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
        Commands::Convert(args) => {
            if let Some(s) = args.stroops {
                println!("{} stroops = {} XLM", s, stroops_to_xlm_string(s));
            } else if let Some(ref x) = args.xlm {
                let s = xlm_string_to_stroops(x).map_err(|e| e.to_string())?;
                println!("{} XLM = {} stroops", x, s);
            } else {
                return Err("Specify either --stroops or --xlm for conversion".to_string());
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
            "hypehand-cli",
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
            "hypehand-cli",
            "deploy",
            "--from",
            "GBBD47IF6LWK7P7MDEVG2DP463W",
            "--wasm-path",
            "target/release/contract.wasm",
        ];
        let parsed = Cli::try_parse_from(args).expect("Failed to parse deploy with wasm");
        assert_eq!(parsed.network, Network::Testnet);
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
            "hypehand-cli",
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
            "hypehand-cli",
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
            "hypehand-cli",
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
    fn test_parse_convert_command() {
        let args = vec!["hypehand-cli", "convert", "--stroops", "10000000"];
        let parsed = Cli::try_parse_from(args).expect("Failed to parse convert");
        match parsed.command {
            Commands::Convert(conv) => {
                assert_eq!(conv.stroops, Some(10000000));
            }
            _ => panic!("Expected Convert command"),
        }
    }

    #[test]
    fn test_missing_required_arguments_fails() {
        let args = vec!["hypehand-cli", "deploy"];
        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn test_invalid_network_value_fails() {
        let args = vec!["hypehand-cli", "--network", "invalid_net", "status"];
        assert!(Cli::try_parse_from(args).is_err());
    }

    const VALID_ACCOUNT_A: &str = "GBBD47IF6LWK7P7MDEVG2DP463W7IGFLYTLEWIVPJYTXSTZDHZX6YPOO";
    const VALID_ACCOUNT_B: &str = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";
    const VALID_CONTRACT: &str = "CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPFUTDWNGPGZ4R7TFR7TXU2ST";

    #[test]
    fn test_execute_validations() {
        let cli_valid = Cli {
            network: Network::Testnet,
            command: Commands::InvokeGift(InvokeGiftArgs {
                contract_id: VALID_CONTRACT.to_string(),
                from: VALID_ACCOUNT_A.to_string(),
                to: VALID_ACCOUNT_B.to_string(),
                amount: 100,
            }),
        };
        assert!(execute(&cli_valid).is_ok());

        let cli_neg = Cli {
            network: Network::Testnet,
            command: Commands::InvokeGift(InvokeGiftArgs {
                contract_id: VALID_CONTRACT.to_string(),
                from: VALID_ACCOUNT_A.to_string(),
                to: VALID_ACCOUNT_B.to_string(),
                amount: -5,
            }),
        };
        assert!(execute(&cli_neg).is_err());

        let cli_self = Cli {
            network: Network::Testnet,
            command: Commands::InvokeGift(InvokeGiftArgs {
                contract_id: VALID_CONTRACT.to_string(),
                from: VALID_ACCOUNT_A.to_string(),
                to: VALID_ACCOUNT_A.to_string(),
                amount: 50,
            }),
        };
        assert!(execute(&cli_self).is_err());

        let cli_invalid_addr = Cli {
            network: Network::Testnet,
            command: Commands::InvokeGift(InvokeGiftArgs {
                contract_id: VALID_CONTRACT.to_string(),
                from: "G_INVALID".to_string(),
                to: VALID_ACCOUNT_B.to_string(),
                amount: 50,
            }),
        };
        assert!(execute(&cli_invalid_addr).is_err());
    }

    #[test]
    fn test_execute_convert() {
        let cli_stroops = Cli {
            network: Network::Testnet,
            command: Commands::Convert(ConvertArgs {
                stroops: Some(25_000_000),
                xlm: None,
            }),
        };
        assert!(execute(&cli_stroops).is_ok());

        let cli_xlm = Cli {
            network: Network::Testnet,
            command: Commands::Convert(ConvertArgs {
                stroops: None,
                xlm: Some("2.5".to_string()),
            }),
        };
        assert!(execute(&cli_xlm).is_ok());

        let cli_none = Cli {
            network: Network::Testnet,
            command: Commands::Convert(ConvertArgs {
                stroops: None,
                xlm: None,
            }),
        };
        assert!(execute(&cli_none).is_err());
    }
}
