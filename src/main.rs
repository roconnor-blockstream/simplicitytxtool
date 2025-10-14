// CLI tool that replicates the Simplicity Web IDE transaction flow
// Based on: https://github.com/BlockstreamResearch/simplicity-webide

use std::str::FromStr;
use std::sync::Arc;

use elements::hashes::Hash;
use elements::secp256k1_zkp as secp256k1;
use elements::{confidential, encode::Encodable};
use simplicity::jet::elements::{ElementsEnv, ElementsUtxo};
use simfony::simplicity::jet::Elements;
use simfony::simplicity::RedeemNode;
use simfony::{elements, simplicity};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "address" => cmd_address(&args[2..]),
        "sighash" => cmd_sighash(&args[2..]),
        "sign" => cmd_sign(&args[2..]),
        "build-tx" => cmd_build_tx(&args[2..]),
        _ => {
            eprintln!("Unknown command: {}", command);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!("Simplicity CLI Tool - Replicates Web IDE functionality");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  address <contract.simf>");
    eprintln!("    Generate P2TR address for a contract");
    eprintln!();
    eprintln!("  sighash <contract.simf> <txid> <vout> <value> <destination> <fee> [-g <genesis_hash>]");
    eprintln!("    Compute sighash for a transaction");
    eprintln!("    -g: Genesis hash (hex). Default: Liquid testnet");
    eprintln!();
    eprintln!("  sign <privkey_wif> <sighash>");
    eprintln!("    Sign a sighash with BIP-340 Schnorr signature");
    eprintln!();
    eprintln!("  build-tx <contract.simf> <txid> <vout> <value> <destination> <fee> <witness.wit> [-g <genesis_hash>]");
    eprintln!("    Build complete transaction with witness");
    eprintln!("    -g: Genesis hash (hex). Default: Liquid testnet");
    eprintln!();
    eprintln!("Genesis hashes:");
    eprintln!("  Liquid testnet (default): a771da8e52ee6ad581ed1e9a99825e5b3b7992225534eaa2ae23244fe26ab1c1");
    eprintln!("  Bitcoin mainnet:          6fe28c0ab6f1b372c1a6a246ae63f74f931e83651e085ae689cd6190000000000");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  simplicity_tx_tool address contract.simf");
    eprintln!("  simplicity_tx_tool sighash contract.simf abc123... 0 100000 tex1q... 1000");
    eprintln!("  simplicity_tx_tool sighash contract.simf abc123... 0 100000 tex1q... 1000 -g a771da8e...");
    eprintln!("  simplicity_tx_tool sign cABC123... abc123def456...");
    eprintln!("  simplicity_tx_tool build-tx contract.simf abc123... 0 100000 tex1q... 1000 witness.wit");
    eprintln!("  simplicity_tx_tool build-tx contract.simf abc123... 0 100000 tex1q... 1000 witness.wit -g a771da8e...");
}

// Command: Sign sighash with BIP-340
fn cmd_sign(args: &[String]) {
    if args.len() != 2 {
        eprintln!("Usage: simplicity_tx_tool sign <privkey_wif> <sighash>");
        std::process::exit(1);
    }
    
    let privkey_wif = &args[0];
    let sighash_hex = &args[1];
    
    // Decode WIF private key
    let privkey = elements::bitcoin::PrivateKey::from_wif(privkey_wif)
        .expect("Invalid WIF private key");
    
    // Parse sighash
    let sighash_bytes = hex::decode(sighash_hex)
        .expect("Invalid sighash hex");
    if sighash_bytes.len() != 32 {
        eprintln!("Sighash must be 32 bytes");
        std::process::exit(1);
    }
    let mut sighash_array = [0u8; 32];
    sighash_array.copy_from_slice(&sighash_bytes);
    
    // Create message for signing
    let message = secp256k1::Message::from_digest(sighash_array);
    
    // Sign with Schnorr (BIP-340)
    let keypair = secp256k1::Keypair::from_secret_key(secp256k1::SECP256K1, &privkey.inner);
    let signature = keypair.sign_schnorr(message);
    
    println!("{}", hex::encode(signature.as_ref()));
}

// Utility functions from web IDE
fn unspendable_internal_key() -> secp256k1::XOnlyPublicKey {
    // Check for environment variable first
    if let Ok(key_hex) = std::env::var("SIMPLICITY_INTERNAL_KEY") {
        match hex::decode(&key_hex) {
            Ok(key_bytes) if key_bytes.len() == 32 => {
                match secp256k1::XOnlyPublicKey::from_slice(&key_bytes) {
                    Ok(key) => {
                        eprintln!("Using custom internal key from SIMPLICITY_INTERNAL_KEY");
                        return key;
                    }
                    Err(e) => {
                        eprintln!("Warning: Invalid internal key in SIMPLICITY_INTERNAL_KEY: {}", e);
                        eprintln!("Falling back to default Web IDE internal key");
                    }
                }
            }
            Ok(_) => {
                eprintln!("Warning: SIMPLICITY_INTERNAL_KEY must be 32 bytes (64 hex chars)");
                eprintln!("Falling back to default Web IDE internal key");
            }
            Err(e) => {
                eprintln!("Warning: Invalid hex in SIMPLICITY_INTERNAL_KEY: {}", e);
                eprintln!("Falling back to default Web IDE internal key");
            }
        }
    }
    
    // Default: Web IDE internal key (provably unspendable NUMS point)
    secp256k1::XOnlyPublicKey::from_slice(&[
        0xf5, 0x91, 0x9f, 0xa6, 0x4c, 0xe4, 0x5f, 0x83, 0x06, 0x84, 0x90, 0x72, 0xb2, 0x6c, 0x1b,
        0xfd, 0xd2, 0x93, 0x7e, 0x6b, 0x81, 0x77, 0x47, 0x96, 0xff, 0x37, 0x2b, 0xd1, 0xeb, 0x53,
        0x62, 0xd2,
    ])
    .expect("key should be valid")
}

fn script_ver(cmr: simplicity::Cmr) -> (elements::Script, elements::taproot::LeafVersion) {
    let script = elements::script::Script::from(cmr.as_ref().to_vec());
    (script, simplicity::leaf_version())
}

fn taproot_spend_info(cmr: simplicity::Cmr) -> elements::taproot::TaprootSpendInfo {
    let builder = elements::taproot::TaprootBuilder::new();
    let (script, version) = script_ver(cmr);
    let builder = builder
        .add_leaf_with_ver(0, script, version)
        .expect("tap tree should be valid");
    builder
        .finalize(secp256k1::SECP256K1, unspendable_internal_key())
        .expect("tap tree should be valid")
}

fn liquid_testnet_address(cmr: simplicity::Cmr) -> elements::Address {
    let info = taproot_spend_info(cmr);
    let blinder = None;
    elements::Address::p2tr(
        secp256k1::SECP256K1,
        info.internal_key(),
        info.merkle_root(),
        blinder,
        &elements::AddressParams::LIQUID_TESTNET,
    )
}

fn liquid_testnet_bitcoin_asset() -> elements::AssetId {
    elements::AssetId::from_inner(elements::hashes::sha256::Midstate([
        0x49, 0x9a, 0x81, 0x85, 0x45, 0xf6, 0xba, 0xe3, 0x9f, 0xc0, 0x3b, 0x63, 0x7f, 0x2a, 0x4e,
        0x1e, 0x64, 0xe5, 0x90, 0xca, 0xc1, 0xbc, 0x3a, 0x6f, 0x6d, 0x71, 0xaa, 0x44, 0x43, 0x65,
        0x4c, 0x14,
    ]))
}

fn liquid_testnet_genesis() -> elements::BlockHash {
    elements::BlockHash::from_byte_array([
        0xc1, 0xb1, 0x6a, 0xe2, 0x4f, 0x24, 0x23, 0xae, 0xa2, 0xea, 0x34, 0x55, 0x22, 0x92, 0x79,
        0x3b, 0x5b, 0x5e, 0x82, 0x99, 0x9a, 0x1e, 0xed, 0x81, 0xd5, 0x6a, 0xee, 0x52, 0x8e, 0xda,
        0x71, 0xa7,
    ])
}

// Parse genesis hash from command line arguments
// Returns the genesis hash, defaulting to Liquid testnet if not provided
fn parse_genesis_flag(args: &[String]) -> elements::BlockHash {
    // Look for -g flag
    for i in 0..args.len() {
        if args[i] == "-g" {
            if i + 1 < args.len() {
                let genesis_hex = &args[i + 1];
                return genesis_hex.parse().expect("invalid genesis hash hex");
            } else {
                eprintln!("Error: -g flag requires a genesis hash argument");
                std::process::exit(1);
            }
        }
    }
    
    // Default to Liquid testnet
    liquid_testnet_genesis()
}

fn control_block(cmr: simplicity::Cmr) -> elements::taproot::ControlBlock {
    let info = taproot_spend_info(cmr);
    let script_ver = script_ver(cmr);
    info.control_block(&script_ver)
        .expect("control block should exist")
}

// Command: Generate address from contract
fn cmd_address(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: simplicity_tx_tool address <contract.simf>");
        std::process::exit(1);
    }
    
    let contract_file = &args[0];
    
    // Read and compile contract
    let source = std::fs::read_to_string(contract_file)
        .expect("Failed to read contract file");
    
    let compiled = simfony::CompiledProgram::new(source.as_str(), simfony::Arguments::default())
        .expect("Failed to compile contract");
    
    let cmr = compiled.commit().cmr();
    let address = liquid_testnet_address(cmr);
    
    println!("P2TR address: {}", address);
    println!("CMR: {}", hex::encode(cmr.as_ref()));
}

// Command: Compute sighash
fn cmd_sighash(args: &[String]) {
    if args.len() < 6 {
        eprintln!("Usage: simplicity_tx_tool sighash <contract.simf> <txid> <vout> <value> <destination> <fee> [-g <genesis_hash>]");
        std::process::exit(1);
    }
    
    let contract_file = &args[0];
    let txid_str = &args[1];
    let vout: u32 = args[2].parse().expect("vout must be a number");
    let value_in: u64 = args[3].parse().expect("value must be a number");
    let destination_str = &args[4];
    let fee: u64 = args[5].parse().expect("fee must be a number");
    
    // Parse optional genesis hash
    let genesis_hash = parse_genesis_flag(&args[6..]);
    
    // Parse inputs
    let txid = elements::Txid::from_str(txid_str).expect("Invalid txid");
    let destination = elements::Address::from_str(destination_str)
        .expect("Invalid destination address");
    
    // Read and compile contract
    let source = std::fs::read_to_string(contract_file)
        .expect("Failed to read contract file");
    
    let compiled = simfony::CompiledProgram::new(source.as_str(), simfony::Arguments::default())
        .expect("Failed to compile contract");
    
    let cmr = compiled.commit().cmr();
    
    // Build transaction parameters (matching web IDE)
    let tx_params = TxParams {
        txid,
        vout,
        value_in,
        recipient_address: Some(destination),
        fee,
        lock_time: elements::LockTime::from_consensus(0),
        sequence: elements::Sequence::from_consensus(0),
        genesis_hash,
    };
    
    // Create transaction environment (exactly like web IDE)
    let tx_env = tx_params.tx_env(cmr);
    
    // Compute sighash (exactly like web IDE)
    let sighash = tx_env.c_tx_env().sighash_all();
    let sighash_bytes: [u8; 32] = sighash.to_byte_array();
    
    println!("{}", hex::encode(sighash_bytes));
}

// Command: Build complete transaction
fn cmd_build_tx(args: &[String]) {
    if args.len() < 7 {
        eprintln!("Usage: simplicity_tx_tool build-tx <contract.simf> <txid> <vout> <value> <destination> <fee> <witness.wit> [-g <genesis_hash>]");
        std::process::exit(1);
    }
    
    let contract_file = &args[0];
    let txid_str = &args[1];
    let vout: u32 = args[2].parse().expect("vout must be a number");
    let value_in: u64 = args[3].parse().expect("value must be a number");
    let destination_str = &args[4];
    let fee: u64 = args[5].parse().expect("fee must be a number");
    let witness_file = &args[6];
    
    // Parse optional genesis hash
    let genesis_hash = parse_genesis_flag(&args[7..]);
    
    // Parse inputs
    let txid = elements::Txid::from_str(txid_str).expect("Invalid txid");
    let destination = elements::Address::from_str(destination_str)
        .expect("Invalid destination address");
    
    // Read and compile contract
    let source = std::fs::read_to_string(contract_file)
        .expect("Failed to read contract file");
    
    let compiled = simfony::CompiledProgram::new(source.as_str(), simfony::Arguments::default())
        .expect("Failed to compile contract");
    
    // Read witness
    let witness_source = std::fs::read_to_string(witness_file)
        .expect("Failed to read witness file");
    let witness_values: simfony::WitnessValues = serde_json::from_str(&witness_source)
        .expect("Failed to parse witness file");
    
    // Satisfy program with witness
    let satisfied = compiled.satisfy(witness_values)
        .expect("Failed to satisfy program");
    
    // Build transaction parameters
    let tx_params = TxParams {
        txid,
        vout,
        value_in,
        recipient_address: Some(destination),
        fee,
        lock_time: elements::LockTime::from_consensus(0),
        sequence: elements::Sequence::from_consensus(0),
        genesis_hash,
    };
    
    // Build complete transaction (exactly like web IDE)
    let tx = tx_params.transaction(&satisfied.redeem());
    
    // Encode to hex
    let mut tx_bytes = Vec::new();
    tx.consensus_encode(&mut tx_bytes).expect("Failed to encode transaction");
    
    println!("{}", hex::encode(tx_bytes));
}

// Transaction parameters struct (from web IDE)
#[derive(Clone, Debug)]
struct TxParams {
    txid: elements::Txid,
    vout: u32,
    value_in: u64,
    recipient_address: Option<elements::Address>,
    fee: u64,
    lock_time: elements::LockTime,
    sequence: elements::Sequence,
    genesis_hash: elements::BlockHash,
}

impl TxParams {
    fn unsatisfied_transaction(&self) -> elements::Transaction {
        elements::Transaction {
            version: 2,
            lock_time: self.lock_time,
            input: vec![elements::TxIn {
                previous_output: elements::OutPoint {
                    txid: self.txid,
                    vout: self.vout,
                },
                is_pegin: false,
                script_sig: elements::Script::new(),
                sequence: self.sequence,
                asset_issuance: elements::AssetIssuance::null(),
                witness: elements::TxInWitness::empty(),
            }],
            output: vec![
                elements::TxOut {
                    asset: confidential::Asset::Explicit(liquid_testnet_bitcoin_asset()),
                    value: confidential::Value::Explicit(self.value_in.saturating_sub(self.fee)),
                    nonce: confidential::Nonce::Null,
                    script_pubkey: self
                        .recipient_address
                        .as_ref()
                        .expect("Destination address required")
                        .script_pubkey(),
                    witness: elements::TxOutWitness::empty(),
                },
                elements::TxOut::new_fee(self.fee, liquid_testnet_bitcoin_asset()),
            ],
        }
    }
    
    fn utxo(&self, script_pubkey: elements::Script) -> ElementsUtxo {
        ElementsUtxo {
            script_pubkey,
            asset: confidential::Asset::Explicit(liquid_testnet_bitcoin_asset()),
            value: confidential::Value::Explicit(self.value_in),
        }
    }
    
    fn tx_env(&self, cmr: simplicity::Cmr) -> ElementsEnv<Arc<elements::Transaction>> {
        let script_pubkey = liquid_testnet_address(cmr).script_pubkey();
        let index = 0;
        let annex = None;
        ElementsEnv::new(
            Arc::new(self.unsatisfied_transaction()),
            vec![self.utxo(script_pubkey)],
            index,
            cmr,
            control_block(cmr),
            annex,
            self.genesis_hash,
        )
    }
    
    fn transaction(&self, pruned: &RedeemNode<Elements>) -> elements::Transaction {
        let mut tx = self.unsatisfied_transaction();
        let (simplicity_program_bytes, simplicity_witness_bytes) = pruned.encode_to_vec();
        let cmr = pruned.cmr();
        tx.input[0].witness = elements::TxInWitness {
            amount_rangeproof: None,
            inflation_keys_rangeproof: None,
            script_witness: vec![
                simplicity_witness_bytes,
                simplicity_program_bytes,
                cmr.as_ref().to_vec(),
                control_block(cmr).serialize(),
            ],
            pegin_witness: vec![],
        };
        tx
    }
}
