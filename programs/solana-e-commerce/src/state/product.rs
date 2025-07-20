use crate::error::ErrorCode;
use anchor_lang::prelude::*;

#[account]
pub struct Product {
    pub id: u64,
    pub merchant: Pubkey,
    pub name: String,
    pub description: String,
    pub price: u64, // SOL价格（lamports）
    pub keywords: Vec<String>,
    pub sales: u32,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,

    // 新增支付相关字段
    pub payment_token: Pubkey, // 支付代币mint（必需）
    pub token_decimals: u8,    // 代币精度
    pub token_price: u64,      // 代币价格

    // 新增产品扩展字段
    pub image_video_urls: Vec<String>,  // 商品图片/视频的链接列表
    pub shipping_location: String,      // 发货地址
    pub sales_regions: Vec<String>,     // 销售地理范围列表
    pub logistics_methods: Vec<String>, // 物流方式列表
}

impl Product {
    pub const LEN: usize = 8        // id
        + 8                     // merchant (Pubkey)
        + 32                    // merchant
        + (4 + 128)             // name
        + (4 + 512)             // description
        + 8                     // price
        + (4 + 10 * (4 + 32))   // keywords
        + 4                     // sales
        + 1                     // is_active
        + 8                     // created_at
        + 8                     // updated_at
        + 1                     // bump
        + 32                    // payment_token
        + 1                     // token_decimals
        + 8                     // token_price
        + (4 + 10 * (4 + 256))  // image_video_urls (最多10个，每个256字符)
        + (4 + 128)             // shipping_location
        + (4 + 20 * (4 + 64))   // sales_regions (最多20个，每个64字符)
        + (4 + 10 * (4 + 64)); // logistics_methods (最多10个，每个64字符)

    pub fn seeds(product_id: u64) -> Vec<Vec<u8>> {
        vec![b"product".to_vec(), product_id.to_le_bytes().to_vec()]
    }

    pub fn validate(&self) -> Result<()> {
        require!(
            self.name.len() <= super::MAX_PRODUCT_NAME_LENGTH,
            ErrorCode::InvalidProductNameLength
        );
        require!(
            self.description.len() <= super::MAX_PRODUCT_DESCRIPTION_LENGTH,
            ErrorCode::InvalidProductDescriptionLength
        );
        require!(
            self.keywords.len() <= super::MAX_KEYWORDS_PER_PRODUCT,
            ErrorCode::InvalidKeywordCount
        );

        for keyword in &self.keywords {
            require!(
                keyword.len() <= super::MAX_KEYWORD_LENGTH,
                ErrorCode::InvalidKeywordLength
            );
        }

        require!(self.price > 0, ErrorCode::InvalidPrice);

        Ok(())
    }

    pub fn update_price(&mut self, new_price: u64) -> Result<()> {
        require!(new_price > 0, ErrorCode::InvalidPrice);
        self.price = new_price;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn update_keywords(&mut self, new_keywords: Vec<String>) -> Result<()> {
        require!(
            new_keywords.len() <= super::MAX_KEYWORDS_PER_PRODUCT,
            ErrorCode::InvalidKeywordCount
        );

        for keyword in &new_keywords {
            require!(
                keyword.len() <= super::MAX_KEYWORD_LENGTH,
                ErrorCode::InvalidKeywordLength
            );
        }

        self.keywords = new_keywords;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn update_sales(&mut self, sales_increment: u32) -> Result<()> {
        self.sales = self.sales.saturating_add(sales_increment);
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn set_active(&mut self, active: bool) -> Result<()> {
        self.is_active = active;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    // SPL代币支付相关方法
    pub fn initialize_payment_config(
        &mut self,
        payment_token: Pubkey,
        token_decimals: u8,
        token_price: u64,
    ) -> Result<()> {
        self.payment_token = payment_token;
        self.token_decimals = token_decimals;
        self.token_price = token_price;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn supports_token_payment(&self, mint: &Pubkey) -> bool {
        &self.payment_token == mint
    }

    pub fn get_token_price(&self) -> u64 {
        self.token_price
    }

    pub fn update_payment_config(
        &mut self,
        payment_token: Pubkey,
        token_decimals: u8,
        token_price: u64,
    ) -> Result<()> {
        self.payment_token = payment_token;
        self.token_decimals = token_decimals;
        self.token_price = token_price;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProductInfo {
    pub name: String,
    pub description: String,
    pub price: u64,
    pub keywords: Vec<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProductUpdates {
    pub update_name: bool,
    pub name: String,
    pub update_description: bool,
    pub description: String,
    pub update_price: bool,
    pub price: u64,
    pub update_keywords: bool,
    pub keywords: Vec<String>,
    pub update_is_active: bool,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum ProductStatus {
    Active,
    Inactive,
    Deleted,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProductSales {
    pub product_id: u64,
    pub sales: u32,
    pub last_update: i64,
}
