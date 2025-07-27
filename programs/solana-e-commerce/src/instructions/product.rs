use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(product_id: u64, hard_delete: bool, force: bool)]
pub struct DeleteProduct<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    // 移除merchant_info账户 - 产品计数统计功能非核心，权限验证通过product.merchant进行
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
    // 移除merchant_info账户 - 权限验证通过product.merchant字段进行，无需额外账户
}

/// 创建ProductBase指令 - 只处理核心业务数据
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

    // 移除merchant_info账户 - 产品计数统计功能非核心，可通过其他方式获取
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

    /// CHECK: 产品账户将在指令中创建
    #[account(mut)]
    pub product_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// 创建ProductExtended指令 - 只处理扩展营销数据
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

/// 创建ProductBase函数 - 只创建核心业务数据
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
    // 验证输入参数
    require!(keywords.len() <= 3, ErrorCode::TooManyKeywords);
    require!(keywords.len() > 0, ErrorCode::InvalidKeyword);
    require!(price > 0, ErrorCode::InvalidPrice);

    // 验证支付代币是否被支持
    require!(
        ctx.accounts
            .payment_config
            .is_token_supported(&payment_token),
        ErrorCode::UnsupportedToken
    );

    // 1. 生成产品ID
    let product_id = generate_next_product_id(
        &mut ctx.accounts.merchant_id_account,
        &mut ctx.accounts.active_chunk,
    )?;

    // 2. 创建产品账户
    create_product_account(
        &ctx.accounts.merchant,
        &ctx.accounts.product_account,
        &ctx.accounts.system_program,
        product_id,
        ctx.program_id,
    )?;

    // 3. 初始化产品基础数据
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
        bump: 0, // 将在后续设置
    };

    // 4. 序列化产品数据
    let mut data = ctx.accounts.product_account.try_borrow_mut_data()?;
    let dst: &mut [u8] = &mut data;
    let mut cursor = std::io::Cursor::new(dst);
    product_data.try_serialize(&mut cursor)?;

    // 5. 商户统计功能已移除 - 可通过查询链上产品账户获取统计信息

    msg!(
        "原子化产品创建成功，ID: {}, 名称: {}, 关键词数量: {}",
        product_id,
        name,
        keywords.len()
    );

    Ok(product_id)
}

/// 创建ProductExtended函数 - 只创建扩展营销数据
pub fn create_product_extended(
    ctx: Context<CreateProductExtended>,
    product_id: u64,
    image_video_urls: Vec<String>,
    sales_regions: Vec<String>,
    logistics_methods: Vec<String>,
) -> Result<()> {
    // 验证输入参数
    require!(image_video_urls.len() <= 10, ErrorCode::TooManyImageUrls);
    require!(sales_regions.len() <= 20, ErrorCode::TooManySalesRegions);
    require!(
        logistics_methods.len() <= 10,
        ErrorCode::TooManyLogisticsMethods
    );

    // 初始化ProductExtended数据
    let product_extended_data = ProductExtended {
        product_id,
        image_video_urls: image_video_urls.join(","),
        sales_regions: sales_regions.join(","),
        logistics_methods: logistics_methods.join(","),
        bump: ctx.bumps.product_extended,
    };

    // 设置账户数据
    ctx.accounts
        .product_extended
        .set_inner(product_extended_data);

    msg!("ProductExtended创建成功，产品ID: {}", product_id);

    Ok(())
}

// ==================== 辅助函数 ====================

/// 创建产品账户的辅助函数
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

// 辅助函数：生成下一个产品ID
fn generate_next_product_id(
    merchant_account: &mut Account<MerchantIdAccount>,
    active_chunk: &mut Account<IdChunk>,
) -> Result<u64> {
    // 检查当前块是否有可用ID
    if active_chunk.is_full() {
        return Err(ErrorCode::NoAvailableId.into());
    }

    // 查找下一个可用的ID
    let mut local_id = active_chunk.next_available;
    while local_id < active_chunk.capacity() {
        if !active_chunk.is_id_used(local_id) {
            // 分配这个ID
            active_chunk.mark_id_used(local_id);
            active_chunk.next_available = local_id + 1;
            merchant_account.last_local_id = local_id;

            // 使用 activeChunk.startId + localId 计算产品ID
            let product_id = active_chunk.start_id + (local_id as u64);

            msg!(
                "生成产品ID: startId {} + 本地ID {} = {}",
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
    // 移除merchant_info引用 - 统计功能已简化
    let product_id = product.id;

    // 权限验证（当force=false时）
    if !force {
        require!(
            product.merchant == ctx.accounts.merchant.key(),
            ErrorCode::Unauthorized
        );
    }

    // 记录需要清理的索引信息
    let keywords = product.parse_keywords();
    let price = product.price;
    let sales = product.sales;

    if hard_delete {
        // 硬删除：账户将通过close约束自动关闭并回收租金到beneficiary
        // 商户统计功能已移除 - 可通过查询链上产品账户获取统计信息

        msg!(
            "商品已硬删除，ID: {}, 强制删除: {}, 租金已回收到受益人",
            product_id,
            force
        );
    } else {
        // 软删除：标记为非活跃
        let product = &mut ctx.accounts.product;
        product.set_active(false)?;

        msg!("商品已软删除，ID: {}, 强制删除: {}", product_id, force);
    }

    // 索引清理逻辑（需要通过专门的指令执行）
    msg!(
        "需要清理的索引信息 - 关键词: {:?}, 价格: {}, 销量: {}",
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
        "商品销量更新成功，ID: {}, 增量: {}",
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

    // 验证权限：只有商品所有者可以修改价格
    require!(
        product.merchant == ctx.accounts.merchant.key(),
        ErrorCode::Unauthorized
    );

    // 更新产品价格
    product.price = new_price;
    product.updated_at = Clock::get()?.unix_timestamp;

    msg!(
        "商品价格更新成功，ID: {}, 旧价格: {} -> 新价格: {}",
        product.id,
        old_price,
        new_price
    );

    Ok(())
}

// 更新商品信息
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
        seeds = [b"payment_config"],
        bump
    )]
    pub payment_config: Account<'info, PaymentConfig>,
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

    // 更新名称
    if let Some(new_name) = name {
        require!(!new_name.is_empty(), ErrorCode::InvalidProductNameLength);
        product.name = new_name;
    }

    // 更新描述
    if let Some(new_description) = description {
        product.description = new_description;
    }

    // 更新价格
    if let Some(new_price) = price {
        require!(new_price > 0, ErrorCode::InvalidPrice);
        product.price = new_price;
    }

    // 更新关键词（现在在ProductBase中）
    if let Some(new_keywords) = keywords {
        require!(new_keywords.len() <= 3, ErrorCode::TooManyKeywords);
        require!(new_keywords.len() > 0, ErrorCode::InvalidKeyword);
        product.update_keywords(new_keywords)?;
    }

    // 更新库存
    if let Some(new_inventory) = inventory {
        product.inventory = new_inventory;
    }

    // 更新支付代币
    if let Some(new_payment_token) = payment_token {
        require!(
            ctx.accounts
                .payment_config
                .is_token_supported(&new_payment_token),
            ErrorCode::UnsupportedToken
        );
        product.payment_token = new_payment_token;
    }

    // 注意：token_decimals 和 token_price 字段已移除，价格统一使用 price 字段

    // TODO: 扩展字段更新需要通过ProductExtended账户处理
    if let Some(_new_image_video_urls) = image_video_urls {
        msg!("图片视频URL更新功能暂时禁用，需要通过ProductExtended账户处理");
    }

    // 更新发货地点（这个字段在ProductBase中）
    if let Some(new_shipping_location) = shipping_location {
        product.shipping_location = new_shipping_location;
    }

    // TODO: 扩展字段更新需要通过ProductExtended账户处理
    if let Some(_new_sales_regions) = sales_regions {
        msg!("销售区域更新功能暂时禁用，需要通过ProductExtended账户处理");
    }

    if let Some(_new_logistics_methods) = logistics_methods {
        msg!("物流方式更新功能暂时禁用，需要通过ProductExtended账户处理");
    }

    // 更新时间戳
    product.updated_at = Clock::get()?.unix_timestamp;

    msg!("商品信息更新成功，ID: {}", product.id);

    Ok(())
}
