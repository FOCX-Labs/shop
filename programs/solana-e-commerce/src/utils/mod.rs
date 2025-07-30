use anchor_lang::prelude::*;

pub mod bloom;
pub mod hash;
pub mod pagination;
pub mod validation;

pub use bloom::*;
pub use hash::*;
pub use pagination::*;
pub use validation::*;

// Bloom filter related constants
pub const BLOOM_FILTER_SIZE: usize = 256;
pub const BLOOM_HASH_COUNT: u8 = 3;

// Pagination related constants
pub const DEFAULT_PAGE_SIZE: u16 = 20;
pub const MAX_PAGE_SIZE: u16 = 100;

// Index sharding related constants
pub const SHARD_SPLIT_THRESHOLD: f32 = 0.8;
pub const SHARD_MERGE_THRESHOLD: f32 = 0.25;
pub const MIN_SHARD_SIZE: usize = 10;

// Search result sorting
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

// Search result filter
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SearchFilter {
    pub price_min: Option<u64>,
    pub price_max: Option<u64>,
    pub sales_min: Option<u32>,
    pub sales_max: Option<u32>,
    pub merchant: Option<Pubkey>,
    pub keywords: Option<Vec<String>>,
    pub is_active_only: bool,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            price_min: None,
            price_max: None,
            sales_min: None,
            sales_max: None,
            merchant: None,
            keywords: None,
            is_active_only: true,
        }
    }
}

// Search result
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SearchResult {
    pub product_ids: Vec<u64>,
    pub total_count: u32,
    pub has_more: bool,
    pub next_offset: u32,
}

impl SearchResult {
    pub fn new(product_ids: Vec<u64>, total_count: u32, offset: u32, limit: u16) -> Self {
        let has_more = (offset + limit as u32) < total_count;
        let next_offset = if has_more {
            offset + limit as u32
        } else {
            total_count
        };

        Self {
            product_ids,
            total_count,
            has_more,
            next_offset,
        }
    }
}

// Performance statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PerformanceStats {
    pub query_time_ms: u64,
    pub total_shards_searched: u32,
    pub bloom_filter_hits: u32,
    pub exact_matches: u32,
    pub false_positives: u32,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            query_time_ms: 0,
            total_shards_searched: 0,
            bloom_filter_hits: 0,
            exact_matches: 0,
            false_positives: 0,
        }
    }
}

// Memory usage statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MemoryStats {
    pub total_accounts: u32,
    pub total_size_bytes: u64,
    pub keyword_indexes: u32,
    pub price_indexes: u32,
    pub sales_indexes: u32,
    pub product_accounts: u32,
}

// Calculate account rent
pub fn calculate_rent(size: usize, rent: &Rent) -> u64 {
    rent.minimum_balance(size)
}

// Verify PDA seeds
pub fn verify_pda(expected_key: &Pubkey, seeds: &[&[u8]], program_id: &Pubkey) -> Result<u8> {
    let (derived_key, bump) = Pubkey::find_program_address(seeds, program_id);
    require!(
        expected_key == &derived_key,
        crate::error::ErrorCode::InvalidPda
    );
    Ok(bump)
}

// Timestamp utilities
pub fn get_current_timestamp() -> Result<i64> {
    Ok(Clock::get()?.unix_timestamp)
}

// Calculate difference between two timestamps (seconds)
pub fn calculate_time_diff(start: i64, end: i64) -> u64 {
    (end - start).max(0) as u64
}

// Generate random seed (for bloom filter, etc.)
pub fn generate_seed(base: u64, salt: u64) -> u64 {
    base.wrapping_mul(0x9e3779b97f4a7c15)
        .wrapping_add(salt)
        .wrapping_mul(0x85ebca6b)
        .rotate_left(13)
}

// Check if string is a valid keyword
pub fn is_valid_keyword(keyword: &str) -> bool {
    !keyword.is_empty()
        && keyword.len() <= crate::state::MAX_KEYWORD_LENGTH
        && keyword
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_')
}

// Normalize keyword (convert to lowercase, remove extra spaces)
pub fn normalize_keyword(keyword: &str) -> String {
    keyword
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

// 计算集合交集
pub fn intersect_sorted_vecs(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                result.push(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }

    result
}

// 计算集合并集
pub fn union_sorted_vecs(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                result.push(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => {
                result.push(a[i]);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                result.push(b[j]);
                j += 1;
            }
        }
    }

    // 添加剩余元素
    while i < a.len() {
        result.push(a[i]);
        i += 1;
    }
    while j < b.len() {
        result.push(b[j]);
        j += 1;
    }

    result
}

// 检查向量是否已排序
pub fn is_sorted(vec: &[u64]) -> bool {
    vec.windows(2).all(|w| w[0] <= w[1])
}

// 二分搜索
pub fn binary_search_range(vec: &[u64], min: u64, max: u64) -> (usize, usize) {
    let start = vec.binary_search(&min).unwrap_or_else(|x| x);
    let end = vec.binary_search(&(max + 1)).unwrap_or_else(|x| x);
    (start, end)
}
