use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;



#[derive(Accounts)]
#[instruction(product_id: u64)]
pub struct RemoveProductFromPriceIndex<'info> {
    #[account(
        mut,
        seeds = [
            b"price_index",
            price_node.price_range_start.to_le_bytes().as_ref(),
            price_node.price_range_end.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub price_node: Account<'info, PriceIndexNode>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(min_price: u64, max_price: u64)]
pub struct SearchPriceRange<'info> {
    /// CHECK: 将在指令中验证正确的价格索引节点
    #[account()]
    pub price_node: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(price_range_start: u64, price_range_end: u64)]
pub struct SplitPriceNode<'info> {
    #[account(
        mut,
        seeds = [
            b"price_index",
            price_range_start.to_le_bytes().as_ref(),
            price_range_end.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub price_node: Account<'info, PriceIndexNode>,

    #[account(
        init,
        payer = payer,
        space = PriceIndexNode::LEN,
        seeds = [
            b"price_index",
            ((price_range_start + price_range_end) / 2 + 1).to_le_bytes().as_ref(),
            price_range_end.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub new_price_node: Account<'info, PriceIndexNode>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}



pub fn remove_product_from_price_index(
    ctx: Context<RemoveProductFromPriceIndex>,
    product_id: u64,
) -> Result<()> {
    let price_node = &mut ctx.accounts.price_node;

    let removed = price_node.remove_product(product_id)?;

    if removed {
        msg!("产品ID {} 成功从价格索引中移除", product_id);
    } else {
        msg!("产品ID {} 不在当前价格索引节点中", product_id);
    }

    Ok(())
}

pub fn search_price_range(
    ctx: Context<SearchPriceRange>,
    min_price: u64,
    max_price: u64,
    offset: u32,
    limit: u16,
) -> Result<Vec<u64>> {
    // 验证价格范围
    require!(min_price <= max_price, ErrorCode::InvalidPriceRange);

    // 反序列化价格索引节点
    let node_data = ctx.accounts.price_node.data.borrow();
    let price_node = PriceIndexNode::try_deserialize(&mut &node_data[8..])?;

    // 获取价格范围内的产品
    let all_products = price_node.get_products_in_range(min_price, max_price);

    // 分页处理
    let start_index = offset as usize;
    let end_index = (start_index + limit as usize).min(all_products.len());

    let results = if start_index < all_products.len() {
        all_products[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };

    msg!(
        "价格范围搜索完成，范围: {} - {}, 找到 {} 个结果",
        min_price,
        max_price,
        results.len()
    );

    Ok(results)
}

pub fn split_price_node(
    ctx: Context<SplitPriceNode>,
    price_range_start: u64,
    price_range_end: u64,
) -> Result<()> {
    let price_node = &mut ctx.accounts.price_node;
    let new_price_node = &mut ctx.accounts.new_price_node;

    // 计算分割点
    let split_point = (price_range_start + price_range_end) / 2;

    // 调整原节点范围
    price_node.price_range_end = split_point;

    // 初始化新节点
    new_price_node.initialize(split_point + 1, price_range_end, ctx.bumps.new_price_node)?;

    // 重新分配产品到对应节点
    let mut products_to_move = Vec::new();
    for &product_id in &price_node.product_ids.clone() {
        // 这里需要获取产品的实际价格来判断应该分配到哪个节点
        // 简化实现：假设产品ID的后几位代表价格区间
        let estimated_price = (product_id % 1000) + price_range_start;
        if estimated_price > split_point {
            products_to_move.push(product_id);
        }
    }

    // 移动产品到新节点
    for product_id in products_to_move {
        price_node.product_ids.retain(|&x| x != product_id);
        new_price_node.product_ids.push(product_id);
    }

    msg!(
        "价格索引节点分裂成功，原范围: {} - {}，新范围: {} - {}",
        price_range_start,
        split_point,
        split_point + 1,
        price_range_end
    );

    Ok(())
}

// 查找适当的价格索引节点
pub fn find_price_node_for_price(price: u64) -> (u64, u64) {
    // 简化实现：使用固定的价格区间划分
    // 实际实现应该遍历价格索引树找到合适的叶子节点
    let interval = 1000u64; // 每1000单位一个区间
    let range_start = (price / interval) * interval;
    let range_end = range_start + interval - 1;
    (range_start, range_end)
}

// 获取价格索引节点的利用率
pub fn get_price_node_utilization(node: &Account<PriceIndexNode>) -> f32 {
    node.product_ids.len() as f32 / MAX_PRODUCTS_PER_SHARD as f32
}

// 检查是否需要平衡价格索引树
pub fn should_rebalance_price_tree(node: &Account<PriceIndexNode>) -> bool {
    node.needs_split() || node.needs_merge()
}

// ============================================================================
// init_if_needed 版本的指令：供product.rs模块使用
// ============================================================================

/// 价格索引初始化（如果需要）的账户结构
#[derive(Accounts)]
#[instruction(price_range_start: u64, price_range_end: u64)]
pub struct InitializePriceIndexIfNeeded<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = PriceIndexNode::LEN,
        seeds = [
            b"price_index",
            price_range_start.to_le_bytes().as_ref(),
            price_range_end.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub price_index: Account<'info, PriceIndexNode>,

    pub system_program: Program<'info, System>,
}

/// 初始化价格索引（如果需要）
pub fn initialize_price_index_if_needed(
    ctx: Context<InitializePriceIndexIfNeeded>,
    price_range_start: u64,
    price_range_end: u64,
) -> Result<()> {
    let price_index = &mut ctx.accounts.price_index;

    // 如果是新创建的账户，初始化数据
    if price_index.price_range_start == 0 && price_index.price_range_end == 0 {
        price_index.price_range_start = price_range_start;
        price_index.price_range_end = price_range_end;
        price_index.product_ids = Vec::new();
        price_index.left_child = None;
        price_index.right_child = None;
        price_index.parent = None;
        price_index.height = 0;
        price_index.bump = ctx.bumps.price_index;

        msg!(
            "价格索引初始化完成，范围: {} - {}",
            price_range_start,
            price_range_end
        );
    }

    Ok(())
}

/// 添加产品到价格索引（如果需要则先初始化）的账户结构
#[derive(Accounts)]
#[instruction(price_range_start: u64, price_range_end: u64, product_id: u64, price: u64)]
pub struct AddProductToPriceIndexIfNeeded<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = PriceIndexNode::LEN,
        seeds = [
            b"price_index",
            price_range_start.to_le_bytes().as_ref(),
            price_range_end.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub price_index: Account<'info, PriceIndexNode>,

    pub system_program: Program<'info, System>,
}

/// 添加产品到价格索引（如果需要则先初始化）
pub fn add_product_to_price_index_if_needed(
    ctx: Context<AddProductToPriceIndexIfNeeded>,
    price_range_start: u64,
    price_range_end: u64,
    product_id: u64,
    price: u64,
) -> Result<()> {
    let price_index = &mut ctx.accounts.price_index;

    // 如果是新创建的账户，先初始化
    if price_index.price_range_start == 0 && price_index.price_range_end == 0 {
        price_index.price_range_start = price_range_start;
        price_index.price_range_end = price_range_end;
        price_index.product_ids = Vec::new();
        price_index.left_child = None;
        price_index.right_child = None;
        price_index.parent = None;
        price_index.height = 0;
        price_index.bump = ctx.bumps.price_index;

        msg!(
            "价格索引初始化完成，范围: {} - {}",
            price_range_start,
            price_range_end
        );
    }

    // 验证价格在范围内
    require!(
        price >= price_index.price_range_start && price <= price_index.price_range_end,
        ErrorCode::InvalidPriceRange
    );

    // 检查产品是否已存在
    if price_index.product_ids.contains(&product_id) {
        return Ok(()); // 已存在，跳过
    }

    // 检查索引是否已满
    if price_index.product_ids.len() >= 1000 {
        return Err(ErrorCode::ShardIsFull.into());
    }

    // 添加产品ID
    price_index.product_ids.push(product_id);

    msg!(
        "产品 {} 已添加到价格索引 [{}, {}]",
        product_id,
        price_range_start,
        price_range_end
    );

    Ok(())
}
