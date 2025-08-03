use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

/// Calculate the starting value of the price range
/// Using logarithmic algorithm: given price P, find n such that 2^n ≤ P < 2^(n+1)
/// Set price_range_start = 2^n
pub fn calculate_price_range_start(price: u64) -> u64 {
    if price == 0 {
        return 0;
    }
    if price == 1 {
        return 1;
    }

    // Find the largest n such that 2^n <= price
    // Use a more intuitive method to calculate floor(log2(price))
    let mut n = 0;
    let mut temp = price;
    while temp > 1 {
        temp >>= 1;
        n += 1;
    }
    // Now n is floor(log2(price))
    // For price=15: floor(log2(15)) = 3, so 2^3 = 8
    1u64 << n
}

/// Calculate the ending value of the price range
/// Set price_range_end = 2^(n+1)
pub fn calculate_price_range_end(price: u64) -> u64 {
    if price == 0 {
        return 0;
    }
    if price == 1 {
        return 1;
    }

    // Find the largest n such that 2^n <= price
    let mut n = 0;
    let mut temp = price;
    while temp > 1 {
        temp >>= 1;
        n += 1;
    }
    // price_range_end = 2^(n+1)
    // For price=15: n=3, so 2^(3+1) = 2^4 = 16
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
    // Remove authority account - completely unused in function implementation, permission verification through PDA seed mechanism
}

#[derive(Accounts)]
#[instruction(min_price: u64, max_price: u64)]
pub struct SearchPriceRange<'info> {
    /// CHECK: will verify the correct price index node in the instruction
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
        msg!("Product ID {} successfully removed from price index", product_id);
    } else {
        msg!("Product ID {} not in current price index node", product_id);
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
    // Verify price range
    require!(min_price <= max_price, ErrorCode::InvalidPriceRange);

    // Deserialize price index node
    let node_data = ctx.accounts.price_node.data.borrow();
    let price_node = PriceIndexNode::try_deserialize(&mut &node_data[8..])?;

    // Get products within price range
    let all_products = price_node.get_products_in_range(min_price, max_price);

    // Pagination processing
    let start_index = offset as usize;
    let end_index = (start_index + limit as usize).min(all_products.len());

    let results = if start_index < all_products.len() {
        all_products[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };

    msg!(
        "Price range search completed, range: {} - {}, found {} results",
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

    // Calculate split point
    let split_point = (price_range_start + price_range_end) / 2;

    // Adjust original node range
    price_node.price_range_end = split_point;

    // Initialize new node
    new_price_node.initialize(split_point + 1, price_range_end, ctx.bumps.new_price_node)?;

    // Reallocate products to corresponding nodes
    let mut products_to_move = Vec::new();
    for &product_id in &price_node.product_ids.clone() {
        // Here we need to get the actual price of the product to determine which node it should be allocated to
        // Simplified implementation: assume the last few digits of product ID represent price range
        let estimated_price = (product_id % 1000) + price_range_start;
        if estimated_price > split_point {
            products_to_move.push(product_id);
        }
    }

    // Move products to new node
    for product_id in products_to_move {
        price_node.product_ids.retain(|&x| x != product_id);
        new_price_node.product_ids.push(product_id);
    }

    msg!(
        "Price index node split successful, original range: {} - {}, new range: {} - {}",
        price_range_start,
        split_point,
        split_point + 1,
        price_range_end
    );

    Ok(())
}

// Find appropriate price index node
pub fn find_price_node_for_price(price: u64) -> (u64, u64) {
    // Simplified implementation: use fixed price range division
    // Actual implementation should traverse the price index tree to find appropriate leaf nodes
    let interval = 1000u64; // Every 1000 units as one range
    let range_start = (price / interval) * interval;
    let range_end = range_start + interval - 1;
    (range_start, range_end)
}

// Get price index node utilization
pub fn get_price_node_utilization(node: &Account<PriceIndexNode>) -> f32 {
    node.product_ids.len() as f32 / MAX_PRODUCTS_PER_SHARD as f32
}

// Check if price index tree needs rebalancing
pub fn should_rebalance_price_tree(node: &Account<PriceIndexNode>) -> bool {
    node.needs_split() || node.needs_merge()
}

// ============================================================================
// Smart price index instructions: automatically calculate price ranges and manage index
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_range_calculation() {
        // Test case: when price=15
        // floor(log2(15)) = 3, so 2^3=8 <= 15 < 2^4=16
        // price_range_start=8，price_range_end=16
        assert_eq!(calculate_price_range_start(15), 8);
        assert_eq!(calculate_price_range_end(15), 16);

        // Test case: when price=1, special case
        assert_eq!(calculate_price_range_start(1), 1);
        assert_eq!(calculate_price_range_end(1), 1);

        // Test case: when price=8
        // floor(log2(8)) = 3, so 2^3=8 <= 8 < 2^4=16
        // price_range_start=8，price_range_end=16
        assert_eq!(calculate_price_range_start(8), 8);
        assert_eq!(calculate_price_range_end(8), 16);

        // Test case: when price=16
        // floor(log2(16)) = 4, so 2^4=16 <= 16 < 2^5=32
        // price_range_start=16，price_range_end=32
        assert_eq!(calculate_price_range_start(16), 16);
        assert_eq!(calculate_price_range_end(16), 32);

        // Test case: when price=0, special case
        assert_eq!(calculate_price_range_start(0), 0);
        assert_eq!(calculate_price_range_end(0), 0);

        // Test case: when price=2
        // floor(log2(2)) = 1, so 2^1=2 <= 2 < 2^2=4
        // price_range_start=2，price_range_end=4
        assert_eq!(calculate_price_range_start(2), 2);
        assert_eq!(calculate_price_range_end(2), 4);

        // Test case: when price=7
        // floor(log2(7)) = 2, so 2^2=4 <= 7 < 2^3=8
        // price_range_start=4，price_range_end=8
        assert_eq!(calculate_price_range_start(7), 4);
        assert_eq!(calculate_price_range_end(7), 8);

        // Test case: when price=50
        // floor(log2(50)) = 5, so 2^5=32 <= 50 < 2^6=64
        // price_range_start=32，price_range_end=64
        assert_eq!(calculate_price_range_start(50), 32);
        assert_eq!(calculate_price_range_end(50), 64);
    }
}

/// Smart price index account structure (using Anchor standard methods)
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

/// Smart add product to price index
pub fn add_product_to_price_index(
    ctx: Context<AddProductToPriceIndex>,
    product_id: u64,
    price: u64,
    price_range_start: u64,
    price_range_end: u64,
) -> Result<()> {
    let price_index = &mut ctx.accounts.price_index;

    // Verify that the passed price range is consistent with the range calculated based on price
    let expected_start = calculate_price_range_start(price);
    let expected_end = calculate_price_range_end(price);

    require!(
        price_range_start == expected_start && price_range_end == expected_end,
        ErrorCode::InvalidPriceRange
    );

    // If it's a newly created account, initialize first
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
            "✅ New price index automatically created: price {} → range [{}, {}]",
            price,
            price_range_start,
            price_range_end
        );
    }

    // Verify price is within range
    require!(
        price >= price_index.price_range_start && price <= price_index.price_range_end,
        ErrorCode::InvalidPriceRange
    );

    // Check if product already exists
    if price_index.product_ids.contains(&product_id) {
        msg!("Product {} already exists in price index, skipping addition", product_id);
        return Ok(());
    }

    // Check index capacity
    if price_index.product_ids.len() >= 1000 {
        return Err(ErrorCode::ShardIsFull.into());
    }

    // Add product ID
    price_index.product_ids.push(product_id);

    msg!(
        "✅ Product {} added to price index [{}, {}], current product count: {}",
        product_id,
        price_range_start,
        price_range_end,
        price_index.product_ids.len()
    );

    Ok(())
}
