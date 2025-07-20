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
        bump
    )]
    pub merchant_info: Account<'info, Merchant>,

    #[account(
        mut,
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump,
        constraint = force || product.merchant == merchant.key() @ ErrorCode::Unauthorized,
        close = beneficiary
    )]
    pub product: Account<'info, Product>,

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
    pub product: Account<'info, Product>,
}

#[derive(Accounts)]
#[instruction(product_id: u64, new_price: u64, new_token_price: u64)]
pub struct UpdateProductPrice<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        mut,
        seeds = [b"product", product_id.to_le_bytes().as_ref()],
        bump,
        constraint = product.merchant == merchant.key() @ ErrorCode::Unauthorized
    )]
    pub product: Account<'info, Product>,

    #[account(
        mut,
        seeds = [b"merchant_info", merchant.key().as_ref()],
        bump
    )]
    pub merchant_info: Account<'info, Merchant>,
}

/// 原子化商品创建指令 - 优化后的账户结构（删除无用的索引账户）
#[derive(Accounts)]
#[instruction(
    name: String,
    description: String,
    price: u64,
    keywords: Vec<String>,
    payment_token: Pubkey,
    token_decimals: u8,
    token_price: u64
)]
pub struct CreateProductAtomic<'info> {
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
        bump
    )]
    pub merchant_info: Account<'info, Merchant>,

    #[account(
        mut,
        constraint = active_chunk.key() == merchant_id_account.active_chunk @ ErrorCode::InvalidActiveChunk
    )]
    pub active_chunk: Account<'info, IdChunk>,

    /// CHECK: 产品账户将在指令中创建
    #[account(mut)]
    pub product_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// 原子化商品创建函数 - 只创建商品，不处理索引
pub fn create_product_atomic(
    ctx: Context<CreateProductAtomic>,
    name: String,
    description: String,
    price: u64,
    keywords: Vec<String>,
    payment_token: Pubkey,
    token_decimals: u8,
    token_price: u64,
) -> Result<u64> {
    // 验证输入参数
    require!(keywords.len() <= 3, ErrorCode::TooManyKeywords);
    require!(keywords.len() > 0, ErrorCode::InvalidKeyword);
    require!(price > 0, ErrorCode::InvalidPrice);
    require!(token_price > 0, ErrorCode::InvalidPrice);

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

    // 3. 初始化产品数据
    let product_data = Product {
        id: product_id,
        merchant: ctx.accounts.merchant.key(),
        name: name.clone(),
        description: description.clone(),
        price,
        keywords: keywords.clone(),
        payment_token,
        token_decimals,
        token_price,
        sales: 0,
        is_active: true,
        created_at: Clock::get()?.unix_timestamp,
        updated_at: Clock::get()?.unix_timestamp,
        bump: 0, // 将在后续设置
        image_video_urls: Vec::new(),
        shipping_location: String::new(),
        sales_regions: Vec::new(),
        logistics_methods: Vec::new(),
    };

    // 4. 序列化产品数据
    let mut data = ctx.accounts.product_account.try_borrow_mut_data()?;
    let dst: &mut [u8] = &mut data;
    let mut cursor = std::io::Cursor::new(dst);
    product_data.try_serialize(&mut cursor)?;

    // 5. 更新商户统计
    ctx.accounts.merchant_info.increment_product_count()?;

    msg!(
        "原子化产品创建成功，ID: {}, 名称: {}, 关键词数量: {}",
        product_id,
        name,
        keywords.len()
    );

    Ok(product_id)
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
    let space = Product::LEN;
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
    let merchant_info = &mut ctx.accounts.merchant_info;
    let product = &ctx.accounts.product;
    let product_id = product.id;

    // 权限验证（当force=false时）
    if !force {
        require!(
            product.merchant == ctx.accounts.merchant.key(),
            ErrorCode::Unauthorized
        );
    }

    // 记录需要清理的索引信息
    let keywords = product.keywords.clone();
    let token_price = product.token_price;
    let sales = product.sales;

    if hard_delete {
        // 硬删除：账户将通过close约束自动关闭并回收租金到beneficiary
        // 更新商户统计（只有在非强制删除时才更新，避免统计错误）
        if !force {
            merchant_info.decrement_product_count()?;
        }

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
        "需要清理的索引信息 - 关键词: {:?}, Token价格: {}, 销量: {}",
        keywords,
        token_price,
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
    new_token_price: u64,
) -> Result<()> {
    require!(new_price > 0, ErrorCode::InvalidPrice);
    require!(new_token_price > 0, ErrorCode::InvalidPrice);

    let product = &mut ctx.accounts.product;
    let old_price = product.price;
    let old_token_price = product.token_price;

    // 验证权限：只有商品所有者可以修改价格
    require!(
        product.merchant == ctx.accounts.merchant.key(),
        ErrorCode::Unauthorized
    );

    // 更新产品价格
    product.price = new_price;
    product.token_price = new_token_price;
    product.updated_at = Clock::get()?.unix_timestamp;

    msg!(
        "商品价格更新成功，ID: {}, 旧价格: {} -> 新价格: {}, 旧Token价格: {} -> 新Token价格: {}",
        product.id,
        old_price,
        new_price,
        old_token_price,
        new_token_price
    );

    Ok(())
}
