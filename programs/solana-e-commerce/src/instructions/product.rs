use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(product_id: u64, hard_delete: bool, force: bool)]
pub struct DeleteProduct<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        mut,
        seeds = [b"merchant_info", merchant.key().as_ref()],
        bump,
        constraint = merchant_info.owner == merchant.key() @ ErrorCode::Unauthorized
    )]
    pub merchant_info: Account<'info, Merchant>,

    #[account(
        mut,
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump,
        constraint = force || product.merchant == merchant.key() @ ErrorCode::Unauthorized,
        close = beneficiary
    )]
    pub product: Account<'info, ProductBase>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(product_id: u64, sales_increment: u32)]
pub struct UpdateSales<'info> {
    /// CHECK: This can be any signer as sales updates might come from various sources
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump
    )]
    pub product: Account<'info, ProductBase>,
}

#[derive(Accounts)]
#[instruction(product_id: u64, new_price: u64)]
pub struct UpdateProductPrice<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        mut,
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump,
        constraint = product.merchant == merchant.key() @ ErrorCode::Unauthorized
    )]
    pub product: Account<'info, ProductBase>,
    // Remove merchant_info account - permission verification through product.merchant field, no additional account needed
}

/// Create ProductBase instruction - only handle core business data
#[derive(Accounts)]
#[instruction(
    name: String,
    description: String,
    price: u64,
    keywords: Vec<String>,
    inventory: u64,
    payment_token: Pubkey,
    shipping_location: String
)]
pub struct CreateProductBase<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        mut,
        seeds = [b"global_id_root"],
        bump
    )]
    pub global_root: Account<'info, GlobalIdRoot>,

    #[account(
        mut,
        seeds = [b"merchant_id", merchant.key().as_ref()],
        bump
    )]
    pub merchant_id_account: Account<'info, MerchantIdAccount>,

    #[account(
        mut,
        seeds = [b"merchant_info", merchant.key().as_ref()],
        bump,
        constraint = merchant_info.owner == merchant.key() @ ErrorCode::Unauthorized
    )]
    pub merchant_info: Account<'info, Merchant>,

    #[account(
        mut,
        constraint = active_chunk.key() == merchant_id_account.active_chunk @ ErrorCode::InvalidActiveChunk
    )]
    pub active_chunk: Account<'info, IdChunk>,

    #[account(
        seeds = [b"payment_config"],
        bump
    )]
    pub payment_config: Account<'info, PaymentConfig>,

    /// CHECK: Product account will be created in the instruction
    #[account(mut)]
    pub product_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// Create ProductExtended instruction - only handle extended marketing data
#[derive(Accounts)]
#[instruction(
    product_id: u64,
    image_video_urls: Vec<String>,
    sales_regions: Vec<String>,
    logistics_methods: Vec<String>
)]
pub struct CreateProductExtended<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        init,
        payer = merchant,
        space = 8 + ProductExtended::INIT_SPACE,
        seeds = [b"product_extended", product_id.to_le_bytes().as_ref()],
        bump
    )]
    pub product_extended: Account<'info, ProductExtended>,

    #[account(
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump,
        constraint = product_base.merchant == merchant.key() @ ErrorCode::Unauthorized
    )]
    pub product_base: Account<'info, ProductBase>,

    pub system_program: Program<'info, System>,
}

/// Create ProductBase function - only create core business data
pub fn create_product_base(
    ctx: Context<CreateProductBase>,
    name: String,
    description: String,
    price: u64,
    keywords: Vec<String>,
    inventory: u64,
    payment_token: Pubkey,
    shipping_location: String,
) -> Result<u64> {
    // Validate input parameters - limit to 3 keywords during creation (considering instruction account size)
    require!(
        keywords.len() <= MAX_KEYWORDS_PER_PRODUCT_CREATE,
        ErrorCode::TooManyKeywords
    );
    require!(keywords.len() > 0, ErrorCode::InvalidKeyword);
    require!(price > 0, ErrorCode::InvalidPrice);

    // Verify if payment token is supported
    require!(
        ctx.accounts
            .payment_config
            .is_token_supported(&payment_token),
        ErrorCode::UnsupportedToken
    );

    // 1. Generate product ID
    let product_id = generate_next_product_id(
        &mut ctx.accounts.merchant_id_account,
        &mut ctx.accounts.active_chunk,
    )?;

    // 2. Create product account
    create_product_account(
        &ctx.accounts.merchant,
        &ctx.accounts.product_account,
        &ctx.accounts.system_program,
        product_id,
        ctx.program_id,
    )?;

    // 3. Initialize product base data
    let product_data = ProductBase {
        id: product_id,
        merchant: ctx.accounts.merchant.key(),
        name: name.clone(),
        description: description.clone(),
        price,
        keywords: keywords.join(","),
        inventory,
        payment_token,
        sales: 0,
        is_active: true,
        created_at: Clock::get()?.unix_timestamp,
        updated_at: Clock::get()?.unix_timestamp,
        shipping_location,
        bump: 0, // Will be set later
    };

    // 4. Serialize product data
    let mut data = ctx.accounts.product_account.try_borrow_mut_data()?;
    let dst: &mut [u8] = &mut data;
    let mut cursor = std::io::Cursor::new(dst);
    product_data.try_serialize(&mut cursor)?;

    // 5. Update merchant product count
    ctx.accounts.merchant_info.increment_product_count()?;

    msg!(
        "Atomic product creation successful, ID: {}, Name: {}, Keyword count: {}",
        product_id,
        name,
        keywords.len()
    );

    Ok(product_id)
}

/// Create ProductExtended function - only create extended marketing data
pub fn create_product_extended(
    ctx: Context<CreateProductExtended>,
    product_id: u64,
    image_video_urls: Vec<String>,
    sales_regions: Vec<String>,
    logistics_methods: Vec<String>,
) -> Result<()> {
    // Validate input parameters
    require!(image_video_urls.len() <= 10, ErrorCode::TooManyImageUrls);
    require!(sales_regions.len() <= 20, ErrorCode::TooManySalesRegions);
    require!(
        logistics_methods.len() <= 10,
        ErrorCode::TooManyLogisticsMethods
    );

    // Initialize ProductExtended data
    let product_extended_data = ProductExtended {
        product_id,
        image_video_urls: image_video_urls.join(","),
        sales_regions: sales_regions.join(","),
        logistics_methods: logistics_methods.join(","),
        bump: ctx.bumps.product_extended,
    };

    // Set account data
    ctx.accounts
        .product_extended
        .set_inner(product_extended_data);

    msg!("ProductExtended created successfully, Product ID: {}", product_id);

    Ok(())
}

// ==================== Helper Functions ====================

/// Update extended fields of ProductExtended account
fn update_product_extended_fields(
    product_extended: &mut ProductExtended,
    image_video_urls: Option<Vec<String>>,
    sales_regions: Option<Vec<String>>,
    logistics_methods: Option<Vec<String>>,
) -> Result<()> {
    // Update image video URLs
    if let Some(new_image_video_urls) = image_video_urls {
        let urls_string = ProductExtended::vec_to_image_urls_string(new_image_video_urls);
        product_extended.image_video_urls = urls_string;
        msg!("Image video URLs updated");
    }

    // Update sales regions
    if let Some(new_sales_regions) = sales_regions {
        let regions_string = ProductExtended::vec_to_sales_regions_string(new_sales_regions);
        product_extended.sales_regions = regions_string;
        msg!("Sales regions updated");
    }

    // Update logistics methods
    if let Some(new_logistics_methods) = logistics_methods {
        let methods_string =
            ProductExtended::vec_to_logistics_methods_string(new_logistics_methods);
        product_extended.logistics_methods = methods_string;
        msg!("Logistics methods updated");
    }

    Ok(())
}

/// Helper function to create product account
fn create_product_account<'info>(
    payer: &Signer<'info>,
    product_account: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    product_id: u64,
    program_id: &Pubkey,
) -> Result<()> {
    let product_id_bytes = product_id.to_le_bytes();
    let product_seeds = &[b"product", product_id_bytes.as_ref()];
    let (expected_product_pda, product_bump) =
        Pubkey::find_program_address(product_seeds, program_id);

    require!(
        product_account.key() == expected_product_pda,
        ErrorCode::InvalidProductAccount
    );

    let rent = Rent::get()?;
    let space = 8 + ProductBase::INIT_SPACE;
    let lamports = rent.minimum_balance(space);

    anchor_lang::system_program::create_account(
        CpiContext::new_with_signer(
            system_program.to_account_info(),
            anchor_lang::system_program::CreateAccount {
                from: payer.to_account_info(),
                to: product_account.to_account_info(),
            },
            &[&[b"product", product_id_bytes.as_ref(), &[product_bump]]],
        ),
        lamports,
        space as u64,
        program_id,
    )?;

    Ok(())
}

// Helper function: generate next product ID
fn generate_next_product_id(
    merchant_account: &mut Account<MerchantIdAccount>,
    active_chunk: &mut Account<IdChunk>,
) -> Result<u64> {
    // Check if current chunk has available IDs
    if active_chunk.is_full() {
        return Err(ErrorCode::NoAvailableId.into());
    }

    // Find next available ID
    let mut local_id = active_chunk.next_available;
    while local_id < active_chunk.capacity() {
        if !active_chunk.is_id_used(local_id) {
            // Allocate this ID
            active_chunk.mark_id_used(local_id);
            active_chunk.next_available = local_id + 1;
            merchant_account.last_local_id = local_id;

            // Use activeChunk.startId + localId to calculate product ID
            let product_id = active_chunk.start_id + local_id;

            msg!(
                "Generated product ID: startId {} + local ID {} = {}",
                active_chunk.start_id,
                local_id,
                product_id
            );

            return Ok(product_id);
        }
        local_id += 1;
    }

    Err(ErrorCode::NoAvailableId.into())
}

#[event]
pub struct ProductEvent {
    pub product_id: u64,
    pub merchant: Pubkey,
    pub name: String,
    pub description: String,
    pub price: u64,
    pub keywords: Vec<String>,
    pub sales_count: u32,
    pub is_active: bool,
    pub timestamp: i64,
    pub event_type: String, // "created", "updated", "deleted", "sold"
}

pub fn delete_product(
    ctx: Context<DeleteProduct>,
    _product_id: u64,
    hard_delete: bool,
    force: bool,
) -> Result<()> {
    let product = &ctx.accounts.product;
    // Remove merchant_info reference - statistics functionality has been simplified
    let product_id = product.id;

    // Permission verification (when force=false)
    if !force {
        require!(
            product.merchant == ctx.accounts.merchant.key(),
            ErrorCode::Unauthorized
        );
    }

    // Record index information that needs cleanup
    let keywords = product.parse_keywords();
    let price = product.price;
    let sales = product.sales;

    if hard_delete {
        // Hard delete: account will be automatically closed and rent reclaimed to beneficiary through close constraint
        // Update merchant product count
        ctx.accounts.merchant_info.decrement_product_count()?;

        msg!(
            "Product hard deleted, ID: {}, force delete: {}, rent reclaimed to beneficiary",
            product_id,
            force
        );
    } else {
        // Soft delete: mark as inactive
        let product = &mut ctx.accounts.product;
        product.set_active(false)?;

        msg!(
            "Product soft deleted, ID: {}, force delete: {}",
            product_id,
            force
        );
    }

    // Index cleanup logic (needs to be executed through dedicated instructions)
    msg!(
        "Index information that needs cleanup - keywords: {:?}, price: {}, sales: {}",
        keywords,
        price,
        sales
    );

    Ok(())
}

pub fn update_sales_count(
    ctx: Context<UpdateSales>,
    _product_id: u64,
    sales_increment: u32,
) -> Result<()> {
    let product = &mut ctx.accounts.product;

    product.update_sales(sales_increment)?;

    msg!(
        "Product sales update successful, ID: {}, increment: {}",
        product.id,
        sales_increment
    );

    Ok(())
}

pub fn update_product_price(
    ctx: Context<UpdateProductPrice>,
    _product_id: u64,
    new_price: u64,
) -> Result<()> {
    require!(new_price > 0, ErrorCode::InvalidPrice);

    let product = &mut ctx.accounts.product;
    let old_price = product.price;

    // Verify permission: only product owner can modify price
    require!(
        product.merchant == ctx.accounts.merchant.key(),
        ErrorCode::Unauthorized
    );

    // Update product price
    product.price = new_price;
    product.updated_at = Clock::get()?.unix_timestamp;

    msg!(
        "Product price update successful, ID: {}, Old price: {} -> New price: {}",
        product.id,
        old_price,
        new_price
    );

    Ok(())
}

// Update product information
#[derive(Accounts)]
#[instruction(product_id: u64)]
pub struct UpdateProduct<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        mut,
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump,
        constraint = product.merchant == merchant.key() @ ErrorCode::Unauthorized
    )]
    pub product: Account<'info, ProductBase>,

    #[account(
        mut,
        seeds = [b"product_extended", product_id.to_le_bytes().as_ref()],
        bump
    )]
    pub product_extended: Option<Account<'info, ProductExtended>>,

    #[account(
        seeds = [b"payment_config"],
        bump
    )]
    pub payment_config: Account<'info, PaymentConfig>,

    pub system_program: Program<'info, System>,
}

pub fn update_product(
    ctx: Context<UpdateProduct>,
    _product_id: u64,
    name: Option<String>,
    description: Option<String>,
    price: Option<u64>,
    keywords: Option<Vec<String>>,
    inventory: Option<u64>,
    payment_token: Option<Pubkey>,
    image_video_urls: Option<Vec<String>>,
    shipping_location: Option<String>,
    sales_regions: Option<Vec<String>>,
    logistics_methods: Option<Vec<String>>,
) -> Result<()> {
    let product = &mut ctx.accounts.product;

    // Update name
    if let Some(new_name) = name {
        require!(!new_name.is_empty(), ErrorCode::InvalidProductNameLength);
        product.name = new_name;
    }

    // Update description
    if let Some(new_description) = description {
        product.description = new_description;
    }

    // Update price
    if let Some(new_price) = price {
        require!(new_price > 0, ErrorCode::InvalidPrice);
        product.price = new_price;
    }

    // Update keywords (now in ProductBase)
    // Allow up to 10 keywords when updating, limit to 3 when creating (considering instruction account size)
    if let Some(new_keywords) = keywords {
        require!(
            new_keywords.len() <= MAX_KEYWORDS_PER_PRODUCT,
            ErrorCode::TooManyKeywords
        );
        require!(new_keywords.len() > 0, ErrorCode::InvalidKeyword);
        product.update_keywords(new_keywords)?;
    }

    // Update inventory
    if let Some(new_inventory) = inventory {
        product.inventory = new_inventory;
    }

    // Update payment token
    if let Some(new_payment_token) = payment_token {
        require!(
            ctx.accounts
                .payment_config
                .is_token_supported(&new_payment_token),
            ErrorCode::UnsupportedToken
        );
        product.payment_token = new_payment_token;
    }

    // Note: token_decimals and token_price fields have been removed, price is unified using the price field

    // Update shipping location (this field is in ProductBase)
    if let Some(new_shipping_location) = shipping_location {
        product.shipping_location = new_shipping_location;
    }

    // Handle ProductExtended extended field updates
    let has_extended_updates =
        image_video_urls.is_some() || sales_regions.is_some() || logistics_methods.is_some();

    if has_extended_updates {
        if let Some(product_extended) = &mut ctx.accounts.product_extended {
            // ProductExtended account exists, update directly
            update_product_extended_fields(
                product_extended,
                image_video_urls,
                sales_regions,
                logistics_methods,
            )?;

            msg!("ProductExtended fields updated successfully");
        } else {
            // ProductExtended account does not exist, need to create first
            msg!("Warning: ProductExtended account does not exist, cannot update extended fields. Please call create_product_extended instruction first to create the extended account.");

            // Record fields that were attempted to be updated
            if image_video_urls.is_some() {
                msg!("Attempted to update image video URLs, but ProductExtended account does not exist");
            }
            if sales_regions.is_some() {
                msg!("Attempted to update sales regions, but ProductExtended account does not exist");
            }
            if logistics_methods.is_some() {
                msg!("Attempted to update logistics methods, but ProductExtended account does not exist");
            }
        }
    }

    // Update timestamp
    product.updated_at = Clock::get()?.unix_timestamp;

    msg!("Product information updated successfully, ID: {}", product.id);

    Ok(())
}
