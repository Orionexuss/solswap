use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::Offer;

#[derive(Accounts)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint_deposit: InterfaceAccount<'info, Mint>,

    pub mint_receive: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = signer,
        space = 8 + Offer::INIT_SPACE,
        seeds = [mint_deposit.key().as_ref(),signer.key().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_deposit,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_create_offer(ctx: Context<CreateOffer>, amount: u64) -> Result<()> {
    *ctx.accounts.offer = Offer {
        mint_deposit: ctx.accounts.mint_deposit.key(),
        mint_receive: ctx.accounts.mint_receive.key(),
        depositor_address: ctx.accounts.signer.key(),
        vault: ctx.accounts.vault.key(),
        amount_deposit: 0,
        status: true,
    };

    let cpi_acounts = TransferChecked {
        from: ctx.accounts.signer.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
        mint: ctx.accounts.mint_deposit.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_acounts);

    transfer_checked(cpi_ctx, amount, ctx.accounts.mint_deposit.decimals)?;

    Ok(())
}
