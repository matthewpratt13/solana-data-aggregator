// Parses and processes the retrieved data to extract relevant information

// Responsibilities:
// * Parse transaction records to extract relevant information (e.g., sender, receiver, amount, timestamp).
// * Organize data into a structured format for storage and analysis.

// Implementation:
// * Use `serde` for JSON deserialization.
// * Implement functions to parse transaction data and extract information.

use log::{error, info};
use regex::Regex;
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
    pub prev_blockhash: String,
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
            UiMessage::Raw(_) => {
                error!("Unexpected message format. Expected parsed message, found raw message");
                return None;
            }
        },
        txn => {
            error!("Unexpected transaction encoding. Expect JSON data, found {txn:?}");
            return None;
        }
    };

    let sender = message
        .account_keys
        .first()
        .cloned()
        .map_or("".to_string(), |acc| acc.pubkey);

    let receiver = message
        .account_keys
        .last()
        .cloned()
        .map_or("".to_string(), |acc| acc.pubkey);

    let meta = if let Some(meta) = txn.transaction.meta.as_ref() {
        meta
    } else {
        error!("Transaction metadata not found");
        return None;
    };

    // calculate SOL amount transferred (simplified example)
    let sol_amount = meta.post_balances[0] - meta.pre_balances[0];

    // get transaction fee
    let fee = meta.fee;

    // get block time
    let timestamp = txn.block_time.unwrap_or_default();

    // get previous block hash
    let prev_blockhash = message.recent_blockhash.clone();

    // build `TransactionData` struct
    let transaction_data = TransactionData {
        signature: signatures
            .first()
            .map_or("".to_string(), |sig| sig.to_string()),
        sender,
        receiver,
        sol_amount,
        fee,
        timestamp,
        prev_blockhash,
    };

    info!("Parsed transaction: {:?}", transaction_data);

    Some(transaction_data)
}

/// Function to process a list of transactions.
pub fn process_transactions(
    transactions: Vec<EncodedConfirmedTransactionWithStatusMeta>,
) -> Vec<TransactionData> {
    info!("Processing transactions…");

    transactions
        .into_iter()
        .filter_map(parse_transaction)
        .filter(|txn| is_valid_transaction(txn))
        .collect::<Vec<_>>()
}

pub fn is_valid_transaction(txn: &TransactionData) -> bool {
    is_valid_signature(&txn.signature)
        && is_valid_pubkey(&txn.sender)
        && is_valid_pubkey(&txn.receiver)
        && is_valid_sender_receiver(&txn.sender, &txn.receiver)
        && is_valid_amount(txn.sol_amount)
        && is_valid_fee(txn.fee)
        && is_valid_timestamp(txn.timestamp)
        && is_valid_blockhash(&txn.prev_blockhash)
}

fn is_valid_signature(signature: &String) -> bool {
    if !signature.to_string().is_empty() {
        true
    } else {
        error!("Transaction signature is empty. Skipping transaction…");
        false
    }
}

fn is_valid_pubkey(pubkey: &String) -> bool {
    let re = Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").expect("Invalid regex");

    if pubkey.len() == 32 && re.is_match(pubkey) {
        true
    } else {
        error!("Invalid public key: `{pubkey}`. Skipping transaction…");
        false
    }
}

fn is_valid_sender_receiver(sender: &String, receiver: &String) -> bool {
    if sender != receiver {
        true
    } else {
        error!("Receiver cannot be sender. Skipping transaction…");
        false
    }
}

fn is_valid_amount(amount: u64) -> bool {
    if amount > 0 {
        true
    } else {
        error!("Transfer amount must be positive. Skipping transaction…");
        false
    }
}

fn is_valid_fee(fee: u64) -> bool {
    if fee > 0 {
        true
    } else {
        error!("Transfer fee must be positive. Skipping transaction…");
        false
    }
}

fn is_valid_timestamp(timestamp: i64) -> bool {
    if timestamp >= 0 {
        true
    } else {
        error!("Block timestamp cannot be negative. Skipping transaction…");
        false
    }
}

fn is_valid_blockhash(hash: &String) -> bool {
    let re = Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").expect("Invalid regex");

    if hash.len() == 32 && re.is_match(hash) {
        true
    } else {
        error!("Invalid blockhash: `{hash}`. Skipping transaction…");
        false
    }
}
