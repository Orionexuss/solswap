use std::rc::Rc;

use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, program_pack::Pack,
        signature::Keypair, signer::Signer, system_instruction::create_account,
        transaction::Transaction,
    },
    Client, Cluster,
};
use anchor_lang::prelude::*;
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token_2022::{
    id,
    instruction::{initialize_mint, mint_to, set_authority, AuthorityType},
    state::Mint,
};

declare_program!(solswap);

use solswap::{client::accounts, client::args};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let payer = Rc::new(Keypair::new());
    let connection =
        RpcClient::new_with_commitment(Cluster::Localnet.url(), CommitmentConfig::confirmed());

    println!("Generated keypair: {}", payer.pubkey());

    // Check balance
    let balance = connection.get_balance(&payer.pubkey())?;
    println!("Initial balance: {} lamports", balance);

    // Fund the account with an airdrop
    let fund_account_signature =
        connection.request_airdrop(&payer.pubkey(), LAMPORTS_PER_SOL * 5)?;

    connection.poll_for_signature(&fund_account_signature)?;

    let new_balance = connection.get_balance(&payer.pubkey())?;
    println!("Balance after airdrop: {} lamports", new_balance);

    // Mint account keypairs and program id
    let mint_account_usdc = Keypair::new();
    let mint_account_sol = Keypair::new();
    let token_program_2022_id = id();

    let rent_lamport = connection.get_minimum_balance_for_rent_exemption(Mint::LEN)?;

    // Create USDC mint
    let create_mint_usdc_ix = create_account(
        &payer.pubkey(),
        &mint_account_usdc.pubkey(),
        rent_lamport,
        Mint::LEN as u64,
        &token_program_2022_id,
    );

    let init_usdc_mint = initialize_mint(
        &token_program_2022_id,
        &mint_account_usdc.pubkey(),
        &payer.pubkey(),
        None,
        6,
    )?;

    let revoke_auth_usdc = set_authority(
        &token_program_2022_id,
        &mint_account_usdc.pubkey(),
        None,
        AuthorityType::MintTokens,
        &payer.pubkey(),
        &[&payer.pubkey()],
    )?;

    // Create SOL mint
    let create_mint_sol_ix = create_account(
        &payer.pubkey(),
        &mint_account_sol.pubkey(),
        rent_lamport,
        Mint::LEN as u64,
        &token_program_2022_id,
    );

    let init_sol_mint = initialize_mint(
        &token_program_2022_id,
        &mint_account_sol.pubkey(),
        &payer.pubkey(),
        None,
        6,
    )?;

    // Associated token account for payer
    let user_token_account = get_associated_token_address_with_program_id(
        &payer.pubkey(),
        &mint_account_sol.pubkey(),
        &token_program_2022_id,
    );

    let create_user_token_account_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &mint_account_sol.pubkey(),
        &token_program_2022_id,
    );

    let amount_to_mint: u64 = 2u64 * 10u64.pow(9); // 2 tokens with 9 decimals

    let mint_to_user_ix = mint_to(
        &token_program_2022_id,
        &mint_account_sol.pubkey(),
        &user_token_account,
        &payer.pubkey(),
        &[],
        amount_to_mint,
    )?;
    let revoke_auth_sol = set_authority(
        &token_program_2022_id,
        &mint_account_sol.pubkey(),
        None,
        AuthorityType::MintTokens,
        &payer.pubkey(),
        &[&payer.pubkey()],
    )?;

    let recent_blockhash = connection.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[
            create_mint_usdc_ix,
            init_usdc_mint,
            revoke_auth_usdc,
            create_mint_sol_ix,
            init_sol_mint,
            create_user_token_account_ix,
            mint_to_user_ix,
            revoke_auth_sol,
        ],
        Some(&payer.pubkey()),
        &[&*payer, &mint_account_usdc, &mint_account_sol],
        recent_blockhash,
    );

    let create_accounts_tx = connection.send_and_confirm_transaction(&tx);

    match create_accounts_tx {
        Ok(sig) => println!("Create acounts signature {:?}", sig),
        Err(e) => {
            let raw = format!("{e}");
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                println!("{}", serde_json::to_string_pretty(&val).unwrap());
            } else {
                println!("{raw}");
            }
        }
    };

    let provider = Client::new_with_options(
        Cluster::Localnet,
        Rc::clone(&payer),
        CommitmentConfig::confirmed(),
    );
    let program = provider.program(solswap::ID)?;

    let (offer_pda, _offer_bump) = Pubkey::find_program_address(
        &[mint_account_sol.pubkey().as_ref(), payer.pubkey().as_ref()],
        &program.id(),
    );

    let vault_pda = spl_associated_token_account::get_associated_token_address_with_program_id(
        &offer_pda,
        &mint_account_sol.pubkey(),
        &token_program_2022_id,
    );

    println!("vault_pda: {}", vault_pda);

    let create_offer = program
        .request()
        .accounts(accounts::CreateOffer {
            signer: payer.pubkey(),
            mint_deposit: mint_account_sol.pubkey(),
            mint_receive: mint_account_usdc.pubkey(),
            offer: offer_pda,
            vault: vault_pda,
            system_program: Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
            token_program: token_program_2022_id,
            associated_token_program: spl_associated_token_account::id(),
            user_token_account,
        })
        .args(args::CreateOffer {
            amount: LAMPORTS_PER_SOL * 2,
        })
        .signer(&payer)
        .send();

    match create_offer {
        Ok(sig) => println!("Instruction signature {:?}", sig),
        Err(e) => {
            let raw = format!("{e}");
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                println!("{}", serde_json::to_string_pretty(&val).unwrap());
            } else {
                println!("{raw}");
            }
        }
    };

    Ok(())
}
