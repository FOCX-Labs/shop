use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

// 创建订单
#[derive(Accounts)]
#[instruction(product_id: u64)]
pub struct CreateOrder<'info> {
    #[account(
        init_if_needed,
        payer = buyer,
        space = 8 + UserPurchaseCount::INIT_SPACE,
        seeds = [
            b"user_purchase_count",
            buyer.key().as_ref()
        ],
        bump
    )]
    pub user_purchase_count: Account<'info, UserPurchaseCount>,

    #[account(
        init,
        payer = buyer,
        space = 8 + Order::INIT_SPACE,
        seeds = [
            b"order",
            buyer.key().as_ref(),
            merchant.key().as_ref(),
            product_id.to_le_bytes().as_ref(),
            user_purchase_count.purchase_count.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    #[account(
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump
    )]
    pub product: Account<'info, ProductBase>,

    #[account(
        seeds = [b"merchant_info", merchant.owner.as_ref()],
        bump = merchant.bump
    )]
    pub merchant: Account<'info, Merchant>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// 更新订单状态
#[derive(Accounts)]
pub struct UpdateOrderStatus<'info> {
    #[account(mut)]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    #[account(
        seeds = [b"merchant_info", merchant.owner.as_ref()],
        bump = merchant.bump,
        constraint = merchant.owner == authority.key() @ ErrorCode::Unauthorized
    )]
    pub merchant: Account<'info, Merchant>,

    pub authority: Signer<'info>,
}

// 买家请求退款
#[derive(Accounts)]
pub struct RequestRefund<'info> {
    #[account(
        mut,
        constraint = order.buyer == buyer.key() @ ErrorCode::Unauthorized
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    pub buyer: Signer<'info>,
}

// 商家批准退款并执行退款
#[derive(Accounts)]
pub struct ApproveRefund<'info> {
    #[account(mut)]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    #[account(
        mut,
        seeds = [b"merchant_info", merchant.owner.as_ref()],
        bump = merchant.bump,
        constraint = merchant.owner == authority.key() @ ErrorCode::Unauthorized
    )]
    pub merchant: Account<'info, Merchant>,

    // 系统配置账户（获取保证金代币mint）
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, crate::SystemConfig>,

    // 支付配置账户（验证支持的代币）
    #[account(
        seeds = [b"payment_config"],
        bump
    )]
    pub payment_config: Account<'info, PaymentConfig>,

    // 保证金托管账户（扣除退款金额）
    #[account(
        mut,
        seeds = [b"deposit_escrow"],
        bump,
        constraint = deposit_escrow_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// CHECK: This is the program authority for token transfers
    pub program_authority: AccountInfo<'info>,

    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// 退款订单（保持向后兼容性，已废弃）
#[derive(Accounts)]
#[instruction(buyer: Pubkey, merchant_key: Pubkey, product_id: u64, timestamp: i64)]
pub struct RefundOrder<'info> {
    #[account(
        mut,
        seeds = [
            b"order",
            buyer.as_ref(),
            merchant_key.as_ref(),
            product_id.to_le_bytes().as_ref(),
            timestamp.to_le_bytes().as_ref()
        ],
        bump = order.bump
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    #[account(
        seeds = [b"merchant_info", merchant.owner.as_ref()],
        bump = merchant.bump,
        constraint = merchant.owner == authority.key() @ ErrorCode::Unauthorized
    )]
    pub merchant: Account<'info, Merchant>,

    #[account(mut)]
    pub merchant_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// 初始化订单统计
#[derive(Accounts)]
pub struct InitializeOrderStats<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + OrderStats::INIT_SPACE,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// 确认收货
#[derive(Accounts)]
#[instruction(buyer_key: Pubkey, merchant: Pubkey, product_id: u64, timestamp: i64)]
pub struct ConfirmDelivery<'info> {
    #[account(
        mut,
        seeds = [
            b"order",
            buyer_key.as_ref(),
            merchant.as_ref(),
            product_id.to_le_bytes().as_ref(),
            timestamp.to_le_bytes().as_ref()
        ],
        bump = order.bump,
        constraint = order.buyer == buyer.key() @ ErrorCode::Unauthorized
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    // 商户信息账户（用于更新保证金余额）
    #[account(
        mut,
        seeds = [b"merchant_info", order.merchant.as_ref()],
        bump
    )]
    pub merchant_info: Account<'info, crate::state::Merchant>,

    // 系统配置账户（获取保证金代币mint和平台手续费配置）
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, crate::SystemConfig>,

    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,

    // 保证金托管账户（接收确认收货的资金）
    #[account(
        mut,
        seeds = [b"deposit_escrow"],
        bump,
        constraint = deposit_escrow_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    // 新增：商户Token账户，用于接收扣除手续费后的金额
    #[account(
        mut,
        constraint = merchant_token_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken,
        constraint = merchant_token_account.owner == order.merchant @ ErrorCode::Unauthorized
    )]
    pub merchant_token_account: Account<'info, TokenAccount>,

    // 新增：平台手续费接收账户
    #[account(
        mut,
        constraint = platform_fee_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken,
        constraint = platform_fee_account.owner == system_config.platform_fee_recipient @ ErrorCode::Unauthorized
    )]
    pub platform_fee_account: Account<'info, TokenAccount>,

    /// CHECK: This is the program authority for token transfers
    pub program_authority: AccountInfo<'info>,

    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// 退货
#[derive(Accounts)]
#[instruction(buyer: Pubkey, merchant_key: Pubkey, product_id: u64, timestamp: i64)]
pub struct ReturnOrder<'info> {
    #[account(
        mut,
        seeds = [
            b"order",
            buyer.as_ref(),
            merchant_key.as_ref(),
            product_id.to_le_bytes().as_ref(),
            timestamp.to_le_bytes().as_ref()
        ],
        bump = order.bump
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    #[account(
        seeds = [b"merchant_info", merchant.owner.as_ref()],
        bump = merchant.bump,
        constraint = merchant.owner == authority.key() @ ErrorCode::Unauthorized
    )]
    pub merchant: Account<'info, Merchant>,

    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// CHECK: This is the program authority for token transfers
    pub program_authority: AccountInfo<'info>,

    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// 获取订单统计
#[derive(Accounts)]
pub struct GetOrderStats<'info> {
    #[account(
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,
}

// 指令实现
pub fn initialize_order_stats(ctx: Context<InitializeOrderStats>) -> Result<()> {
    let order_stats = &mut ctx.accounts.order_stats;

    order_stats.total_orders = 0;
    order_stats.pending_orders = 0;
    order_stats.confirmed_orders = 0;
    order_stats.shipped_orders = 0;
    order_stats.refund_requested_orders = 0;
    order_stats.delivered_orders = 0;
    order_stats.refunded_orders = 0;
    order_stats.total_revenue = 0;
    order_stats.bump = ctx.bumps.order_stats;

    msg!("订单统计系统初始化成功");

    Ok(())
}

pub fn create_order(
    ctx: Context<CreateOrder>,
    product_id: u64,
    timestamp: i64,
    quantity: u32,
    shipping_address: String,
    notes: String,
    transaction_signature: String,
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;
    let product = &ctx.accounts.product;
    let merchant = &ctx.accounts.merchant;
    let buyer = &ctx.accounts.buyer;
    let user_purchase_count = &mut ctx.accounts.user_purchase_count;

    // 验证产品ID匹配
    require!(product.id == product_id, ErrorCode::InvalidProduct);

    // 验证商户匹配
    require!(
        product.merchant == merchant.owner,
        ErrorCode::InvalidMerchant
    );

    // 初始化或更新用户购买计数
    if user_purchase_count.buyer == Pubkey::default() {
        user_purchase_count.initialize(buyer.key(), ctx.bumps.user_purchase_count)?;
    }

    let _purchase_count = user_purchase_count.increment_count()?;
    let current_time = Clock::get()?.unix_timestamp;

    // 初始化订单 - 移除id字段，使用PDA确保唯一性
    order.buyer = buyer.key();
    order.merchant = merchant.owner;
    order.product_id = product_id;
    order.quantity = quantity;
    order.price = product.price;
    order.total_amount = product.price.checked_mul(quantity as u64).unwrap();
    order.payment_token = product.payment_token;
    order.status = OrderManagementStatus::Pending;
    order.shipping_address = shipping_address;
    order.notes = notes;
    order.created_at = current_time;
    order.updated_at = current_time;
    order.confirmed_at = None;
    order.shipped_at = None;
    order.delivered_at = None;
    order.refunded_at = None;
    order.refund_requested_at = None;
    order.refund_reason = String::new();
    order.transaction_signature = transaction_signature;
    order.bump = ctx.bumps.order;

    // 验证订单数据
    order.validate()?;

    // 更新订单统计
    order_stats.update_for_new_order(order);

    msg!(
        "订单创建成功: ID {}, 买家: {}, 商户: {}, 商品: {}, 数量: {}, 总金额: {} lamports",
        timestamp,
        buyer.key(),
        merchant.owner,
        product_id,
        quantity,
        order.total_amount
    );

    Ok(())
}

pub fn update_order_status(
    ctx: Context<UpdateOrderStatus>,
    new_status: OrderManagementStatus,
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;

    let old_status = order.status.clone();
    let current_time = Clock::get()?.unix_timestamp;

    // 更新订单状态
    order.update_status(new_status.clone(), current_time)?;

    // 更新统计信息
    order_stats.update_for_status_change(&old_status, &new_status, order.total_amount);

    msg!(
        "订单状态更新成功: 从 {:?} 更新为 {:?}",
        old_status,
        new_status
    );

    Ok(())
}

// 买家请求退款
pub fn request_refund(ctx: Context<RequestRefund>, refund_reason: String) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;

    // 验证订单状态必须是已发货
    require!(order.can_request_refund(), ErrorCode::OrderCannotBeRefunded);

    // 验证退款原因长度
    require!(
        refund_reason.len() <= 500,
        ErrorCode::InvalidOrderNotesLength
    );

    let old_status = order.status.clone();
    let current_time = Clock::get()?.unix_timestamp;

    // 更新订单状态为退款请求中
    order.update_status(OrderManagementStatus::RefundRequested, current_time)?;
    order.refund_reason = refund_reason.clone();

    // 更新统计信息
    order_stats.update_for_status_change(
        &old_status,
        &OrderManagementStatus::RefundRequested,
        order.total_amount,
    );

    msg!(
        "退款请求提交成功: 买家: {}, 退款原因: {}",
        order.buyer,
        refund_reason
    );

    Ok(())
}

// 商家批准退款并执行退款
pub fn approve_refund(ctx: Context<ApproveRefund>) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;
    let merchant = &mut ctx.accounts.merchant;

    // 验证订单状态必须是退款请求中
    require!(order.can_approve_refund(), ErrorCode::OrderCannotBeRefunded);

    // 验证商户保证金余额充足
    require!(
        merchant.has_sufficient_deposit(order.total_amount),
        ErrorCode::InsufficientDeposit
    );

    // 验证订单的支付Token是否在系统支持的代币列表中
    require!(
        ctx.accounts
            .payment_config
            .is_token_supported(&order.payment_token),
        ErrorCode::InvalidDepositToken
    );

    // 执行代币退款：从保证金托管账户转给买家
    let deposit_escrow_bump = ctx.bumps.deposit_escrow_account;
    let seeds = &[b"deposit_escrow".as_ref(), &[deposit_escrow_bump]];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.deposit_escrow_account.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        authority: ctx.accounts.deposit_escrow_account.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

    transfer(cpi_ctx, order.total_amount)?;

    // 扣除商户保证金余额
    merchant.deduct_deposit(order.total_amount)?;

    let old_status = order.status.clone();
    let current_time = Clock::get()?.unix_timestamp;

    // 更新为退款状态
    order.update_status(OrderManagementStatus::Refunded, current_time)?;

    // 更新统计信息
    order_stats.update_for_status_change(
        &old_status,
        &OrderManagementStatus::Refunded,
        order.total_amount,
    );

    msg!(
        "退款批准并执行成功: 退款金额: {} tokens, 商户保证金余额: {} tokens",
        order.total_amount,
        merchant.deposit_amount
    );

    Ok(())
}

#[deprecated(note = "Use request_refund() and approve_refund() instead")]
pub fn refund_order(
    ctx: Context<RefundOrder>,
    _buyer: Pubkey,
    _merchant_key: Pubkey,
    _product_id: u64,
    timestamp: i64,
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;

    require!(order.can_approve_refund(), ErrorCode::OrderCannotBeRefunded);

    // 执行代币退款
    let cpi_accounts = Transfer {
        from: ctx.accounts.merchant_token_account.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer(cpi_ctx, order.total_amount)?;

    let old_status = order.status.clone();
    let current_time = Clock::get()?.unix_timestamp;

    // 更新为退款状态
    order.update_status(OrderManagementStatus::Refunded, current_time)?;

    // 更新统计信息
    order_stats.update_for_status_change(
        &old_status,
        &OrderManagementStatus::Refunded,
        order.total_amount,
    );

    msg!(
        "订单退款成功: ID {}, 退款金额: {} tokens",
        timestamp,
        order.total_amount
    );

    Ok(())
}

pub fn get_order_stats(ctx: Context<GetOrderStats>) -> Result<()> {
    let order_stats = &ctx.accounts.order_stats;

    msg!("订单统计信息:");
    msg!("总订单数: {}", order_stats.total_orders);
    msg!("待处理: {}", order_stats.pending_orders);
    msg!("已确认: {}", order_stats.confirmed_orders);
    msg!("已发货: {}", order_stats.shipped_orders);
    msg!("退款请求中: {}", order_stats.refund_requested_orders);
    msg!("已送达: {}", order_stats.delivered_orders);
    msg!("已退款: {}", order_stats.refunded_orders);
    msg!("总收入: {} lamports", order_stats.total_revenue);

    Ok(())
}

pub fn confirm_delivery(ctx: Context<ConfirmDelivery>) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;
    let merchant_info = &mut ctx.accounts.merchant_info;
    let system_config = &ctx.accounts.system_config;

    // 验证订单状态必须是已发货
    require!(
        order.status == OrderManagementStatus::Shipped,
        ErrorCode::InvalidOrderStatusTransition
    );

    // 验证订单的支付Token与保证金Token一致
    require!(
        order.payment_token == system_config.deposit_token_mint,
        ErrorCode::InvalidDepositToken
    );

    // 智能确认收货逻辑：检查是否为自动确认收货
    let current_time = Clock::get()?.unix_timestamp;
    let is_auto_confirm = if let Some(shipped_at) = order.shipped_at {
        let auto_confirm_seconds = system_config.auto_confirm_days as i64 * 24 * 60 * 60;
        let auto_confirm_time = shipped_at + auto_confirm_seconds;
        current_time >= auto_confirm_time
    } else {
        false
    };

    // 如果是自动确认收货，记录日志
    if is_auto_confirm {
        msg!(
            "自动确认收货触发: 买家 {}, 发货时间: {}, 当前时间: {}, 自动确认天数: {}天",
            order.buyer,
            order.shipped_at.unwrap_or(0),
            current_time,
            system_config.auto_confirm_days
        );
    } else {
        msg!(
            "手动确认收货: 买家 {}, 确认时间: {}",
            order.buyer,
            current_time
        );
    }

    // 计算平台手续费
    let total_amount = order.total_amount;
    let platform_fee_rate = ctx.accounts.system_config.platform_fee_rate as u64;
    let platform_fee = total_amount
        .checked_mul(platform_fee_rate)
        .and_then(|x| x.checked_div(10000))
        .ok_or(ErrorCode::IntegerOverflow)?;
    let merchant_amount = total_amount
        .checked_sub(platform_fee)
        .ok_or(ErrorCode::IntegerOverflow)?;

    // 使用保证金托管账户作为权限进行转账
    let deposit_escrow_bump = ctx.bumps.deposit_escrow_account;
    let seeds = &[b"deposit_escrow".as_ref(), &[deposit_escrow_bump]];
    let signer = &[&seeds[..]];

    // 1. 转账平台手续费到平台手续费账户
    if platform_fee > 0 {
        let cpi_accounts_fee = Transfer {
            from: ctx.accounts.program_token_account.to_account_info(),
            to: ctx.accounts.platform_fee_account.to_account_info(),
            authority: ctx.accounts.deposit_escrow_account.to_account_info(),
        };
        let cpi_program_fee = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_fee = CpiContext::new_with_signer(cpi_program_fee, cpi_accounts_fee, signer);

        transfer(cpi_ctx_fee, platform_fee)?;

        msg!(
            "平台手续费收取成功: 订单金额: {}, 手续费率: {}基点, 手续费金额: {}",
            total_amount,
            platform_fee_rate,
            platform_fee
        );
    }

    // 2. 转账剩余金额到商户Token账户
    let cpi_accounts_merchant = Transfer {
        from: ctx.accounts.program_token_account.to_account_info(),
        to: ctx.accounts.merchant_token_account.to_account_info(),
        authority: ctx.accounts.deposit_escrow_account.to_account_info(),
    };
    let cpi_program_merchant = ctx.accounts.token_program.to_account_info();
    let cpi_ctx_merchant =
        CpiContext::new_with_signer(cpi_program_merchant, cpi_accounts_merchant, signer);

    transfer(cpi_ctx_merchant, merchant_amount)?;

    // 3. 同时转账到保证金托管账户（用于保证金管理）
    let cpi_accounts_deposit = Transfer {
        from: ctx.accounts.program_token_account.to_account_info(),
        to: ctx.accounts.deposit_escrow_account.to_account_info(),
        authority: ctx.accounts.deposit_escrow_account.to_account_info(),
    };
    let cpi_program_deposit = ctx.accounts.token_program.to_account_info();
    let cpi_ctx_deposit =
        CpiContext::new_with_signer(cpi_program_deposit, cpi_accounts_deposit, signer);

    transfer(cpi_ctx_deposit, merchant_amount)?;

    // 更新商户保证金余额（扣除手续费后的金额）
    merchant_info.add_deposit(merchant_amount)?;

    let old_status = order.status.clone();
    let current_time = Clock::get()?.unix_timestamp;

    // 更新为已送达状态
    order.update_status(OrderManagementStatus::Delivered, current_time)?;

    // 更新统计信息
    order_stats.update_for_status_change(
        &old_status,
        &OrderManagementStatus::Delivered,
        order.total_amount,
    );

    // 验证代币转账是否成功
    let deposit_balance_after = ctx.accounts.deposit_escrow_account.amount;
    let program_balance_after = ctx.accounts.program_token_account.amount;

    msg!(
        "确认收货成功: 买家: {}, 确认时间: {}, 订单总金额: {} tokens",
        order.buyer,
        current_time,
        total_amount
    );

    msg!(
        "平台手续费处理: 手续费率: {}基点, 手续费金额: {} tokens, 商户实收: {} tokens",
        platform_fee_rate,
        platform_fee,
        merchant_amount
    );

    msg!(
        "商户保证金更新: 商户 {}, 新增保证金: {} tokens, 当前总保证金: {} tokens",
        order.merchant,
        merchant_amount,
        merchant_info.deposit_amount
    );
    msg!(
        "代币余额验证: 保证金账户余额: {}, 主程序托管账户余额: {}",
        deposit_balance_after,
        program_balance_after
    );

    Ok(())
}

pub fn return_order(
    ctx: Context<ReturnOrder>,
    _buyer: Pubkey,
    _merchant_key: Pubkey,
    _product_id: u64,
    timestamp: i64,
    return_reason: Option<String>,
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;

    // 验证订单状态必须是已送达
    require!(
        order.status == OrderManagementStatus::Delivered,
        ErrorCode::InvalidOrderStatusTransition
    );

    // 验证退货原因长度（如果提供）
    if let Some(ref reason) = return_reason {
        require!(reason.len() <= 500, ErrorCode::InvalidOrderNotesLength);
    }

    // 执行代币退货：从主程序托管账户退回给买家
    let cpi_accounts = Transfer {
        from: ctx.accounts.program_token_account.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        authority: ctx.accounts.program_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer(cpi_ctx, order.total_amount)?;

    let old_status = order.status.clone();
    let current_time = Clock::get()?.unix_timestamp;

    // 更新为已退货状态
    order.update_status(OrderManagementStatus::Refunded, current_time)?;

    // 更新统计信息
    order_stats.update_for_status_change(
        &old_status,
        &OrderManagementStatus::Refunded,
        order.total_amount,
    );

    // 验证代币退款是否成功
    let buyer_balance_after = ctx.accounts.buyer_token_account.amount;
    let program_balance_after = ctx.accounts.program_token_account.amount;

    let reason_msg = return_reason.unwrap_or_else(|| "无退货原因".to_string());
    msg!(
        "退货成功: 订单ID {}, 退货金额: {} tokens, 退货原因: {}",
        timestamp,
        order.total_amount,
        reason_msg
    );
    msg!(
        "代币余额验证: 买家账户余额: {}, 主程序托管账户余额: {}",
        buyer_balance_after,
        program_balance_after
    );

    Ok(())
}
