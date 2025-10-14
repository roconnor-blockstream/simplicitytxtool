# Simplicity Transaction Tool (TESTNET ONLY)

## Work in progress not official tool, no association with blockstream etc.

CLI tool that replicates the Simplicity Web IDE functionality for deploying contracts with witness data.

Based on: https://github.com/BlockstreamResearch/simplicity-webide

## What It Does

This tool provides command-line access to:
- Generate P2TR addresses for Simplicity contracts
- Compute correct sighash for transactions
- Build complete transactions with witness data
- Deploy contracts to Liquid testnet

**Default Constants**
- **Internal Key**: `0xf5919fa64ce45f8306849072b26c1bfdd2937e6b81774796ff372bd1eb5362d2` (same as Web IDE)
- **Genesis Hash**: `a771da8e52ee6ad581ed1e9a99825e5b3b7992225534eaa2ae23244fe26ab1c1` (Liquid testnet, can be overridden with `-g`)

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
simplicity_tx_tool sighash <contract.simf> <txid> <vout> <value> <destination> <fee> [-g <genesis_hash>]
```

**Arguments:**
- `contract.simf`: SimplicityHL contract file
- `txid`: Funding transaction ID
- `vout`: Output index (usually 0)
- `value`: Input value in satoshis
- `destination`: Destination address (tex1...)
- `fee`: Fee in satoshis
- `-g <genesis_hash>`: (Optional) Genesis hash in hex. Default: Liquid testnet

**Output:** 32-byte hex sighash

**Examples:**
```bash
# Use default Liquid testnet genesis hash
simplicity_tx_tool sighash p2pk_embedded.simf cbb3ac22... 0 100000 tex1q... 1000

# Override with Bitcoin mainnet genesis hash
simplicity_tx_tool sighash p2pk_embedded.simf cbb3ac22... 0 100000 bc1q... 1000 \
  -g 6fe28c0ab6f1b372c1a6a246ae63f74f931e83651e085ae689cd6190000000000
```

### Sign Sighash

```bash
simplicity_tx_tool sign <privkey_wif> <sighash>
```

**Arguments:**
- `privkey_wif`: Private key in WIF format (from elements-cli dumpprivkey)
- `sighash`: 32-byte hex sighash (from sighash command)

**Output:** 64-byte hex BIP-340 Schnorr signature

**Example:**
```bash
simplicity_tx_tool sign cSJofGp2qowy... 2f6b093e917fb945...
```

### Build Transaction

```bash
simplicity_tx_tool build-tx <contract.simf> <txid> <vout> <value> <destination> <fee> <witness.wit> [-g <genesis_hash>]
```

**Arguments:**
- Same as sighash command, plus:
- `witness.wit`: Witness file with signatures
- `-g <genesis_hash>`: (Optional) Genesis hash in hex. Default: Liquid testnet

**Output:** Complete transaction hex ready to broadcast

**Examples:**
```bash
# Use default Liquid testnet genesis hash
simplicity_tx_tool build-tx p2pk_embedded.simf cbb3ac22... 0 100000 tex1q... 1000 witness.wit

# Override with custom genesis hash
simplicity_tx_tool build-tx p2pk_embedded.simf cbb3ac22... 0 100000 bc1q... 1000 witness.wit \
  -g 6fe28c0ab6f1b372c1a6a246ae63f74f931e83651e085ae689cd6190000000000
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
simplicity_tx_tool sign <privkey_wif> <sighash>
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

## Genesis Hash Support

The `-g` flag allows you to specify which blockchain network to target:

**Liquid testnet (default):**
```
a771da8e52ee6ad581ed1e9a99825e5b3b7992225534eaa2ae23244fe26ab1c1
```

**Bitcoin mainnet:**
```
6fe28c0ab6f1b372c1a6a246ae63f74f931e83651e085ae689cd6190000000000
```

**Why it matters:** The genesis hash is included in the sighash computation, making signatures network-specific. A signature created for Liquid testnet will not be valid on Bitcoin mainnet, and vice versa. This prevents replay attacks across different networks.

## How It Works

This tool uses the same libraries as the Simplicity Web IDE:
- **simfony**: SimplicityHL compiler
- **simplicity-lang**: Rust Simplicity implementation
- **elements**: Elements/Liquid transaction handling

It replicates the Web IDE's transaction building logic exactly:
1. Compiles SimplicityHL to Simplicity bytecode
2. Computes CMR (Commitment Merkle Root)
3. Creates transaction environment with real transaction data
4. Computes sighash using `ElementsEnv::c_tx_env().sighash_all()` with genesis hash
5. Builds complete transaction with witness data

## Requirements

- Rust toolchain
- SimplicityHL contracts (.simf files)
- Witness data (.wit files) with valid signatures
- elements-cli for key management and destination addresses
- curl for broadcasting (or elements-cli)

## Differences from Web IDE

- **CLI-based**: No browser required
- **Same logic**: Uses identical transaction building code
- **Built-in signing**: BIP-340 Schnorr signing included (same as Web IDE)
- **Same output**: Generates identical transactions

## License

GPLv2

## Credits

Based on the Simplicity Web IDE by Blockstream Research:
https://github.com/BlockstreamResearch/simplicity-webide
