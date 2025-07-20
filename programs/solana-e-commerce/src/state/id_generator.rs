use anchor_lang::prelude::*;

pub const DEFAULT_CHUNK_SIZE: u32 = 10_000;
pub const MAX_CHUNKS_PER_MERCHANT: u32 = 100;
pub const ID_CHUNK_BITMAP_SIZE: usize = 1250; // 10,000位/8 = 1250字节

#[account]
pub struct GlobalIdRoot {
    pub last_merchant_id: u32,
    pub last_global_id: u64,
    pub chunk_size: u32,
    pub merchants: Vec<Pubkey>,
    pub max_products_per_shard: u16,
    pub max_keywords_per_product: u8,
    pub bloom_filter_size: u16,
    pub cache_ttl: u32,
    pub bump: u8,
}

impl GlobalIdRoot {
    pub const LEN: usize = 8 + 4 + 8 + 4 + (4 + 32 * 100) + 2 + 1 + 2 + 4 + 1; // 预留100个商户，避免超过10KB限制

    pub fn seeds() -> &'static [&'static [u8]] {
        &[b"global_id_root"]
    }
}

#[account]
pub struct MerchantIdAccount {
    pub merchant_id: u32,
    pub last_chunk_index: u32,
    pub last_local_id: u32,
    pub active_chunk: Pubkey,
    pub unused_chunks: Vec<Pubkey>,
    pub bump: u8,
}

impl MerchantIdAccount {
    pub const LEN: usize = 8 + 4 + 4 + 4 + 32 + (4 + 32 * 100) + 1; // 预留100个块

    pub fn seeds(merchant: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"merchant_id".to_vec(), merchant.as_ref().to_vec()]
    }
}

#[account]
pub struct IdChunk {
    pub merchant_id: u32,
    pub chunk_index: u32,
    pub start_id: u64,
    pub end_id: u64,
    pub next_available: u32,
    pub bitmap: Vec<u8>, // 改为Vec<u8>以避免栈溢出
    pub bump: u8,
}

impl IdChunk {
    pub const LEN: usize = 8 + 4 + 4 + 8 + 8 + 4 + (4 + ID_CHUNK_BITMAP_SIZE) + 1; // Vec<u8>需要4字节长度前缀

    pub fn seeds(merchant_id: u32, chunk_index: u32) -> Vec<Vec<u8>> {
        vec![
            b"id_chunk".to_vec(),
            merchant_id.to_le_bytes().to_vec(),
            chunk_index.to_le_bytes().to_vec(),
        ]
    }

    pub fn capacity(&self) -> u32 {
        ((self.end_id - self.start_id) + 1) as u32
    }

    pub fn is_id_used(&self, local_id: u32) -> bool {
        let byte_index = (local_id / 8) as usize;
        let bit_index = local_id % 8;
        if byte_index >= self.bitmap.len() {
            return false;
        }
        (self.bitmap[byte_index] >> bit_index) & 1 == 1
    }

    pub fn mark_id_used(&mut self, local_id: u32) {
        let byte_index = (local_id / 8) as usize;
        let bit_index = local_id % 8;
        if byte_index < self.bitmap.len() {
            self.bitmap[byte_index] |= 1 << bit_index;
        }
    }

    pub fn clear_id(&mut self, local_id: u32) {
        let byte_index = (local_id / 8) as usize;
        let bit_index = local_id % 8;
        if byte_index < self.bitmap.len() {
            self.bitmap[byte_index] &= !(1 << bit_index);
        }
    }

    /// 安全初始化bitmap
    pub fn initialize_bitmap(&mut self) {
        self.bitmap = vec![0u8; ID_CHUNK_BITMAP_SIZE];
    }

    pub fn is_full(&self) -> bool {
        self.next_available >= self.capacity()
    }

    pub fn utilization_rate(&self) -> f32 {
        self.next_available as f32 / self.capacity() as f32
    }
}
