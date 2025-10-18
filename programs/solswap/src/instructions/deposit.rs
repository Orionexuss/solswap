use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::Offer;
use crate::{error::ErrorCode, Config};

#[derive(Accounts)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub config: Account<'info, Config>,
    pub token_mint_in: InterfaceAccount<'info, Mint>,
    pub token_mint_out: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + Offer::INIT_SPACE,
        seeds = [token_mint_in.key().as_ref(),signer.key().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = token_mint_in,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
    mut,
    associated_token::mint = token_mint_in,
    associated_token::authority = signer,
    associated_token::token_program = token_program,
)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_create_offer(ctx: Context<CreateOffer>, amount: u64) -> Result<()> {
    msg!("amount: {}", amount);
    if amount == 0 {
        return Err(ErrorCode::AmountZero.into());
    }

    let token_in = ctx.accounts.token_mint_in.key();
    let token_out = ctx.accounts.token_mint_out.key();
    let usdc_mint = ctx.accounts.config.usdc_mint;
    let wsol_mint = pubkey!("So11111111111111111111111111111111111111112");

    // Only USDC and WSOL are allowed
    let valid_mints = [usdc_mint, wsol_mint];

    // Check that mint_in is allowed
    if !valid_mints.contains(&token_in) {
        return Err(ErrorCode::InvalidTokenIn.into());
    }

    // Check that mint_out is allowed
    if !valid_mints.contains(&token_out) {
        return Err(ErrorCode::InvalidTokenOut.into());
    }

    // Prevent them from being the same (meaningless swap)
    if token_in == token_out {
        return Err(ErrorCode::SameToken.into());
    }

    let cpi_acounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
        mint: ctx.accounts.token_mint_in.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_acounts);

    transfer_checked(cpi_ctx, amount, ctx.accounts.token_mint_in.decimals)?;

    *ctx.accounts.offer = Offer {
        token_mint_in: ctx.accounts.token_mint_in.key(),
        token_mint_out: ctx.accounts.token_mint_out.key(),
        depositor: ctx.accounts.signer.key(),
        vault: ctx.accounts.vault.key(),
        amount_deposit: amount,
        bump: ctx.bumps.offer,
    };

    msg!("Offer created: {}", ctx.accounts.offer.key());

    Ok(())
}
