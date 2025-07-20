// 简单的哈希函数，用于布隆过滤器
pub fn hash_keyword(keyword: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in keyword.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// 多重哈希函数，用于布隆过滤器
pub fn multi_hash(data: u64, seed: u8) -> u64 {
    let mut hash = data;
    hash ^= seed as u64;
    hash = hash.wrapping_mul(0x9e3779b97f4a7c15);
    hash ^= hash >> 30;
    hash = hash.wrapping_mul(0xbf58476d1ce4e5b9);
    hash ^= hash >> 27;
    hash = hash.wrapping_mul(0x94d049bb133111eb);
    hash ^= hash >> 31;
    hash
}

// 计算字符串的哈希值
pub fn hash_string(s: &str) -> u64 {
    let mut hash = 0u64;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

// 计算数字的哈希值
pub fn hash_u64(value: u64) -> u64 {
    let mut hash = value;
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51afd7ed558ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ceb9fe1a85ec53);
    hash ^= hash >> 33;
    hash
}
