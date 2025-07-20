use super::hash::multi_hash;
use anchor_lang::prelude::*;

pub const BLOOM_FILTER_SIZE: usize = 256;
pub const BLOOM_HASH_COUNT: u8 = 3;

// 布隆过滤器操作
pub struct BloomFilter;

impl BloomFilter {
    // 添加元素到布隆过滤器
    pub fn add(filter: &mut [u8; BLOOM_FILTER_SIZE], value: u64) {
        for i in 0..BLOOM_HASH_COUNT {
            let hash = multi_hash(value, i);
            let bit_index = (hash % (BLOOM_FILTER_SIZE as u64 * 8)) as usize;
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;
            filter[byte_index] |= 1 << bit_offset;
        }
    }

    // 检查元素是否可能存在于布隆过滤器中
    pub fn might_contain(filter: &[u8; BLOOM_FILTER_SIZE], value: u64) -> bool {
        for i in 0..BLOOM_HASH_COUNT {
            let hash = multi_hash(value, i);
            let bit_index = (hash % (BLOOM_FILTER_SIZE as u64 * 8)) as usize;
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;
            if (filter[byte_index] & (1 << bit_offset)) == 0 {
                return false;
            }
        }
        true
    }

    // 清空布隆过滤器
    pub fn clear(filter: &mut [u8; BLOOM_FILTER_SIZE]) {
        filter.fill(0);
    }

    // 计算布隆过滤器的填充率
    pub fn fill_rate(filter: &[u8; BLOOM_FILTER_SIZE]) -> f32 {
        let mut set_bits = 0;
        for byte in filter {
            set_bits += byte.count_ones();
        }
        set_bits as f32 / (BLOOM_FILTER_SIZE as f32 * 8.0)
    }

    // 合并两个布隆过滤器
    pub fn merge(dest: &mut [u8; BLOOM_FILTER_SIZE], src: &[u8; BLOOM_FILTER_SIZE]) {
        for i in 0..BLOOM_FILTER_SIZE {
            dest[i] |= src[i];
        }
    }

    // 估算布隆过滤器中的元素数量
    pub fn estimate_count(filter: &[u8; BLOOM_FILTER_SIZE]) -> u32 {
        let fill_rate = Self::fill_rate(filter);
        if fill_rate >= 1.0 {
            return u32::MAX;
        }

        let m = BLOOM_FILTER_SIZE as f32 * 8.0; // 总位数
        let k = BLOOM_HASH_COUNT as f32; // 哈希函数数量

        // 使用布隆过滤器的标准估算公式
        let estimated = -(m / k) * (1.0 - fill_rate).ln();
        estimated.max(0.0) as u32
    }
}

// 布隆过滤器摘要（用于快速比较）
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub struct BloomSummary {
    pub checksum: u64,
    pub fill_rate: u16, // 填充率 * 10000
    pub estimated_count: u32,
}

impl BloomSummary {
    pub fn from_filter(filter: &[u8; BLOOM_FILTER_SIZE]) -> Self {
        let mut checksum = 0u64;
        for chunk in filter.chunks(8) {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            checksum ^= u64::from_le_bytes(bytes);
        }

        let fill_rate = (BloomFilter::fill_rate(filter) * 10000.0) as u16;
        let estimated_count = BloomFilter::estimate_count(filter);

        Self {
            checksum,
            fill_rate,
            estimated_count,
        }
    }

    pub fn get_fill_rate(&self) -> f32 {
        self.fill_rate as f32 / 10000.0
    }
}
