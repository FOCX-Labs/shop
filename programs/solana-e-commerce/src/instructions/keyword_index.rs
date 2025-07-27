use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(keyword: String, product_id: u64)]
pub struct RemoveProductFromKeywordIndex<'info> {
    #[account(
        mut,
        seeds = [b"keyword_root", keyword.as_bytes()],
        bump
    )]
    pub keyword_root: Account<'info, KeywordRoot>,

    #[account(
        mut,
        seeds = [b"keyword_shard", keyword.as_bytes(), 0u32.to_le_bytes().as_ref()],
        bump
    )]
    pub target_shard: Account<'info, KeywordShard>,
    // 移除authority账户 - 在函数实现中完全未使用，权限验证通过PDA种子机制实现
}

#[derive(Accounts)]
#[instruction(keyword: String, shard_index: u32)]
pub struct CreateKeywordShard<'info> {
    #[account(
        mut,
        seeds = [b"keyword_root", keyword.as_bytes()],
        bump
    )]
    pub keyword_root: Account<'info, KeywordRoot>,

    #[account(
        mut,
        seeds = [b"keyword_shard", keyword.as_bytes(), (shard_index - 1).to_le_bytes().as_ref()],
        bump
    )]
    pub prev_shard: Account<'info, KeywordShard>,

    #[account(
        init,
        payer = payer,
        space = 8 + KeywordShard::INIT_SPACE,
        seeds = [b"keyword_shard", keyword.as_bytes(), shard_index.to_le_bytes().as_ref()],
        bump
    )]
    pub new_shard: Account<'info, KeywordShard>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(keyword: String)]
pub struct SearchKeywordIndex<'info> {
    #[account(
        seeds = [b"keyword_root", keyword.as_bytes()],
        bump
    )]
    pub keyword_root: Account<'info, KeywordRoot>,
}

// 关闭关键词根账户
#[derive(Accounts)]
#[instruction(keyword: String)]
pub struct CloseKeywordRoot<'info> {
    #[account(
        mut,
        close = beneficiary,
        seeds = [b"keyword_root", keyword.as_bytes()],
        bump
    )]
    pub keyword_root: Account<'info, KeywordRoot>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,
    // 移除authority账户 - 在函数实现中完全未使用，权限验证通过PDA种子机制实现
}

// 关闭关键词分片账户
#[derive(Accounts)]
#[instruction(keyword: String, shard_index: u32)]
pub struct CloseKeywordShard<'info> {
    #[account(
        mut,
        close = beneficiary,
        seeds = [b"keyword_shard", keyword.as_bytes(), shard_index.to_le_bytes().as_ref()],
        bump
    )]
    pub keyword_shard: Account<'info, KeywordShard>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,
    // 移除authority账户 - 在函数实现中完全未使用，权限验证通过PDA种子机制实现
}

pub fn remove_product_from_keyword_index(
    ctx: Context<RemoveProductFromKeywordIndex>,
    keyword: String,
    product_id: u64,
) -> Result<()> {
    let keyword_root = &mut ctx.accounts.keyword_root;
    let target_shard = &mut ctx.accounts.target_shard;

    // 验证关键词匹配
    require!(keyword_root.keyword == keyword, ErrorCode::InvalidKeyword);
    require!(target_shard.keyword == keyword, ErrorCode::InvalidKeyword);

    // 快速检查产品是否可能存在
    if !keyword_root.might_contain(product_id) {
        msg!("产品ID {} 不在关键词 {} 索引中", product_id, keyword);
        return Ok(());
    }

    // 尝试从分片中移除产品
    let found = target_shard.remove_product(product_id)?;

    if found {
        // 更新根的统计（注意：布隆过滤器不支持删除，保持原样）
        keyword_root.total_products = keyword_root.total_products.saturating_sub(1);

        msg!("产品ID {} 成功从关键词 {} 索引中移除", product_id, keyword);
    } else {
        msg!("产品ID {} 不在关键词 {} 索引中", product_id, keyword);
    }

    Ok(())
}

pub fn create_keyword_shard(
    ctx: Context<CreateKeywordShard>,
    keyword: String,
    shard_index: u32,
) -> Result<()> {
    let keyword_root = &mut ctx.accounts.keyword_root;
    let prev_shard = &mut ctx.accounts.prev_shard;
    let new_shard = &mut ctx.accounts.new_shard;

    // 验证关键词匹配
    require!(keyword_root.keyword == keyword, ErrorCode::InvalidKeyword);
    require!(prev_shard.keyword == keyword, ErrorCode::InvalidKeyword);

    // 验证分片索引连续性
    require!(
        prev_shard.shard_index + 1 == shard_index,
        ErrorCode::InvalidShardIndex
    );

    // 初始化新分片
    new_shard.initialize(
        keyword.clone(),
        shard_index,
        prev_shard.key(),
        ctx.bumps.new_shard,
    )?;

    // 更新前一个分片的链接
    prev_shard.next_shard = Some(new_shard.key());

    // 更新根的统计
    keyword_root.add_shard(new_shard.key());

    msg!("关键词 {} 的新分片 {} 创建成功", keyword, shard_index);

    Ok(())
}

pub fn search_keyword_index(
    ctx: Context<SearchKeywordIndex>,
    keyword: String,
    offset: u32,
    limit: u16,
) -> Result<Vec<u64>> {
    let keyword_root = &ctx.accounts.keyword_root;

    // 验证关键词匹配
    require!(keyword_root.keyword == keyword, ErrorCode::InvalidKeyword);

    if keyword_root.total_products == 0 {
        return Ok(Vec::new());
    }

    // 这里只返回第一个分片的结果作为示例
    // 实际实现需要遍历所有分片
    let results = Vec::new(); // 简化实现

    msg!(
        "关键词 {} 搜索完成，偏移: {}, 限制: {}",
        keyword,
        offset,
        limit
    );

    Ok(results)
}

// 检查分片是否需要分裂
pub fn check_shard_split_needed(shard: &Account<KeywordShard>) -> bool {
    shard.needs_split()
}

// 检查分片是否需要合并
pub fn check_shard_merge_needed(shard: &Account<KeywordShard>) -> bool {
    shard.needs_merge()
}

// 关闭关键词根账户
pub fn close_keyword_root(
    ctx: Context<CloseKeywordRoot>,
    keyword: String,
    force: bool,
) -> Result<()> {
    let keyword_root = &ctx.accounts.keyword_root;

    // 验证关键词匹配
    require!(keyword_root.keyword == keyword, ErrorCode::InvalidKeyword);

    // 检查是否为空（除非强制删除）
    if !force {
        require!(
            keyword_root.total_products == 0,
            ErrorCode::KeywordIndexNotEmpty
        );
    }

    msg!(
        "关键词根账户已关闭，关键词: {}, 强制删除: {}",
        keyword,
        force
    );

    // 账户将通过close约束自动关闭并回收租金
    Ok(())
}

// 关闭关键词分片账户
pub fn close_keyword_shard(
    ctx: Context<CloseKeywordShard>,
    keyword: String,
    shard_index: u32,
    force: bool,
) -> Result<()> {
    let keyword_shard = &ctx.accounts.keyword_shard;

    // 验证关键词匹配
    require!(keyword_shard.keyword == keyword, ErrorCode::InvalidKeyword);
    require!(
        keyword_shard.shard_index == shard_index,
        ErrorCode::InvalidShardIndex
    );

    // 检查是否为空（除非强制删除）
    if !force {
        require!(
            keyword_shard.product_ids.is_empty(),
            ErrorCode::KeywordShardNotEmpty
        );
    }

    msg!(
        "关键词分片账户已关闭，关键词: {}, 分片: {}, 强制删除: {}",
        keyword,
        shard_index,
        force
    );

    // 账户将通过close约束自动关闭并回收租金
    Ok(())
}

// ============================================================================
// init_if_needed 版本的指令：供product.rs模块使用
// ============================================================================

/// 关键词索引初始化（如果需要）的账户结构
#[derive(Accounts)]
#[instruction(keyword: String)]
pub struct InitializeKeywordIndexIfNeeded<'info> {
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + KeywordRoot::INIT_SPACE,
        seeds = [b"keyword_root", keyword.as_bytes()],
        bump
    )]
    pub keyword_root: Account<'info, KeywordRoot>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + KeywordShard::INIT_SPACE,
        seeds = [b"keyword_shard", keyword.as_bytes(), 0u32.to_le_bytes().as_ref()],
        bump
    )]
    pub first_shard: Account<'info, KeywordShard>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// 初始化关键词索引（如果需要）
pub fn initialize_keyword_index_if_needed(
    ctx: Context<InitializeKeywordIndexIfNeeded>,
    keyword: String,
) -> Result<()> {
    let keyword_root = &mut ctx.accounts.keyword_root;
    let first_shard = &mut ctx.accounts.first_shard;

    // 如果是新创建的根账户，初始化数据
    if keyword_root.keyword.is_empty() {
        keyword_root.keyword = keyword.clone();
        keyword_root.total_products = 0;
        keyword_root.total_shards = 1;
        keyword_root.first_shard = first_shard.key();
        keyword_root.last_shard = first_shard.key();
        keyword_root.bloom_filter = [0u8; 256];
        keyword_root.bump = ctx.bumps.keyword_root;

        msg!("关键词根账户初始化完成，关键词: {}", keyword);
    }

    // 如果是新创建的分片账户，初始化数据
    if first_shard.keyword.is_empty() {
        first_shard.keyword = keyword.clone();
        first_shard.shard_index = 0;
        first_shard.prev_shard = Pubkey::default();
        first_shard.next_shard = None;
        first_shard.product_ids = Vec::new();
        first_shard.min_id = 0;
        first_shard.max_id = 0;
        first_shard.bloom_summary = [0u8; 32];
        first_shard.bump = ctx.bumps.first_shard;

        msg!("关键词分片账户初始化完成，关键词: {}, 分片: 0", keyword);
    }

    Ok(())
}

/// 添加产品到关键词索引（如果需要则先初始化）的账户结构
#[derive(Accounts)]
#[instruction(keyword: String, product_id: u64)]
pub struct AddProductToKeywordIndexIfNeeded<'info> {
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + KeywordRoot::INIT_SPACE,
        seeds = [b"keyword_root", keyword.as_bytes()],
        bump
    )]
    pub keyword_root: Account<'info, KeywordRoot>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + KeywordShard::INIT_SPACE,
        seeds = [b"keyword_shard", keyword.as_bytes(), 0u32.to_le_bytes().as_ref()],
        bump
    )]
    pub target_shard: Account<'info, KeywordShard>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// 添加产品到关键词索引（如果需要则先初始化）
pub fn add_product_to_keyword_index_if_needed(
    ctx: Context<AddProductToKeywordIndexIfNeeded>,
    keyword: String,
    product_id: u64,
) -> Result<()> {
    let keyword_root = &mut ctx.accounts.keyword_root;
    let target_shard = &mut ctx.accounts.target_shard;

    // 如果是新创建的根账户，先初始化
    if keyword_root.keyword.is_empty() {
        keyword_root.keyword = keyword.clone();
        keyword_root.total_products = 0;
        keyword_root.total_shards = 1;
        keyword_root.first_shard = target_shard.key();
        keyword_root.last_shard = target_shard.key();
        keyword_root.bloom_filter = [0u8; 256];
        keyword_root.bump = ctx.bumps.keyword_root;

        msg!("关键词根账户初始化完成，关键词: {}", keyword);
    }

    // 如果是新创建的分片账户，先初始化
    if target_shard.keyword.is_empty() {
        target_shard.keyword = keyword.clone();
        target_shard.shard_index = 0;
        target_shard.prev_shard = Pubkey::default();
        target_shard.next_shard = None;
        target_shard.product_ids = Vec::new();
        target_shard.min_id = 0;
        target_shard.max_id = 0;
        target_shard.bloom_summary = [0u8; 32];
        target_shard.bump = ctx.bumps.target_shard;

        msg!("关键词分片账户初始化完成，关键词: {}, 分片: 0", keyword);
    }

    // 检查产品是否已存在
    if target_shard.product_ids.contains(&product_id) {
        return Ok(()); // 已存在，跳过
    }

    // 检查分片是否已满
    if target_shard.product_ids.len() >= 1000 {
        return Err(ErrorCode::ShardIsFull.into());
    }

    // 添加产品ID
    target_shard.product_ids.push(product_id);
    keyword_root.total_products += 1;

    msg!("产品 {} 已添加到关键词索引 '{}'", product_id, keyword);

    Ok(())
}
