use anchor_client::{
    solana_client::rpc_client::RpcClient, solana_sdk::commitment_config::CommitmentConfig,
};
use anchor_lang::prelude::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;

// Helper function to print balances in a table format
pub fn print_balances(
    depositor: &Pubkey,
    taker: Option<&Pubkey>,
    token_mint_in: &Pubkey,
    token_mint_out: &Pubkey,
    token_program_id: &Pubkey,
    title: &str,
) {
    println!("\n=== {} ===", title);
    println!(
        "{:<20} | {:<44} | {:<12}",
        "Account", "Associated Token Address", "Balance"
    );
    println!("{}", "-".repeat(80));
    print_spl_balance(depositor, token_mint_in, token_program_id, "Depositor WSOL");
    print_spl_balance(
        depositor,
        token_mint_out,
        token_program_id,
        "Depositor USDC",
    );
    if let Some(taker) = taker {
        print_spl_balance(taker, token_mint_in, token_program_id, "Taker WSOL");
        print_spl_balance(taker, token_mint_out, token_program_id, "Taker USDC");
    }
}

// Format a number with thousands separators and fixed decimals
fn format_human_amount(amount: f64, symbol: &str) -> String {
    let formatted = format!("{:.2}", amount);
    let mut parts = formatted.split('.');
    let integer = parts.next().unwrap_or("0");
    let decimal = parts.next().unwrap_or("00");

    let chars: Vec<char> = integer.chars().rev().collect();
    let mut formatted_int = String::new();
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            formatted_int.push(',');
        }
        formatted_int.push(*ch);
    }
    format!(
        "{}.{} {}",
        formatted_int.chars().rev().collect::<String>(),
        decimal,
        symbol
    )
}

// Print SPL token balance for a given owner and mint
pub fn print_spl_balance(owner: &Pubkey, mint: &Pubkey, token_program_id: &Pubkey, label: &str) {
    let connection = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    let ata = get_associated_token_address_with_program_id(owner, mint, token_program_id);

    let balance_str = match connection.get_token_account_balance(&ata) {
        Ok(balance) => {
            let raw_amount = balance.amount.parse::<u64>().unwrap_or(0);
            let (decimals, symbol) =
                if mint.to_string() == "So11111111111111111111111111111111111111112" {
                    (9, "SOL")
                } else {
                    (6, "USDC")
                };
            format_human_amount(raw_amount as f64 / 10f64.powi(decimals), symbol)
        }
        Err(_) => format!(
            "0.00 {}",
            if mint.to_string() == "So11111111111111111111111111111111111111112" {
                "SOL"
            } else {
                "USDC"
            }
        ),
    };

    println!("{:<20} | {} | {}", label, ata, balance_str);
}
