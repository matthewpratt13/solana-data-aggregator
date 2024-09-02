mod api;
mod data_processing;
mod data_retrieval;
mod data_storage;

use solana_sdk::pubkey::Pubkey;
use tokio::task;

use std::{env, str::FromStr, sync::Arc};

use data_retrieval::SolanaClient;
use data_storage::get_pool;

// TODO: add logging (general)
// TODO: update error handling (general)
// TODO: add `submit_transaction()` function to submit transactions to devnet
// TODO: (add `SECRET_KEY` env var)
// TODO: validate transaction data
// TODO: where do we implement the `insert_transactions()` function?
// TODO: test

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    env_logger::init();

    // RPC client setup
    let rpc_url = env::var("RPC_URL")?;
    let solana_client = SolanaClient::new(&rpc_url);

    // database setup
    let db_url = env::var("DATABASE_URL")?;
    let db = Arc::new(get_pool(&db_url).await?);

    // monitored address's public key
    let address = Pubkey::from_str(&env::var("ADDRESS")?)?;

    // start monitoring the blockchain
    task::spawn(async move {
        solana_client.monitor_blockchain(address).await;
    });

    // run API server
    api::main(db)?;

    Ok(())
}
