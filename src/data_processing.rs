// Parses and processes the retrieved data to extract relevant information

// Responsibilities:
// * Parse transaction records to extract relevant information (e.g., sender, receiver, amount, timestamp).
// * Organize data into a structured format for storage and analysis.

// Implementation:
// * Use `serde` for JSON deserialization.
// * Implement functions to parse transaction data and extract information.

use serde::{Deserialize, Serialize};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiMessage, UiTransaction,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub signature: String,
    pub sender: String,
    pub receiver: String,
    pub sol_amount: u64,
    pub fee: u64,
    pub timestamp: i64,
    pub prev_block_hash: String,
}

/// Function to parse transaction data and extract relevant fields.
pub fn parse_transaction(
    txn: EncodedConfirmedTransactionWithStatusMeta,
) -> Option<TransactionData> {
    // extract sender, receiver and parsed message
    let (signatures, message) = match &txn.transaction.transaction {
        EncodedTransaction::Json(UiTransaction {
            signatures,
            message,
        }) => match message {
            UiMessage::Parsed(msg) => (signatures, msg),
            UiMessage::Raw(_) => todo!(),
        },
        _ => todo!(),
    };

    let sender_pubkey = message.account_keys.first()?.pubkey.clone();
    let receiver_pubkey = message.account_keys.last()?.pubkey.clone();

    let meta = txn.transaction.meta.as_ref()?;

    // calculate SOL amount transferred (simplified example)
    let sol_amount = meta.post_balances[0] - meta.pre_balances[0];

    // get transaction fee
    let fee = meta.fee;

    // get block time
    let timestamp = txn.block_time.unwrap_or_default();

    // get previous block hash
    let prev_block_hash = message.recent_blockhash.clone();

    // build `TransactionData` struct
    Some(TransactionData {
        signature: signatures.first()?.to_string(),
        sender: sender_pubkey,
        receiver: receiver_pubkey,
        sol_amount,
        fee,
        timestamp,
        prev_block_hash: prev_block_hash,
    })
}

/// Function to process a list of transactions.
pub fn process_transactions(
    transactions: Vec<EncodedConfirmedTransactionWithStatusMeta>,
) -> Vec<TransactionData> {
    transactions
        .into_iter()
        .filter_map(parse_transaction)
        .collect()
}
