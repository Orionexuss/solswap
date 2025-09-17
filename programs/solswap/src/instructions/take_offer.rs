use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use switchboard_on_demand::QuoteVerifier;

use crate::Offer;

#[derive(Accounts)]
pub struct Sysvars<'info> {
    pub clock: Sysvar<'info, Clock>,
    pub slothashes: Sysvar<'info, SlotHashes>,

    #[account(
        address = pubkey!("Sysvar1nstructions1111111111111111111111111")
    )]
    pub instructions: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    pub depositor: SystemAccount<'info>,

    pub mint_deposit: InterfaceAccount<'info, Mint>, // The mint deposit refers to the token stored in the offer vault

    pub mint_receive: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_deposit,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_deposit_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_receive,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_receive_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_receive,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_receive_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = taker,
        has_one = depositor,
        has_one = mint_deposit,
        has_one = mint_receive,
        seeds = [mint_deposit.key().as_ref(),depositor.key().as_ref()],
        bump


    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = mint_deposit,
        associated_token::authority = offer,
        associated_token::token_program = token_program

    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub queue: AccountInfo<'info>,
    pub oracle: AccountInfo<'info>,
    pub sysvars: Sysvars<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn process_take_offer(ctx: Context<TakeOffer>) -> anchor_lang::Result<()> {
    let TakeOffer {
        queue,
        oracle,
        sysvars,
        ..
    } = ctx.accounts;
    let clock_slot = switchboard_on_demand::clock::get_slot(&sysvars.clock);

    let mut verifier = QuoteVerifier::new()
        .queue(&queue)
        .slothash_sysvar(&sysvars.slothashes)
        .ix_sysvar(&sysvars.instructions)
        .clock_slot(&clock_slot)
        .verify_account(&oracle)
        .unwrap();

    println!("Data {:?}", verifier.feeds());

    // Transfer receive tokens from taker to depositor
    // let cpi_accounts = TransferChecked {
    //     from: ctx.accounts.taker_receive_ata.to_account_info(),
    //     mint: ctx.accounts.mint_receive.to_account_info(),
    //     to: ctx.accounts.depositor_receive_ata.to_account_info(),
    //     authority: ctx.accounts.taker.to_account_info(),
    // };
    //
    // let cpi_program = ctx.accounts.token_program.to_account_info();
    // let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    //
    // transfer_checked(cpi_context, amount, ctx.accounts.mint_receive.decimals);

    Ok(())
}
