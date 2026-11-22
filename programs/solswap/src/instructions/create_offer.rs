use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::error::ErrorCode;
use crate::{Config, Offer, OfferCreated, CONFIG_SEED, OFFER_SEED, WSOL_MINT};

#[derive(Accounts)]
#[instruction(offer_id: u64)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(mint::token_program = token_program)]
    pub token_mint_in: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub token_mint_out: InterfaceAccount<'info, Mint>,

    /// `init` (not `init_if_needed`): an existing offer can never be overwritten.
    #[account(
        init,
        payer = depositor,
        space = 8 + Offer::INIT_SPACE,
        seeds = [OFFER_SEED, depositor.key().as_ref(), &offer_id.to_le_bytes()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        init,
        payer = depositor,
        associated_token::mint = token_mint_in,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint_in,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_create_offer(ctx: Context<CreateOffer>, offer_id: u64, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::AmountZero);

    let token_in = ctx.accounts.token_mint_in.key();
    let token_out = ctx.accounts.token_mint_out.key();
    let valid_mints = [ctx.accounts.config.usdc_mint, WSOL_MINT];

    require!(valid_mints.contains(&token_in), ErrorCode::InvalidTokenIn);
    require!(valid_mints.contains(&token_out), ErrorCode::InvalidTokenOut);
    require_keys_neq!(token_in, token_out, ErrorCode::SameToken);

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.depositor_token_account.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
                mint: ctx.accounts.token_mint_in.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.token_mint_in.decimals,
    )?;

    ctx.accounts.offer.set_inner(Offer {
        id: offer_id,
        depositor: ctx.accounts.depositor.key(),
        token_mint_in: token_in,
        token_mint_out: token_out,
        deposited_amount: amount,
        created_at: Clock::get()?.unix_timestamp,
        bump: ctx.bumps.offer,
    });

    emit!(OfferCreated {
        offer: ctx.accounts.offer.key(),
        depositor: ctx.accounts.depositor.key(),
        token_mint_in: token_in,
        token_mint_out: token_out,
        amount,
    });

    Ok(())
}
