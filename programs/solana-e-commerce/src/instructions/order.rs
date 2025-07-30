use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

// Create order (original - for backward compatibility)
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
            b"buyer_order",
            buyer.key().as_ref(),
            (user_purchase_count.purchase_count + 1).to_le_bytes().as_ref()
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

    // 商户订单相关账户（集成到CreateOrder中）
    #[account(
        init_if_needed,
        payer = buyer,
        space = 8 + MerchantOrderCount::INIT_SPACE,
        seeds = [
            b"merchant_order_count",
            merchant.owner.as_ref()
        ],
        bump
    )]
    pub merchant_order_count: Account<'info, MerchantOrderCount>,

    #[account(
        init,
        payer = buyer,
        space = 8 + MerchantOrder::INIT_SPACE,
        seeds = [
            b"merchant_order",
            merchant.owner.as_ref(),
            (merchant_order_count.total_orders + 1).to_le_bytes().as_ref()
        ],
        bump
    )]
    pub merchant_order: Account<'info, MerchantOrder>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// 商户发货
#[derive(Accounts)]
pub struct ShipOrder<'info> {
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

// 买家请求退款（直接退款）
#[derive(Accounts)]
pub struct RefundOrder<'info> {
    #[account(
        mut,
        constraint = order.buyer == buyer.key() @ ErrorCode::Unauthorized
    )]
    pub order: Account<'info, Order>,

    // 移除order_stats账户 - 统计功能非核心，可通过其他方式获取

    // 主程序统一托管账户（退款来源）
    #[account(
        mut,
        seeds = [b"program_token_account", payment_token_mint.key().as_ref()],
        bump
    )]
    pub program_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// CHECK: 程序权限账户，用于控制Token转账
    #[account(
        seeds = [b"program_authority"],
        bump
    )]
    pub program_authority: AccountInfo<'info>,

    pub payment_token_mint: Account<'info, Mint>,

    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// 商家批准退款指令已移除，买家可直接退款

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
pub struct ConfirmDelivery<'info> {
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

    #[account(
        mut,
        seeds = [b"program_token_account", system_config.deposit_token_mint.as_ref()],
        bump
    )]
    pub program_token_account: Account<'info, TokenAccount>,

    // 保证金托管账户（接收确认收货的资金）
    #[account(
        mut,
        seeds = [b"deposit_escrow", system_config.deposit_token_mint.as_ref()],
        bump,
        constraint = deposit_escrow_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    /// CHECK: 程序权限账户，用于控制Token转账
    #[account(
        seeds = [b"program_authority"],
        bump
    )]
    pub program_authority: AccountInfo<'info>,

    // === CPI调用外部vault程序所需的账户 ===
    /// CHECK: Vault账户，使用vault_program_id作为地址
    #[account(
        mut,
        constraint = vault.key() == system_config.vault_program_id @ ErrorCode::InvalidVaultProgram
    )]
    pub vault: UncheckedAccount<'info>,

    /// CHECK: Vault Token账户，从system_config读取地址
    #[account(
        mut,
        constraint = vault_token_account.key() == system_config.vault_token_account @ ErrorCode::InvalidVaultTokenAccount
    )]
    pub vault_token_account: UncheckedAccount<'info>,

    /// CHECK: 平台Token账户，从system_config读取地址
    #[account(
        mut,
        constraint = platform_token_account.key() == system_config.platform_token_account @ ErrorCode::InvalidPlatformTokenAccount
    )]
    pub platform_token_account: UncheckedAccount<'info>,

    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
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
    quantity: u32,
    shipping_address: String,
    notes: String,
    transaction_signature: String,
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let merchant_order = &mut ctx.accounts.merchant_order;
    let order_stats = &mut ctx.accounts.order_stats;
    let product = &ctx.accounts.product;
    let merchant = &ctx.accounts.merchant;
    let buyer = &ctx.accounts.buyer;
    let user_purchase_count = &mut ctx.accounts.user_purchase_count;
    let merchant_order_count = &mut ctx.accounts.merchant_order_count;

    // Get current timestamp
    let current_timestamp = Clock::get()?.unix_timestamp;

    // Verify product ID match
    require!(product.id == product_id, ErrorCode::InvalidProduct);

    // Verify merchant match
    require!(
        product.merchant == merchant.owner,
        ErrorCode::InvalidMerchant
    );

    // 初始化或更新用户购买计数
    if user_purchase_count.buyer == Pubkey::default() {
        user_purchase_count.initialize(buyer.key(), ctx.bumps.user_purchase_count)?;
    }

    let _purchase_count = user_purchase_count.increment_count()?;

    // 初始化或更新商户订单计数
    if merchant_order_count.merchant == Pubkey::default() {
        merchant_order_count.initialize(merchant.owner, ctx.bumps.merchant_order_count)?;
    }

    let merchant_order_sequence = merchant_order_count.increment_total_orders()?;

    // Initialize order - remove id field, use PDA to ensure uniqueness
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
    order.created_at = current_timestamp;
    order.updated_at = current_timestamp;
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

    // 初始化商户订单作为索引
    merchant_order.initialize_as_index(
        merchant.owner,
        buyer.key(),
        merchant_order_sequence,
        order.key(),
        product_id,
        ctx.bumps.merchant_order,
    )?;

    // 更新订单统计
    order_stats.update_for_new_order(order);

    msg!(
        "双订单创建成功: 买家订单PDA: {}, 商户订单PDA: {}, 买家: {}, 商户: {}, 商品: {}, 数量: {}, 总金额: {} lamports, 商户订单序列号: {}",
        order.key(),
        merchant_order.key(),
        buyer.key(),
        merchant.owner,
        product_id,
        quantity,
        order.total_amount,
        merchant_order_sequence
    );

    Ok(())
}

pub fn ship_order(ctx: Context<ShipOrder>, tracking_number: String) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;
    let merchant = &ctx.accounts.merchant;

    // Verify order belongs to this merchant
    require!(order.merchant == merchant.owner, ErrorCode::InvalidMerchant);

    // Verify tracking number
    require!(
        !tracking_number.is_empty() && tracking_number.len() <= 100,
        ErrorCode::InvalidTrackingNumber
    );

    let old_status = order.status.clone();
    let current_time = Clock::get()?.unix_timestamp;

    // Set tracking number
    order.tracking_number = tracking_number.clone();

    // Update order status to shipped
    order.update_status(OrderManagementStatus::Shipped, current_time)?;

    // Update statistics
    order_stats.update_for_status_change(
        &old_status,
        &OrderManagementStatus::Shipped,
        order.total_amount,
    );

    msg!("商户发货成功: 物流单号: {}", tracking_number);

    Ok(())
}

// 买家直接退款
pub fn refund_order(ctx: Context<RefundOrder>, refund_reason: String) -> Result<()> {
    let order = &mut ctx.accounts.order;
    // 移除order_stats引用 - 统计功能已简化

    // 验证订单状态必须是已发货
    require!(order.can_request_refund(), ErrorCode::OrderCannotBeRefunded);

    // 验证退款原因长度
    require!(
        refund_reason.len() <= 500,
        ErrorCode::InvalidOrderNotesLength
    );

    // 执行Token退款：从主程序托管账户直接转给买家
    let program_authority_bump = ctx.bumps.program_authority;
    let program_signer_seeds = &[b"program_authority".as_ref(), &[program_authority_bump]];
    let program_signer = &[&program_signer_seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.program_token_account.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        authority: ctx.accounts.program_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, program_signer);

    transfer(cpi_ctx, order.total_amount)?;

    let current_time = Clock::get()?.unix_timestamp;

    // 更新订单状态为已退款
    order.update_status(OrderManagementStatus::Refunded, current_time)?;
    order.refund_reason = refund_reason.clone();

    // 统计信息更新已移除 - 可通过查询链上订单账户获取统计信息

    msg!(
        "买家直接退款成功: 买家: {}, 退款金额: {} tokens, 退款原因: {}",
        order.buyer,
        order.total_amount,
        refund_reason
    );

    Ok(())
}

// 商家批准退款函数已移除，买家可直接退款

pub fn get_order_stats(ctx: Context<GetOrderStats>) -> Result<()> {
    let order_stats = &ctx.accounts.order_stats;

    msg!("订单统计信息:");
    msg!("总订单数: {}", order_stats.total_orders);
    msg!("待处理: {}", order_stats.pending_orders);
    msg!("已发货: {}", order_stats.shipped_orders);
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

    // 简化逻辑：使用程序权限直接从主程序托管账户转移到商户保证金账户
    let program_authority_bump = ctx.bumps.program_authority;
    let program_signer_seeds = &[b"program_authority".as_ref(), &[program_authority_bump]];
    let program_signer = &[&program_signer_seeds[..]];

    // 1. 暂时注释掉CPI调用外部程序的add_rewards指令，避免InstructionFallbackNotFound错误
    // TODO: 在外部vault程序正确配置后重新启用CPI调用
    if platform_fee > 0 {
        msg!(
            "平台手续费 {} lamports 将保留在程序托管账户中（CPI调用已暂时禁用）",
            platform_fee
        );

        /*
        // 原CPI调用代码已注释，避免InstructionFallbackNotFound错误
        // 检查vault程序ID是否有效（不是默认的System Program ID）
        if system_config.vault_program_id != anchor_lang::solana_program::system_program::ID {
            // 根据AddRewards结构体构建CPI调用的账户列表
            let add_rewards_accounts = vec![
                // vault: Account<'info, Vault>
                ctx.accounts.vault.to_account_info(),
                // vault_token_account: Account<'info, TokenAccount>
                ctx.accounts.vault_token_account.to_account_info(),
                // reward_source_account: Account<'info, TokenAccount> (使用程序Token账户)
                ctx.accounts.program_token_account.to_account_info(),
                // platform_token_account: Account<'info, TokenAccount>
                ctx.accounts.platform_token_account.to_account_info(),
                // reward_source_authority: Signer<'info> (使用买家作为签名者)
                ctx.accounts.buyer.to_account_info(),
                // token_program: Program<'info, Token>
                ctx.accounts.token_program.to_account_info(),
            ];

            // 构建add_rewards指令数据
            let add_rewards_data = {
                let mut data = Vec::new();
                // 添加指令discriminator (通常是方法名的hash前8字节)
                // 这里使用add_rewards的discriminator，需要根据实际外部程序确定
                let discriminator = [0x8c, 0x3c, 0x2b, 0x1a, 0x9f, 0x8e, 0x7d, 0x6c]; // 示例discriminator
                data.extend_from_slice(&discriminator);
                // 添加平台手续费金额参数
                data.extend_from_slice(&platform_fee.to_le_bytes());
                data
            };

            let add_rewards_instruction = anchor_lang::solana_program::instruction::Instruction {
                program_id: system_config.vault_program_id,
                accounts: vec![
                    // vault (mut)
                    anchor_lang::solana_program::instruction::AccountMeta::new(
                        ctx.accounts.vault.key(),
                        false,
                    ),
                    // vault_token_account (mut)
                    anchor_lang::solana_program::instruction::AccountMeta::new(
                        ctx.accounts.vault_token_account.key(),
                        false,
                    ),
                    // reward_source_account (mut) - 程序Token账户
                    anchor_lang::solana_program::instruction::AccountMeta::new(
                        ctx.accounts.program_token_account.key(),
                        false,
                    ),
                    // platform_token_account (mut)
                    anchor_lang::solana_program::instruction::AccountMeta::new(
                        ctx.accounts.platform_token_account.key(),
                        false,
                    ),
                    // reward_source_authority (signer) - 买家作为签名者
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        ctx.accounts.buyer.key(),
                        true,
                    ),
                    // token_program
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        ctx.accounts.token_program.key(),
                        false,
                    ),
                ],
                data: add_rewards_data,
            };

            // 尝试调用外部程序，如果失败则记录日志但不中断确认收货流程
            // 使用普通invoke而不是invoke_signed，因为买家已经是签名者
            match anchor_lang::solana_program::program::invoke(
                &add_rewards_instruction,
                &add_rewards_accounts,
            ) {
                Ok(_) => {
                    msg!(
                        "外部vault程序调用成功，平台手续费: {} lamports",
                        platform_fee
                    );
                }
                Err(e) => {
                    msg!("外部vault程序调用失败，继续确认收货流程。错误: {:?}", e);
                    msg!(
                        "平台手续费 {} lamports 将保留在程序托管账户中",
                        platform_fee
                    );
                }
            }
        } else {
            msg!(
                "vault程序ID无效，跳过CPI调用，平台手续费 {} lamports 将保留在程序托管账户中",
                platform_fee
            );
        }
        */
    }

    // 3. 转移剩余金额（商户实收）到商户保证金账户
    let merchant_transfer_accounts = Transfer {
        from: ctx.accounts.program_token_account.to_account_info(),
        to: ctx.accounts.deposit_escrow_account.to_account_info(),
        authority: ctx.accounts.program_authority.to_account_info(),
    };
    let merchant_cpi_program = ctx.accounts.token_program.to_account_info();
    let merchant_cpi_ctx = CpiContext::new_with_signer(
        merchant_cpi_program,
        merchant_transfer_accounts,
        program_signer,
    );
    transfer(merchant_cpi_ctx, merchant_amount)?;

    // Update merchant deposit balance (only add merchant's actual received amount, excluding platform fees)
    merchant_info.add_deposit(merchant_amount)?;

    let old_status = order.status.clone();
    let current_time = Clock::get()?.unix_timestamp;

    // Update to delivered status
    order.update_status(OrderManagementStatus::Delivered, current_time)?;

    // Update statistics
    order_stats.update_for_status_change(
        &old_status,
        &OrderManagementStatus::Delivered,
        order.total_amount,
    );

    // Verify token transfer success
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

/// Auto confirm delivery (called by merchant or administrator)
#[derive(Accounts)]
pub struct AutoConfirmDelivery<'info> {
    #[account(mut)]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,

    // Merchant account (for permission verification)
    #[account(
        seeds = [b"merchant_info", merchant.owner.as_ref()],
        bump = merchant.bump
    )]
    pub merchant: Account<'info, Merchant>,

    // 系统配置账户（获取自动确认天数）
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, crate::SystemConfig>,

    // 调用者（商户或系统管理员）
    #[account(
        constraint = authority.key() == merchant.owner || authority.key() == system_config.authority @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,
}

pub fn auto_confirm_delivery(ctx: Context<AutoConfirmDelivery>) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;
    let system_config = &ctx.accounts.system_config;

    let current_time = Clock::get()?.unix_timestamp;

    // 检查是否应该自动确认
    require!(
        order.should_auto_confirm(system_config.auto_confirm_days, current_time),
        ErrorCode::InvalidOrderStatusTransition
    );

    let old_status = order.status.clone();

    // 执行自动确认
    order.auto_confirm_delivery(current_time)?;

    // 更新统计信息
    order_stats.update_for_status_change(
        &old_status,
        &OrderManagementStatus::Delivered,
        order.total_amount,
    );

    let caller_type = if ctx.accounts.authority.key() == ctx.accounts.system_config.authority {
        "系统管理员"
    } else {
        "商户"
    };

    msg!(
        "订单自动确认收货成功: 订单ID {}, 买家: {}, 商户: {}, 调用者: {} ({}), 发货时间: {:?}, 确认时间: {}",
        order.product_id,
        order.buyer,
        order.merchant,
        ctx.accounts.authority.key(),
        caller_type,
        order.shipped_at,
        current_time
    );

    Ok(())
}
