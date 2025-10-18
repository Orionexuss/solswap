use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::Offer;

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    pub depositor: SystemAccount<'info>,

    pub token_mint_in: InterfaceAccount<'info, Mint>, // The mint deposit refers to the token stored in the offer vault

    pub token_mint_out: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_in,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_deposit_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint_out,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_receive_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_out,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_receive_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = taker,
        has_one = depositor,
        has_one = token_mint_in,
        has_one = token_mint_out,
        seeds = [token_mint_in.key().as_ref(),depositor.key().as_ref()],
        bump


    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = token_mint_in,
        associated_token::authority = offer,
        associated_token::token_program = token_program

    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn process_take_offer(_ctx: Context<TakeOffer>) -> Result<()> {
    Ok(())
}
