use std::{fs::File, str::FromStr};

use anyhow::{Context, Result};
use clap::Parser;
use csv::Writer;
use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_pubkey::Pubkey;

// Constants
const VALIDATOR_HISTORY_PROGRAM: &str = "HistoryJTGbKQD2mRgLZ3XhqHnN811Qpez8X9kCcGHoa";
const DEFAULT_RPC: &str = "https://api.mainnet-beta.solana.com";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// RPC URL to connect to
    #[arg(short, long, default_value = DEFAULT_RPC)]
    rpc_url: String,

    /// Output CSV file path
    #[arg(short, long, default_value = "validator_history_accounts.csv")]
    output: String,

    /// Include account balances
    #[arg(short, long, default_value = "true")]
    include_balance: bool,
}

#[derive(Debug, Serialize)]
struct ValidatorHistoryAccount {
    address: String,
    balance_lamports: u64,
    balance_sol: f64,
    data_size: usize,
}

struct ValidatorHistoryExporter {
    client: RpcClient,
    program_id: Pubkey,
}

impl ValidatorHistoryExporter {
    fn new(rpc_url: &str) -> Result<Self> {
        let client = RpcClient::new(rpc_url.to_string());
        let program_id =
            Pubkey::from_str(VALIDATOR_HISTORY_PROGRAM).context("Failed to parse program ID")?;

        Ok(Self { client, program_id })
    }

    fn get_all_accounts(&self) -> Result<Vec<ValidatorHistoryAccount>> {
        println!(
            "Fetching accounts from program: {}",
            VALIDATOR_HISTORY_PROGRAM
        );
        println!("Using RPC: {}", self.client.url());

        let accounts = self
            .client
            .get_program_accounts(&self.program_id)
            .context("Failed to fetch program accounts")?;

        println!("Found {} accounts", accounts.len());

        let validator_accounts: Vec<ValidatorHistoryAccount> = accounts
            .into_iter()
            .map(|(pubkey, account)| {
                let balance_lamports = account.lamports;
                let balance_sol = balance_lamports as f64 / 1e9;
                let data_size = account.data.len();

                ValidatorHistoryAccount {
                    address: pubkey.to_string(),
                    balance_lamports,
                    balance_sol,
                    data_size,
                }
            })
            .collect();

        Ok(validator_accounts)
    }

    fn export_to_csv(&self, accounts: &[ValidatorHistoryAccount], output_path: &str) -> Result<()> {
        let file = File::create(output_path)
            .context(format!("Failed to create output file: {}", output_path))?;

        let mut writer = Writer::from_writer(file);

        for account in accounts {
            writer
                .serialize(account)
                .context("Failed to write account to CSV")?;
        }

        writer.flush().context("Failed to flush CSV writer")?;

        println!(
            "Successfully exported {} accounts to {}",
            accounts.len(),
            output_path
        );

        // Print summary statistics
        let total_rent_sol: f64 = accounts.iter().map(|a| a.balance_sol).sum();
        println!("\n=== Summary ===");
        println!("Total accounts: {}", accounts.len());
        println!("Total rent locked: {:.4} SOL", total_rent_sol);
        println!(
            "Average rent per account: {:.4} SOL",
            total_rent_sol / accounts.len() as f64
        );

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Jito Validator History Account Exporter");
    println!("=========================================\n");

    let exporter = ValidatorHistoryExporter::new(&args.rpc_url)?;

    println!("Fetching validator history accounts...");
    let accounts = exporter.get_all_accounts()?;

    println!("Exporting to CSV...");
    exporter.export_to_csv(&accounts, &args.output)?;

    Ok(())
}
