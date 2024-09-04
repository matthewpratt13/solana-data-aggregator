mod api;
mod data_processing;
mod data_retrieval;
mod data_storage;

use solana_sdk::pubkey::Pubkey;
use tokio::task;

use std::{env, str::FromStr, sync::Arc};

use data_retrieval::SolanaClient;
use data_storage::get_pool;

// TODO: test

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    env_logger::init();

    // RPC client setup
    let rpc_url = env::var("RPC_URL")?;
    let solana_client = SolanaClient::new(&rpc_url);

    // monitored address's public key
    let address = Pubkey::from_str(&env::var("ADDRESS_A")?)?;

    // database setup
    let db_url = env::var("DATABASE_URL")?;
    let db = Arc::new(get_pool(&db_url).await?);
    let db_clone = Arc::clone(&db);

    // start monitoring the blockchain
    task::spawn(async move {
        solana_client
            .monitor_blockchain(address, Some(&db_clone))
            .await;
    });

    // run API server
    api::main(db)?;

    Ok(())
}
