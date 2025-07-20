use crate::error::ErrorCode;
use anchor_lang::prelude::*;

#[account]
pub struct KeywordRoot {
    pub keyword: String,
    pub total_shards: u8,
    pub first_shard: Pubkey,
    pub last_shard: Pubkey,
    pub total_products: u32,
    pub bloom_filter: [u8; super::BLOOM_FILTER_SIZE],
    pub bump: u8,
}

impl KeywordRoot {
    pub const LEN: usize = 8 + (4 + 128) + 1 + 32 + 32 + 4 + super::BLOOM_FILTER_SIZE + 1;

    pub fn seeds(keyword: &str) -> Vec<Vec<u8>> {
        vec![b"keyword_root".to_vec(), keyword.as_bytes().to_vec()]
    }

    pub fn initialize(&mut self, keyword: String, bump: u8) -> Result<()> {
        require!(
            keyword.len() <= super::MAX_KEYWORD_LENGTH,
            ErrorCode::InvalidKeywordLength
        );

        self.keyword = keyword;
        self.total_shards = 0;
        self.first_shard = Pubkey::default();
        self.last_shard = Pubkey::default();
        self.total_products = 0;
        self.bloom_filter = [0; super::BLOOM_FILTER_SIZE];
        self.bump = bump;

        Ok(())
    }

    pub fn add_shard(&mut self, shard_key: Pubkey) {
        if self.total_shards == 0 {
            self.first_shard = shard_key;
            self.last_shard = shard_key;
        } else {
            self.last_shard = shard_key;
        }
        self.total_shards += 1;
    }

    pub fn update_bloom_filter(&mut self, product_id: u64, add: bool) {
        let hash1 = (product_id as usize) % (super::BLOOM_FILTER_SIZE * 8);
        let hash2 = ((product_id * 31) as usize) % (super::BLOOM_FILTER_SIZE * 8);
        let hash3 = ((product_id * 37) as usize) % (super::BLOOM_FILTER_SIZE * 8);

        for hash in [hash1, hash2, hash3] {
            let byte_index = hash / 8;
            let bit_index = hash % 8;
            if add {
                self.bloom_filter[byte_index] |= 1 << bit_index;
            } else {
                self.bloom_filter[byte_index] &= !(1 << bit_index);
            }
        }
    }

    pub fn might_contain(&self, product_id: u64) -> bool {
        let hash1 = (product_id as usize) % (super::BLOOM_FILTER_SIZE * 8);
        let hash2 = ((product_id * 31) as usize) % (super::BLOOM_FILTER_SIZE * 8);
        let hash3 = ((product_id * 37) as usize) % (super::BLOOM_FILTER_SIZE * 8);

        for hash in [hash1, hash2, hash3] {
            let byte_index = hash / 8;
            let bit_index = hash % 8;
            if (self.bloom_filter[byte_index] >> bit_index) & 1 == 0 {
                return false;
            }
        }
        true
    }
}

#[account]
pub struct KeywordShard {
    pub keyword: String,
    pub shard_index: u32,
    pub prev_shard: Pubkey,
    pub next_shard: Option<Pubkey>,
    pub product_ids: Vec<u64>,
    pub min_id: u64,
    pub max_id: u64,
    pub bloom_summary: [u8; super::BLOOM_SUMMARY_SIZE],
    pub bump: u8,
}

impl KeywordShard {
    pub const LEN: usize = 8
        + (4 + 128)
        + 4
        + 32
        + 33
        + (4 + super::MAX_PRODUCTS_PER_SHARD * 8)
        + 8
        + 8
        + super::BLOOM_SUMMARY_SIZE
        + 1;

    pub fn seeds(keyword: &str, shard_index: u32) -> Vec<Vec<u8>> {
        vec![
            b"keyword_shard".to_vec(),
            keyword.as_bytes().to_vec(),
            shard_index.to_le_bytes().to_vec(),
        ]
    }

    pub fn initialize(
        &mut self,
        keyword: String,
        shard_index: u32,
        prev_shard: Pubkey,
        bump: u8,
    ) -> Result<()> {
        require!(
            keyword.len() <= super::MAX_KEYWORD_LENGTH,
            ErrorCode::InvalidKeywordLength
        );

        self.keyword = keyword;
        self.shard_index = shard_index;
        self.prev_shard = prev_shard;
        self.next_shard = None;
        self.product_ids = Vec::new();
        self.min_id = u64::MAX;
        self.max_id = 0;
        self.bloom_summary = [0; super::BLOOM_SUMMARY_SIZE];
        self.bump = bump;

        Ok(())
    }

    pub fn add_product(&mut self, product_id: u64) -> Result<()> {
        require!(
            self.product_ids.len() < super::MAX_PRODUCTS_PER_SHARD,
            ErrorCode::ShardIsFull
        );

        if !self.product_ids.contains(&product_id) {
            self.product_ids.push(product_id);
            self.update_min_max(product_id);
            self.update_bloom_summary(product_id, true);
        }

        Ok(())
    }

    pub fn remove_product(&mut self, product_id: u64) -> Result<bool> {
        if let Some(index) = self.product_ids.iter().position(|&x| x == product_id) {
            self.product_ids.remove(index);
            self.recalculate_min_max();
            self.recalculate_bloom_summary();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn update_min_max(&mut self, product_id: u64) {
        if self.min_id == u64::MAX {
            self.min_id = product_id;
            self.max_id = product_id;
        } else {
            self.min_id = self.min_id.min(product_id);
            self.max_id = self.max_id.max(product_id);
        }
    }

    fn recalculate_min_max(&mut self) {
        if self.product_ids.is_empty() {
            self.min_id = u64::MAX;
            self.max_id = 0;
        } else {
            self.min_id = *self.product_ids.iter().min().unwrap();
            self.max_id = *self.product_ids.iter().max().unwrap();
        }
    }

    fn update_bloom_summary(&mut self, product_id: u64, add: bool) {
        let hash1 = (product_id as usize) % (super::BLOOM_SUMMARY_SIZE * 8);
        let hash2 = ((product_id * 31) as usize) % (super::BLOOM_SUMMARY_SIZE * 8);

        for hash in [hash1, hash2] {
            let byte_index = hash / 8;
            let bit_index = hash % 8;
            if add {
                self.bloom_summary[byte_index] |= 1 << bit_index;
            } else {
                self.bloom_summary[byte_index] &= !(1 << bit_index);
            }
        }
    }

    fn recalculate_bloom_summary(&mut self) {
        self.bloom_summary = [0; super::BLOOM_SUMMARY_SIZE];
        let product_ids = self.product_ids.clone();
        for product_id in product_ids {
            self.update_bloom_summary(product_id, true);
        }
    }

    pub fn is_full(&self) -> bool {
        self.product_ids.len() >= super::MAX_PRODUCTS_PER_SHARD
    }

    pub fn utilization_rate(&self) -> f32 {
        self.product_ids.len() as f32 / super::MAX_PRODUCTS_PER_SHARD as f32
    }

    pub fn needs_split(&self) -> bool {
        self.utilization_rate() > 0.8
    }

    pub fn needs_merge(&self) -> bool {
        self.utilization_rate() < 0.25
    }
}
