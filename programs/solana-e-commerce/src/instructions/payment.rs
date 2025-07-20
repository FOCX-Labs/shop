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
        space = PaymentConfig::LEN,
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





/// 托管购买商品（新的托管支付系统）- 集成订单创建
#[derive(Accounts)]
#[instruction(product_id: u64, amount: u64, timestamp: i64)]
pub struct PurchaseProductEscrow<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump
    )]
    pub product: Account<'info, Product>,

    #[account(
        seeds = [b"payment_config"],
        bump
    )]
    pub payment_config: Account<'info, PaymentConfig>,

    // 托管账户
    #[account(
        init,
        payer = buyer,
        space = EscrowAccount::LEN,
        seeds = [b"escrow", buyer.key().as_ref(), product_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    // 订单账户 - 新增（使用复合种子）
    #[account(
        init,
        payer = buyer,
        space = Order::LEN,
        seeds = [
            b"order",
            buyer.key().as_ref(),
            product.merchant.as_ref(),
            product_id.to_le_bytes().as_ref(),
            timestamp.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub order: Account<'info, Order>,

    // 订单统计账户 - 新增
    #[account(
        mut,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    // 商户信息账户 - 新增
    #[account(
        seeds = [b"merchant_info", product.merchant.as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,

    // 系统配置账户 - 用于保证金验证
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, crate::SystemConfig>,

    // 托管代币账户（程序控制）
    #[account(
        init,
        payer = buyer,
        token::mint = payment_token_mint,
        token::authority = escrow_account,
        seeds = [b"escrow_token", buyer.key().as_ref(), product_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    // 买家代币账户
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    // 支付代币mint
    pub payment_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn purchase_product_escrow(
    ctx: Context<PurchaseProductEscrow>,
    product_id: u64,
    amount: u64,
    timestamp: i64,
    shipping_address: String,
    notes: String,
) -> Result<()> {
    let product = &ctx.accounts.product;
    let payment_config = &ctx.accounts.payment_config;
    let escrow_account = &mut ctx.accounts.escrow_account;
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;
    let merchant = &ctx.accounts.merchant;

    // 验证商品是否激活
    require!(product.is_active, ErrorCode::InvalidProduct);

    // 验证产品ID匹配
    require!(product.id == product_id, ErrorCode::InvalidProduct);

    // 验证商户匹配
    require!(
        product.merchant == merchant.owner,
        ErrorCode::InvalidMerchant
    );

    // 验证商户保证金是否满足要求
    let system_config = &ctx.accounts.system_config;
    let required_deposit = system_config.get_deposit_requirement();
    require!(
        merchant.has_sufficient_deposit(required_deposit),
        ErrorCode::MerchantDepositInsufficient
    );

    // 确定支付方式和价格 - 只支持SPL代币
    let payment_token = product.payment_token;
    require!(
        payment_config.is_token_supported(&payment_token),
        ErrorCode::UnsupportedToken
    );

    let token_info = payment_config
        .get_token_info(&payment_token)
        .ok_or(ErrorCode::UnsupportedToken)?;

    token_info.validate_amount(product.token_price.saturating_mul(amount))?;

    let total_price = product.token_price.saturating_mul(amount);

    // 计算手续费
    let fee_amount = total_price
        .saturating_mul(payment_config.fee_rate as u64)
        .saturating_div(10000);

    let current_time = Clock::get()?.unix_timestamp;

    // 初始化托管账户
    let bump = ctx.bumps.escrow_account;
    escrow_account.initialize(
        timestamp as u64, // 使用timestamp作为订单ID
        ctx.accounts.buyer.key(),
        product.merchant,
        product_id,
        payment_token,
        amount,
        total_price,
        fee_amount,
        bump,
    )?;

    // 创建订单记录 - 新增逻辑
    order.id = timestamp as u64;
    order.buyer = ctx.accounts.buyer.key();
    order.merchant = merchant.owner;
    order.product_id = product_id;
    order.quantity = amount as u32;
    order.unit_price = product.price;
    order.total_amount = product.price.checked_mul(amount).unwrap();
    order.payment_token = product.payment_token;
    order.token_decimals = product.token_decimals;
    order.token_unit_price = product.token_price;
    order.token_total_amount = total_price;
    order.status = OrderManagementStatus::Pending;
    order.shipping_address = shipping_address;
    order.notes = notes;
    order.created_at = current_time;
    order.updated_at = current_time;
    order.confirmed_at = None;
    order.shipped_at = None;
    order.delivered_at = None;
    order.refunded_at = None;
    order.transaction_signature = "".to_string(); // 将在客户端设置
    order.bump = ctx.bumps.order;

    // 验证订单数据
    order.validate()?;

    // 更新订单统计
    order_stats.update_for_new_order(order);

    // 将买家的代币转入托管账户
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer_token_account.to_account_info(),
                to: ctx.accounts.escrow_token_account.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        total_price,
    )?;

    msg!(
        "原子性购买成功: 买家 {}, 商品 {}, 数量 {}, 总费用 {} tokens, 订单ID {}, 订单状态: 待确认收货",
        ctx.accounts.buyer.key(),
        product_id,
        amount,
        total_price,
        timestamp
    );

    Ok(())
}
