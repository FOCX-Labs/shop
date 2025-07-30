use crate::error::ErrorCode;
use anchor_lang::prelude::*;

// Product base information account (core business data)
#[account]
#[derive(InitSpace)]
pub struct ProductBase {
    pub id: u64,
    pub merchant: Pubkey,
    #[max_len(64)]
    pub name: String,
    #[max_len(256)]
    pub description: String,
    pub price: u64, // Token price (unified using token units)
    #[max_len(128)]
    pub keywords: String, // Keywords, comma-separated (core search field)
    pub inventory: u64, // Inventory quantity
    pub sales: u32,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub payment_token: Pubkey, // Payment token mint (required)
    #[max_len(128)]
    pub shipping_location: String, // Shipping address
    pub bump: u8,
}

// 产品扩展信息账户（可选的营销和展示数据）
#[account]
#[derive(InitSpace)]
pub struct ProductExtended {
    pub product_id: u64, // 关联到ProductBase的ID
    #[max_len(512)]
    pub image_video_urls: String, // 图片/视频URL，用逗号分隔
    #[max_len(256)]
    pub sales_regions: String, // 销售地区，用逗号分隔
    #[max_len(256)]
    pub logistics_methods: String, // 物流方式，用逗号分隔
    pub bump: u8,
}

// 为了向后兼容，保留Product结构体作为账户（使用ProductBase的数据）
pub type Product = ProductBase;

impl ProductBase {
    pub fn seeds(&self) -> Vec<Vec<u8>> {
        vec![b"product".to_vec(), self.id.to_le_bytes().to_vec()]
    }

    // 基础验证方法
    pub fn validate(&self) -> Result<()> {
        require!(!self.name.is_empty(), ErrorCode::InvalidProductName);
        require!(
            self.name.len() <= MAX_PRODUCT_NAME_LENGTH,
            ErrorCode::InvalidProductNameLength
        );
        require!(
            !self.description.is_empty(),
            ErrorCode::InvalidProductDescription
        );
        require!(
            self.description.len() <= MAX_PRODUCT_DESCRIPTION_LENGTH,
            ErrorCode::InvalidProductDescriptionLength
        );
        require!(self.price > 0, ErrorCode::InvalidPrice);
        Ok(())
    }

    pub fn update_price(&mut self, new_price: u64) -> Result<()> {
        require!(new_price > 0, ErrorCode::InvalidPrice);
        self.price = new_price;
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
    pub fn initialize_payment_config(&mut self, payment_token: Pubkey, price: u64) -> Result<()> {
        self.payment_token = payment_token;
        self.price = price;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn supports_token_payment(&self, mint: &Pubkey) -> bool {
        &self.payment_token == mint
    }

    pub fn get_token_price(&self) -> u64 {
        self.price
    }

    pub fn update_payment_config(&mut self, payment_token: Pubkey, price: u64) -> Result<()> {
        self.payment_token = payment_token;
        self.price = price;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    // 静态辅助方法
    pub fn seeds_static(product_id: u64) -> Vec<Vec<u8>> {
        vec![b"product".to_vec(), product_id.to_le_bytes().to_vec()]
    }

    // Keywords相关方法
    pub fn parse_keywords(&self) -> Vec<String> {
        if self.keywords.is_empty() {
            Vec::new()
        } else {
            self.keywords
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        }
    }

    pub fn update_keywords(&mut self, new_keywords: Vec<String>) -> Result<()> {
        require!(
            new_keywords.len() <= MAX_KEYWORDS_PER_PRODUCT,
            ErrorCode::TooManyKeywords
        );

        for keyword in &new_keywords {
            require!(
                keyword.len() <= MAX_KEYWORD_LENGTH,
                ErrorCode::InvalidKeyword
            );
        }

        self.keywords = new_keywords.join(",");
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

impl ProductExtended {
    pub fn seeds(&self) -> Vec<Vec<u8>> {
        vec![
            b"product_extended".to_vec(),
            self.product_id.to_le_bytes().to_vec(),
            b"v1".to_vec(),
        ]
    }

    // 辅助方法：解析逗号分隔的字符串为Vec
    pub fn parse_image_urls(&self) -> Vec<String> {
        if self.image_video_urls.is_empty() {
            Vec::new()
        } else {
            self.image_video_urls
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        }
    }

    pub fn parse_sales_regions(&self) -> Vec<String> {
        if self.sales_regions.is_empty() {
            Vec::new()
        } else {
            self.sales_regions
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        }
    }

    pub fn parse_logistics_methods(&self) -> Vec<String> {
        if self.logistics_methods.is_empty() {
            Vec::new()
        } else {
            self.logistics_methods
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        }
    }

    // 辅助方法：将Vec转换为逗号分隔的字符串
    pub fn vec_to_image_urls_string(vec: Vec<String>) -> String {
        vec.join(",")
    }

    pub fn vec_to_sales_regions_string(vec: Vec<String>) -> String {
        vec.join(",")
    }

    pub fn vec_to_logistics_methods_string(vec: Vec<String>) -> String {
        vec.join(",")
    }
}

// 为了向后兼容，为Product类型别名添加一些辅助方法
impl Product {
    // 向后兼容的辅助函数（现在返回字符串而不是数组）
    pub fn vec_to_keywords_array(vec: Vec<String>) -> String {
        vec.join(",")
    }

    pub fn vec_to_image_urls_array(vec: Vec<String>) -> String {
        ProductExtended::vec_to_image_urls_string(vec)
    }

    pub fn vec_to_sales_regions_array(vec: Vec<String>) -> String {
        ProductExtended::vec_to_sales_regions_string(vec)
    }

    pub fn vec_to_logistics_methods_array(vec: Vec<String>) -> String {
        ProductExtended::vec_to_logistics_methods_string(vec)
    }
}

// 常量定义
pub const MAX_PRODUCT_NAME_LENGTH: usize = 64;
pub const MAX_PRODUCT_DESCRIPTION_LENGTH: usize = 256;
pub const MAX_KEYWORDS_PER_PRODUCT: usize = 10;
pub const MAX_KEYWORD_LENGTH: usize = 32;

// 产品信息结构体（用于序列化）
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProductInfo {
    pub name: String,
    pub description: String,
    pub price: u64,
    pub keywords: Vec<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProductSearchResult {
    pub id: u64,
    pub merchant: Pubkey,
    pub name: String,
    pub description: String,
    pub price: u64,
    pub keywords: Vec<String>,
    pub inventory: u64,
    pub sales: u32,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ProductSales {
    pub product_id: u64,
    pub merchant: Pubkey,
    #[max_len(32)]
    pub name: String,
    pub price: u64,
    pub sales: u32,
    pub last_update: i64,
}
