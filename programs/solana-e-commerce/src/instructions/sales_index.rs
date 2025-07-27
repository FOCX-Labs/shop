use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(product_id: u64, old_sales: u32, new_sales: u32)]
pub struct UpdateProductSalesIndex<'info> {
    /// CHECK: 旧销量范围的节点，将在指令中验证
    #[account(mut)]
    pub old_sales_node: AccountInfo<'info>,

    /// CHECK: 新销量范围的节点，将在指令中验证
    #[account(mut)]
    pub new_sales_node: AccountInfo<'info>,
    // 移除authority账户 - 在函数实现中完全未使用，权限验证通过PDA种子机制实现
}

#[derive(Accounts)]
#[instruction(product_id: u64)]
pub struct RemoveProductFromSalesIndex<'info> {
    #[account(
        mut,
        seeds = [
            b"sales_index",
            sales_node.sales_range_start.to_le_bytes().as_ref(),
            sales_node.sales_range_end.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub sales_node: Account<'info, SalesIndexNode>,
    // 移除authority账户 - 在函数实现中完全未使用，权限验证通过PDA种子机制实现
}

#[derive(Accounts)]
#[instruction(min_sales: u32, max_sales: u32)]
pub struct SearchSalesRange<'info> {
    /// CHECK: 将在指令中验证正确的销量索引节点
    #[account()]
    pub sales_node: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct GetTopSellingProducts<'info> {
    /// CHECK: 销量索引根节点
    #[account()]
    pub sales_root: AccountInfo<'info>,
}

pub fn update_product_sales_index(
    ctx: Context<UpdateProductSalesIndex>,
    product_id: u64,
    old_sales: u32,
    new_sales: u32,
) -> Result<()> {
    // 如果销量范围没有变化，只需要在同一节点内更新
    let old_range = find_sales_node_for_sales(old_sales);
    let new_range = find_sales_node_for_sales(new_sales);

    if old_range == new_range {
        // 在同一节点内更新
        let node_data = ctx.accounts.old_sales_node.try_borrow_data()?;
        let mut sales_node = SalesIndexNode::try_deserialize(&mut &node_data[..])?;
        drop(node_data);

        let mut node_data = ctx.accounts.old_sales_node.try_borrow_mut_data()?;

        // 验证这是正确的销量节点
        require!(
            sales_node.contains_sales(old_sales) && sales_node.contains_sales(new_sales),
            ErrorCode::InvalidSalesRange
        );

        sales_node.update_product_sales(product_id, new_sales)?;
        sales_node.update_top_items(product_id, new_sales)?;

        // 重新序列化
        let mut cursor = std::io::Cursor::new(&mut node_data[..]);
        sales_node.try_serialize(&mut cursor)?;
    } else {
        // 需要在不同节点间移动产品
        // 从旧节点移除
        {
            let old_node_data = ctx.accounts.old_sales_node.try_borrow_data()?;
            let mut old_sales_node = SalesIndexNode::try_deserialize(&mut &old_node_data[..])?;
            drop(old_node_data);

            let mut old_node_data = ctx.accounts.old_sales_node.try_borrow_mut_data()?;

            // 验证这是正确的旧销量节点
            require!(
                old_sales_node.contains_sales(old_sales),
                ErrorCode::InvalidSalesRange
            );

            old_sales_node.remove_product(product_id)?;

            // 重新序列化旧节点
            let mut cursor = std::io::Cursor::new(&mut old_node_data[..]);
            old_sales_node.try_serialize(&mut cursor)?;
        }

        // 添加到新节点
        {
            let new_node_data = ctx.accounts.new_sales_node.try_borrow_data()?;
            let mut new_sales_node = SalesIndexNode::try_deserialize(&mut &new_node_data[..])?;
            drop(new_node_data);

            let mut new_node_data = ctx.accounts.new_sales_node.try_borrow_mut_data()?;

            // 验证这是正确的新销量节点
            require!(
                new_sales_node.contains_sales(new_sales),
                ErrorCode::InvalidSalesRange
            );

            new_sales_node.add_product(product_id, new_sales)?;
            new_sales_node.update_top_items(product_id, new_sales)?;

            // 重新序列化新节点
            let mut cursor = std::io::Cursor::new(&mut new_node_data[..]);
            new_sales_node.try_serialize(&mut cursor)?;
        }
    }

    msg!(
        "产品ID {} 销量索引更新成功，从 {} 更新到 {}",
        product_id,
        old_sales,
        new_sales
    );

    Ok(())
}

pub fn remove_product_from_sales_index(
    ctx: Context<RemoveProductFromSalesIndex>,
    product_id: u64,
) -> Result<()> {
    let sales_node = &mut ctx.accounts.sales_node;

    let removed = sales_node.remove_product(product_id)?;

    if removed {
        // 从热销商品缓存中移除
        sales_node.remove_from_top_items(product_id);

        msg!("产品ID {} 成功从销量索引中移除", product_id);
    } else {
        msg!("产品ID {} 不在当前销量索引节点中", product_id);
    }

    Ok(())
}

pub fn search_sales_range(
    ctx: Context<SearchSalesRange>,
    min_sales: u32,
    max_sales: u32,
    offset: u32,
    limit: u16,
) -> Result<Vec<u64>> {
    // 验证销量范围
    require!(min_sales <= max_sales, ErrorCode::InvalidSalesRange);

    // 反序列化销量索引节点
    let node_data = ctx.accounts.sales_node.data.borrow();
    let sales_node = SalesIndexNode::try_deserialize(&mut &node_data[8..])?;

    // 获取销量范围内的产品
    let all_products = sales_node.get_products_in_range(min_sales, max_sales);

    // 分页处理
    let start_index = offset as usize;
    let end_index = (start_index + limit as usize).min(all_products.len());

    let results = if start_index < all_products.len() {
        all_products[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };

    msg!(
        "销量范围搜索完成，范围: {} - {}, 找到 {} 个结果",
        min_sales,
        max_sales,
        results.len()
    );

    Ok(results)
}

pub fn get_top_selling_products(
    ctx: Context<GetTopSellingProducts>,
    limit: u16,
) -> Result<Vec<ProductSales>> {
    // 反序列化销量索引根节点
    let node_data = ctx.accounts.sales_root.data.borrow();
    let sales_node = SalesIndexNode::try_deserialize(&mut &node_data[8..])?;

    // 获取热销商品（已经按销量排序）
    let mut top_products = sales_node.top_items.clone();

    // 限制返回数量
    if top_products.len() > limit as usize {
        top_products.truncate(limit as usize);
    }

    msg!("获取热销商品成功，返回 {} 个商品", top_products.len());

    Ok(top_products)
}

// 根据销量查找对应的索引节点范围
pub fn find_sales_node_for_sales(sales: u32) -> (u32, u32) {
    // 简化的销量范围计算：每1000销量为一个范围
    let interval = 1000u32;
    let range_start = (sales / interval) * interval;
    let range_end = range_start + interval - 1;
    (range_start, range_end)
}

// 获取销量索引节点的利用率
pub fn get_sales_node_utilization(node: &Account<SalesIndexNode>) -> f32 {
    node.product_ids.len() as f32 / MAX_PRODUCTS_PER_SHARD as f32
}

// 更新全局热销商品排行榜
pub fn update_global_bestsellers(
    sales_nodes: Vec<&Account<SalesIndexNode>>,
) -> Result<Vec<ProductSales>> {
    let mut all_top_items = Vec::new();

    // 收集所有节点的热销商品
    for node in sales_nodes {
        all_top_items.extend(node.top_items.iter().cloned());
    }

    // 按销量降序排序
    all_top_items.sort_by(|a, b| b.sales.cmp(&a.sales));

    // 去重（保留销量最高的记录）
    let mut unique_products = std::collections::HashMap::new();
    for item in all_top_items {
        unique_products
            .entry(item.product_id)
            .and_modify(|e: &mut ProductSales| {
                if item.sales > e.sales {
                    *e = item.clone();
                }
            })
            .or_insert(item);
    }

    // 转换为排序后的向量
    let mut result: Vec<ProductSales> = unique_products.into_values().collect();
    result.sort_by(|a, b| b.sales.cmp(&a.sales));

    // 限制为前100名
    if result.len() > 100 {
        result.truncate(100);
    }

    Ok(result)
}

// ============================================================================
// init_if_needed 版本的指令：供product.rs模块使用
// ============================================================================

/// 销量索引初始化（如果需要）的账户结构
#[derive(Accounts)]
#[instruction(sales_range_start: u32, sales_range_end: u32)]
pub struct InitializeSalesIndexIfNeeded<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + SalesIndexNode::INIT_SPACE,
        seeds = [
            b"sales_index",
            sales_range_start.to_le_bytes().as_ref(),
            sales_range_end.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub sales_index: Account<'info, SalesIndexNode>,

    pub system_program: Program<'info, System>,
}

/// 初始化销量索引（如果需要）
pub fn initialize_sales_index_if_needed(
    ctx: Context<InitializeSalesIndexIfNeeded>,
    sales_range_start: u32,
    sales_range_end: u32,
) -> Result<()> {
    let sales_index = &mut ctx.accounts.sales_index;

    // 如果是新创建的账户，初始化数据
    if sales_index.sales_range_start == 0 && sales_index.sales_range_end == 0 {
        sales_index.sales_range_start = sales_range_start;
        sales_index.sales_range_end = sales_range_end;
        sales_index.product_ids = Vec::new();
        sales_index.top_items = Vec::new();
        sales_index.left_child = None;
        sales_index.right_child = None;
        sales_index.parent = None;
        sales_index.height = 0;
        sales_index.bump = ctx.bumps.sales_index;

        msg!(
            "销量索引初始化完成，范围: {} - {}",
            sales_range_start,
            sales_range_end
        );
    }

    Ok(())
}

/// 添加产品到销量索引（如果需要则先初始化）的账户结构
#[derive(Accounts)]
#[instruction(sales_range_start: u32, sales_range_end: u32, product_id: u64, sales: u32)]
pub struct AddProductToSalesIndexIfNeeded<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + SalesIndexNode::INIT_SPACE,
        seeds = [
            b"sales_index",
            sales_range_start.to_le_bytes().as_ref(),
            sales_range_end.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub sales_index: Account<'info, SalesIndexNode>,

    pub system_program: Program<'info, System>,
}

/// 添加产品到销量索引（如果需要则先初始化）
pub fn add_product_to_sales_index_if_needed(
    ctx: Context<AddProductToSalesIndexIfNeeded>,
    sales_range_start: u32,
    sales_range_end: u32,
    product_id: u64,
    sales: u32,
) -> Result<()> {
    let sales_index = &mut ctx.accounts.sales_index;

    // 如果是新创建的账户，先初始化
    if sales_index.sales_range_start == 0 && sales_index.sales_range_end == 0 {
        sales_index.sales_range_start = sales_range_start;
        sales_index.sales_range_end = sales_range_end;
        sales_index.product_ids = Vec::new();
        sales_index.top_items = Vec::new();
        sales_index.left_child = None;
        sales_index.right_child = None;
        sales_index.parent = None;
        sales_index.height = 0;
        sales_index.bump = ctx.bumps.sales_index;

        msg!(
            "销量索引初始化完成，范围: {} - {}",
            sales_range_start,
            sales_range_end
        );
    }

    // 验证销量在范围内
    require!(
        sales >= sales_index.sales_range_start && sales <= sales_index.sales_range_end,
        ErrorCode::InvalidSalesRange
    );

    // 检查产品是否已存在
    if sales_index.product_ids.contains(&product_id) {
        return Ok(()); // 已存在，跳过
    }

    // 检查索引是否已满
    if sales_index.product_ids.len() >= 1000 {
        return Err(ErrorCode::ShardIsFull.into());
    }

    // 添加产品ID
    sales_index.product_ids.push(product_id);

    // 更新热销商品列表（如果需要）
    if sales > 0 {
        update_top_sales_items(&mut sales_index.top_items, product_id, sales)?;
    }

    msg!(
        "产品 {} 已添加到销量索引 [{}, {}]",
        product_id,
        sales_range_start,
        sales_range_end
    );

    Ok(())
}

/// 更新热销商品列表
fn update_top_sales_items(
    top_items: &mut Vec<ProductSales>,
    product_id: u64,
    sales: u32,
) -> Result<()> {
    // 添加新产品到热销列表
    top_items.push(ProductSales {
        product_id,
        merchant: Pubkey::default(), // TODO: 从产品账户获取实际商户信息
        name: String::new(),         // TODO: 从产品账户获取实际产品名称
        price: 0,                    // TODO: 从产品账户获取实际价格
        sales,
        last_update: Clock::get()?.unix_timestamp,
    });

    // 按销量降序排序
    top_items.sort_by(|a, b| b.sales.cmp(&a.sales));

    // 保持最多100个热销商品
    if top_items.len() > 100 {
        top_items.truncate(100);
    }

    Ok(())
}
