# Starcoin VM2 Transaction Factory

A minimal transaction generation and submission tool for Starcoin blockchain testing.

## Features

- **Account Management**: Load accounts from CSV files or generate new ones
- **Automated Testing**: Continuously generate and submit transfer transactions
- **Balance Management**: Automatic balance top-up from funding account
- **Transaction Confirmation**: Real-time transaction confirmation via block events

## Usage

### Generate Test Accounts
```bash
starcoin-vm2-txfactory generate <account_file> <count>
```
Creates `count` new accounts and saves private keys to `account_file`.

### Run Transaction Factory
```bash
starcoin-vm2-txfactory run <node_url> <account_file> [target_address]
```
- `node_url`: RPC endpoint (WebSocket or IPC)
- `account_file`: File containing account private keys
- `target_address`: Optional recipient address (defaults to funding account)

## How It Works

1. **Account Loading**: Loads existing accounts or creates new ones
2. **Balance Check**: Ensures accounts have sufficient balance for transactions
3. **Transaction Flow**: Each account continuously:
   - Checks balance and requests top-up if needed
   - Creates and submits transfer transactions
   - Waits for confirmation via block events
   - Repeats the cycle

## Configuration

- Default transfer amount: 1,000 nano STC
- Minimum balance: 1,000,000,000 nano STC
- Max gas: 10,000,000 units

Perfect for blockchain stress testing and performance evaluation.