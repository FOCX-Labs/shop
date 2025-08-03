use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(product_id: u64, old_sales: u32, new_sales: u32)]
pub struct UpdateProductSalesIndex<'info> {
    /// CHECK: Node for old sales range, will be verified in instruction
    #[account(mut)]
    pub old_sales_node: AccountInfo<'info>,

    /// CHECK: Node for new sales range, will be verified in instruction
    #[account(mut)]
    pub new_sales_node: AccountInfo<'info>,
    // Remove authority account - completely unused in function implementation, permission verification through PDA seed mechanism
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
    // Remove authority account - completely unused in function implementation, permission verification through PDA seed mechanism
}

#[derive(Accounts)]
#[instruction(min_sales: u32, max_sales: u32)]
pub struct SearchSalesRange<'info> {
    /// CHECK: Will verify correct sales index node in instruction
    #[account()]
    pub sales_node: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct GetTopSellingProducts<'info> {
    /// CHECK: Sales index root node
    #[account()]
    pub sales_root: AccountInfo<'info>,
}

pub fn update_product_sales_index(
    ctx: Context<UpdateProductSalesIndex>,
    product_id: u64,
    old_sales: u32,
    new_sales: u32,
) -> Result<()> {
    // If sales range hasn't changed, only need to update within the same node
    let old_range = find_sales_node_for_sales(old_sales);
    let new_range = find_sales_node_for_sales(new_sales);

    if old_range == new_range {
        // Update within the same node
        let node_data = ctx.accounts.old_sales_node.try_borrow_data()?;
        let mut sales_node = SalesIndexNode::try_deserialize(&mut &node_data[..])?;
        drop(node_data);

        let mut node_data = ctx.accounts.old_sales_node.try_borrow_mut_data()?;

        // Verify this is the correct sales node
        require!(
            sales_node.contains_sales(old_sales) && sales_node.contains_sales(new_sales),
            ErrorCode::InvalidSalesRange
        );

        sales_node.update_product_sales(product_id, new_sales)?;
        sales_node.update_top_items(product_id, new_sales)?;

        // Re-serialize
        let mut cursor = std::io::Cursor::new(&mut node_data[..]);
        sales_node.try_serialize(&mut cursor)?;
    } else {
        // Need to move product between different nodes
        // Remove from old node
        {
            let old_node_data = ctx.accounts.old_sales_node.try_borrow_data()?;
            let mut old_sales_node = SalesIndexNode::try_deserialize(&mut &old_node_data[..])?;
            drop(old_node_data);

            let mut old_node_data = ctx.accounts.old_sales_node.try_borrow_mut_data()?;

            // Verify this is the correct old sales node
            require!(
                old_sales_node.contains_sales(old_sales),
                ErrorCode::InvalidSalesRange
            );

            old_sales_node.remove_product(product_id)?;

            // Re-serialize old node
            let mut cursor = std::io::Cursor::new(&mut old_node_data[..]);
            old_sales_node.try_serialize(&mut cursor)?;
        }

        // Add to new node
        {
            let new_node_data = ctx.accounts.new_sales_node.try_borrow_data()?;
            let mut new_sales_node = SalesIndexNode::try_deserialize(&mut &new_node_data[..])?;
            drop(new_node_data);

            let mut new_node_data = ctx.accounts.new_sales_node.try_borrow_mut_data()?;

            // Verify this is the correct new sales node
            require!(
                new_sales_node.contains_sales(new_sales),
                ErrorCode::InvalidSalesRange
            );

            new_sales_node.add_product(product_id, new_sales)?;
            new_sales_node.update_top_items(product_id, new_sales)?;

            // Re-serialize new node
            let mut cursor = std::io::Cursor::new(&mut new_node_data[..]);
            new_sales_node.try_serialize(&mut cursor)?;
        }
    }

    msg!(
        "Product ID {} sales index update successful, updated from {} to {}",
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
        // Remove from bestselling products cache
        sales_node.remove_from_top_items(product_id);

        msg!(
            "Product ID {} successfully removed from sales index",
            product_id
        );
    } else {
        msg!("Product ID {} not in current sales index node", product_id);
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
    // Verify sales range
    require!(min_sales <= max_sales, ErrorCode::InvalidSalesRange);

    // Deserialize sales index node
    let node_data = ctx.accounts.sales_node.data.borrow();
    let sales_node = SalesIndexNode::try_deserialize(&mut &node_data[8..])?;

    // Get products within sales range
    let all_products = sales_node.get_products_in_range(min_sales, max_sales);

    // Pagination processing
    let start_index = offset as usize;
    let end_index = (start_index + limit as usize).min(all_products.len());

    let results = if start_index < all_products.len() {
        all_products[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };

    msg!(
        "Sales range search completed, range: {} - {}, found {} results",
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
    // Deserialize sales index root node
    let node_data = ctx.accounts.sales_root.data.borrow();
    let sales_node = SalesIndexNode::try_deserialize(&mut &node_data[8..])?;

    // Get bestselling products (already sorted by sales)
    let mut top_products = sales_node.top_items.clone();

    // Limit return quantity
    if top_products.len() > limit as usize {
        top_products.truncate(limit as usize);
    }

    msg!("Successfully retrieved bestselling products, returned {} products", top_products.len());

    Ok(top_products)
}

// Find corresponding index node range by sales volume
pub fn find_sales_node_for_sales(sales: u32) -> (u32, u32) {
    // Simplified sales range calculation: each 1000 sales as a range
    let interval = 1000u32;
    let range_start = (sales / interval) * interval;
    let range_end = range_start + interval - 1;
    (range_start, range_end)
}

// Get sales index node utilization
pub fn get_sales_node_utilization(node: &Account<SalesIndexNode>) -> f32 {
    node.product_ids.len() as f32 / MAX_PRODUCTS_PER_SHARD as f32
}

// Update global bestseller product rankings
pub fn update_global_bestsellers(
    sales_nodes: Vec<&Account<SalesIndexNode>>,
) -> Result<Vec<ProductSales>> {
    let mut all_top_items = Vec::new();

    // Collect bestselling products from all nodes
    for node in sales_nodes {
        all_top_items.extend(node.top_items.iter().cloned());
    }

    // Sort by sales in descending order
    all_top_items.sort_by(|a, b| b.sales.cmp(&a.sales));

    // Deduplicate (keep records with highest sales)
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

    // Convert to sorted vector
    let mut result: Vec<ProductSales> = unique_products.into_values().collect();
    result.sort_by(|a, b| b.sales.cmp(&a.sales));

    // Limit to top 100
    if result.len() > 100 {
        result.truncate(100);
    }

    Ok(result)
}

// ============================================================================
// init_if_needed version instructions: for use by product.rs module
// ============================================================================

/// Account structure for sales index initialization (if needed)
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

/// Initialize sales index (if needed)
pub fn initialize_sales_index_if_needed(
    ctx: Context<InitializeSalesIndexIfNeeded>,
    sales_range_start: u32,
    sales_range_end: u32,
) -> Result<()> {
    let sales_index = &mut ctx.accounts.sales_index;

    // If it's a newly created account, initialize data
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
            "Sales index initialization completed, range: {} - {}",
            sales_range_start,
            sales_range_end
        );
    }

    Ok(())
}

/// Account structure for adding product to sales index (initialize first if needed)
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

/// Add product to sales index (initialize first if needed)
pub fn add_product_to_sales_index_if_needed(
    ctx: Context<AddProductToSalesIndexIfNeeded>,
    sales_range_start: u32,
    sales_range_end: u32,
    product_id: u64,
    sales: u32,
) -> Result<()> {
    let sales_index = &mut ctx.accounts.sales_index;

    // If it's a newly created account, initialize first
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
            "Sales index initialization completed, range: {} - {}",
            sales_range_start,
            sales_range_end
        );
    }

    // Verify sales within range
    require!(
        sales >= sales_index.sales_range_start && sales <= sales_index.sales_range_end,
        ErrorCode::InvalidSalesRange
    );

    // Check if product already exists
    if sales_index.product_ids.contains(&product_id) {
        return Ok(()); // Already exists, skip
    }

    // Check if index is full
    if sales_index.product_ids.len() >= 1000 {
        return Err(ErrorCode::ShardIsFull.into());
    }

    // Add product ID
    sales_index.product_ids.push(product_id);

    // Update bestselling products list (if needed)
    if sales > 0 {
        update_top_sales_items(&mut sales_index.top_items, product_id, sales)?;
    }

    msg!(
        "Product {} added to sales index [{}, {}]",
        product_id,
        sales_range_start,
        sales_range_end
    );

    Ok(())
}

/// Update bestselling products list
fn update_top_sales_items(
    top_items: &mut Vec<ProductSales>,
    product_id: u64,
    sales: u32,
) -> Result<()> {
    // Add new product to bestselling list
    top_items.push(ProductSales {
        product_id,
        merchant: Pubkey::default(), // TODO: Get actual merchant info from product account
        name: String::new(),         // TODO: Get actual product name from product account
        price: 0,                    // TODO: Get actual price from product account
        sales,
        last_update: Clock::get()?.unix_timestamp,
    });

    // Sort by sales in descending order
    top_items.sort_by(|a, b| b.sales.cmp(&a.sales));

    // Keep at most 100 bestselling products
    if top_items.len() > 100 {
        top_items.truncate(100);
    }

    Ok(())
}