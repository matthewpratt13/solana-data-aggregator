// Stores the processed data in a database or an in-memory data structure

// Responsibilities:
// * Store the processed data securely.
// * Provide fast and efficient querying capabilities.

// Implementation Options:
// * In-memory storage: Use a thread-safe data structure (e.g., `HashMap` or `Vec`) to store data temporarily.
// * Database storage: Use `sqlx` to interact with a PostgreSQL database.

use std::sync::Arc;

use crate::data_processing::TransactionData;

use log::info;
use sqlx::{postgres::PgPoolOptions, PgPool};

// use std::{
//     collections::HashMap,
//     sync::{Arc, Mutex},
// };

// /// In-memory storage with a thread-safe `HashMap`.
// pub struct InMemoryStorage {
//     transactions: Arc<Mutex<HashMap<String, TransactionData>>>,
// }

// impl InMemoryStorage {
//     pub fn new() -> Self {
//         InMemoryStorage {
//             transactions: Arc::new(Mutex::new(HashMap::new())),
//         }
//     }

//     pub fn insert_transaction(&self, txn_data: TransactionData) {
//         let mut transactions = self.transactions.lock().unwrap();
//         transactions.insert(txn_data.signature.clone(), txn_data);
//     }

//     pub fn get_all_transactions(&self) -> Vec<TransactionData> {
//         let transactions = self.transactions.lock().unwrap();
//         transactions.values().cloned().collect()
//     }
// }

pub async fn get_pool(db_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS transactions (
        id SERIAL PRIMARY KEY,
        signature VARCHAR NOT NULL,
        sender VARCHAR NOT NULL,
        receiver VARCHAR NOT NULL,
        sol_amount BIGINT NOT NULL,
        fee BIGINT NOT NULL,
        timestamp BIGINT NOT NULL,
        prev_blockhash VARCHAR NOT NULL
    )"
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

#[allow(dead_code)]
pub async fn insert_transaction(
    pool: &Arc<PgPool>,
    txn_data: &TransactionData,
) -> anyhow::Result<()> {
    sqlx::query!(
            "INSERT INTO transactions (signature, sender, receiver, sol_amount, fee, timestamp, prev_blockhash)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
            txn_data.signature,
            txn_data.sender,
            txn_data.receiver,
            txn_data.sol_amount as i64,
            txn_data.fee as i64,
            txn_data.timestamp,
            txn_data.prev_blockhash
        )
        .execute(pool.as_ref())
        .await?;

    info!(
        "Inserted transaction in PostgreSQL database: {:?}",
        txn_data
    );

    Ok(())
}

pub async fn get_all_transactions(pool: &Arc<PgPool>) -> anyhow::Result<Vec<TransactionData>> {
    let rows = sqlx::query!(
            "SELECT signature, sender, receiver, sol_amount, fee, timestamp, prev_blockhash FROM transactions"
        )
        .fetch_all(pool.as_ref())
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| TransactionData {
            signature: row.signature,
            sender: row.sender,
            receiver: row.receiver,
            sol_amount: row.sol_amount as u64,
            fee: row.fee as u64,
            timestamp: row.timestamp,
            prev_blockhash: row.prev_blockhash,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    use sqlx::PgPool;

    #[tokio::test]
    async fn test_store_transaction() -> Result<(), anyhow::Error> {
        let _ = dotenvy::dotenv();

        let sender = env::var("ADDRESS_A").expect("`ADDRESS_A` must be set");
        let receiver = env::var("ADDRESS_B").expect("`ADDRESS_B` must be set");

        // Setup: Initialize a test database (using a different test database connection)
        let pool = PgPool::connect("postgres://postgres@localhost/test_database").await?;

        let valid_transaction = TransactionData {
        signature: "5NzT3RMAGiJjxGqAXgy6xakdcTfV7oF2dt2m5x8y7vc48pmQ9JVDd8LfPtkMRNZkNmJmhYoP2cFHGip7vRtXVcdv".to_string(),
        sender,
        receiver,
        sol_amount: 1000,
        fee: 500,
        timestamp: 1625077743,
        prev_blockhash: "4sZ76MsNd8y3WSw2L1nfd3AqLoYxdmC98sERoMRbHV14".to_string(),
    };

        // Act: Store the transaction
        insert_transaction(&Arc::new(pool.clone()), &valid_transaction).await?;

        // Assert: Check that the transaction was stored
        let result = sqlx::query!(
            "SELECT * FROM transactions WHERE signature = $1",
            valid_transaction.signature
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(result.signature, valid_transaction.signature);
        assert_eq!(result.sender, valid_transaction.sender);
        assert_eq!(result.receiver, valid_transaction.receiver);
        assert_eq!(result.sol_amount, valid_transaction.sol_amount as i64);
        assert_eq!(result.fee, valid_transaction.fee as i64);
        assert_eq!(result.timestamp, valid_transaction.timestamp);
        assert_eq!(result.prev_blockhash, valid_transaction.prev_blockhash);

        Ok(())
    }
}
