use anchor_lang::prelude::*;

// 验证产品名称
pub fn validate_product_name(name: &str) -> Result<()> {
    require!(!name.is_empty(), ErrorCode::EmptyProductName);
    require!(name.len() <= 100, ErrorCode::ProductNameTooLong);
    require!(name.trim() == name, ErrorCode::InvalidProductName);
    Ok(())
}

// 验证产品描述
pub fn validate_product_description(description: &str) -> Result<()> {
    require!(description.len() <= 1000, ErrorCode::DescriptionTooLong);
    Ok(())
}

// 验证价格
pub fn validate_price(price: u64) -> Result<()> {
    require!(price > 0, ErrorCode::InvalidPrice);
    require!(price <= 1_000_000_000_000, ErrorCode::PriceTooHigh); // 1万亿 lamports
    Ok(())
}

// 验证关键词
pub fn validate_keyword(keyword: &str) -> Result<()> {
    require!(!keyword.is_empty(), ErrorCode::EmptyKeyword);
    require!(keyword.len() <= 50, ErrorCode::KeywordTooLong);
    require!(keyword.trim() == keyword, ErrorCode::InvalidKeyword);
    require!(!keyword.contains(' '), ErrorCode::KeywordContainsSpace);
    Ok(())
}

// 验证关键词列表
pub fn validate_keywords(keywords: &[String]) -> Result<()> {
    require!(!keywords.is_empty(), ErrorCode::NoKeywords);
    require!(keywords.len() <= 10, ErrorCode::TooManyKeywords);

    for keyword in keywords {
        validate_keyword(keyword)?;
    }

    // 检查重复关键词
    for i in 0..keywords.len() {
        for j in i + 1..keywords.len() {
            require!(keywords[i] != keywords[j], ErrorCode::DuplicateKeyword);
        }
    }

    Ok(())
}

// 验证商户名称
pub fn validate_merchant_name(name: &str) -> Result<()> {
    require!(!name.is_empty(), ErrorCode::EmptyMerchantName);
    require!(name.len() <= 50, ErrorCode::MerchantNameTooLong);
    require!(name.trim() == name, ErrorCode::InvalidMerchantName);
    Ok(())
}

// 验证商户邮箱
pub fn validate_email(email: &str) -> Result<()> {
    require!(!email.is_empty(), ErrorCode::EmptyEmail);
    require!(email.len() <= 100, ErrorCode::EmailTooLong);
    require!(email.contains('@'), ErrorCode::InvalidEmailFormat);
    require!(
        email.matches('@').count() == 1,
        ErrorCode::InvalidEmailFormat
    );

    let parts: Vec<&str> = email.split('@').collect();
    require!(parts[0].len() > 0, ErrorCode::InvalidEmailFormat);
    require!(parts[1].len() > 0, ErrorCode::InvalidEmailFormat);
    require!(parts[1].contains('.'), ErrorCode::InvalidEmailFormat);

    Ok(())
}

// 验证URL
pub fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Ok(()); // URL是可选的
    }

    require!(url.len() <= 200, ErrorCode::UrlTooLong);
    require!(
        url.starts_with("http://") || url.starts_with("https://"),
        ErrorCode::InvalidUrlFormat
    );

    Ok(())
}

// 验证库存数量
pub fn validate_stock(stock: u32) -> Result<()> {
    require!(stock <= 1_000_000, ErrorCode::StockTooHigh);
    Ok(())
}

// 验证销量
pub fn validate_sales_count(sales: u32) -> Result<()> {
    require!(sales <= 10_000_000, ErrorCode::SalesTooHigh);
    Ok(())
}

// 验证评分
pub fn validate_rating(rating: u8) -> Result<()> {
    require!(rating <= 5, ErrorCode::InvalidRating);
    Ok(())
}

// 验证分片ID
pub fn validate_shard_id(shard_id: u32) -> Result<()> {
    require!(shard_id < 10000, ErrorCode::InvalidShardId);
    Ok(())
}

// 验证产品ID
pub fn validate_product_id(product_id: u64) -> Result<()> {
    require!(product_id > 0, ErrorCode::InvalidProductId);
    Ok(())
}

// 验证商户ID
pub fn validate_merchant_id(merchant_id: u64) -> Result<()> {
    require!(merchant_id > 0, ErrorCode::InvalidMerchantId);
    Ok(())
}

// 验证价格范围
pub fn validate_price_range(min_price: u64, max_price: u64) -> Result<()> {
    require!(min_price <= max_price, ErrorCode::InvalidPriceRange);
    validate_price(min_price)?;
    validate_price(max_price)?;
    Ok(())
}

// 验证销量范围
pub fn validate_sales_range(min_sales: u32, max_sales: u32) -> Result<()> {
    require!(min_sales <= max_sales, ErrorCode::InvalidSalesRange);
    validate_sales_count(min_sales)?;
    validate_sales_count(max_sales)?;
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Product name cannot be empty")]
    EmptyProductName,
    #[msg("Product name too long")]
    ProductNameTooLong,
    #[msg("Invalid product name")]
    InvalidProductName,
    #[msg("Description too long")]
    DescriptionTooLong,
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Price too high")]
    PriceTooHigh,
    #[msg("Keyword cannot be empty")]
    EmptyKeyword,
    #[msg("Keyword too long")]
    KeywordTooLong,
    #[msg("Invalid keyword")]
    InvalidKeyword,
    #[msg("Keyword cannot contain spaces")]
    KeywordContainsSpace,
    #[msg("No keywords provided")]
    NoKeywords,
    #[msg("Too many keywords")]
    TooManyKeywords,
    #[msg("Duplicate keyword")]
    DuplicateKeyword,
    #[msg("Merchant name cannot be empty")]
    EmptyMerchantName,
    #[msg("Merchant name too long")]
    MerchantNameTooLong,
    #[msg("Invalid merchant name")]
    InvalidMerchantName,
    #[msg("Email cannot be empty")]
    EmptyEmail,
    #[msg("Email too long")]
    EmailTooLong,
    #[msg("Invalid email format")]
    InvalidEmailFormat,
    #[msg("URL too long")]
    UrlTooLong,
    #[msg("Invalid URL format")]
    InvalidUrlFormat,
    #[msg("Stock too high")]
    StockTooHigh,
    #[msg("Sales count too high")]
    SalesTooHigh,
    #[msg("Invalid rating")]
    InvalidRating,
    #[msg("Invalid shard ID")]
    InvalidShardId,
    #[msg("Invalid product ID")]
    InvalidProductId,
    #[msg("Invalid merchant ID")]
    InvalidMerchantId,
    #[msg("Invalid price range")]
    InvalidPriceRange,
    #[msg("Invalid sales range")]
    InvalidSalesRange,
}
