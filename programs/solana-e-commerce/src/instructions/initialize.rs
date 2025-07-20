use crate::state::*;
use crate::SystemConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeSystem<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = GlobalIdRoot::LEN,
        seeds = [b"global_id_root"],
        bump
    )]
    pub global_root: Account<'info, GlobalIdRoot>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_system(ctx: Context<InitializeSystem>, config: SystemConfig) -> Result<()> {
    let global_root = &mut ctx.accounts.global_root;

    global_root.last_merchant_id = 0;
    global_root.last_global_id = 0;
    global_root.chunk_size = config.chunk_size;
    global_root.merchants = Vec::new();
    global_root.max_products_per_shard = config.max_products_per_shard;
    global_root.max_keywords_per_product = config.max_keywords_per_product;
    global_root.bloom_filter_size = config.bloom_filter_size;
    global_root.cache_ttl = config.cache_ttl;
    global_root.bump = ctx.bumps.global_root;

    msg!("系统初始化成功，块大小: {}", config.chunk_size);

    Ok(())
}

/// 初始化系统配置账户
#[derive(Accounts)]
pub struct InitializeSystemConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<SystemConfig>(),
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, SystemConfig>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_system_config(
    ctx: Context<InitializeSystemConfig>,
    config: SystemConfig,
) -> Result<()> {
    let system_config = &mut ctx.accounts.system_config;

    system_config.authority = config.authority;
    system_config.max_products_per_shard = config.max_products_per_shard;
    system_config.max_keywords_per_product = config.max_keywords_per_product;
    system_config.chunk_size = config.chunk_size;
    system_config.bloom_filter_size = config.bloom_filter_size;
    system_config.cache_ttl = config.cache_ttl;
    system_config.merchant_deposit_required = config.merchant_deposit_required;
    system_config.deposit_token_mint = config.deposit_token_mint;
    system_config.deposit_token_decimals = config.deposit_token_decimals;

    msg!(
        "系统配置初始化成功，管理员: {}, 保证金要求: {} tokens",
        config.authority,
        config.merchant_deposit_required
    );

    Ok(())
}
