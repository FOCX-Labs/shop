use crate::error::ErrorCode;
use crate::state::{GlobalIdRoot, IdChunk, Merchant, MerchantIdAccount, MerchantStats};
use anchor_lang::prelude::*;
// 移除未使用的token导入，因为保证金管理已统一到deposit.rs模块

// 初始化商户账户
#[derive(Accounts)]
pub struct InitializeMerchant<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Merchant::INIT_SPACE,
        seeds = [b"merchant_info", owner.key().as_ref()],
        bump
    )]
    pub merchant_info: Account<'info, Merchant>,

    // 系统配置账户，用于获取保证金代币mint
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, crate::SystemConfig>,

    pub owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// 更新商户信息
#[derive(Accounts)]
pub struct UpdateMerchant<'info> {
    #[account(
        mut,
        seeds = [b"merchant_info", owner.key().as_ref()],
        bump,
        constraint = merchant_info.owner == owner.key() @ ErrorCode::Unauthorized
    )]
    pub merchant_info: Account<'info, Merchant>,

    pub owner: Signer<'info>,
}

pub fn update_merchant_info(
    ctx: Context<UpdateMerchant>,
    name: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let merchant_info = &mut ctx.accounts.merchant_info;
    merchant_info.update_info(name, description)?;

    Ok(())
}

// 获取商户统计信息
#[derive(Accounts)]
pub struct GetMerchantStats<'info> {
    #[account(
        seeds = [b"merchant_info", owner.key().as_ref()],
        bump
    )]
    pub merchant_info: Account<'info, Merchant>,

    pub owner: Signer<'info>,
}

pub fn get_merchant_stats(ctx: Context<GetMerchantStats>) -> Result<MerchantStats> {
    let merchant_info = &ctx.accounts.merchant_info;

    Ok(MerchantStats {
        product_count: merchant_info.product_count,
        total_sales: merchant_info.total_sales,
        active_products: merchant_info.product_count, // 简化处理
        total_keywords: 0,                            // 需要从其他地方获取
        avg_product_price: 0,                         // 需要计算
        last_updated: merchant_info.updated_at,
    })
}

// 设置商户状态
pub fn set_merchant_status(ctx: Context<UpdateMerchant>, is_active: bool) -> Result<()> {
    let merchant_info = &mut ctx.accounts.merchant_info;
    merchant_info.set_active(is_active)?;

    Ok(())
}

// 增加商户销量
pub fn add_merchant_sales(ctx: Context<UpdateMerchant>, sales_amount: u64) -> Result<()> {
    let merchant_info = &mut ctx.accounts.merchant_info;
    merchant_info.add_sales(sales_amount)?;

    Ok(())
}

// 关闭商户账户
#[derive(Accounts)]
pub struct CloseMerchant<'info> {
    #[account(
        mut,
        close = beneficiary,
        seeds = [b"merchant_info", owner.key().as_ref()],
        bump,
        constraint = merchant_info.owner == owner.key() @ ErrorCode::Unauthorized
    )]
    pub merchant_info: Account<'info, Merchant>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,

    pub owner: Signer<'info>,
}

// 关闭商户账户实现
pub fn close_merchant(ctx: Context<CloseMerchant>, force: bool) -> Result<()> {
    let merchant_info = &ctx.accounts.merchant_info;

    // 检查是否还有活跃产品（除非强制删除）
    if !force {
        require!(
            merchant_info.product_count == 0,
            ErrorCode::MerchantHasActiveProducts
        );
    }

    msg!(
        "商户账户已关闭，商户: {}, 强制删除: {}",
        merchant_info.owner,
        force
    );

    // 账户将通过close约束自动关闭并回收租金
    Ok(())
}

// 事件定义
#[event]
pub struct MerchantRegisteredAtomic {
    pub merchant: Pubkey,
    pub merchant_id: u32,
    pub name: String,
    pub initial_id_range_start: u64,
    pub initial_id_range_end: u64,
}

// ==================== 完整商户注册功能（包含ID块分配） ====================

/// 原子性商户注册账户结构（包含ID块分配）
#[derive(Accounts)]
#[instruction(name: String, description: String)]
pub struct RegisterMerchantAtomic<'info> {
    pub merchant: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"global_id_root"],
        bump
    )]
    pub global_root: Account<'info, GlobalIdRoot>,

    #[account(
        init,
        payer = payer,
        space = 8 + Merchant::INIT_SPACE,
        seeds = [b"merchant_info", merchant.key().as_ref()],
        bump
    )]
    pub merchant_info: Account<'info, Merchant>,

    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, crate::SystemConfig>,

    #[account(
        init,
        payer = payer,
        space = 8 + MerchantIdAccount::INIT_SPACE,
        seeds = [b"merchant_id", merchant.key().as_ref()],
        bump
    )]
    pub merchant_id_account: Account<'info, MerchantIdAccount>,

    #[account(
        init,
        payer = payer,
        space = 8 + IdChunk::INIT_SPACE,
        seeds = [
            b"id_chunk",
            merchant.key().as_ref(),
            &[0u8] // chunk_index = 0
        ],
        bump
    )]
    pub initial_chunk: Account<'info, IdChunk>,

    pub system_program: Program<'info, System>,
}

/// 原子性商户注册函数（包含ID块分配）
pub fn register_merchant_atomic(
    ctx: Context<RegisterMerchantAtomic>,
    name: String,
    description: String,
) -> Result<()> {
    // 获取初始块的key（在借用之前）
    let initial_chunk_key = ctx.accounts.initial_chunk.key();

    let global_root = &mut ctx.accounts.global_root;
    let merchant_info = &mut ctx.accounts.merchant_info;
    let merchant_id_account = &mut ctx.accounts.merchant_id_account;
    let initial_chunk = &mut ctx.accounts.initial_chunk;

    // 1. 分配商户ID
    global_root.last_merchant_id += 1;
    let merchant_id = global_root.last_merchant_id;

    // 2. 初始化商户信息账户
    let merchant_info_bump = ctx.bumps.merchant_info;
    let system_config = &ctx.accounts.system_config;
    merchant_info.initialize(
        ctx.accounts.merchant.key(),
        name.clone(),
        description,
        system_config.deposit_token_mint,
        merchant_info_bump,
    )?;

    // 3. 初始化第一个ID块 - 使用基于商户ID的范围
    let merchant_start_id = merchant_id as u64 * 10000; // 每个商户预留10000个ID
    initial_chunk.merchant_id = merchant_id;
    initial_chunk.chunk_index = 0;
    initial_chunk.start_id = merchant_start_id;
    initial_chunk.end_id = merchant_start_id + global_root.chunk_size as u64 - 1;
    initial_chunk.next_available = 0;

    // 安全初始化bitmap（使用Vec<u8>避免栈溢出）
    initial_chunk.initialize_bitmap();
    initial_chunk.bump = ctx.bumps.initial_chunk;

    // 4. 初始化商户ID分配账户
    merchant_id_account.merchant_id = merchant_id;
    merchant_id_account.last_chunk_index = 0;
    merchant_id_account.last_local_id = 0;
    merchant_id_account.active_chunk = initial_chunk_key;
    merchant_id_account.unused_chunks = Vec::new();
    merchant_id_account.bump = ctx.bumps.merchant_id_account;

    // 5. 更新全局状态
    global_root.last_global_id = initial_chunk.end_id + 1;
    global_root
        .merchants
        .push(ctx.accounts.merchant_id_account.key());

    // 发射事件
    emit!(MerchantRegisteredAtomic {
        merchant: ctx.accounts.merchant.key(),
        merchant_id,
        name: merchant_info.name.clone(),
        initial_id_range_start: initial_chunk.start_id,
        initial_id_range_end: initial_chunk.end_id,
    });

    msg!(
        "完整商户注册成功，ID: {}, 名称: {}, 初始ID范围: {} - {}",
        merchant_id,
        name,
        initial_chunk.start_id,
        initial_chunk.end_id
    );

    Ok(())
}
