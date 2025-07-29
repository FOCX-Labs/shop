use crate::error::ErrorCode;
use crate::state::merchant::Merchant;
use crate::SystemConfig;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

/// 商户缴纳/补充保证金（统一指令）
#[derive(Accounts)]
pub struct ManageDeposit<'info> {
    #[account(mut)]
    pub merchant_owner: Signer<'info>,

    // 商户信息账户
    #[account(
        mut,
        seeds = [b"merchant_info", merchant_owner.key().as_ref()],
        bump,
        constraint = merchant.owner == merchant_owner.key() @ ErrorCode::InvalidMerchant
    )]
    pub merchant: Account<'info, Merchant>,

    // 系统配置账户
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, SystemConfig>,

    // 商户的代币账户（支付保证金）
    #[account(
        mut,
        constraint = merchant_token_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub merchant_token_account: Account<'info, TokenAccount>,

    // 保证金代币mint账户
    #[account(
        constraint = deposit_token_mint.key() == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_token_mint: Account<'info, Mint>,

    // 系统保证金托管账户
    #[account(
        init_if_needed,
        payer = merchant_owner,
        seeds = [b"deposit_escrow", deposit_token_mint.key().as_ref()],
        bump,
        token::mint = deposit_token_mint,
        token::authority = deposit_escrow_account,
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// 商户缴纳/补充保证金（统一处理）
pub fn manage_deposit(ctx: Context<ManageDeposit>, amount: u64) -> Result<()> {
    let merchant = &mut ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;

    // 验证保证金金额
    require!(amount > 0, ErrorCode::InvalidDepositAmount);

    // 验证商户代币账户余额
    require!(
        ctx.accounts.merchant_token_account.amount >= amount,
        ErrorCode::InsufficientFunds
    );

    // 验证保证金代币类型
    require!(
        merchant.is_valid_deposit_token(&system_config.deposit_token_mint),
        ErrorCode::InvalidDepositToken
    );

    // 记录操作前的保证金状态
    let old_deposit = merchant.deposit_amount;
    let is_initial_deposit = old_deposit == 0;

    // 执行代币转账到系统托管账户
    let cpi_accounts = Transfer {
        from: ctx.accounts.merchant_token_account.to_account_info(),
        to: ctx.accounts.deposit_escrow_account.to_account_info(),
        authority: ctx.accounts.merchant_owner.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, amount)?;

    // 更新商户保证金余额
    merchant.add_deposit(amount)?;

    // 根据操作类型输出不同的日志
    if is_initial_deposit {
        msg!(
            "商户 {} 首次缴纳保证金 {} tokens，当前保证金余额: {}",
            merchant.owner,
            amount,
            merchant.deposit_amount
        );
    } else {
        msg!(
            "商户 {} 补充保证金 {} tokens，保证金余额: {} -> {}",
            merchant.owner,
            amount,
            old_deposit,
            merchant.deposit_amount
        );
    }

    // 强制验证是否满足最低保证金要求
    // 获取Token精度
    let token_decimals = ctx.accounts.deposit_token_mint.decimals;
    let required_deposit = system_config.get_deposit_requirement(token_decimals);
    require!(
        merchant.deposit_amount >= required_deposit,
        ErrorCode::MerchantDepositInsufficient
    );

    msg!(
        "商户保证金验证通过: {} >= {} (要求), Token精度: {}",
        merchant.deposit_amount,
        required_deposit,
        token_decimals
    );

    Ok(())
}

/// 商户提取保证金（支持商户和管理员双重权限）
#[derive(Accounts)]
pub struct WithdrawMerchantDeposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // 商户信息账户
    #[account(
        mut,
        seeds = [b"merchant_info", merchant_owner.key().as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,

    /// 商户所有者（签名者）
    pub merchant_owner: Signer<'info>,

    // 系统配置账户
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, SystemConfig>,

    // 接收提取保证金的代币账户
    #[account(
        mut,
        constraint = recipient_token_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    // 保证金Token mint账户（用于获取精度）
    #[account(
        constraint = deposit_token_mint.key() == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_token_mint: Account<'info, Mint>,

    // 系统保证金托管账户
    #[account(
        mut,
        seeds = [b"deposit_escrow", deposit_token_mint.key().as_ref()],
        bump,
        constraint = deposit_escrow_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// 商户提取保证金（仅限商户本人操作）
pub fn withdraw_merchant_deposit(ctx: Context<WithdrawMerchantDeposit>, amount: u64) -> Result<()> {
    let merchant = &mut ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;
    let merchant_owner = &ctx.accounts.merchant_owner;

    // 验证提取金额
    require!(amount > 0, ErrorCode::InvalidDepositAmount);

    // 验证商户可用保证金余额
    require!(
        merchant.get_available_deposit() >= amount,
        ErrorCode::InsufficientDeposit
    );

    // 权限验证：仅限商户本人
    require!(
        merchant.owner == merchant_owner.key(),
        ErrorCode::Unauthorized
    );

    // 检查提取后是否满足最低保证金限制
    let remaining_deposit = merchant.get_available_deposit().saturating_sub(amount);
    // 获取Token精度
    let token_decimals = ctx.accounts.deposit_token_mint.decimals;
    let required_deposit = system_config.get_deposit_requirement(token_decimals);
    require!(
        remaining_deposit >= required_deposit,
        ErrorCode::MerchantDepositInsufficient
    );

    msg!(
        "商户 {} 提取保证金，提取后剩余: {} tokens，最低要求: {} tokens，Token精度: {}",
        merchant.owner,
        remaining_deposit,
        required_deposit,
        token_decimals
    );

    // 验证托管账户余额
    require!(
        ctx.accounts.deposit_escrow_account.amount >= amount,
        ErrorCode::InsufficientFunds
    );

    // 执行代币转账从系统托管账户到接收账户
    let deposit_escrow_bump = ctx.bumps.deposit_escrow_account;
    let token_mint_key = ctx.accounts.deposit_token_mint.key();
    let seeds = &[
        b"deposit_escrow".as_ref(),
        token_mint_key.as_ref(),
        &[deposit_escrow_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.deposit_escrow_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.deposit_escrow_account.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    token::transfer(cpi_ctx, amount)?;

    // 更新商户保证金余额
    merchant.deduct_deposit(amount)?;

    msg!(
        "保证金提取成功: 商户 {}, 提取金额: {} tokens, 当前保证金余额: {} tokens, 操作者: {}",
        merchant.owner,
        amount,
        merchant.deposit_amount,
        merchant_owner.key()
    );

    Ok(())
}

/// 查询商户保证金信息
#[derive(Accounts)]
pub struct GetMerchantDepositInfo<'info> {
    // 商户信息账户
    #[account(
        seeds = [b"merchant_info", merchant_owner.key().as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,

    pub merchant_owner: Signer<'info>,

    // 系统配置账户
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, SystemConfig>,

    // 保证金Token mint账户（用于获取精度）
    #[account(
        constraint = deposit_token_mint.key() == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_token_mint: Account<'info, Mint>,
}

/// 查询商户保证金信息
pub fn get_merchant_deposit_info(
    ctx: Context<GetMerchantDepositInfo>,
) -> Result<MerchantDepositInfo> {
    let merchant = &ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;
    let token_decimals = ctx.accounts.deposit_token_mint.decimals;
    let required_deposit = system_config.get_deposit_requirement(token_decimals);

    Ok(MerchantDepositInfo {
        total_deposit: merchant.deposit_amount,
        locked_deposit: merchant.deposit_locked,
        available_deposit: merchant.get_available_deposit(),
        required_deposit,
        is_sufficient: merchant.deposit_amount >= required_deposit,
        deposit_token_mint: merchant.deposit_token_mint,
        last_updated: merchant.deposit_updated_at,
    })
}

/// 商户保证金信息结构
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MerchantDepositInfo {
    pub total_deposit: u64,         // 总保证金
    pub locked_deposit: u64,        // 锁定保证金
    pub available_deposit: u64,     // 可用保证金
    pub required_deposit: u64,      // 要求保证金
    pub is_sufficient: bool,        // 是否满足要求
    pub deposit_token_mint: Pubkey, // 保证金代币mint
    pub last_updated: i64,          // 最后更新时间
}

/// 系统管理员更新保证金要求
#[derive(Accounts)]
pub struct UpdateDepositRequirement<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    // 系统配置账户
    #[account(
        mut,
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, SystemConfig>,
}

/// 更新保证金要求
pub fn update_deposit_requirement(
    ctx: Context<UpdateDepositRequirement>,
    new_requirement: u64,
) -> Result<()> {
    let system_config = &mut ctx.accounts.system_config;

    require!(new_requirement > 0, ErrorCode::InvalidDepositAmount);

    let old_requirement = system_config.merchant_deposit_required;
    system_config.merchant_deposit_required = new_requirement;

    msg!(
        "系统保证金要求已更新: {} -> {} tokens",
        old_requirement,
        new_requirement
    );

    Ok(())
}

/// 管理员扣除商户保证金（用于违规处罚等）
#[derive(Accounts)]
pub struct DeductMerchantDeposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    // 商户信息账户
    #[account(
        mut,
        seeds = [b"merchant_info", merchant_owner.key().as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,

    /// 商户所有者公钥（用于PDA计算）
    /// CHECK: 这是商户的公钥，用于PDA计算
    pub merchant_owner: UncheckedAccount<'info>,

    // 系统配置账户
    #[account(
        seeds = [b"system_config"],
        bump,
        constraint = system_config.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub system_config: Account<'info, SystemConfig>,

    // 保证金Token mint账户（用于获取精度）
    #[account(
        constraint = deposit_token_mint.key() == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_token_mint: Account<'info, Mint>,

    // 系统保证金托管账户
    #[account(
        mut,
        seeds = [b"deposit_escrow", deposit_token_mint.key().as_ref()],
        bump,
        constraint = deposit_escrow_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    // 管理员接收扣除保证金的代币账户
    #[account(
        mut,
        constraint = admin_token_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub admin_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// 管理员扣除商户保证金
pub fn deduct_merchant_deposit(
    ctx: Context<DeductMerchantDeposit>,
    amount: u64,
    reason: String,
) -> Result<()> {
    let merchant = &mut ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;

    // 验证扣除金额
    require!(amount > 0, ErrorCode::InvalidDepositAmount);
    require!(!reason.is_empty(), ErrorCode::InvalidDepositAmount);

    // 验证商户保证金余额
    require!(
        merchant.deposit_amount >= amount,
        ErrorCode::InsufficientDeposit
    );

    // 验证托管账户余额
    require!(
        ctx.accounts.deposit_escrow_account.amount >= amount,
        ErrorCode::InsufficientFunds
    );

    // 执行代币转账从系统托管账户到管理员账户
    let deposit_escrow_bump = ctx.bumps.deposit_escrow_account;
    let seeds = &[b"deposit_escrow".as_ref(), &[deposit_escrow_bump]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.deposit_escrow_account.to_account_info(),
        to: ctx.accounts.admin_token_account.to_account_info(),
        authority: ctx.accounts.deposit_escrow_account.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    token::transfer(cpi_ctx, amount)?;

    // 更新商户保证金余额
    merchant.deduct_deposit(amount)?;

    msg!(
        "管理员 {} 扣除商户 {} 保证金 {} tokens，原因: {}，剩余保证金: {}",
        ctx.accounts.authority.key(),
        merchant.owner,
        amount,
        reason,
        merchant.deposit_amount
    );

    // 检查保证金是否低于要求
    let token_decimals = ctx.accounts.deposit_token_mint.decimals;
    let required_deposit = system_config.get_deposit_requirement(token_decimals);
    if merchant.deposit_amount < required_deposit {
        msg!(
            "警告：商户保证金不足，当前: {} < 要求: {}，Token精度: {}",
            merchant.deposit_amount,
            required_deposit,
            token_decimals
        );
    }

    Ok(())
}
