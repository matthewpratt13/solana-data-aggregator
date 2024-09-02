// Interacts with the Solana blockchain to fetch transactions and account data

// Responsibilities:
// * Connect to the Solana devnet or testnet.
// * Fetch transactions and account data for the current epoch.
// * Continuously monitor the blockchain for new transactions and account changes.

// Implementation:
// * Use the `solana-client` crate to create an RPC client that connects to the Solana devnet or testnet.
// * Implement a function to retrieve transactions and account data. This function will use asynchronous requests to fetch data.
// * Use a background task (using `tokio::spawn`) to periodically poll the blockchain for new transactions.

use solana_client::{
    rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient},
    rpc_config::RpcTransactionConfig,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use tokio::time::{self, Duration};

use std::str::FromStr;

use crate::data_processing::process_transactions;

pub struct SolanaClient {
    client: RpcClient,
}

impl SolanaClient {
    pub fn new(rpc_url: &str) -> Self {
        let client =
            RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
        SolanaClient { client }
    }

    /// Fetch transaction signatures for a given address.
    pub fn fetch_transaction_signatures(&self, address: &Pubkey) -> anyhow::Result<Vec<Signature>> {
        let mut signature_list: Vec<Signature> = Vec::new();

        let config = GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: None,
            limit: Some(3),
            commitment: Some(CommitmentConfig::confirmed()),
        };

        let signatures = self
            .client
            .get_signatures_for_address_with_config(address, config)?;

        for txn in signatures {
            let sig = Signature::from_str(&txn.signature)?;
            signature_list.push(sig);
        }

        Ok(signature_list)
    }

    /// Fetch transactions based on their signatures.
    pub fn fetch_transactions(
        &self,
        signatures: &[Signature],
    ) -> anyhow::Result<Vec<EncodedConfirmedTransactionWithStatusMeta>> {
        let mut transactions = Vec::new();

        for sig in signatures {
            let config = RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                ..Default::default()
            };

            if let Ok(txn) = self.client.get_transaction_with_config(sig, config.clone()) {
                transactions.push(txn);
            }
        }

        Ok(transactions)
    }

    /// Fetch and process epoch data.
    pub async fn fetch_epoch_data(
        &self,
        address: &Pubkey,
    ) -> anyhow::Result<Vec<EncodedConfirmedTransactionWithStatusMeta>> {
        let signatures = self.fetch_transaction_signatures(address)?;
        let transactions = self.fetch_transactions(&signatures)?;

        Ok(transactions)
    }

    /// Continuously monitor the blockchain for new data.
    pub async fn monitor_blockchain(&self, address: Pubkey) {
        let mut interval = time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            match self.fetch_epoch_data(&address).await {
                Ok(txns) => {
                    let processed_txns = process_transactions(txns);
                    println!("{:?}", processed_txns);
                }
                Err(e) => eprintln!("Error fetching data: {:?}", e),
            }
        }
    }
}
