# SolSwap

Oracle-priced escrow swaps between **WSOL** and **USDC** on Solana, built with Anchor.

A depositor escrows one asset in a PDA-owned vault. Any taker can fill the whole offer by paying
the other asset at the live **Chainlink SOL / USD** price, read on-chain at the moment of the swap.
There is no order matching and no pool: each offer is a self-contained escrow.

## Program

| | |
|---|---|
| Program ID (devnet) | `4RGdWYYuAVYWUH23MQxZomSRUYpstb5zTVbrSGocHYBn` |
| Oracle | Chainlink SOL / USD, devnet feed `99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR` |
| USDC (devnet) | `Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr` ([spl-token-faucet.com](https://spl-token-faucet.com/?token-name=USDC-Dev)) |
| Token program | SPL Token via `token_interface` |

## Instructions

| Instruction | Who | What it does |
|---|---|---|
| `init_config(usdc_mint, price_feed, max_price_age_secs)` | Upgrade authority only | Creates the config. Checked against the program's `ProgramData`, so it cannot be front-run. |
| `update_config(price_feed, max_price_age_secs)` | Config admin | Rotates the feed or the freshness window (1 to 3600 s). |
| `create_offer(offer_id, amount)` | Anyone | Escrows `amount` of WSOL or USDC in a vault owned by the offer PDA. |
| `take_offer(max_amount_out)` | Anyone | Reads Chainlink, pays the depositor at the oracle price, receives the vault, closes the offer. |
| `cancel_offer()` | Depositor | Returns the escrow and closes the offer. |

## Accounts

- **Config** `["swap_config"]`: admin, USDC mint, Chainlink feed, max price age.
- **Offer** `["offer", depositor, offer_id (u64 LE)]`: one PDA per offer, so a depositor can keep several open.
- **Vault**: associated token account of the offer PDA. Closed (rent to the depositor) when the offer is taken or cancelled.

## Pricing

`take_offer` reads the latest round with the official `chainlink_solana::v2::read_feed_v2`, which
validates the account owner and layout. The program then checks:

- the account is the configured feed,
- the round is at most `max_price_age_secs` old (60 s on devnet),
- the answer is positive.

Conversions use `u128` checked math with the mints' and the feed's decimals, and round **up** in the
depositor's favor. The taker passes `max_amount_out`, a slippage bound, and the swap fails with
`SlippageExceeded` if the oracle moved past it.

## Events

`OfferCreated`, `OfferTaken` (including the price and its timestamp), `OfferCancelled`.

## Build, test, deploy

```bash
anchor build
cargo test -p solswap --lib      # conversion math
anchor deploy --provider.cluster devnet
```

## Client example

```bash
cp target/idl/solswap.json client/idls/
cd client && cargo run
```

`client/fixtures/depositor_wallet.json` needs devnet SOL (the client wraps 0.05 SOL).
`client/fixtures/taker_wallet.json` needs devnet USDC-Dev.

## Security notes

- Only WSOL and the configured USDC mint are accepted, and they must differ.
- Offers use `init`, never `init_if_needed`, so an open offer can't be overwritten.
- The vault authority is the offer PDA. Only the program can move escrowed funds.
- Stale or non-positive oracle prices are rejected, and the taker sets a slippage bound.
- Offer and vault rent goes back to the depositor.
