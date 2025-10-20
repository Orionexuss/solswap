use std::rc::Rc;

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
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::id;
mod utils;
use crate::utils::print_balances;

declare_program!(solswap);

use solswap::{client::accounts, client::args};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load depositor
    let depositor_wallet_path = std::path::Path::new("fixtures/depositor_wallet.json");
    let depositor = Rc::new(Keypair::read_from_file(depositor_wallet_path)?);

    let taker_wallet_path = std::path::Path::new("fixtures/taker_wallet.json");
    let taker = Rc::new(Keypair::read_from_file(taker_wallet_path)?);

    // Constants for token mints (Devnet USDC and WSOL)
    const USDC_PUBKEY: Pubkey = pubkey!("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr");
    const WSOL_PUBKEY: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

    let token_mint_in = WSOL_PUBKEY;
    let token_mint_out = USDC_PUBKEY;
    let token_program_id = id();

    // Set up Anchor client provider for Devnet
    let provider = Client::new_with_options(
        Cluster::Devnet,
        Rc::clone(&depositor),
        CommitmentConfig::confirmed(),
    );
    let program = provider.program(solswap::ID)?;

    // Derive config PDA (Program Derived Address)
    let (config_pda, _config_bump) = Pubkey::find_program_address(&[b"config"], &program.id());

    // Initialize config account on-chain
    let init_config_sig = program
        .request()
        .accounts(accounts::InitConfig {
            payer: depositor.pubkey(),
            config: config_pda,
            system_program: Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
        })
        .args(args::InitConfig {
            usdc_mint: USDC_PUBKEY,
        })
        .signer(&depositor)
        .send();

    match init_config_sig {
        Ok(sig) => println!("\nConfig initialized with signature: {:?}", sig),
        Err(e) => {
            let raw = format!("{e}");
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                println!("{}", serde_json::to_string_pretty(&val).unwrap());
            } else {
                println!("{raw}");
            }
        }
    }

    // Derive offer PDA for this user and token
    let (offer_pda, _offer_bump) = Pubkey::find_program_address(
        &[token_mint_in.as_ref(), depositor.pubkey().as_ref()],
        &program.id(),
    );

    // Derive vault PDA (associated token account for offer PDA)
    let vault_pda =
        get_associated_token_address_with_program_id(&offer_pda, &token_mint_in, &token_program_id);

    // Find depositor's associated token account for input token
    let user_token_account = get_associated_token_address_with_program_id(
        &depositor.pubkey(),
        &token_mint_in,
        &token_program_id,
    );

    // Print balances before offer creation
    print_balances(
        &depositor.pubkey(),
        Some(&taker.pubkey()),
        &token_mint_in,
        &token_mint_out,
        &token_program_id,
        "Balances Before Offer Creation",
    );

    // Create offer instruction
    let create_offer_sig = program
        .request()
        .accounts(accounts::CreateOffer {
            signer: depositor.pubkey(),
            offer: offer_pda,
            config: config_pda,
            token_mint_in,
            token_mint_out,
            vault: vault_pda,
            user_token_account,
            system_program: Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
            token_program: token_program_id,
            associated_token_program: spl_associated_token_account::id(),
        })
        .args(args::CreateOffer {
            amount: (LAMPORTS_PER_SOL as f64 * 0.05) as u64, // Offer 0.05 SOL
        })
        .signer(&depositor)
        .send();

    // Print result or error in readable form
    match create_offer_sig {
        Ok(sig) => {
            println!("\nOffer created with signature: {:?}", sig);
        }
        Err(e) => {
            let raw = format!("{e}");
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                println!("{}", serde_json::to_string_pretty(&val).unwrap());
            } else {
                println!("{raw}");
            }
        }
    }

    const PRICE_FEED_ACCOUNT: Pubkey =
        Pubkey::from_str_const("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

    let taker_token_in_ata = get_associated_token_address_with_program_id(
        &taker.pubkey(),
        &token_mint_in,
        &token_program_id,
    );

    let taker_token_out_ata = get_associated_token_address_with_program_id(
        &taker.pubkey(),
        &token_mint_out,
        &token_program_id,
    );

    let depositor_receive_ata = get_associated_token_address_with_program_id(
        &depositor.pubkey(),
        &token_mint_out,
        &token_program_id,
    );

    // Take offer instruction
    let take_offer_sig = program
        .request()
        .accounts(accounts::TakeOffer {
            taker: taker.pubkey(),
            depositor: depositor.pubkey(),
            token_mint_in,
            token_mint_out,
            taker_token_in_ata,
            taker_token_out_ata,
            depositor_receive_ata,
            offer: offer_pda,
            vault: vault_pda,
            price_update: PRICE_FEED_ACCOUNT,
            associated_token_program: spl_associated_token_account::id(),
            system_program: Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
            token_program: token_program_id,
        })
        .args(args::TakeOffer {})
        .signer(&taker)
        .payer(Rc::clone(&taker))
        .send();

    match take_offer_sig {
        Ok(sig) => {
            // Print balances after offer taken
            print_balances(
                &depositor.pubkey(),
                Some(&taker.pubkey()),
                &token_mint_in,
                &token_mint_out,
                &token_program_id,
                "Balances After Taking Offer",
            );
            println!("\nOffer taken with signature: {:?}", sig);
        }
        Err(e) => {
            let raw = format!("{e}");
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                println!("{}", serde_json::to_string_pretty(&val).unwrap());
            } else {
                println!("{raw}");
            }
        }
    }

    Ok(())
}
