use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{release_vault, Offer, OfferCancelled, OFFER_SEED};

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    pub token_mint_in: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        close = depositor,
        has_one = depositor,
        has_one = token_mint_in,
        seeds = [OFFER_SEED, depositor.key().as_ref(), &offer.id.to_le_bytes()],
        bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = token_mint_in,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = token_mint_in,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_token_account: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn process_cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
    let accounts = &ctx.accounts;
    let amount_returned = accounts.vault.amount;

    release_vault(
        &accounts.offer,
        &accounts.vault,
        &accounts.token_mint_in,
        accounts.depositor_token_account.to_account_info(),
        accounts.depositor.to_account_info(),
        &accounts.token_program,
    )?;

    emit!(OfferCancelled {
        offer: accounts.offer.key(),
        depositor: accounts.depositor.key(),
        amount_returned,
    });

    Ok(())
}
