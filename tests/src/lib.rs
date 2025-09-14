#[cfg(test)]
mod tests {
    use litesvm::LiteSVM;
    use sha2::{Digest, Sha256};
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_pubkey::{pubkey, Pubkey};
    use solana_signer::Signer;
    use solana_system_interface::program::id as system_program_id;
    use solana_transaction::Transaction;
    use spl_associated_token_account::{
        instruction::create_associated_token_account,
        solana_program::{native_token::LAMPORTS_PER_SOL, program_pack::Pack},
    };
    use spl_token_2022::{
        instruction::{initialize_mint2, mint_to_checked},
        state::Mint,
    };

    #[test]
    fn integration_test() {
        let mut svm = LiteSVM::new();

        let program_id = pubkey!("Bw51Xa4JoAiyhE2e8cQAmFNjB5F7pazMWjDxdwKL6Giv");
        let bytes = include_bytes!("../fixtures/solswap.so");
        let _ = svm.add_program(program_id, bytes);

        let create_offer_ix_discriminator: [u8; 8] = Sha256::digest(b"global:create_offer")[..8]
            .try_into()
            .unwrap();

        // Create depositor and fund account with 5 SOLs
        let depositor_keypair = Keypair::new();
        let depositor_pubkey = depositor_keypair.pubkey();
        let _ = svm.airdrop(&depositor_pubkey, LAMPORTS_PER_SOL * 5);

        let mint_rent = svm.minimum_balance_for_rent_exemption(Mint::LEN);

        // Create mint for deposit token
        let mint_deposit = Keypair::new();

        let create_mint_deposit_ix = solana_system_interface::instruction::create_account(
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

        let create_mint_create_ix = solana_system_interface::instruction::create_account(
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

        // Create offer account PDA
        let (offer_pda, _offer_bump) = Pubkey::find_program_address(
            &[mint_deposit.pubkey().as_ref(), depositor_pubkey.as_ref()],
            &program_id,
        );

        // Create vault ATA
        let vault_account =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &offer_pda,
                &mint_deposit.pubkey(),
                &spl_token_2022::id(),
            );

        let accounts = vec![
            AccountMeta::new(depositor_pubkey, true), // Signer
            AccountMeta::new_readonly(mint_deposit.pubkey(), false), // Mint deposit
            AccountMeta::new_readonly(mint_receive.pubkey(), false), // Mint receive
            AccountMeta::new(offer_pda, false),       // Offer account
            AccountMeta::new(vault_account, false),   // Vault account
            AccountMeta::new(user_associated_token_account, false), // User token account
            AccountMeta::new_readonly(system_program_id(), false), // System program
            AccountMeta::new_readonly(spl_token_2022::id(), false), // Token program
            AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
        ];
        let amount_to_offer: u64 = 100 * 10_u64.pow(6);

        let create_offer_ix = Instruction::new_with_borsh(
            program_id,
            &(create_offer_ix_discriminator, amount_to_offer),
            accounts,
        );

        let latest_blockhash = svm.latest_blockhash();

        println!("Latest blockhash: {:?}", latest_blockhash);

        let msg = Message::new_with_blockhash(
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
            &latest_blockhash,
        );

        let tx = Transaction::new(
            &[&depositor_keypair, &mint_receive, &mint_deposit],
            msg,
            latest_blockhash,
        );

        let meta = svm.send_transaction(tx).unwrap();

        println!("{:#?}", meta.logs)
    }
}
