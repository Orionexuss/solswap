# SolSwap

A Solana program for token swaps built with Anchor framework. Currently in development with basic offer creation functionality.

## Current Status

**⚠️ Work in Progress** 

### What's Implemented
- `create_offer` - Creates a token swap offer by depositing tokens into a vault

### What's Missing
- Accept offer functionality
- Complete swap execution
- Offer cancellation

## Quick Start

### Prerequisites
- Rust
- Solana CLI
- Anchor framework

### Build & Deploy
```bash
anchor build
anchor deploy
```

### Run Client Example
```bash
cd client
cargo run
```

## Program Details

- **Program ID**: `Bw51Xa4JoAiyhE2e8cQAmFNjB5F7pazMWjDxdwKL6Giv`
- **Current Instruction**: `create_offer(amount: u64)`
- **Uses**: SPL Token 2022, PDAs, Associated Token Accounts

## Development

This is an early-stage project. The current implementation demonstrates offer creation with token deposits, but the full swap functionality is still being developed.