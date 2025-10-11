use std::fs::File;

use clap::Parser;
use csv::Writer;

use crate::exporter::Exporter;

pub mod exporter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// RPC URL to connect to
    #[arg(short, long)]
    rpc_url: String,

    /// Output CSV file path
    #[arg(short, long, default_value = "jito_all_accounts.csv")]
    output: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let programs = [
        ("steward", "Stewardf95sJbmtcZsyagb2dg4Mo8eVQho8gpECvLx8"),
        (
            "validator_history",
            "HistoryJTGbKQD2mRgLZ3XhqHnN811Qpez8X9kCcGHoa",
        ),
        ("vault", "Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8"),
        ("tip_router", "RouterBmuRBkPUbgEDMtdvTZ75GBdSREZR5uGUxxxpb"),
        (
            "interceptor",
            "5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV",
        ),
    ];

    println!("Jito Snapshot");
    println!("=========================================\n");

    let file = File::create(&args.output)?;
    let mut writer = Writer::from_writer(file);

    let mut total_accounts = 0;
    let mut total_rent_sol = 0.0;

    for (program_name, program_id) in programs {
        let exporter = Exporter::new(&args.rpc_url, program_name.to_string(), program_id)?;

        println!("Fetching validator history accounts...");
        let accounts = exporter.get_all_accounts()?;

        for account in &accounts {
            writer.serialize(account)?;
            total_rent_sol += account.balance_sol;
        }

        total_accounts += accounts.len();
    }

    writer.flush()?;

    println!("\n=== Summary ===");
    println!("Output file: {}", args.output);
    println!("Total accounts: {}", total_accounts);
    println!("Total rent locked: {:.4} SOL", total_rent_sol);
    println!(
        "Average rent per account: {:.4} SOL",
        total_rent_sol / total_accounts as f64
    );

    Ok(())
}
