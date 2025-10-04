pub fn create_offer_ix(
    mint_deposit: &solana_pubkey::Pubkey,
    mint_receive: &solana_pubkey::Pubkey,
    depositor: &solana_pubkey::Pubkey,
    program_id: &solana_pubkey::Pubkey,
    user_associated_token_account: &solana_pubkey::Pubkey,
    amount: u64,
) -> solana_instruction::Instruction {
    // Anchor instruction discriminator for create_offer
    // Calculate the correct 8-byte discriminator from SHA256 hash of "global:create_offer"
    use sha2::{Digest, Sha256};
    let discriminator = &Sha256::digest(b"global:create_offer")[..8];
    let mut data = discriminator.to_vec();
    data.extend_from_slice(&amount.to_le_bytes());

    let (offer_pda, _bump) = solana_pubkey::Pubkey::find_program_address(
        &[mint_deposit.as_ref(), depositor.as_ref()],
        program_id,
    );

    let vault_account = spl_associated_token_account::get_associated_token_address_with_program_id(
        &offer_pda,
        mint_deposit,
        &spl_token_2022::id(),
    );

    let accounts = vec![
        // Depositor (signer) - mut
        solana_instruction::AccountMeta::new(*depositor, true),
        // Mints
        solana_instruction::AccountMeta::new_readonly(*mint_deposit, false),
        solana_instruction::AccountMeta::new_readonly(*mint_receive, false),
        // Offer PDA (init)
        solana_instruction::AccountMeta::new(offer_pda, false),
        // Vault ATA (init)
        solana_instruction::AccountMeta::new(vault_account, false),
        // User token account (mut)
        solana_instruction::AccountMeta::new(*user_associated_token_account, false),
        // System program
        solana_instruction::AccountMeta::new_readonly(
            solana_system_interface::program::id(),
            false,
        ),
        // Token program
        solana_instruction::AccountMeta::new_readonly(spl_token_2022::id(), false),
        // Associated token program
        solana_instruction::AccountMeta::new_readonly(spl_associated_token_account::id(), false),
    ];

    solana_instruction::Instruction {
        program_id: *program_id,
        accounts,
        data,
    }
}
