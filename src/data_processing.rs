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
    let sol_amount = meta.post_balances[1] - meta.pre_balances[1];

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

fn is_valid_signature(signature: &str) -> bool {
    if !signature.to_string().is_empty() {
        true
    } else {
        error!("Transaction signature is empty. Skipping transaction…");
        false
    }
}

fn is_valid_pubkey(pubkey: &str) -> bool {
    let re = Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").expect("Invalid regex");

    if pubkey.len() == 44 && re.is_match(pubkey) {
        true
    } else {
        error!("Invalid public key: `{pubkey}`. Skipping transaction…");
        false
    }
}

fn is_valid_sender_receiver(sender: &str, receiver: &str) -> bool {
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

fn is_valid_blockhash(hash: &str) -> bool {
    let re = Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").expect("Invalid regex");

    if hash.len() == 44 && re.is_match(hash) {
        true
    } else {
        error!("Invalid blockhash: `{hash}`. Skipping transaction…");
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::{message::MessageHeader, pubkey::Pubkey, signature::Signature};
    use solana_transaction_status::{
        option_serializer::OptionSerializer, parse_accounts::ParsedAccount,
        EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
        EncodedTransactionWithStatusMeta, UiMessage, UiParsedMessage, UiRawMessage, UiTransaction,
        UiTransactionStatusMeta,
    };

    use std::env;

    #[test]
    fn test_valid_signature() {
        let valid_signature = "5NzT3RMAGiJjxGqAXgy6xakdcTfV7oF2dt2m5x8y7vc48pmQ9JVDd8LfPtkMRNZkNmJmhYoP2cFHGip7vRtXVcdv";
        assert!(is_valid_signature(&valid_signature.to_string()));

        let invalid_signature = "";
        assert!(!is_valid_signature(&invalid_signature.to_string()));
    }

    #[test]
    fn test_valid_pubkey() {
        let _ = dotenvy::dotenv();

        let valid_pubkey = env::var("ADDRESS_A").expect("`ADDRESS_A` must be set");
        assert!(is_valid_pubkey(&valid_pubkey.to_string()));

        let invalid_pubkey = "InvalidPubkeyString";
        assert!(!is_valid_pubkey(&invalid_pubkey.to_string()));
    }

    #[test]
    fn test_valid_amount() {
        let valid_amount = 1000;
        assert!(is_valid_amount(valid_amount));

        let invalid_amount = 0;
        assert!(!is_valid_amount(invalid_amount));
    }

    #[test]
    fn test_valid_fee() {
        let valid_fee = 500;
        assert!(is_valid_fee(valid_fee));

        let invalid_fee = 0;
        assert!(!is_valid_fee(invalid_fee));
    }

    #[test]
    fn test_valid_timestamp() {
        let valid_timestamp = 1625077743;
        assert!(is_valid_timestamp(valid_timestamp));

        let invalid_timestamp = -1625077743;
        assert!(!is_valid_timestamp(invalid_timestamp));
    }

    #[test]
    fn test_valid_blockhash() {
        let valid_hash = "4sZ76MsNd8y3WSw2L1nfd3AqLoYxdmC98sERoMRbHV14";
        assert!(is_valid_blockhash(&valid_hash.to_string()));

        let invalid_hash = "InvalidHashString";
        assert!(!is_valid_blockhash(&invalid_hash.to_string()));
    }

    #[test]
    fn test_valid_sender_receiver() {
        let _ = dotenvy::dotenv();

        let sender = env::var("ADDRESS_A").expect("`ADDRESS_A` must be set");
        let receiver = env::var("ADDRESS_B").expect("`ADDRESS_B` must be set");

        assert!(is_valid_sender_receiver(&sender, &receiver));

        let same_pubkey = sender.clone();

        assert!(!is_valid_sender_receiver(&sender, &same_pubkey));
    }

    #[test]
    fn test_valid_transaction() {
        let _ = dotenvy::dotenv();

        let sender = env::var("ADDRESS_A").expect("`ADDRESS_A` must be set");
        let receiver = env::var("ADDRESS_B").expect("`ADDRESS_B` must be set");

        let valid_transaction = TransactionData {
        signature: "5NzT3RMAGiJjxGqAXgy6xakdcTfV7oF2dt2m5x8y7vc48pmQ9JVDd8LfPtkMRNZkNmJmhYoP2cFHGip7vRtXVcdv".to_string(),
        sender,
        receiver,
        sol_amount: 1000,
        fee: 500,
        timestamp: 1625077743,
        prev_blockhash: "4sZ76MsNd8y3WSw2L1nfd3AqLoYxdmC98sERoMRbHV14".to_string(),
    };
        assert!(is_valid_transaction(&valid_transaction));

        let invalid_transaction = TransactionData {
            signature: "".to_string(),
            sender: "9WgXgM4UQftvDStk9SMeLBjQ1tr1sVpYzVv9ekDwpa5X".to_string(),
            receiver: "InvalidPubkeyString".to_string(),
            sol_amount: 0,
            fee: 0,
            timestamp: -1625077743,
            prev_blockhash: "InvalidHashString".to_string(),
        };

        assert!(!is_valid_transaction(&invalid_transaction));
    }

    #[test]
    fn test_parse_transaction() {
        // Test 1: Valid parsed transaction
        let txn = EncodedConfirmedTransactionWithStatusMeta {
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec![Signature::new_unique().to_string()],
                    message: UiMessage::Parsed(UiParsedMessage {
                        account_keys: vec![
                            ParsedAccount {
                                pubkey: Pubkey::new_unique().to_string(),
                                writable: true,
                                signer: true,
                                source: None,
                            },
                            ParsedAccount {
                                pubkey: Pubkey::new_unique().to_string(),
                                writable: false,
                                signer: false,
                                source: None,
                            },
                        ],
                        recent_blockhash: "recent_blockhash".to_string(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),

                meta: Some(UiTransactionStatusMeta {
                    err: None,
                    status: Ok(()),
                    fee: 5000,
                    pre_balances: vec![100_000, 50_000],
                    post_balances: vec![85_000, 60_000],
                    inner_instructions: OptionSerializer::Skip,
                    log_messages: OptionSerializer::Skip,
                    pre_token_balances: OptionSerializer::Skip,
                    post_token_balances: OptionSerializer::Skip,
                    rewards: OptionSerializer::Skip,
                    loaded_addresses: OptionSerializer::Skip,
                    return_data: OptionSerializer::Skip,
                    compute_units_consumed: OptionSerializer::Skip,
                }),
                version: None,
            },
            slot: 42,
            block_time: Some(1625077743),
        };

        let parsed_transaction = parse_transaction(txn).unwrap();
        assert_eq!(parsed_transaction.sol_amount, 10_000);
        assert_eq!(parsed_transaction.fee, 5000);
        assert_eq!(parsed_transaction.timestamp, 1625077743);
        assert_eq!(parsed_transaction.prev_blockhash, "recent_blockhash");

        // Test 2: Raw message instead of parsed
        let txn = EncodedConfirmedTransactionWithStatusMeta {
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec![Signature::new_unique().to_string()],
                    message: UiMessage::Raw(UiRawMessage {
                        header: MessageHeader {
                            ..Default::default()
                        },
                        account_keys: vec![],
                        recent_blockhash: "recent_blockhash".to_string(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),

                meta: Some(UiTransactionStatusMeta {
                    err: None,
                    status: Ok(()),
                    fee: 5000,
                    pre_balances: vec![100_000, 50_000],
                    post_balances: vec![85_000, 60_000],
                    inner_instructions: OptionSerializer::Skip,
                    log_messages: OptionSerializer::Skip,
                    pre_token_balances: OptionSerializer::Skip,
                    post_token_balances: OptionSerializer::Skip,
                    rewards: OptionSerializer::Skip,
                    loaded_addresses: OptionSerializer::Skip,
                    return_data: OptionSerializer::Skip,
                    compute_units_consumed: OptionSerializer::Skip,
                }),
                version: None,
            },
            slot: 42,
            block_time: Some(1625077743),
        };

        assert!(parse_transaction(txn).is_none());

        // Test 3: Transaction metadata missing
        let txn = EncodedConfirmedTransactionWithStatusMeta {
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec![Signature::new_unique().to_string()],
                    message: UiMessage::Parsed(UiParsedMessage {
                        account_keys: vec![
                            ParsedAccount {
                                pubkey: Pubkey::new_unique().to_string(),
                                writable: true,
                                signer: true,
                                source: None,
                            },
                            ParsedAccount {
                                pubkey: Pubkey::new_unique().to_string(),
                                writable: false,
                                signer: false,
                                source: None,
                            },
                        ],
                        recent_blockhash: "recent_blockhash".to_string(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),

                meta: None,
                version: None,
            },
            slot: 42,
            block_time: Some(1625077743),
        };

        assert!(parse_transaction(txn).is_none());
    }
}
