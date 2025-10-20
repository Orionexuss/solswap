use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{lamports_to_usdc, usdc_to_lamports, Offer, FEED_ID, MAXIMUM_AGE};

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    pub depositor: SystemAccount<'info>,

    pub token_mint_in: InterfaceAccount<'info, Mint>,
    pub token_mint_out: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_in,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_in_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint_out,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_out_ata: InterfaceAccount<'info, TokenAccount>,

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

    pub price_update: Account<'info, PriceUpdateV2>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn process_take_offer(ctx: Context<TakeOffer>) -> Result<()> {
    let price_update = &ctx.accounts.price_update;
    let price_info = price_update.get_price_no_older_than(
        &Clock::get()?,
        MAXIMUM_AGE,
        &get_feed_id_from_hex(FEED_ID)?,
    )?;

    let price = price_info.price;
    msg!("Current price: {}", price);

    let offer = &ctx.accounts.offer;
    let taker = &ctx.accounts.taker;
    let token_program = &ctx.accounts.token_program;

    let depositor_gave_usdc =
        offer.token_mint_in != pubkey!("So11111111111111111111111111111111111111112");

    if depositor_gave_usdc {
        // depositor deposited USDC, taker sends SOL
        let lamports_amount = usdc_to_lamports(offer.deposited_amount, price);
        msg!(
            "USDC -> SOL | {} USDC ≈ {} lamports",
            offer.deposited_amount,
            lamports_amount
        );

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.taker_token_out_ata.to_account_info(),
            to: ctx.accounts.depositor_receive_ata.to_account_info(),
            authority: taker.to_account_info(),
            mint: ctx.accounts.token_mint_out.to_account_info(),
        };

        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, lamports_amount, 9)?;
    } else {
        // depositor deposited SOL, taker sends USDC
        let usdc_amount = lamports_to_usdc(offer.deposited_amount, price);
        msg!(
            "SOL -> USDC | {} lamports ≈ {} USDC",
            offer.deposited_amount,
            usdc_amount
        );

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.taker_token_out_ata.to_account_info(),
            to: ctx.accounts.depositor_receive_ata.to_account_info(),
            authority: taker.to_account_info(),
            mint: ctx.accounts.token_mint_out.to_account_info(),
        };

        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, usdc_amount, 6)?;
    }

    // Transfer the offered tokens from the vault to the taker
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.taker_token_in_ata.to_account_info(),
        authority: ctx.accounts.offer.to_account_info(),
        mint: ctx.accounts.token_mint_in.to_account_info(),
    };

    let cpi_program = token_program.to_account_info();
    let token_mint_in = ctx.accounts.token_mint_in.key();
    let depositor = ctx.accounts.depositor.key();

    let offer_seeds = &[
        token_mint_in.as_ref(),
        depositor.as_ref(),
        &[ctx.accounts.offer.bump],
    ];
    let signer_seeds = &[&offer_seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    transfer_checked(
        cpi_ctx,
        offer.deposited_amount,
        ctx.accounts.token_mint_in.decimals,
    )?;

    Ok(())
}
