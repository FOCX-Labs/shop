use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

// 创建订单
#[derive(Accounts)]
#[instruction(product_id: u64, timestamp: i64)]
pub struct CreateOrder<'info> {
    #[account(
        init,
        payer = buyer,
        space = Order::LEN,
        seeds = [
            b"order",
            buyer.key().as_ref(),
            merchant.key().as_ref(),
            product_id.to_le_bytes().as_ref(),
            timestamp.to_le_bytes().as_ref()
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
    pub product: Account<'info, Product>,

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
#[instruction(buyer_key: Pubkey, merchant_key: Pubkey, product_id: u64, timestamp: i64)]
pub struct UpdateOrderStatus<'info> {
    #[account(
        mut,
        seeds = [
            b"order",
            buyer_key.as_ref(),
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

    pub authority: Signer<'info>,
}

// 退款订单
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
        space = OrderStats::LEN,
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

    // 系统配置账户（获取保证金代币mint）
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

    // 验证产品ID匹配
    require!(product.id == product_id, ErrorCode::InvalidProduct);

    // 验证商户匹配
    require!(
        product.merchant == merchant.owner,
        ErrorCode::InvalidMerchant
    );

    let current_time = Clock::get()?.unix_timestamp;

    // 初始化订单 - 使用时间戳作为订单ID确保唯一性
    order.id = timestamp as u64;
    order.buyer = buyer.key();
    order.merchant = merchant.owner;
    order.product_id = product_id;
    order.quantity = quantity;
    order.unit_price = product.price;
    order.total_amount = product.price.checked_mul(quantity as u64).unwrap();
    order.payment_token = product.payment_token;
    order.token_decimals = product.token_decimals;
    order.token_unit_price = product.token_price;
    order.token_total_amount = product.token_price.checked_mul(quantity as u64).unwrap();
    order.status = OrderManagementStatus::Pending;
    order.shipping_address = shipping_address;
    order.notes = notes;
    order.created_at = current_time;
    order.updated_at = current_time;
    order.confirmed_at = None;
    order.shipped_at = None;
    order.delivered_at = None;
    order.refunded_at = None;
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
    buyer_key: Pubkey,
    merchant_key: Pubkey,
    product_id: u64,
    timestamp: i64,
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
        "订单状态更新成功: ID {}, 从 {:?} 更新为 {:?}",
        timestamp,
        old_status,
        new_status
    );

    Ok(())
}

pub fn refund_order(
    ctx: Context<RefundOrder>,
    buyer: Pubkey,
    merchant_key: Pubkey,
    product_id: u64,
    timestamp: i64
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;

    require!(order.can_refund(), ErrorCode::OrderCannotBeRefunded);

    // 执行代币退款
    let cpi_accounts = Transfer {
        from: ctx.accounts.merchant_token_account.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer(cpi_ctx, order.token_total_amount)?;

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
        order.token_total_amount
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
    msg!("已送达: {}", order_stats.delivered_orders);
    msg!("已退款: {}", order_stats.refunded_orders);
    msg!("总收入: {} lamports", order_stats.total_revenue);

    Ok(())
}

pub fn confirm_delivery(
    ctx: Context<ConfirmDelivery>,
    buyer_key: Pubkey,
    merchant: Pubkey,
    product_id: u64,
    timestamp: i64
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;
    let merchant_info = &mut ctx.accounts.merchant_info;

    // 验证订单状态必须是已发货
    require!(
        order.status == OrderManagementStatus::Shipped,
        ErrorCode::InvalidOrderStatusTransition
    );

    // 验证订单的支付Token与保证金Token一致
    require!(
        order.payment_token == ctx.accounts.system_config.deposit_token_mint,
        ErrorCode::InvalidDepositToken
    );

    // 执行代币转账：从主程序托管账户转到保证金托管账户
    // 使用保证金托管账户作为权限进行转账
    let deposit_escrow_bump = ctx.bumps.deposit_escrow_account;
    let seeds = &[b"deposit_escrow".as_ref(), &[deposit_escrow_bump]];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.program_token_account.to_account_info(),
        to: ctx.accounts.deposit_escrow_account.to_account_info(),
        authority: ctx.accounts.deposit_escrow_account.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

    transfer(cpi_ctx, order.token_total_amount)?;

    // 更新商户保证金余额
    merchant_info.add_deposit(order.token_total_amount)?;

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
        "确认收货成功: 订单ID {}, 买家: {}, 确认时间: {}, 转账金额: {} tokens, 转入保证金账户",
        timestamp,
        order.buyer,
        current_time,
        order.token_total_amount
    );

    msg!(
        "商户保证金更新: 商户 {}, 新增保证金: {} tokens, 当前总保证金: {} tokens",
        order.merchant,
        order.token_total_amount,
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
    buyer: Pubkey,
    merchant_key: Pubkey,
    product_id: u64,
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

    transfer(cpi_ctx, order.token_total_amount)?;

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
        order.token_total_amount,
        reason_msg
    );
    msg!(
        "代币余额验证: 买家账户余额: {}, 主程序托管账户余额: {}",
        buyer_balance_after,
        program_balance_after
    );

    Ok(())
}
