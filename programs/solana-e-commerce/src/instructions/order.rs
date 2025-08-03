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

    // Merchant order related accounts (integrated into CreateOrder)
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

// Merchant shipping
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

// Buyer requests refund (direct refund)
#[derive(Accounts)]
pub struct RefundOrder<'info> {
    #[account(
        mut,
        constraint = order.buyer == buyer.key() @ ErrorCode::Unauthorized
    )]
    pub order: Account<'info, Order>,

    // Remove order_stats account - statistics functionality is not core, can be obtained through other methods

    // Main program unified escrow account (refund source)
    #[account(
        mut,
        seeds = [b"program_token_account", payment_token_mint.key().as_ref()],
        bump
    )]
    pub program_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// CHECK: Program authority account, used to control token transfers
    #[account(
        seeds = [b"program_authority"],
        bump
    )]
    pub program_authority: AccountInfo<'info>,

    pub payment_token_mint: Account<'info, Mint>,

    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// Merchant approval refund instruction has been removed, buyer can refund directly

// Initialize order statistics
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

// Confirm delivery
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

    // Merchant info account (for updating deposit balance)
    #[account(
        mut,
        seeds = [b"merchant_info", order.merchant.as_ref()],
        bump
    )]
    pub merchant_info: Account<'info, crate::state::Merchant>,

    // System config account (get deposit token mint and platform fee configuration)
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

    // Deposit escrow account (receive funds from confirmed delivery)
    #[account(
        mut,
        seeds = [b"deposit_escrow", system_config.deposit_token_mint.as_ref()],
        bump,
        constraint = deposit_escrow_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    /// CHECK: Program authority account, used to control token transfers
    #[account(
        seeds = [b"program_authority"],
        bump
    )]
    pub program_authority: AccountInfo<'info>,

    // === CPI call external vault program required accounts ===
    /// CHECK: Vault account, read address from system_config
    #[account(
        mut,
        constraint = vault.key() == system_config.vault_account @ ErrorCode::InvalidVaultAccount
    )]
    pub vault: UncheckedAccount<'info>,

    /// CHECK: Vault Token account, read address from system_config
    #[account(
        mut,
        constraint = vault_token_account.key() == system_config.vault_token_account @ ErrorCode::InvalidVaultTokenAccount
    )]
    pub vault_token_account: UncheckedAccount<'info>,

    /// CHECK: Platform Token account, read address from system_config
    #[account(
        mut,
        constraint = platform_token_account.key() == system_config.platform_token_account @ ErrorCode::InvalidPlatformTokenAccount
    )]
    pub platform_token_account: UncheckedAccount<'info>,

    /// CHECK: Vault program, read program ID from system_config
    #[account(
        constraint = vault_program.key() == system_config.vault_program_id @ ErrorCode::InvalidVaultProgram
    )]
    pub vault_program: UncheckedAccount<'info>,

    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// Get order statistics
#[derive(Accounts)]
pub struct GetOrderStats<'info> {
    #[account(
        seeds = [b"order_stats"],
        bump
    )]
    pub order_stats: Account<'info, OrderStats>,
}

// Instruction implementation
pub fn initialize_order_stats(ctx: Context<InitializeOrderStats>) -> Result<()> {
    let order_stats = &mut ctx.accounts.order_stats;

    order_stats.total_orders = 0;
    order_stats.pending_orders = 0;
    order_stats.shipped_orders = 0;
    order_stats.delivered_orders = 0;
    order_stats.refunded_orders = 0;
    order_stats.total_revenue = 0;
    order_stats.bump = ctx.bumps.order_stats;

    msg!("Order statistics system initialized successfully");

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

    // Initialize or update user purchase count
    if user_purchase_count.buyer == Pubkey::default() {
        user_purchase_count.initialize(buyer.key(), ctx.bumps.user_purchase_count)?;
    }

    let _purchase_count = user_purchase_count.increment_count()?;

    // Initialize or update merchant order count
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

    // Validate order data
    order.validate()?;

    // Initialize merchant order as index
    merchant_order.initialize_as_index(
        merchant.owner,
        buyer.key(),
        merchant_order_sequence,
        order.key(),
        product_id,
        ctx.bumps.merchant_order,
    )?;

    // Update order statistics
    order_stats.update_for_new_order(order);

    msg!(
        "Dual order creation successful: Buyer order PDA: {}, Merchant order PDA: {}, Buyer: {}, Merchant: {}, Product: {}, Quantity: {}, Total amount: {} lamports, Merchant order sequence: {}",
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

    msg!("Merchant shipping successful: Tracking number: {}", tracking_number);

    Ok(())
}

// Buyer direct refund
pub fn refund_order(ctx: Context<RefundOrder>, refund_reason: String) -> Result<()> {
    let order = &mut ctx.accounts.order;
    // Remove order_stats reference - statistics functionality has been simplified

    // Verify order status must be shipped
    require!(order.can_request_refund(), ErrorCode::OrderCannotBeRefunded);

    // Verify refund reason length
    require!(
        refund_reason.len() <= 500,
        ErrorCode::InvalidOrderNotesLength
    );

    // Execute token refund: transfer directly from main program escrow account to buyer
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

    // Update order status to refunded
    order.update_status(OrderManagementStatus::Refunded, current_time)?;
    order.refund_reason = refund_reason.clone();

    // Statistics update removed - can get statistics by querying on-chain order accounts

    msg!(
        "Buyer direct refund successful: Buyer: {}, Refund amount: {} tokens, Refund reason: {}",
        order.buyer,
        order.total_amount,
        refund_reason
    );

    Ok(())
}

// Merchant approval refund function has been removed, buyer can refund directly

pub fn get_order_stats(ctx: Context<GetOrderStats>) -> Result<()> {
    let order_stats = &ctx.accounts.order_stats;

    msg!("Order statistics:");
    msg!("Total orders: {}", order_stats.total_orders);
    msg!("Pending: {}", order_stats.pending_orders);
    msg!("Shipped: {}", order_stats.shipped_orders);
    msg!("Delivered: {}", order_stats.delivered_orders);
    msg!("Refunded: {}", order_stats.refunded_orders);
    msg!("Total revenue: {} lamports", order_stats.total_revenue);

    Ok(())
}

pub fn confirm_delivery(ctx: Context<ConfirmDelivery>) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let order_stats = &mut ctx.accounts.order_stats;
    let merchant_info = &mut ctx.accounts.merchant_info;
    let system_config = &ctx.accounts.system_config;

    // Verify order status must be shipped
    require!(
        order.status == OrderManagementStatus::Shipped,
        ErrorCode::InvalidOrderStatusTransition
    );

    // Verify that order payment token matches deposit token
    require!(
        order.payment_token == system_config.deposit_token_mint,
        ErrorCode::InvalidDepositToken
    );

    // Smart delivery confirmation logic: check if it's auto-confirmation
    let current_time = Clock::get()?.unix_timestamp;
    let is_auto_confirm = if let Some(shipped_at) = order.shipped_at {
        let auto_confirm_seconds = system_config.auto_confirm_days as i64 * 24 * 60 * 60;
        let auto_confirm_time = shipped_at + auto_confirm_seconds;
        current_time >= auto_confirm_time
    } else {
        false
    };

    // If it's auto-confirmation, log it
    if is_auto_confirm {
        msg!(
            "Auto-confirmation triggered: Buyer {}, Shipped at: {}, Current time: {}, Auto-confirm days: {} days",
            order.buyer,
            order.shipped_at.unwrap_or(0),
            current_time,
            system_config.auto_confirm_days
        );
    } else {
        msg!(
            "Manual delivery confirmation: Buyer {}, Confirmation time: {}",
            order.buyer,
            current_time
        );
    }

    // Calculate platform fee
    let total_amount = order.total_amount;
    let platform_fee_rate = ctx.accounts.system_config.platform_fee_rate as u64;
    let platform_fee = total_amount
        .checked_mul(platform_fee_rate)
        .and_then(|x| x.checked_div(10000))
        .ok_or(ErrorCode::IntegerOverflow)?;
    let merchant_amount = total_amount
        .checked_sub(platform_fee)
        .ok_or(ErrorCode::IntegerOverflow)?;

    // Simplified logic: use program authority to transfer directly from main program escrow account to merchant deposit account
    let program_authority_bump = ctx.bumps.program_authority;
    let program_signer_seeds = &[b"program_authority".as_ref(), &[program_authority_bump]];
    let program_signer = &[&program_signer_seeds[..]];

    // 1. Process platform fee through CPI call to external vault program
    if platform_fee > 0 {
        msg!(
            "Start processing platform fee: {} lamports, calling vault program for distribution",
            platform_fee
        );
        // Check if vault program ID is valid (not default System Program ID)
        if system_config.vault_program_id != anchor_lang::solana_program::system_program::ID {
            // Build CPI call account list based on AddRewards struct
            let add_rewards_accounts = vec![
                // vault: Account<'info, Vault>
                ctx.accounts.vault.to_account_info(),
                // vault_token_account: Account<'info, TokenAccount>
                ctx.accounts.vault_token_account.to_account_info(),
                // reward_source_account: Account<'info, TokenAccount> (use program token account)
                ctx.accounts.program_token_account.to_account_info(),
                // platform_token_account: Account<'info, TokenAccount>
                ctx.accounts.platform_token_account.to_account_info(),
                // reward_source_authority: Signer<'info> (use program authority PDA as signer)
                ctx.accounts.program_authority.to_account_info(),
                // token_program: Program<'info, Token>
                ctx.accounts.token_program.to_account_info(),
                // vault_program: Program<'info, VaultProgram> (add vault program account)
                ctx.accounts.vault_program.to_account_info(),
            ];

            // Build add_rewards instruction data
            let add_rewards_data = {
                let mut data = Vec::new();
                // Add instruction discriminator - correct discriminator determined from vault.json IDL
                let discriminator = [88, 186, 25, 227, 38, 137, 81, 23]; // correct discriminator for add_rewards instruction
                data.extend_from_slice(&discriminator);
                // Add platform fee amount parameter
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
                    // reward_source_account (mut) - program token account
                    anchor_lang::solana_program::instruction::AccountMeta::new(
                        ctx.accounts.program_token_account.key(),
                        false,
                    ),
                    // platform_token_account (mut)
                    anchor_lang::solana_program::instruction::AccountMeta::new(
                        ctx.accounts.platform_token_account.key(),
                        false,
                    ),
                    // reward_source_authority (signer) - program authority PDA as signer
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        ctx.accounts.program_authority.key(),
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

            // Try to call external program, if it fails log but don't interrupt delivery confirmation process
            // Use invoke_signed because program authority PDA needs to sign
            match anchor_lang::solana_program::program::invoke_signed(
                &add_rewards_instruction,
                &add_rewards_accounts,
                program_signer,
            ) {
                Ok(_) => {
                    msg!(
                        "External vault program call successful, platform fee: {} lamports",
                        platform_fee
                    );
                }
                Err(e) => {
                    msg!("External vault program call failed, continue delivery confirmation process. Error: {:?}", e);
                    msg!(
                        "Platform fee {} lamports will remain in program escrow account",
                        platform_fee
                    );
                }
            }
        } else {
            msg!(
                "Vault program ID invalid, skip CPI call, platform fee {} lamports will remain in program escrow account",
                platform_fee
            );
        }
    }

    // 3. Transfer remaining amount (merchant's actual received) to merchant deposit account
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
        "Delivery confirmation successful: Buyer: {}, Confirmation time: {}, Order total amount: {} tokens",
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
