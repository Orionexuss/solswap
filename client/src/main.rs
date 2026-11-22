use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io, path::Path};

use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        native_token::LAMPORTS_PER_SOL,
        signature::Keypair,
        signer::{EncodableKey, Signer},
    },
    Client, Cluster,
};
use anchor_lang::prelude::*;
use spl_associated_token_account::{
    get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent,
};
use spl_token::{id, instruction::sync_native, native_mint};
mod utils;
use crate::utils::print_balances;

declare_program!(solswap);

use solswap::{client::accounts, client::args};

/// Devnet USDC-Dev (spl-token-faucet.com) and the Chainlink SOL / USD devnet feed.
const USDC_PUBKEY: Pubkey = pubkey!("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr");
const CHAINLINK_SOL_USD_FEED: Pubkey = pubkey!("99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR");
const OFFER_LAMPORTS: u64 = LAMPORTS_PER_SOL / 20; // 0.05 SOL

fn load_wallet(path: &str, label: &str) -> std::result::Result<Rc<Keypair>, Box<dyn std::error::Error>> {
    let wallet_path = Path::new(path);

    if !wallet_path.exists() {
        let cwd = env::current_dir()?.display().to_string();
        let msg = format!(
            "Missing {label} wallet file: '{path}'.\nExpected relative to: {cwd}\n\
Create it with:\n\
  mkdir -p fixtures\n\
  solana-keygen new --no-bip39-passphrase -o {path}"
        );
        return Err(io::Error::new(io::ErrorKind::NotFound, msg).into());
    }

    Keypair::read_from_file(wallet_path).map(Rc::new).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to read {label} wallet at '{path}': {e}"),
        )
        .into()
    })
}

fn print_result(label: &str, result: std::result::Result<impl std::fmt::Debug, impl std::fmt::Display>) {
    match result {
        Ok(signature) => println!("\n{label}: {signature:?}"),
        Err(error) => {
            let raw = format!("{error}");
            match serde_json::from_str::<serde_json::Value>(&raw) {
                Ok(value) => println!("\n{label} failed:\n{}", serde_json::to_string_pretty(&value).unwrap()),
                Err(_) => println!("\n{label} failed: {raw}"),
            }
        }
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let depositor = load_wallet("fixtures/depositor_wallet.json", "depositor")?;
    let taker = load_wallet("fixtures/taker_wallet.json", "taker")?;

    let token_mint_in = native_mint::id();
    let token_mint_out = USDC_PUBKEY;
    let token_program_id = id();

    let provider = Client::new_with_options(Cluster::Devnet, Rc::clone(&depositor), CommitmentConfig::confirmed());
    let program = provider.program(solswap::ID)?;

    // The config is created once by the program's upgrade authority (see README).
    let (config_pda, _) = Pubkey::find_program_address(&[b"swap_config"], &program.id());
    let config: solswap::accounts::Config = program.account(config_pda)?;
    println!(
        "Config: USDC mint {}, Chainlink feed {}, max price age {}s",
        config.usdc_mint, config.price_feed, config.max_price_age_secs
    );

    // Each offer gets its own id, so a depositor can keep several open at once.
    let offer_id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let (offer_pda, _) = Pubkey::find_program_address(
        &[b"offer", depositor.pubkey().as_ref(), &offer_id.to_le_bytes()],
        &program.id(),
    );
    let vault = get_associated_token_address_with_program_id(&offer_pda, &token_mint_in, &token_program_id);
    let depositor_wsol = get_associated_token_address_with_program_id(&depositor.pubkey(), &token_mint_in, &token_program_id);

    print_balances(
        &depositor.pubkey(),
        Some(&taker.pubkey()),
        &token_mint_in,
        &token_mint_out,
        &token_program_id,
        "Balances Before Offer Creation",
    );

    // Wrap 0.05 SOL and escrow it in the same transaction.
    let create_offer = program
        .request()
        .instruction(create_associated_token_account_idempotent(
            &depositor.pubkey(),
            &depositor.pubkey(),
            &token_mint_in,
            &token_program_id,
        ))
        .instruction(anchor_lang::solana_program::system_instruction::transfer(
            &depositor.pubkey(),
            &depositor_wsol,
            OFFER_LAMPORTS,
        ))
        .instruction(sync_native(&token_program_id, &depositor_wsol)?)
        .accounts(accounts::CreateOffer {
            depositor: depositor.pubkey(),
            config: config_pda,
            token_mint_in,
            token_mint_out,
            offer: offer_pda,
            vault,
            depositor_token_account: depositor_wsol,
            system_program: Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
            token_program: token_program_id,
            associated_token_program: spl_associated_token_account::id(),
        })
        .args(args::CreateOffer { offer_id, amount: OFFER_LAMPORTS })
        .signer(&depositor)
        .send();
    print_result("Offer created", create_offer);

    let taker_usdc = get_associated_token_address_with_program_id(&taker.pubkey(), &token_mint_out, &token_program_id);
    // Slippage bound: the taker accepts paying up to its whole USDC balance for this demo.
    let max_amount_out: u64 = program.rpc().get_token_account_balance(&taker_usdc)?.amount.parse()?;

    let take_offer = program
        .request()
        .accounts(accounts::TakeOffer {
            taker: taker.pubkey(),
            depositor: depositor.pubkey(),
            config: config_pda,
            token_mint_in,
            token_mint_out,
            taker_token_in_ata: get_associated_token_address_with_program_id(&taker.pubkey(), &token_mint_in, &token_program_id),
            taker_token_out_ata: taker_usdc,
            depositor_receive_ata: get_associated_token_address_with_program_id(
                &depositor.pubkey(),
                &token_mint_out,
                &token_program_id,
            ),
            offer: offer_pda,
            vault,
            price_feed: CHAINLINK_SOL_USD_FEED,
            associated_token_program: spl_associated_token_account::id(),
            system_program: Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
            token_program: token_program_id,
        })
        .args(args::TakeOffer { max_amount_out })
        .signer(&taker)
        .payer(Rc::clone(&taker))
        .send();
    print_result("Offer taken", take_offer);

    print_balances(
        &depositor.pubkey(),
        Some(&taker.pubkey()),
        &token_mint_in,
        &token_mint_out,
        &token_program_id,
        "Balances After Taking Offer",
    );

    Ok(())
}
