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
