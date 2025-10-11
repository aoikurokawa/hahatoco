use std::str::FromStr;

use anyhow::Context;
use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_pubkey::Pubkey;

#[derive(Debug, Serialize)]
pub struct JitoAccount {
    pub program_name: String,
    pub program_id: String,
    pub address: String,
    pub balance_lamports: u64,
    pub balance_sol: f64,
    pub data_size: usize,
}

pub struct Exporter {
    /// Client
    client: RpcClient,

    /// Program name
    program_name: String,

    /// Program ID
    program_id: Pubkey,
}

impl Exporter {
    pub fn new(rpc_url: &str, program_name: String, program_id: &str) -> anyhow::Result<Self> {
        let client = RpcClient::new(rpc_url.to_string());
        let program_id = Pubkey::from_str(program_id).context("Failed to parse program ID")?;

        Ok(Self {
            client,
            program_name,
            program_id,
        })
    }

    pub fn get_all_accounts(&self) -> anyhow::Result<Vec<JitoAccount>> {
        println!("Fetching accounts from program: {}", self.program_id);
        println!("Using RPC: {}", self.client.url());

        let accounts = self
            .client
            .get_program_accounts(&self.program_id)
            .context("Failed to fetch program accounts")?;

        println!("Found {} accounts", accounts.len());

        let validator_accounts: Vec<JitoAccount> = accounts
            .into_iter()
            .map(|(pubkey, account)| {
                let balance_lamports = account.lamports;
                let balance_sol = balance_lamports as f64 / 1e9;
                let data_size = account.data.len();

                JitoAccount {
                    program_name: self.program_name.clone(),
                    program_id: self.program_id.to_string(),
                    address: pubkey.to_string(),
                    balance_lamports,
                    balance_sol,
                    data_size,
                }
            })
            .collect();

        Ok(validator_accounts)
    }
}
