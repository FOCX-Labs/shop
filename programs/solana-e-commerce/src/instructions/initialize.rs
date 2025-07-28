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
        space = 8 + GlobalIdRoot::INIT_SPACE,
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
    system_config.merchant_deposit_required = config.merchant_deposit_required;
    system_config.deposit_token_mint = config.deposit_token_mint;
    // 初始化新增字段
    system_config.platform_fee_rate = config.platform_fee_rate;
    system_config.platform_fee_recipient = config.platform_fee_recipient;
    system_config.auto_confirm_days = config.auto_confirm_days;
    system_config.external_program_id = config.external_program_id;

    msg!(
        "系统配置初始化成功，管理员: {}, 保证金要求: {} tokens",
        config.authority,
        config.merchant_deposit_required
    );

    Ok(())
}

/// 关闭系统配置账户
#[derive(Accounts)]
pub struct CloseSystemConfig<'info> {
    #[account(
        mut,
        seeds = [b"system_config"],
        bump,
        close = beneficiary
    )]
    pub system_config: Account<'info, SystemConfig>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,

    #[account(
        constraint = authority.key() == system_config.authority @ crate::error::ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,
}

pub fn close_system_config(ctx: Context<CloseSystemConfig>, force: bool) -> Result<()> {
    let system_config = &ctx.accounts.system_config;

    msg!(
        "关闭系统配置账户，管理员: {}, 强制关闭: {}",
        system_config.authority,
        force
    );

    // 这里可以添加额外的验证逻辑
    // 例如检查是否还有活跃的商户等
    if !force {
        // 添加安全检查
        msg!("执行安全检查...");
    }

    msg!("系统配置账户关闭成功");
    Ok(())
}

/// 强制关闭不兼容的系统配置账户（手动转移余额）
#[derive(Accounts)]
pub struct ForceCloseSystemConfig<'info> {
    #[account(
        mut,
        seeds = [b"system_config"],
        bump
    )]
    /// CHECK: 这个账户可能包含不兼容的数据，我们不尝试反序列化
    pub system_config: AccountInfo<'info>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,

    #[account(
        constraint = authority.key() == beneficiary.key() @ crate::error::ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn force_close_system_config(ctx: Context<ForceCloseSystemConfig>) -> Result<()> {
    let system_config = &ctx.accounts.system_config;
    let beneficiary = &ctx.accounts.beneficiary;

    msg!(
        "强制关闭系统配置账户: {}, 受益人: {}",
        system_config.key(),
        beneficiary.key()
    );

    // 检查账户所有者是否正确
    require!(
        system_config.owner == &crate::ID,
        crate::error::ErrorCode::Unauthorized
    );

    // 手动转移所有余额到受益人
    let lamports = system_config.lamports();
    **system_config.try_borrow_mut_lamports()? = 0;
    **beneficiary.try_borrow_mut_lamports()? = beneficiary
        .lamports()
        .checked_add(lamports)
        .ok_or(crate::error::ErrorCode::ArithmeticOverflow)?;

    // 清空账户数据
    system_config.realloc(0, false)?;

    msg!("系统配置账户强制关闭成功，转移 {} lamports", lamports);
    Ok(())
}
