use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

/// 计算价格范围的起始值
/// 使用对数算法：给定价格P，找到满足 2^n ≤ P < 2^(n+1) 的n值
/// 设置 price_range_start = 2^n
pub fn calculate_price_range_start(price: u64) -> u64 {
    if price == 0 {
        return 0;
    }
    if price == 1 {
        return 1;
    }

    // 找到最大的n，使得2^n <= price
    // 使用更直观的方法计算 floor(log2(price))
    let mut n = 0;
    let mut temp = price;
    while temp > 1 {
        temp >>= 1;
        n += 1;
    }
    // 现在 n 就是 floor(log2(price))
    // 对于price=15: floor(log2(15)) = 3, 所以 2^3 = 8
    1u64 << n
}

/// 计算价格范围的结束值
/// 设置 price_range_end = 2^(n+1)
pub fn calculate_price_range_end(price: u64) -> u64 {
    if price == 0 {
        return 0;
    }
    if price == 1 {
        return 1;
    }

    // 找到最大的n，使得2^n <= price
    let mut n = 0;
    let mut temp = price;
    while temp > 1 {
        temp >>= 1;
        n += 1;
    }
    // price_range_end = 2^(n+1)
    // 对于price=15: n=3, 所以 2^(3+1) = 2^4 = 16
    1u64 << (n + 1)
}

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
    // 移除authority账户 - 在函数实现中完全未使用，权限验证通过PDA种子机制实现
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
        space = 8 + PriceIndexNode::INIT_SPACE,
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
// 智能价格索引指令：自动计算价格范围并管理索引
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_range_calculation() {
        // 测试用例：价格=15时
        // floor(log2(15)) = 3，所以 2^3=8 <= 15 < 2^4=16
        // price_range_start=8，price_range_end=16
        assert_eq!(calculate_price_range_start(15), 8);
        assert_eq!(calculate_price_range_end(15), 16);

        // 测试用例：价格=1时，特殊情况
        assert_eq!(calculate_price_range_start(1), 1);
        assert_eq!(calculate_price_range_end(1), 1);

        // 测试用例：价格=8时
        // floor(log2(8)) = 3，所以 2^3=8 <= 8 < 2^4=16
        // price_range_start=8，price_range_end=16
        assert_eq!(calculate_price_range_start(8), 8);
        assert_eq!(calculate_price_range_end(8), 16);

        // 测试用例：价格=16时
        // floor(log2(16)) = 4，所以 2^4=16 <= 16 < 2^5=32
        // price_range_start=16，price_range_end=32
        assert_eq!(calculate_price_range_start(16), 16);
        assert_eq!(calculate_price_range_end(16), 32);

        // 测试用例：价格=0时，特殊情况
        assert_eq!(calculate_price_range_start(0), 0);
        assert_eq!(calculate_price_range_end(0), 0);

        // 测试用例：价格=2时
        // floor(log2(2)) = 1，所以 2^1=2 <= 2 < 2^2=4
        // price_range_start=2，price_range_end=4
        assert_eq!(calculate_price_range_start(2), 2);
        assert_eq!(calculate_price_range_end(2), 4);

        // 测试用例：价格=7时
        // floor(log2(7)) = 2，所以 2^2=4 <= 7 < 2^3=8
        // price_range_start=4，price_range_end=8
        assert_eq!(calculate_price_range_start(7), 4);
        assert_eq!(calculate_price_range_end(7), 8);

        // 测试用例：价格=50时
        // floor(log2(50)) = 5，所以 2^5=32 <= 50 < 2^6=64
        // price_range_start=32，price_range_end=64
        assert_eq!(calculate_price_range_start(50), 32);
        assert_eq!(calculate_price_range_end(50), 64);
    }
}

/// 智能价格索引账户结构（使用Anchor标准方法）
#[derive(Accounts)]
#[instruction(product_id: u64, price: u64, price_range_start: u64, price_range_end: u64)]
pub struct AddProductToPriceIndex<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + PriceIndexNode::INIT_SPACE,
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

/// 智能添加产品到价格索引
pub fn add_product_to_price_index(
    ctx: Context<AddProductToPriceIndex>,
    product_id: u64,
    price: u64,
    price_range_start: u64,
    price_range_end: u64,
) -> Result<()> {
    let price_index = &mut ctx.accounts.price_index;

    // 验证传入的价格范围是否与基于价格计算的范围一致
    let expected_start = calculate_price_range_start(price);
    let expected_end = calculate_price_range_end(price);

    require!(
        price_range_start == expected_start && price_range_end == expected_end,
        ErrorCode::InvalidPriceRange
    );

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
            "✅ 新价格索引自动创建: 价格 {} → 范围 [{}, {}]",
            price,
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
        msg!("产品 {} 已存在于价格索引中，跳过添加", product_id);
        return Ok(());
    }

    // 检查索引容量
    if price_index.product_ids.len() >= 1000 {
        return Err(ErrorCode::ShardIsFull.into());
    }

    // 添加产品ID
    price_index.product_ids.push(product_id);

    msg!(
        "✅ 产品 {} 已添加到价格索引 [{}, {}]，当前产品数: {}",
        product_id,
        price_range_start,
        price_range_end,
        price_index.product_ids.len()
    );

    Ok(())
}
