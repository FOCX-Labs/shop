use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;
use state::SupportedToken;

declare_id!("5XZ74thixMBX2tQN9P3yLTugUK4YMdRLznDNa2mRdGNT");

#[program]
pub mod solana_e_commerce {
    use super::*;

    // System initialization
    pub fn initialize_system(ctx: Context<InitializeSystem>, config: SystemConfig) -> Result<()> {
        instructions::initialize::initialize_system(ctx, config)
    }

    // System configuration initialization
    pub fn initialize_system_config(
        ctx: Context<InitializeSystemConfig>,
        config: SystemConfig,
    ) -> Result<()> {
        instructions::initialize::initialize_system_config(ctx, config)
    }

    // Close system configuration
    pub fn close_system_config(ctx: Context<CloseSystemConfig>, force: bool) -> Result<()> {
        instructions::initialize::close_system_config(ctx, force)
    }

    // Force close incompatible system configuration account
    pub fn force_close_system_config(ctx: Context<ForceCloseSystemConfig>) -> Result<()> {
        instructions::initialize::force_close_system_config(ctx)
    }

    // ID generator instructions
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

    // ==================== Merchant Management Instructions ====================

    // Atomic merchant registration instruction
    pub fn register_merchant_atomic(
        ctx: Context<RegisterMerchantAtomic>,
        name: String,
        description: String,
    ) -> Result<()> {
        instructions::merchant::register_merchant_atomic(ctx, name, description)
    }

    // Update merchant information
    pub fn update_merchant_info(
        ctx: Context<UpdateMerchant>,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<()> {
        instructions::merchant::update_merchant_info(ctx, name, description)
    }

    // Get merchant statistics information
    pub fn get_merchant_stats(ctx: Context<GetMerchantStats>) -> Result<state::MerchantStats> {
        instructions::merchant::get_merchant_stats(ctx)
    }

    // Close merchant account
    pub fn close_merchant(ctx: Context<CloseMerchant>, force: bool) -> Result<()> {
        instructions::merchant::close_merchant(ctx, force)
    }

    // Product management instructions

    // Create ProductBase (core business data)
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

    // Create ProductExtended (extended marketing data)
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

    // Product modification instruction
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

    // Payment system instructions
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

    // Program Token account initialization instruction
    pub fn initialize_program_token_account(
        ctx: Context<InitializeProgramTokenAccount>,
    ) -> Result<()> {
        instructions::payment::initialize_program_token_account(ctx)
    }

    // Escrow purchase product instruction - integrates order creation
    pub fn purchase_product_escrow(
        ctx: Context<PurchaseProductEscrow>,
        product_id: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::payment::purchase_product_escrow(ctx, product_id, amount)
    }

    // Keyword index management instructions (removed old functions, only keep if_needed versions)

    pub fn remove_product_from_keyword_index(
        ctx: Context<RemoveProductFromKeywordIndex>,
        keyword: String,
        product_id: u64,
    ) -> Result<()> {
        instructions::keyword_index::remove_product_from_keyword_index(ctx, keyword, product_id)
    }

    // Keyword index instructions
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

    // Price index management instructions

    // Smart price index instructions
    pub fn add_product_to_price_index(
        ctx: Context<AddProductToPriceIndex>,
        product_id: u64,
        price: u64,
        price_range_start: u64,
        price_range_end: u64,
    ) -> Result<()> {
        instructions::price_index::add_product_to_price_index(
            ctx,
            product_id,
            price,
            price_range_start,
            price_range_end,
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

    // Sales index management instructions
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

    // Account closing instructions
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

    pub fn close_id_chunk(
        ctx: Context<CloseIdChunk>,
        merchant_key: Pubkey,
        chunk_index: u32,
        force: bool,
    ) -> Result<()> {
        instructions::id_generator::close_id_chunk(ctx, merchant_key, chunk_index, force)
    }

    pub fn close_merchant_id_account(
        ctx: Context<CloseMerchantIdAccount>,
        merchant_key: Pubkey,
        force: bool,
    ) -> Result<()> {
        instructions::id_generator::close_merchant_id_account(ctx, merchant_key, force)
    }

    // Order management instructions
    pub fn initialize_order_stats(ctx: Context<InitializeOrderStats>) -> Result<()> {
        instructions::order::initialize_order_stats(ctx)
    }

    pub fn create_order(
        ctx: Context<CreateOrder>,
        product_id: u64,
        quantity: u32,
        shipping_address: String,
        notes: String,
        transaction_signature: String,
    ) -> Result<()> {
        instructions::order::create_order(
            ctx,
            product_id,
            quantity,
            shipping_address,
            notes,
            transaction_signature,
        )
    }

    pub fn ship_order(ctx: Context<ShipOrder>, tracking_number: String) -> Result<()> {
        instructions::order::ship_order(ctx, tracking_number)
    }

    // Buyer requests refund
    pub fn refund_order(ctx: Context<RefundOrder>, refund_reason: String) -> Result<()> {
        instructions::order::refund_order(ctx, refund_reason)
    }

    // Merchant approve refund instruction removed, buyers can refund directly

    pub fn get_order_stats(ctx: Context<GetOrderStats>) -> Result<()> {
        instructions::order::get_order_stats(ctx)
    }

    pub fn confirm_delivery(ctx: Context<ConfirmDelivery>) -> Result<()> {
        instructions::order::confirm_delivery(ctx)
    }

    // Auto confirm delivery (system call)
    pub fn auto_confirm_delivery(ctx: Context<AutoConfirmDelivery>) -> Result<()> {
        instructions::order::auto_confirm_delivery(ctx)
    }

    // ==================== 保证金管理指令 ====================

    // 商户缴纳/补充保证金（统一指令）
    pub fn manage_deposit(ctx: Context<ManageDeposit>, amount: u64) -> Result<()> {
        instructions::deposit::manage_deposit(ctx, amount)
    }

    // 商户提取保证金
    pub fn withdraw_merchant_deposit(
        ctx: Context<WithdrawMerchantDeposit>,
        amount: u64,
    ) -> Result<()> {
        instructions::deposit::withdraw_merchant_deposit(ctx, amount)
    }

    // 管理员扣除商户保证金
    pub fn deduct_merchant_deposit(
        ctx: Context<DeductMerchantDeposit>,
        amount: u64,
        reason: String,
    ) -> Result<()> {
        instructions::deposit::deduct_merchant_deposit(ctx, amount, reason)
    }

    // 查询商户保证金信息
    pub fn get_merchant_deposit_info(
        ctx: Context<GetMerchantDepositInfo>,
    ) -> Result<MerchantDepositInfo> {
        instructions::deposit::get_merchant_deposit_info(ctx)
    }

    // Update deposit requirement (system administrator)
    pub fn update_deposit_requirement(
        ctx: Context<UpdateDepositRequirement>,
        new_requirement: u64,
    ) -> Result<()> {
        instructions::deposit::update_deposit_requirement(ctx, new_requirement)
    }
}

#[account]
pub struct SystemConfig {
    pub authority: Pubkey, // System administrator address
    pub max_products_per_shard: u16,
    pub max_keywords_per_product: u8,
    pub chunk_size: u32,
    pub bloom_filter_size: u16,
    // Deposit configuration
    pub merchant_deposit_required: u64, // Merchant deposit requirement (in smallest units)
    pub deposit_token_mint: Pubkey,     // Deposit token mint address
    // Platform fee configuration
    pub platform_fee_rate: u16, // Platform fee rate (basis points, default 40 = 0.4%)
    pub platform_fee_recipient: Pubkey, // Platform fee recipient account
    // Auto confirm delivery configuration
    pub auto_confirm_days: u32, // Auto confirm delivery days (default 30 days)
    // Vault program ID configuration
    pub vault_program_id: Pubkey, // Vault program ID for CPI calls to add_rewards

    // Vault相关账户配置
    pub vault_account: Pubkey,          // Vault数据账户地址（PDA）
    pub vault_token_account: Pubkey,    // Vault的Token账户地址
    pub platform_token_account: Pubkey, // 平台Token账户地址
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            authority: Pubkey::default(), // Needs to be set during initialization
            max_products_per_shard: 100,
            max_keywords_per_product: 10,
            chunk_size: 10_000,
            bloom_filter_size: 256,
            // Default deposit configuration: base units, needs dynamic calculation based on Token precision during initialization
            merchant_deposit_required: 1000, // Base units, needs conversion based on Token precision during actual use
            deposit_token_mint: Pubkey::default(), // Needs to be set during initialization
            // Default platform fee configuration
            platform_fee_rate: 40,                     // 0.4% (40 basis points)
            platform_fee_recipient: Pubkey::default(), // Needs to be set during initialization
            // Default auto confirm delivery configuration
            auto_confirm_days: 30, // 30 days auto confirm delivery
            // Default vault program ID configuration
            vault_program_id: Pubkey::default(), // Needs to be set during initialization

            // Default vault账户配置
            vault_account: Pubkey::default(), // Needs to be set during initialization
            vault_token_account: Pubkey::default(), // Needs to be set during initialization
            platform_token_account: Pubkey::default(), // Needs to be set during initialization
        }
    }
}

impl SystemConfig {
    /// 获取保证金要求（以代币最小单位计算）
    ///
    /// # 参数
    /// - `token_decimals`: Token的精度（从mint账户获取）
    ///
    /// # 返回
    /// 根据Token精度计算的实际保证金要求
    pub fn get_deposit_requirement(&self, token_decimals: u8) -> u64 {
        // merchant_deposit_required 存储的是基础单位（如1000）
        // 需要根据Token精度转换为最小单位
        self.merchant_deposit_required
            .saturating_mul(10_u64.saturating_pow(token_decimals as u32))
    }

    /// 获取基础保证金要求（不考虑精度）
    pub fn get_base_deposit_requirement(&self) -> u64 {
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

    /// 根据Token精度计算实际的保证金要求
    ///
    /// # 参数
    /// - `base_amount`: 基础金额（如1000）
    /// - `decimals`: Token精度（从mint账户获取）
    ///
    /// # 返回
    /// 转换后的最小单位金额
    pub fn calculate_deposit_with_decimals(base_amount: u64, decimals: u8) -> u64 {
        base_amount.saturating_mul(10_u64.saturating_pow(decimals as u32))
    }

    /// 设置保证金要求（根据Token精度动态计算）
    pub fn set_deposit_requirement_with_decimals(&mut self, base_amount: u64, decimals: u8) {
        self.merchant_deposit_required =
            Self::calculate_deposit_with_decimals(base_amount, decimals);
    }
}
