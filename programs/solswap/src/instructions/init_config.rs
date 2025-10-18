use crate::Config;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space  = 8 + Config::INIT_SPACE,
        seeds = [b"config"], bump
        )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

pub fn process_init_config(ctx: Context<InitConfig>, usdc_mint: Pubkey) -> Result<()> {
    ctx.accounts.config.usdc_mint = usdc_mint;
    Ok(())
}
