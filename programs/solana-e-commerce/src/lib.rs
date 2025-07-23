use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::order::{ApproveRefund, RequestRefund};
use instructions::*;
use state::{OrderManagementStatus, SupportedToken};

declare_id!("mo5xPstZDm27CAkcyoTJnEovMYcW45tViAU6PZikv5q");

#[program]
pub mod solana_e_commerce {
    use super::*;

    // 系统初始化
    pub fn initialize_system(ctx: Context<InitializeSystem>, config: SystemConfig) -> Result<()> {
        instructions::initialize::initialize_system(ctx, config)
    }

    // 系统配置初始化
    pub fn initialize_system_config(
        ctx: Context<InitializeSystemConfig>,
        config: SystemConfig,
    ) -> Result<()> {
        instructions::initialize::initialize_system_config(ctx, config)
    }

    // 关闭系统配置
    pub fn close_system_config(ctx: Context<CloseSystemConfig>, force: bool) -> Result<()> {
        instructions::initialize::close_system_config(ctx, force)
    }

    // ID生成器指令
    pub fn generate_product_id(ctx: Context<GenerateId>) -> Result<u64> {
        instructions::id_generator::generate_product_id(ctx)
    }

    pub fn batch_generate_ids(ctx: Context<BatchGenerate>, count: u16) -> Result<Vec<u64>> {
        instructions::id_generator::batch_generate_ids(ctx, count)
    }

    pub fn is_id_exists(ctx: Context<VerifyId>, id: u64) -> Result<bool> {
        instructions::id_generator::is_id_exists(ctx, id)
    }

    pub fn allocate_new_chunk(ctx: Context<AllocateChunk>) -> Result<Pubkey> {
        instructions::id_generator::allocate_new_chunk(ctx)
    }

    // 商户管理指令

    // 原子性商户注册指令
    pub fn register_merchant_atomic(
        ctx: Context<RegisterMerchantAtomic>,
        name: String,
        description: String,
    ) -> Result<()> {
        instructions::merchant::register_merchant_atomic(ctx, name, description)
    }

    pub fn update_merchant_info(
        ctx: Context<UpdateMerchant>,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<()> {
        instructions::merchant::update_merchant_info(ctx, name, description)
    }

    pub fn get_merchant_stats(ctx: Context<GetMerchantStats>) -> Result<state::MerchantStats> {
        instructions::merchant::get_merchant_stats(ctx)
    }

    // 商品管理指令

    // 创建ProductBase（核心业务数据）
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
        instructions::product::create_product_base(
            ctx,
            name,
            description,
            price,
            keywords,
            inventory,
            payment_token,
            shipping_location,
        )
    }

    // 创建ProductExtended（扩展营销数据）
    pub fn create_product_extended(
        ctx: Context<CreateProductExtended>,
        product_id: u64,
        image_video_urls: Vec<String>,
        sales_regions: Vec<String>,
        logistics_methods: Vec<String>,
    ) -> Result<()> {
        instructions::product::create_product_extended(
            ctx,
            product_id,
            image_video_urls,
            sales_regions,
            logistics_methods,
        )
    }

    // 产品修改指令
    pub fn update_product(
        ctx: Context<UpdateProduct>,
        product_id: u64,
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
        instructions::product::update_product(
            ctx,
            product_id,
            name,
            description,
            price,
            keywords,
            inventory,
            payment_token,
            image_video_urls,
            shipping_location,
            sales_regions,
            logistics_methods,
        )
    }

    pub fn delete_product(
        ctx: Context<DeleteProduct>,
        product_id: u64,
        hard_delete: bool,
        force: bool,
    ) -> Result<()> {
        instructions::product::delete_product(ctx, product_id, hard_delete, force)
    }

    pub fn update_product_price(
        ctx: Context<UpdateProductPrice>,
        product_id: u64,
        new_price: u64,
    ) -> Result<()> {
        instructions::product::update_product_price(ctx, product_id, new_price)
    }

    pub fn update_sales_count(
        ctx: Context<UpdateSales>,
        product_id: u64,
        sales_increment: u32,
    ) -> Result<()> {
        instructions::product::update_sales_count(ctx, product_id, sales_increment)
    }

    // 支付系统指令
    pub fn initialize_payment_system(
        ctx: Context<InitializePaymentSystem>,
        supported_tokens: Vec<SupportedToken>,
        fee_rate: u16,
        fee_recipient: Pubkey,
    ) -> Result<()> {
        instructions::payment::initialize_payment_system(
            ctx,
            supported_tokens,
            fee_rate,
            fee_recipient,
        )
    }

    pub fn update_supported_tokens(
        ctx: Context<UpdatePaymentConfig>,
        supported_tokens: Vec<SupportedToken>,
    ) -> Result<()> {
        instructions::payment::update_supported_tokens(ctx, supported_tokens)
    }

    pub fn update_fee_rate(ctx: Context<UpdatePaymentConfig>, fee_rate: u16) -> Result<()> {
        instructions::payment::update_fee_rate(ctx, fee_rate)
    }

    pub fn close_payment_config(ctx: Context<ClosePaymentConfig>, force: bool) -> Result<()> {
        instructions::payment::close_payment_config(ctx, force)
    }

    // 托管购买商品指令 - 集成订单创建
    pub fn purchase_product_escrow(
        ctx: Context<PurchaseProductEscrow>,
        product_id: u64,
        amount: u64,
        timestamp: i64,
        shipping_address: String,
        notes: String,
    ) -> Result<()> {
        instructions::payment::purchase_product_escrow(
            ctx,
            product_id,
            amount,
            timestamp,
            shipping_address,
            notes,
        )
    }

    // 关键词索引管理指令（已删除老旧函数，只保留if_needed版本）

    pub fn remove_product_from_keyword_index(
        ctx: Context<RemoveProductFromKeywordIndex>,
        keyword: String,
        product_id: u64,
    ) -> Result<()> {
        instructions::keyword_index::remove_product_from_keyword_index(ctx, keyword, product_id)
    }

    // 关键词索引指令
    pub fn initialize_keyword_index(
        ctx: Context<InitializeKeywordIndexIfNeeded>,
        keyword: String,
    ) -> Result<()> {
        instructions::keyword_index::initialize_keyword_index_if_needed(ctx, keyword)
    }

    pub fn add_product_to_keyword_index(
        ctx: Context<AddProductToKeywordIndexIfNeeded>,
        keyword: String,
        product_id: u64,
    ) -> Result<()> {
        instructions::keyword_index::add_product_to_keyword_index_if_needed(
            ctx, keyword, product_id,
        )
    }

    pub fn create_keyword_shard(
        ctx: Context<CreateKeywordShard>,
        keyword: String,
        shard_index: u32,
    ) -> Result<()> {
        instructions::keyword_index::create_keyword_shard(ctx, keyword, shard_index)
    }

    // 价格索引管理指令
    pub fn initialize_price_index(
        ctx: Context<InitializePriceIndexIfNeeded>,
        price_range_start: u64,
        price_range_end: u64,
    ) -> Result<()> {
        instructions::price_index::initialize_price_index_if_needed(
            ctx,
            price_range_start,
            price_range_end,
        )
    }

    pub fn add_product_to_price_index(
        ctx: Context<AddProductToPriceIndexIfNeeded>,
        price_range_start: u64,
        price_range_end: u64,
        product_id: u64,
        price: u64,
    ) -> Result<()> {
        instructions::price_index::add_product_to_price_index_if_needed(
            ctx,
            price_range_start,
            price_range_end,
            product_id,
            price,
        )
    }

    pub fn remove_product_from_price_index(
        ctx: Context<RemoveProductFromPriceIndex>,
        product_id: u64,
    ) -> Result<()> {
        instructions::price_index::remove_product_from_price_index(ctx, product_id)
    }

    pub fn split_price_node(
        ctx: Context<SplitPriceNode>,
        price_range_start: u64,
        price_range_end: u64,
    ) -> Result<()> {
        instructions::price_index::split_price_node(ctx, price_range_start, price_range_end)
    }

    // 销量索引管理指令
    pub fn initialize_sales_index(
        ctx: Context<InitializeSalesIndexIfNeeded>,
        sales_range_start: u32,
        sales_range_end: u32,
    ) -> Result<()> {
        instructions::sales_index::initialize_sales_index_if_needed(
            ctx,
            sales_range_start,
            sales_range_end,
        )
    }

    pub fn add_product_to_sales_index(
        ctx: Context<AddProductToSalesIndexIfNeeded>,
        sales_range_start: u32,
        sales_range_end: u32,
        product_id: u64,
        sales: u32,
    ) -> Result<()> {
        instructions::sales_index::add_product_to_sales_index_if_needed(
            ctx,
            sales_range_start,
            sales_range_end,
            product_id,
            sales,
        )
    }

    pub fn remove_product_from_sales_index(
        ctx: Context<RemoveProductFromSalesIndex>,
        product_id: u64,
    ) -> Result<()> {
        instructions::sales_index::remove_product_from_sales_index(ctx, product_id)
    }

    pub fn update_product_sales_index(
        ctx: Context<UpdateProductSalesIndex>,
        product_id: u64,
        old_sales: u32,
        new_sales: u32,
    ) -> Result<()> {
        instructions::sales_index::update_product_sales_index(ctx, product_id, old_sales, new_sales)
    }

    // 账户关闭指令
    pub fn close_keyword_root(
        ctx: Context<CloseKeywordRoot>,
        keyword: String,
        force: bool,
    ) -> Result<()> {
        instructions::keyword_index::close_keyword_root(ctx, keyword, force)
    }

    pub fn close_keyword_shard(
        ctx: Context<CloseKeywordShard>,
        keyword: String,
        shard_index: u32,
        force: bool,
    ) -> Result<()> {
        instructions::keyword_index::close_keyword_shard(ctx, keyword, shard_index, force)
    }

    pub fn close_merchant(ctx: Context<CloseMerchant>, force: bool) -> Result<()> {
        instructions::merchant::close_merchant(ctx, force)
    }

    pub fn close_id_chunk(
        ctx: Context<CloseIdChunk>,
        merchant_id: u64,
        chunk_index: u32,
        force: bool,
    ) -> Result<()> {
        instructions::id_generator::close_id_chunk(ctx, merchant_id, chunk_index, force)
    }

    pub fn close_merchant_id_account(
        ctx: Context<CloseMerchantIdAccount>,
        force: bool,
    ) -> Result<()> {
        instructions::id_generator::close_merchant_id_account(ctx, force)
    }

    // 订单管理指令
    pub fn initialize_order_stats(ctx: Context<InitializeOrderStats>) -> Result<()> {
        instructions::order::initialize_order_stats(ctx)
    }

    pub fn create_order(
        ctx: Context<CreateOrder>,
        product_id: u64,
        timestamp: i64,
        quantity: u32,
        shipping_address: String,
        notes: String,
        transaction_signature: String,
    ) -> Result<()> {
        instructions::order::create_order(
            ctx,
            product_id,
            timestamp,
            quantity,
            shipping_address,
            notes,
            transaction_signature,
        )
    }

    pub fn update_order_status(
        ctx: Context<UpdateOrderStatus>,
        new_status: OrderManagementStatus,
    ) -> Result<()> {
        instructions::order::update_order_status(ctx, new_status)
    }

    // 买家请求退款
    pub fn request_refund(ctx: Context<RequestRefund>, refund_reason: String) -> Result<()> {
        instructions::order::request_refund(ctx, refund_reason)
    }

    // 商家批准退款并执行退款
    pub fn approve_refund(ctx: Context<ApproveRefund>) -> Result<()> {
        instructions::order::approve_refund(ctx)
    }

    pub fn get_order_stats(ctx: Context<GetOrderStats>) -> Result<()> {
        instructions::order::get_order_stats(ctx)
    }

    pub fn confirm_delivery(ctx: Context<ConfirmDelivery>) -> Result<()> {
        instructions::order::confirm_delivery(ctx)
    }

    pub fn return_order(
        ctx: Context<ReturnOrder>,
        buyer: Pubkey,
        merchant_key: Pubkey,
        product_id: u64,
        timestamp: i64,
        return_reason: Option<String>,
    ) -> Result<()> {
        instructions::order::return_order(
            ctx,
            buyer,
            merchant_key,
            product_id,
            timestamp,
            return_reason,
        )
    }

    // ==================== 保证金管理指令 ====================

    // 商户缴纳保证金
    pub fn deposit_merchant_deposit(
        ctx: Context<DepositMerchantDeposit>,
        amount: u64,
    ) -> Result<()> {
        instructions::deposit::deposit_merchant_deposit(ctx, amount)
    }

    // 商户提取保证金
    pub fn withdraw_merchant_deposit(
        ctx: Context<WithdrawMerchantDeposit>,
        amount: u64,
    ) -> Result<()> {
        instructions::deposit::withdraw_merchant_deposit(ctx, amount)
    }

    // 查询商户保证金信息
    pub fn get_merchant_deposit_info(
        ctx: Context<GetMerchantDepositInfo>,
    ) -> Result<MerchantDepositInfo> {
        instructions::deposit::get_merchant_deposit_info(ctx)
    }

    // 更新保证金要求（系统管理员）
    pub fn update_deposit_requirement(
        ctx: Context<UpdateDepositRequirement>,
        new_requirement: u64,
    ) -> Result<()> {
        instructions::deposit::update_deposit_requirement(ctx, new_requirement)
    }
}

#[account]
pub struct SystemConfig {
    pub authority: Pubkey, // 系统管理员地址
    pub max_products_per_shard: u16,
    pub max_keywords_per_product: u8,
    pub chunk_size: u32,
    pub bloom_filter_size: u16,
    // 保证金配置
    pub merchant_deposit_required: u64, // 商户保证金要求（USDC，以最小单位计算）
    pub deposit_token_mint: Pubkey,     // 保证金代币mint地址（USDC）
    pub deposit_token_decimals: u8,     // 保证金代币精度
    // 新增平台手续费配置
    pub platform_fee_rate: u16, // 平台手续费率（基点，默认40 = 0.4%）
    pub platform_fee_recipient: Pubkey, // 平台手续费接收账户
    // 新增自动确认收货配置
    pub auto_confirm_days: u32, // 自动确认收货天数（默认30天）
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            authority: Pubkey::default(), // 需要在初始化时设置
            max_products_per_shard: 100,
            max_keywords_per_product: 10,
            chunk_size: 10_000,
            bloom_filter_size: 256,
            // 默认保证金配置：1000 USDC (6位精度)
            merchant_deposit_required: 1000 * 1_000_000, // 1000 USDC
            deposit_token_mint: Pubkey::default(),       // 需要在初始化时设置
            deposit_token_decimals: 6,                   // USDC精度
            // 默认平台手续费配置
            platform_fee_rate: 40,                     // 0.4% (40基点)
            platform_fee_recipient: Pubkey::default(), // 需要在初始化时设置
            // 默认自动确认收货配置
            auto_confirm_days: 30, // 30天自动确认收货
        }
    }
}

impl SystemConfig {
    /// 获取保证金要求（以代币最小单位计算）
    pub fn get_deposit_requirement(&self) -> u64 {
        self.merchant_deposit_required
    }

    /// 验证保证金代币
    pub fn is_valid_deposit_token(&self, token_mint: &Pubkey) -> bool {
        self.deposit_token_mint == *token_mint
    }

    /// 设置保证金代币mint
    pub fn set_deposit_token_mint(&mut self, token_mint: Pubkey) {
        self.deposit_token_mint = token_mint;
    }
}
