use crate::error::ErrorCode;
use crate::state::merchant::Merchant;
use crate::SystemConfig;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

/// 商户缴纳保证金
#[derive(Accounts)]
pub struct DepositMerchantDeposit<'info> {
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
        seeds = [b"deposit_escrow"],
        bump,
        token::mint = deposit_token_mint,
        token::authority = deposit_escrow_account,
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// 商户追加保证金
pub fn deposit_merchant_deposit(ctx: Context<DepositMerchantDeposit>, amount: u64) -> Result<()> {
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

    msg!(
        "商户 {} 成功缴纳保证金 {} tokens，当前保证金余额: {}",
        merchant.owner,
        amount,
        merchant.deposit_amount
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

    /// 商户所有者公钥（用于PDA计算和权限验证）
    /// CHECK: 这是商户的公钥，用于PDA计算
    pub merchant_owner: UncheckedAccount<'info>,

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

    // 系统保证金托管账户
    #[account(
        mut,
        seeds = [b"deposit_escrow"],
        bump,
        constraint = deposit_escrow_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// 商户提取保证金（支持商户和管理员双重权限）
pub fn withdraw_merchant_deposit(ctx: Context<WithdrawMerchantDeposit>, amount: u64) -> Result<()> {
    let merchant = &mut ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;
    let signer = &ctx.accounts.signer;

    // 验证提取金额
    require!(amount > 0, ErrorCode::InvalidDepositAmount);

    // 验证商户可用保证金余额
    require!(
        merchant.get_available_deposit() >= amount,
        ErrorCode::InsufficientDeposit
    );

    // 权限验证：商户本人或系统管理员
    let is_merchant_owner = merchant.owner == signer.key();
    let is_system_admin = system_config.authority == signer.key(); // 假设系统配置有authority字段

    require!(
        is_merchant_owner || is_system_admin,
        ErrorCode::Unauthorized
    );

    // 如果是商户提取，需要检查最低保证金限制
    // 如果是管理员提取，可以提取任意金额（用于违规处罚等）
    if is_merchant_owner {
        let remaining_deposit = merchant.get_available_deposit().saturating_sub(amount);
        require!(
            remaining_deposit >= system_config.get_deposit_requirement(),
            ErrorCode::MerchantDepositInsufficient
        );

        msg!(
            "商户 {} 提取保证金，提取后剩余: {} tokens，最低要求: {} tokens",
            merchant.owner,
            remaining_deposit,
            system_config.get_deposit_requirement()
        );
    } else {
        msg!(
            "管理员 {} 提取商户 {} 的保证金 {} tokens（管理操作）",
            signer.key(),
            merchant.owner,
            amount
        );
    }

    // 验证托管账户余额
    require!(
        ctx.accounts.deposit_escrow_account.amount >= amount,
        ErrorCode::InsufficientFunds
    );

    // 执行代币转账从系统托管账户到接收账户
    let deposit_escrow_bump = ctx.bumps.deposit_escrow_account;
    let seeds = &[b"deposit_escrow".as_ref(), &[deposit_escrow_bump]];
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
        signer.key()
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
}

/// 查询商户保证金信息
pub fn get_merchant_deposit_info(
    ctx: Context<GetMerchantDepositInfo>,
) -> Result<MerchantDepositInfo> {
    let merchant = &ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;

    Ok(MerchantDepositInfo {
        total_deposit: merchant.deposit_amount,
        locked_deposit: merchant.deposit_locked,
        available_deposit: merchant.get_available_deposit(),
        required_deposit: system_config.get_deposit_requirement(),
        is_sufficient: merchant.has_sufficient_deposit(system_config.get_deposit_requirement()),
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
