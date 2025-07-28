use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

/// 初始化支付系统配置
#[derive(Accounts)]
pub struct InitializePaymentSystem<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PaymentConfig::INIT_SPACE,
        seeds = [b"payment_config"],
        bump
    )]
    pub payment_config: Account<'info, PaymentConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_payment_system(
    ctx: Context<InitializePaymentSystem>,
    supported_tokens: Vec<SupportedToken>,
    fee_rate: u16,
    fee_recipient: Pubkey,
) -> Result<()> {
    let payment_config = &mut ctx.accounts.payment_config;
    let bump = ctx.bumps.payment_config;

    payment_config.initialize(
        ctx.accounts.authority.key(),
        supported_tokens,
        fee_rate,
        fee_recipient,
        bump,
    )?;

    msg!("支付系统初始化成功");
    Ok(())
}

/// 更新支付系统配置
#[derive(Accounts)]
pub struct UpdatePaymentConfig<'info> {
    #[account(
        mut,
        seeds = [b"payment_config"],
        bump,
        constraint = payment_config.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub payment_config: Account<'info, PaymentConfig>,

    pub authority: Signer<'info>,
}

pub fn update_supported_tokens(
    ctx: Context<UpdatePaymentConfig>,
    supported_tokens: Vec<SupportedToken>,
) -> Result<()> {
    let payment_config = &mut ctx.accounts.payment_config;
    payment_config.update_tokens(supported_tokens)?;

    msg!("支持的代币列表已更新");
    Ok(())
}

pub fn update_fee_rate(ctx: Context<UpdatePaymentConfig>, fee_rate: u16) -> Result<()> {
    let payment_config = &mut ctx.accounts.payment_config;
    payment_config.update_fee_rate(fee_rate)?;

    msg!("手续费率已更新为: {}基点", fee_rate);
    Ok(())
}

/// 关闭支付系统配置
#[derive(Accounts)]
pub struct ClosePaymentConfig<'info> {
    #[account(
        mut,
        close = beneficiary,
        seeds = [b"payment_config"],
        bump,
        constraint = payment_config.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub payment_config: Account<'info, PaymentConfig>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,

    pub authority: Signer<'info>,
}

pub fn close_payment_config(ctx: Context<ClosePaymentConfig>, force: bool) -> Result<()> {
    let payment_config = &ctx.accounts.payment_config;

    // 检查是否还有活跃代币（除非强制删除）
    if !force {
        let active_tokens = payment_config
            .supported_tokens
            .iter()
            .filter(|token| token.is_active)
            .count();

        require!(active_tokens == 0, ErrorCode::TooManyTokens);
    }

    msg!(
        "支付配置账户已关闭，权限者: {}, 强制删除: {}",
        payment_config.authority,
        force
    );

    // 账户将通过close约束自动关闭并回收租金
    Ok(())
}

/// 简化的购买商品指令
#[derive(Accounts)]
#[instruction(product_id: u64, amount: u64)]
pub struct PurchaseProductEscrow<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    // 产品账户 - 验证产品存在和价格
    #[account(
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump
    )]
    pub product: Account<'info, ProductBase>,

    // 主程序统一托管代币账户
    #[account(
        init_if_needed,
        payer = buyer,
        token::mint = payment_token_mint,
        token::authority = program_authority,
        seeds = [b"program_token_account"],
        bump
    )]
    pub program_token_account: Account<'info, TokenAccount>,

    /// CHECK: 程序权限账户，用于控制Token转账
    #[account(
        seeds = [b"program_authority"],
        bump
    )]
    pub program_authority: AccountInfo<'info>,

    // 买家代币账户
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    // 支付代币mint
    pub payment_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn purchase_product_escrow(
    ctx: Context<PurchaseProductEscrow>,
    product_id: u64,
    amount: u64,
) -> Result<()> {
    let product = &ctx.accounts.product;

    // 验证商品是否激活
    require!(product.is_active, ErrorCode::InvalidProduct);

    // 验证产品ID匹配
    require!(product.id == product_id, ErrorCode::InvalidProduct);

    // 验证购买数量
    require!(amount > 0, ErrorCode::InvalidAmount);

    // 计算总价格
    let total_price = product
        .price
        .checked_mul(amount)
        .ok_or(ErrorCode::IntegerOverflow)?;

    // 将买家的代币转入主程序统一托管账户
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer_token_account.to_account_info(),
                to: ctx.accounts.program_token_account.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        total_price,
    )?;

    msg!(
        "购买成功: 买家: {}, 产品ID: {}, 数量: {}, 总价: {} tokens",
        ctx.accounts.buyer.key(),
        product_id,
        amount,
        total_price
    );

    Ok(())
}

/// 初始化程序Token账户
#[derive(Accounts)]
pub struct InitializeProgramTokenAccount<'info> {
    #[account(
        init,
        payer = authority,
        token::mint = payment_token_mint,
        token::authority = program_authority,
        seeds = [b"program_token_account"],
        bump
    )]
    pub program_token_account: Account<'info, TokenAccount>,

    /// CHECK: 程序权限账户，用于控制Token转账
    #[account(
        seeds = [b"program_authority"],
        bump
    )]
    pub program_authority: UncheckedAccount<'info>,

    pub payment_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_program_token_account(ctx: Context<InitializeProgramTokenAccount>) -> Result<()> {
    msg!(
        "程序Token账户初始化成功: {}, Token Mint: {}, 权限账户: {}",
        ctx.accounts.program_token_account.key(),
        ctx.accounts.payment_token_mint.key(),
        ctx.accounts.program_authority.key()
    );

    Ok(())
}
