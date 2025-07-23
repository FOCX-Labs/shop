pub mod id_generator;
pub mod keyword_index;
pub mod merchant;
pub mod order;
pub mod payment;
pub mod price_index;
pub mod product;
pub mod sales_index;
pub mod user_purchase_count;

pub use id_generator::*;
pub use keyword_index::*;
pub use merchant::*;
pub use order::*;
pub use payment::*;
pub use price_index::*;
pub use product::*;
pub use sales_index::*;
pub use user_purchase_count::*;

// 系统常量
pub const MAX_PRODUCTS_PER_SHARD: usize = 100;
pub const MAX_KEYWORDS_PER_PRODUCT: usize = 10;
pub const MAX_KEYWORD_LENGTH: usize = 32;
pub const MAX_PRODUCT_NAME_LENGTH: usize = 100;
pub const MAX_PRODUCT_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_MERCHANT_NAME_LENGTH: usize = 100;
pub const MAX_MERCHANT_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_SHARDS_PER_KEYWORD: usize = 100;
pub const BLOOM_FILTER_SIZE: usize = 256;
pub const BLOOM_SUMMARY_SIZE: usize = 32;
