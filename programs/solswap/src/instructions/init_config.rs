use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::program::Solswap;
use crate::{Config, CONFIG_SEED, MAX_ALLOWED_PRICE_AGE_SECS};

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + Config::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, Config>,

    /// Only the upgrade authority may create the config, so nobody can front-run deployment.
    #[account(constraint = program.programdata_address()? == Some(program_data.key()))]
    pub program: Program<'info, Solswap>,

    #[account(constraint = program_data.upgrade_authority_address == Some(admin.key()) @ ErrorCode::Unauthorized)]
    pub program_data: Account<'info, ProgramData>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    pub admin: Signer<'info>,

    #[account(mut, seeds = [CONFIG_SEED], bump = config.bump, has_one = admin @ ErrorCode::Unauthorized)]
    pub config: Account<'info, Config>,
}

fn validate_max_age(max_price_age_secs: u32) -> Result<()> {
    require!(
        (1..=MAX_ALLOWED_PRICE_AGE_SECS).contains(&max_price_age_secs),
        ErrorCode::InvalidMaxPriceAge
    );
    Ok(())
}

pub fn process_init_config(
    ctx: Context<InitConfig>,
    usdc_mint: Pubkey,
    price_feed: Pubkey,
    max_price_age_secs: u32,
) -> Result<()> {
    validate_max_age(max_price_age_secs)?;

    ctx.accounts.config.set_inner(Config {
        admin: ctx.accounts.admin.key(),
        usdc_mint,
        price_feed,
        max_price_age_secs,
        bump: ctx.bumps.config,
    });

    Ok(())
}

pub fn process_update_config(
    ctx: Context<UpdateConfig>,
    price_feed: Pubkey,
    max_price_age_secs: u32,
) -> Result<()> {
    validate_max_age(max_price_age_secs)?;

    let config = &mut ctx.accounts.config;
    config.price_feed = price_feed;
    config.max_price_age_secs = max_price_age_secs;

    Ok(())
}
