# SolSwap

A Solana program for trustless token swaps built with Anchor framework.

## Features

- **Create offers**: Deposit tokens into a vault and specify what you want in return
- **Take offers**: Pay the counter token and receive the escrowed tokens
- **Price-aware**: Uses Pyth price feeds for SOL/USDC conversions
- **Supported tokens**: WSOL and USDC only

## Program Details

- **Program ID**: `3c9wj6bDT9opsUWPAPdGjdddv1GKF8R7yDpR9ZH7VpvX`
- **Token Program**: SPL Token 2022
- **Supported mints**: WSOL, USDC (configurable)

## Instructions

1. **`init_config(usdc_mint)`** - Set the USDC mint address
2. **`create_offer(amount)`** - Deposit tokens and create an offer
3. **`take_offer()`** - Take an existing offer using Pyth price data

## Quick Start

### Prerequisites
- Rust, Solana CLI, Anchor framework

### Build & Deploy
```bash
anchor build
anchor deploy
```

### Run Client Example

**Setup required:**
1. Build the program first:
```bash
anchor build
```

2. Copy the generated IDL to the client:
```bash
cp target/idl/solswap.json client/idls/
```

3. Ensure fixture keypairs exist and are funded:
   - `client/fixtures/depositor_wallet.json` - must have WSOL balance
   - `client/fixtures/taker_wallet.json` - must have USDC balance

4. Run the client:
```bash
cd client
cargo run
```

## Architecture

- **Config PDA**: `["config"]` - stores USDC mint
- **Offer PDA**: `[token_mint_in, depositor]` - represents an active offer
- **Vault ATA**: owned by Offer PDA, holds escrowed tokens

## Security

- Only WSOL and USDC allowed
- Vault authority is the Offer PDA
- Offers are closed after being taken
- Pyth price validation required
