use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
    TransferChecked,
};

use crate::{Offer, OFFER_SEED};

/// Moves the whole vault balance to `recipient`, then closes the vault and returns its rent
/// to `rent_receiver`. The offer PDA signs as vault authority.
pub fn release_vault<'info>(
    offer: &Account<'info, Offer>,
    vault: &InterfaceAccount<'info, TokenAccount>,
    mint: &InterfaceAccount<'info, Mint>,
    recipient: AccountInfo<'info>,
    rent_receiver: AccountInfo<'info>,
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    let id_bytes = offer.id.to_le_bytes();
    let seeds: &[&[u8]] = &[
        OFFER_SEED,
        offer.depositor.as_ref(),
        &id_bytes,
        &[offer.bump],
    ];
    let signer_seeds = &[seeds];

    transfer_checked(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            TransferChecked {
                from: vault.to_account_info(),
                to: recipient,
                authority: offer.to_account_info(),
                mint: mint.to_account_info(),
            },
            signer_seeds,
        ),
        vault.amount,
        mint.decimals,
    )?;

    close_account(CpiContext::new_with_signer(
        token_program.to_account_info(),
        CloseAccount {
            account: vault.to_account_info(),
            destination: rent_receiver,
            authority: offer.to_account_info(),
        },
        signer_seeds,
    ))
}
