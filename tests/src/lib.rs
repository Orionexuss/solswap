pub mod instructions;
pub use instructions::*;

#[cfg(test)]
mod tests {
    use super::*;
    use solana_client::rpc_client::RpcClient;
    use solana_keypair::Keypair;
    use solana_pubkey::pubkey;
    use solana_sdk::{
        commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, program_pack::Pack,
    };
    use solana_signer::Signer;
    use solana_system_interface::instruction::create_account;
    use spl_associated_token_account::instruction::create_associated_token_account;
    use spl_token_2022::{
        instruction::{initialize_mint2, mint_to_checked},
        state::Mint,
    };

    fn airdrop_and_confirm(rpc_client: &RpcClient, pubkey: &solana_pubkey::Pubkey, lamports: u64) {
        let signature = rpc_client.request_airdrop(pubkey, lamports).unwrap();
        rpc_client
            .poll_for_signature_with_commitment(&signature, CommitmentConfig::confirmed())
            .unwrap();
    }

    #[test]
    fn integration_test() {
        // Point RPC client at the local test validator
        let rpc_client = RpcClient::new_with_commitment(
            "http://127.0.0.1:8899".to_string(),
            CommitmentConfig::confirmed(),
        );

        let program_id = pubkey!("Bw51Xa4JoAiyhE2e8cQAmFNjB5F7pazMWjDxdwKL6Giv");

        // Deploy the program to the test validator
        let program_keypair = Keypair::new();
        airdrop_and_confirm(&rpc_client, &program_keypair.pubkey(), LAMPORTS_PER_SOL);

        // Create depositor and fund account with 5 SOLs
        let depositor_keypair = Keypair::new();
        let depositor_pubkey = depositor_keypair.pubkey();
        airdrop_and_confirm(&rpc_client, &depositor_pubkey, 5 * LAMPORTS_PER_SOL);

        let mint_rent = rpc_client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)
            .unwrap();

        println!("Depositor Pubkey: {}", depositor_pubkey);

        // Create mint for deposit token
        let mint_deposit = Keypair::new();

        let create_mint_deposit_ix = create_account(
            &depositor_pubkey,
            &mint_deposit.pubkey(),
            mint_rent,
            Mint::LEN as u64,
            &spl_token_2022::id(),
        );

        let init_mint_deposit_ix = initialize_mint2(
            &spl_token_2022::id(),
            &mint_deposit.pubkey(),
            &depositor_pubkey,
            None,
            6,
        )
        .unwrap();

        // Create mint for receive token
        let mint_receive = Keypair::new();

        let create_mint_create_ix = create_account(
            &depositor_pubkey,
            &mint_receive.pubkey(),
            mint_rent,
            Mint::LEN as u64,
            &spl_token_2022::id(),
        );

        let init_mint_receive_ix = initialize_mint2(
            &spl_token_2022::id(),
            &mint_receive.pubkey(),
            &depositor_pubkey,
            None,
            6,
        )
        .unwrap();

        // Derive user token account
        let user_associated_token_account =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &depositor_pubkey,
                &mint_deposit.pubkey(),
                &spl_token_2022::id(),
            );

        let init_user_ata_ix = create_associated_token_account(
            &depositor_pubkey,
            &depositor_pubkey,
            &mint_deposit.pubkey(),
            &spl_token_2022::id(),
        );

        let mint_to_depositor_ix = mint_to_checked(
            &spl_token_2022::id(),
            &mint_deposit.pubkey(),
            &user_associated_token_account,
            &depositor_pubkey,
            &[],
            200 * 10_u64.pow(6),
            6,
        )
        .unwrap();

        let create_offer_ix = create_offer_ix(
            &mint_deposit.pubkey(),
            &mint_receive.pubkey(),
            &depositor_pubkey,
            &program_id,
            &user_associated_token_account,
            100 * 10_u64.pow(6),
        );

        let latest_blockhash = rpc_client.get_latest_blockhash().unwrap();

        let setup_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[
                create_mint_deposit_ix,
                init_mint_deposit_ix,
                create_mint_create_ix,
                init_mint_receive_ix,
                init_user_ata_ix,
                mint_to_depositor_ix,
                create_offer_ix,
            ],
            Some(&depositor_pubkey),
            &[&depositor_keypair, &mint_receive, &mint_deposit],
            latest_blockhash,
        );

        println!("Simulating transaction before sending...");
        match rpc_client.simulate_transaction(&setup_tx) {
            Ok(sim) => {
                if let Some(logs) = sim.value.logs {
                    println!("Simulation logs:");
                    for log in logs {
                        println!("{}", log);
                    }
                } else {
                    println!("No logs found in simulation");
                }
                if let Some(err) = sim.value.err {
                    println!("Simulation error: {:?}", err);
                }
            }
            Err(e) => println!("simulate_transaction failed: {:?}", e),
        }
        let setup_result = rpc_client.send_and_confirm_transaction(&setup_tx);

        match setup_result {
            Ok(signature) => {
                println!("Setup transaction succeeded: {}", signature);
                // Aquí puedes seguir con get_signature_status o cualquier otra verificación
            }
            Err(e) => {
                println!("Setup transaction failed: {:#?}", e);
                panic!("Setup transaction failed");
            }
        }
    }
}
