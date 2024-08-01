use serde::{Deserialize, Serialize};

use std::fs;
//use std::path::Path;

use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut, Sequence}; 
use bitcoin::hashes::sha256d;
use bitcoin::absolute::LockTime;
use bitcoin::{Address, PublicKey, Network, merkle_tree};
use bitcoin::secp256k1::{rand, Secp256k1};
use bitcoin::blockdata::script::ScriptBuf;



#[derive(Serialize, Deserialize, Debug, Clone)]
struct MempoolTransaction {
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



fn read_transactions(mempool_dir: &str) -> Vec<MempoolTransaction> {
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
            let transaction: Result<MempoolTransaction, _> = serde_json::from_str(&data);
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


fn validate_transaction(tx: &MempoolTransaction) -> bool {
    // Check if the transaction has at least one input and one output
    if tx.vin.is_empty() || tx.vout.is_empty() {
        return false;
    }

    // Check if the transaction size and weight are within acceptable limits
    if tx.size > 100_000 || tx.weight > 400_000 {
        return false;
    }

    // Check if sum of inputs is greater than or equal to the sum of outputs
    let input_sum: u64 = tx.vin.iter().map(|vin| vin.prevout.value).sum();
    let output_sum: u64 = tx.vout.iter().map(|vout| vout.value).sum();

    if input_sum < output_sum {
        return false;
    }

    // should add checks on spent or unspent UTXO
    // // Check if the UTXO is unspent
    // match client.get_tx_out(txid, vout, None) {
    //     Ok(Some(utxo)) => {
    //         println!("UTXO is unspent: {:?}", utxo);
    //     }
    //     Ok(None) => {
    //         println!("UTXO is spent or does not exist");
    //     }
    //     Err(e) => {
    //         println!("Error querying UTXO: {}", e);
    //     }
    // }

    true
}

fn validate_transactions(transactions: &[MempoolTransaction]) -> Vec<MempoolTransaction> {
    transactions.iter().filter(|tx| validate_transaction(tx)).cloned().collect()
}

fn create_coinbase_transaction(address: &Address) -> Transaction {
    Transaction {
        version: bitcoin::transaction::Version(1),
        lock_time: LockTime::ZERO,        
        input: vec![TxIn {
            previous_output: Default::default(),
            script_sig: bitcoin::blockdata::script::ScriptBuf::new(),
            sequence: bitcoin::blockdata::transaction::Sequence::MAX,
            witness: bitcoin::blockdata::witness::Witness::default(),
        }],
        output: vec![TxOut {
            value: bitcoin::Amount::from_sat(50 * 100_000_000), // 50 BTC reward
            script_pubkey: address.script_pubkey(),
        }],
    }
}

fn calculate_merkle_root(transactions: &[Transaction]) -> sha256d::Hash {
    let txids: Vec<_> = transactions.iter().map(|tx| tx.txid().to_raw_hash()).collect();
    merkle_tree::calculate_root(txids.into_iter()).expect("Failed to calculate Merkle root")
}
// fn mine_block(header: &BlockHeader, difficulty_target: &str) -> (BlockHeader, u64) {
//     let mut nonce = 0;
//     let mut rng = rand::thread_rng();
//     let target = sha256d::Hash::from_str(difficulty_target).unwrap();
//     let mut block_header = header.clone();

//     loop {
//         block_header.nonce = nonce;
//         let block_hash = block_header.block_hash();
//         if block_hash <= target {
//             return (block_header, nonce);
//         }
//         nonce = rng.gen();
//     }
// }

fn main() {
    let mempool_dir = "../../mempool";
    //let difficulty_target = "0000ffff00000000000000000000000000000000000000000000000000000000";
    
    // Generate a random key pair and create a new address
    let s = Secp256k1::new();
    let public_key = PublicKey::new(s.generate_keypair(&mut rand::thread_rng()).1);
    let address = Address::p2pkh(&public_key, Network::Bitcoin);
    println!("generated new address : {:?}", address);


    let mempool_transactions = read_transactions(mempool_dir);
    if !mempool_transactions.is_empty() {
        println!("reading transactions sucess");
    } else {
        println!("No transactions found.");
    }
    // Validate transactions
    let valid_mempool_transactions = validate_transactions(&mempool_transactions);

    // Calculate the number of transactions that did not pass validation
    let original_count = mempool_transactions.len();
    let valid_count = valid_mempool_transactions.len();
    let invalid_count = original_count - valid_count;

    println!("Total transactions in mempool: {}", original_count);
    println!("Valid transactions: {}", valid_count);
    println!("Invalid transactions: {}", invalid_count);
    
    let coinbase_tx = create_coinbase_transaction(&address);
    //println!("Coinbase transaction: {:?}", coinbase_tx);

    let mut block_transactions = vec![coinbase_tx.clone()];
    block_transactions.extend(valid_mempool_transactions.iter().map(|tx| Transaction {
        version: bitcoin::blockdata::transaction::Version(tx.version as i32),
        lock_time: LockTime::ZERO, 
        input: tx.vin.iter().map(|vin| TxIn {
            previous_output: bitcoin::blockdata::transaction::OutPoint::default(), // Assuming a default value for simplicity
            script_sig: ScriptBuf::from_hex(&vin.scriptsig).unwrap(),
            sequence: Sequence::from_consensus(vin.sequence),
            witness: bitcoin::blockdata::witness::Witness::default(),
        }).collect(),
        output: tx.vout.iter().map(|vout| TxOut {
            value: bitcoin::Amount::from_sat(vout.value),
            script_pubkey: ScriptBuf::from_hex(&vout.scriptpubkey).unwrap(),
        }).collect(),
    }));

    let merkle_root = calculate_merkle_root(&block_transactions);
    println!("merkle root: {:?}", merkle_root);
    // let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    // let header = BlockHeader {
    //     version: 1,
    //     prev_blockhash: prev_block_hash,
    //     merkle_root,
    //     time: timestamp,
    //     bits: 0x1d00ffff,
    //     nonce: 0,
    // };
    // let (mined_header, nonce) = mine_block(&header, difficulty_target);
    // let block = Block {
    //     header: mined_header,
    //     txdata: block_transactions,
    // };

}

