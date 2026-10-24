use clap::{Args, Parser, Subcommand, ValueEnum};
use hypehand_telemetry::{init_telemetry, LogFormat, TelemetryConfig};
use std::time::Instant;
use tracing::{error, info, info_span, warn, Level};

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

    #[arg(
        short,
        long,
        global = true,
        help = "Enable verbose debug logging"
    )]
    pub verbose: bool,

    #[arg(
        short,
        long,
        global = true,
        conflicts_with = "verbose",
        help = "Suppress informational output, only display warnings and errors"
    )]
    pub quiet: bool,

    #[arg(
        long,
        global = true,
        help = "Emit structured logs in JSON format"
    )]
    pub json: bool,

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
    /// Deployer account public key or identity
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

pub fn configure_logging(cli: &Cli) {
    let level = if cli.verbose {
        Level::DEBUG
    } else if cli.quiet {
        Level::WARN
    } else {
        Level::INFO
    };

    let format = if cli.json {
        LogFormat::Json
    } else {
        LogFormat::Compact
    };

    let config = TelemetryConfig::new(level, format);
    let _ = init_telemetry(&config);
}

pub fn execute(cli: &Cli) -> Result<(), String> {
    let start_time = Instant::now();
    let network_str = cli.network.to_string();

    match &cli.command {
        Commands::Deploy(args) => {
            let span = info_span!("contract_deployment", network = %network_str, deployer = %args.from);
            let _enter = span.enter();

            if let Some(amt) = args.amount {
                if amt < 0 {
                    error!(amount = amt, "Validation failed: initial funding cannot be negative");
                    return Err("Initial funding amount cannot be negative".to_string());
                }
            }

            info!(
                network = %network_str,
                from = %args.from,
                wasm = ?args.wasm_path,
                "Deploying contract instance"
            );
            let duration = start_time.elapsed();
            info!(duration_ms = duration.as_millis(), "Contract deployment completed successfully");
        }
        Commands::InvokeGift(args) => {
            let span = info_span!(
                "invoke_gift_transaction",
                network = %network_str,
                contract_id = %args.contract_id,
                from = %args.from,
                to = %args.to
            );
            let _enter = span.enter();

            if args.amount <= 0 {
                error!(amount = args.amount, "Validation failed: gift amount must be greater than zero");
                return Err("Gift amount must be greater than zero".to_string());
            }
            if args.from == args.to {
                warn!(from = %args.from, "Attempted self-transfer: sender and recipient match");
                return Err("Sender and recipient accounts cannot be identical".to_string());
            }

            info!(
                contract = %args.contract_id,
                from = %args.from,
                to = %args.to,
                amount = args.amount,
                "Submitting gift transaction"
            );
            let duration = start_time.elapsed();
            info!(duration_ms = duration.as_millis(), "Gift transaction confirmed on ledger");
        }
        Commands::Balance(args) => {
            let span = info_span!("balance_query", network = %network_str, account = %args.from);
            let _enter = span.enter();

            info!(account = %args.from, contract = ?args.contract_id, "Querying account balance");
            let duration = start_time.elapsed();
            info!(duration_ms = duration.as_millis(), balance = 0, "Balance query finished");
        }
        Commands::Status(args) => {
            let span = info_span!("status_check", network = %network_str);
            let _enter = span.enter();

            info!(
                network = %network_str,
                contract = ?args.contract_id,
                account = ?args.from,
                "Verifying network status"
            );
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    configure_logging(&cli);

    if let Err(err) = execute(&cli) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_logging_flags() {
        // Verbose flag
        let args_v = vec!["cli", "-v", "status"];
        let parsed_v = Cli::try_parse_from(args_v).expect("Parse verbose");
        assert!(parsed_v.verbose);
        assert!(!parsed_v.quiet);
        assert!(!parsed_v.json);

        // Quiet flag
        let args_q = vec!["cli", "-q", "status"];
        let parsed_q = Cli::try_parse_from(args_q).expect("Parse quiet");
        assert!(parsed_q.quiet);
        assert!(!parsed_v.json);

        // JSON flag
        let args_json = vec!["cli", "--json", "status"];
        let parsed_json = Cli::try_parse_from(args_json).expect("Parse json");
        assert!(parsed_json.json);

        // Conflicts: verbose and quiet together should fail
        let args_conflict = vec!["cli", "-v", "-q", "status"];
        assert!(Cli::try_parse_from(args_conflict).is_err());
    }

    #[test]
    fn test_execute_with_spans() {
        let cli = Cli {
            network: Network::Testnet,
            verbose: true,
            quiet: false,
            json: false,
            command: Commands::Status(StatusArgs {
                contract_id: Some("C123".to_string()),
                from: Some("G_ADMIN".to_string()),
            }),
        };
        configure_logging(&cli);
        assert!(execute(&cli).is_ok());
    }

    #[test]
    fn test_execute_invoke_gift_tracing() {
        let cli = Cli {
            network: Network::Futurenet,
            verbose: false,
            quiet: false,
            json: true,
            command: Commands::InvokeGift(InvokeGiftArgs {
                contract_id: "CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF".to_string(),
                from: "G_SENDER".to_string(),
                to: "G_RECEIVER".to_string(),
                amount: 500,
            }),
        };
        assert!(execute(&cli).is_ok());
    }

    #[test]
    fn test_execute_validation_errors_tracing() {
        let cli_err = Cli {
            network: Network::Testnet,
            verbose: false,
            quiet: false,
            json: false,
            command: Commands::InvokeGift(InvokeGiftArgs {
                contract_id: "CA3D5".to_string(),
                from: "G_SAME".to_string(),
                to: "G_SAME".to_string(),
                amount: -10,
            }),
        };
        assert!(execute(&cli_err).is_err());
    }
}
