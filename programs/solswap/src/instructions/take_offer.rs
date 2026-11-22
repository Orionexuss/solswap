use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::error::ErrorCode;
use crate::{
    lamports_to_usdc, read_fresh_price, release_vault, usdc_to_lamports, Config, Offer, OfferTaken,
    CONFIG_SEED, OFFER_SEED, WSOL_MINT,
};

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    /// Receives the payment and, when the offer closes, its rent.
    #[account(mut)]
    pub depositor: SystemAccount<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

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
        close = depositor,
        has_one = depositor,
        has_one = token_mint_in,
        has_one = token_mint_out,
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

    /// CHECK: must be the configured Chainlink feed; owner and layout are validated by
    /// `chainlink_solana::v2::read_feed_v2`.
    #[account(address = config.price_feed @ ErrorCode::InvalidPriceFeed)]
    pub price_feed: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

/// `max_amount_out`: the most the taker accepts to pay, in `token_mint_out` base units.
pub fn process_take_offer(ctx: Context<TakeOffer>, max_amount_out: u64) -> Result<()> {
    let accounts = &ctx.accounts;
    let oracle = read_fresh_price(
        &accounts.price_feed.to_account_info(),
        accounts.config.max_price_age_secs,
        Clock::get()?.unix_timestamp,
    )?;

    let deposited = accounts.offer.deposited_amount;
    let mint_in = &accounts.token_mint_in;
    let mint_out = &accounts.token_mint_out;

    let amount_out = if mint_in.key() == WSOL_MINT {
        // Vault holds WSOL; taker pays USDC.
        lamports_to_usdc(deposited, oracle.price, oracle.decimals, mint_out.decimals, mint_in.decimals)?
    } else {
        // Vault holds USDC; taker pays WSOL.
        usdc_to_lamports(deposited, oracle.price, oracle.decimals, mint_in.decimals, mint_out.decimals)?
    };
    require!(amount_out <= max_amount_out, ErrorCode::SlippageExceeded);

    transfer_checked(
        CpiContext::new(
            accounts.token_program.to_account_info(),
            TransferChecked {
                from: accounts.taker_token_out_ata.to_account_info(),
                to: accounts.depositor_receive_ata.to_account_info(),
                authority: accounts.taker.to_account_info(),
                mint: mint_out.to_account_info(),
            },
        ),
        amount_out,
        mint_out.decimals,
    )?;

    release_vault(
        &accounts.offer,
        &accounts.vault,
        mint_in,
        accounts.taker_token_in_ata.to_account_info(),
        accounts.depositor.to_account_info(),
        &accounts.token_program,
    )?;

    emit!(OfferTaken {
        offer: accounts.offer.key(),
        taker: accounts.taker.key(),
        amount_received: deposited,
        amount_paid: amount_out,
        price: oracle.price as i128,
        price_decimals: oracle.decimals,
        price_timestamp: oracle.timestamp,
    });

    Ok(())
}
