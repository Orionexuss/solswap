use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::{Client, Cluster};
use anchor_lang::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let payer = Keypair::new();
    let provider = Client::new(Cluster::Localnet, &payer);
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

    // Program interaction
    let program = provider.program(program_id)
    Ok(())
}
