# Simplicity Transaction Tool (TESTNET ONLY)

CLI tool that replicates the Simplicity Web IDE functionality for deploying contracts with witness data.

Based on: https://github.com/BlockstreamResearch/simplicity-webide

## What It Does

This tool provides command-line access to:
- Generate P2TR addresses for Simplicity contracts
- Compute correct sighash for transactions
- Build complete transactions with witness data
- Deploy contracts to Liquid testnet

## Installation

```bash
cargo install --path .
```

Or install directly from GitHub:
```bash
cargo install --git https://github.com/iajhff/simplicitytxtool
```

## Commands

### Generate Address

```bash
simplicity_tx_tool address <contract.simf>
```

**Output:**
- P2TR address for the contract
- CMR (Commitment Merkle Root)

**Example:**
```bash
simplicity_tx_tool address p2pk_embedded.simf
```

### Compute Sighash

```bash
simplicity_tx_tool sighash <contract.simf> <txid> <vout> <value> <destination> <fee>
```

**Arguments:**
- `contract.simf`: SimplicityHL contract file
- `txid`: Funding transaction ID
- `vout`: Output index (usually 0)
- `value`: Input value in satoshis
- `destination`: Destination address (tex1...)
- `fee`: Fee in satoshis

**Output:** 32-byte hex sighash

**Example:**
```bash
simplicity_tx_tool sighash p2pk_embedded.simf cbb3ac22... 0 100000 tex1q... 1000
```

### Build Transaction

```bash
simplicity_tx_tool build-tx <contract.simf> <txid> <vout> <value> <destination> <fee> <witness.wit>
```

**Arguments:**
- Same as sighash command, plus:
- `witness.wit`: Witness file with signatures

**Output:** Complete transaction hex ready to broadcast

**Example:**
```bash
simplicity_tx_tool build-tx p2pk_embedded.simf cbb3ac22... 0 100000 tex1q... 1000 witness.wit
```

## Complete Workflow

### 1. Generate Address

```bash
simplicity_tx_tool address contract.simf
```

Copy the P2TR address.

### 2. Fund Address

Go to https://liquidtestnet.com/faucet and send funds to your address.

### 3. Compute Sighash

```bash
simplicity_tx_tool sighash contract.simf <txid> <vout> <value> <destination> <fee>
```

Copy the sighash output.

### 4. Sign Sighash

```bash
hal key schnorr-sign <privkey_wif> <sighash>
```

Copy the signature.

### 5. Create Witness File

```bash
cat > witness.wit << 'EOF'
{
    "YOUR_WITNESS_NAME": {
        "value": "0xYOUR_SIGNATURE",
        "type": "Signature"
    }
}
EOF
```

### 6. Build Transaction

```bash
simplicity_tx_tool build-tx contract.simf <txid> <vout> <value> <destination> <fee> witness.wit
```

Copy the transaction hex.

### 7. Broadcast

```bash
curl -X POST "https://blockstream.info/liquidtestnet/api/tx" -d "<tx_hex>"
```

Or use elements-cli:
```bash
elements-cli -chain=liquidtestnet sendrawtransaction <tx_hex>
```

## How It Works

This tool uses the same libraries as the Simplicity Web IDE:
- **simfony**: SimplicityHL compiler
- **simplicity-lang**: Rust Simplicity implementation
- **elements**: Elements/Liquid transaction handling

It replicates the Web IDE's transaction building logic exactly:
1. Compiles SimplicityHL to Simplicity bytecode
2. Computes CMR (Commitment Merkle Root)
3. Creates transaction environment with real transaction data
4. Computes sighash using `ElementsEnv::c_tx_env().sighash_all()`
5. Builds complete transaction with witness data

## Requirements

- Rust toolchain
- SimplicityHL contracts (.simf files)
- Witness data (.wit files) with valid signatures
- hal tool for BIP-340 signing
- elements-cli or curl for broadcasting

## Differences from Web IDE

- **CLI-based**: No browser required
- **Same logic**: Uses identical transaction building code
- **Manual signing**: Use hal tool for signatures (Web IDE has built-in key management)
- **Same output**: Generates identical transactions

## Differences from `simply`

The `simply` tool uses a dummy environment for witness validation, which makes it difficult to use with custom signatures. This tool uses the real transaction environment, matching the Web IDE approach.

## License

Same as parent repository

## Credits

Based on the Simplicity Web IDE by Blockstream Research:
https://github.com/BlockstreamResearch/simplicity-webide
