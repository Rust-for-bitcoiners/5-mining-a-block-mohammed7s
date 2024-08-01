use serde::{Deserialize, Serialize};

use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    txid: String,
    version: u32,
    locktime: u32,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
    size: usize,
    weight: usize,
    fee: u64,
    status: Status,
    hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Vin {
    txid: String,
    vout: u32,
    prevout: Prevout,
    scriptsig: String,
    scriptsig_asm: String,
    witness: Option<Vec<String>>,
    is_coinbase: bool,
    sequence: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Prevout {
    scriptpubkey: String,
    scriptpubkey_asm: String,
    scriptpubkey_type: String,
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Vout {
    scriptpubkey: String,
    scriptpubkey_asm: String,
    scriptpubkey_type: String,
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Status {
    confirmed: bool,
    block_height: u32,
    block_hash: String,
    block_time: u64,
}



fn read_transactions(mempool_dir: &str) -> Vec<Transaction> {
    let mut transactions = Vec::new();
    let paths = fs::read_dir(mempool_dir).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if path.extension().unwrap() == "json" {
            if path.file_name().unwrap() == "mempool.json" {
                println!("Skipping mempool.json");
                continue;
            }
            let data = fs::read_to_string(&path).unwrap();
            let transaction: Result<Transaction, _> = serde_json::from_str(&data);
            match transaction {
                Ok(tx) => transactions.push(tx),
                Err(e) => {
                    println!("Failed to deserialize transaction in file {:?}: {:?}", path, e);
                    //println!("JSON content: {:?}", data);
                }
            }
        }
    }
    transactions
}


fn validate_transaction(tx: &Transaction) -> bool {
    // Check if the transaction has at least one input and one output
    if tx.vin.is_empty() || tx.vout.is_empty() {
        return false;
    }

    // Check if the transaction size and weight are within acceptable limits
    if tx.size > 100_000 || tx.weight > 400_000 {
        return false;
    }

    true
}

fn validate_transactions(transactions: &[Transaction]) -> Vec<Transaction> {
    transactions.iter().filter(|tx| validate_transaction(tx)).cloned().collect()
}




fn main() {
    let mempool_dir = "../../mempool";
    //let difficulty_target = "0000ffff00000000000000000000000000000000000000000000000000000000";
    let transactions = read_transactions(mempool_dir);
    if !transactions.is_empty() {
        println!("reading transactions sucess");
    } else {
        println!("No transactions found.");
    }
    // Validate transactions
    let valid_transactions = validate_transactions(&transactions);

    // Calculate the number of transactions that did not pass validation
    let original_count = transactions.len();
    let valid_count = valid_transactions.len();
    let invalid_count = original_count - valid_count;

    println!("Total transactions in mempool: {}", original_count);
    println!("Valid transactions: {}", valid_count);
    println!("Invalid transactions: {}", invalid_count);
    
}

