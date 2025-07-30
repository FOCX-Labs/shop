use anchor_lang::prelude::*;

/// 商家订单账户 - 纯索引功能，用于商家快速查询和定位买家订单
#[account]
#[derive(InitSpace)]
pub struct MerchantOrder {
    pub merchant: Pubkey,                   // 商家地址 (32字节)
    pub buyer: Pubkey,                      // 买家地址 (32字节)
    pub merchant_order_sequence: u64,       // 商家订单序列号 (8字节)
    pub buyer_order_pda: Pubkey,            // 关联的买家订单PDA (32字节)
    pub product_id: u64,                    // 产品ID (8字节)
    pub created_at: i64,                    // 创建时间 (8字节)
    pub bump: u8,                           // PDA bump (1字节)
}

impl MerchantOrder {
    /// 生成商家订单PDA种子
    pub fn seeds(
        merchant: &Pubkey,
        merchant_order_sequence: u64,
    ) -> Vec<Vec<u8>> {
        vec![
            b"merchant_order".to_vec(),
            merchant.to_bytes().to_vec(),
            merchant_order_sequence.to_le_bytes().to_vec(),
        ]
    }

    /// 初始化商家订单作为索引
    pub fn initialize_as_index(
        &mut self,
        merchant: Pubkey,
        buyer: Pubkey,
        merchant_order_sequence: u64,
        buyer_order_pda: Pubkey,
        product_id: u64,
        bump: u8,
    ) -> Result<()> {
        self.merchant = merchant;
        self.buyer = buyer;
        self.merchant_order_sequence = merchant_order_sequence;
        self.buyer_order_pda = buyer_order_pda;
        self.product_id = product_id;
        self.created_at = Clock::get()?.unix_timestamp;
        self.bump = bump;
        Ok(())
    }
}
