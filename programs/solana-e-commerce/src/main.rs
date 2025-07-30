use anchor_lang::solana_program::pubkey::Pubkey;
use std::str::FromStr;

fn main() {
    // Replace with your seeds
    let seed1 = b"prefix_index";
    let seed2 = "test"; // Only use prefix, keep consistent with TypeScript side

    println!("Rust seed1 bytes: {:?}", seed1);
    println!("Rust seed2 string: {}", seed2);
    println!("Rust seed2 bytes: {:?}", seed2.as_bytes());

    // Replace with your program ID
    let program_id = Pubkey::from_str("XNHBi5iSC9AL23JUoGGJJsPWyRL7drM55bLEy214KPP")
        .expect("Invalid program ID");

    println!("Rust program ID: {}", program_id);

    // PDA calculation - use byte representation of string
    let seeds = &[seed1, seed2.as_bytes()];
    println!("Rust seeds: {:?}", seeds);

    let (pda, bump) = Pubkey::find_program_address(seeds, &program_id);

    println!("PDA: {}", pda);
    println!("Bump: {}", bump);
}
