use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_client::solana_sdk::program_pack::Pack;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_instruction::create_account;
use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::{Client, ClientError, Cluster};
use anchor_lang::{prelude::*, system_program};
use spl_token_2022::instruction::{
    initialize_account, initialize_mint, set_authority, AuthorityType,
};
use spl_token_2022::{id, native_mint};

declare_program!(solswap);
use solswap::{accounts::Offer, client::accounts, client::args};
use spl_token_2022::state::Mint;
use std::rc::Rc;

fn main() -> std::result::Result<(), ClientError> {
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

    // create mint account keypair
    let mint_account_usdc = Keypair::new();
    let token_program_2022_id = id();

    let rent_lamport = connection.get_minimum_balance_for_rent_exemption(Mint::LEN)?;

    let create_mint_ix = create_account(
        &payer.pubkey(),
        &mint_account_usdc.pubkey(),
        rent_lamport,
        Mint::LEN as u64,
        &token_program_2022_id,
    );

    let ix_1 = initialize_mint(
        &token_program_2022_id,
        &mint_account_usdc.pubkey(),
        &payer.pubkey(),
        None,
        6,
    )?;

    let ix_2 = set_authority(
        &token_program_2022_id,
        &mint_account_usdc.pubkey(),
        None,
        AuthorityType::MintTokens,
        &payer.pubkey(),
        &[&payer.pubkey()],
    )?;

    let recent_blockhash = connection.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix, ix_1, ix_2],
        Some(&payer.pubkey()),
        &[&*payer, &mint_account_usdc],
        recent_blockhash,
    );

    let create_mint_tx = connection.send_and_confirm_transaction(&tx)?;
    println!("Create mint signature {}", create_mint_tx);

    let provider = Client::new_with_options(
        Cluster::Localnet,
        Rc::clone(&payer),
        CommitmentConfig::confirmed(),
    );

    let program = provider.program(solswap::ID)?;
    let offer_account = Keypair::new();
    let vault_account = Keypair::new();

    let create_offer = program
        .request()
        .accounts(accounts::CreateOffer {
            signer: payer.pubkey(),
            mint_deposit: native_mint::ID,
            mint_receive: mint_account_usdc.pubkey(),
            offer: offer_account.pubkey(),
            vault: vault_account.pubkey(),
            system_program: system_program::ID,
            token_program: token_program_2022_id,
            associated_token_program: spl_associated_token_account::id(),
        })
        .args(args::CreateOffer {
            amount: LAMPORTS_PER_SOL * 2,
        })
        .signer(&payer)
        .send()?;

    println!("instruction signature {}", create_offer);

    Ok(())
}
