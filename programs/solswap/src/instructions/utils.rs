const SCALE: u128 = 100_000_000_000;

pub fn usdc_to_lamports(usdc_base: u64, price: i64) -> u64 {
    assert!(price > 0);
    let num = (usdc_base as u128) * SCALE;
    let den = price as u128;
    (num.div_ceil(den)) as u64
}

pub fn lamports_to_usdc(lamports: u64, price: i64) -> u64 {
    assert!(price > 0);
    let num = (lamports as u128) * (price as u128);
    (num / SCALE) as u64 // floor
}
