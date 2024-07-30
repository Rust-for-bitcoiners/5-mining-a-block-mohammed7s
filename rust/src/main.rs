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

fn main() {
    let mempool_dir = "../../mempool";
    //let difficulty_target = "0000ffff00000000000000000000000000000000000000000000000000000000";
    let transactions = read_transactions(mempool_dir);
    if !transactions.is_empty() {
        println!("reading transactions sucess");
    } else {
        println!("No transactions found.");
    }
}

