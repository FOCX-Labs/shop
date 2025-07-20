use anchor_lang::solana_program::pubkey::Pubkey;
use std::str::FromStr;

fn main() {
    // 替换为你的 seeds
    let seed1 = b"prefix_index";
    let seed2 = "test"; // 只使用前缀，和 TypeScript 端保持一致

    println!("Rust seed1 bytes: {:?}", seed1);
    println!("Rust seed2 string: {}", seed2);
    println!("Rust seed2 bytes: {:?}", seed2.as_bytes());

    // 替换为你的程序ID
    let program_id = Pubkey::from_str("XNHBi5iSC9AL23JUoGGJJsPWyRL7drM55bLEy214KPP")
        .expect("Invalid program ID");

    println!("Rust program ID: {}", program_id);

    // PDA计算 - 使用字符串的字节表示
    let seeds = &[seed1, seed2.as_bytes()];
    println!("Rust seeds: {:?}", seeds);

    let (pda, bump) = Pubkey::find_program_address(seeds, &program_id);

    println!("PDA: {}", pda);
    println!("Bump: {}", bump);
}
