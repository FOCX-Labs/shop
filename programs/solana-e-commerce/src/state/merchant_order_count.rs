use anchor_lang::prelude::*;

/// 商家订单计数账户 - 记录商家接收的订单总数，为商家订单提供唯一序列号
#[account]
#[derive(InitSpace)]
pub struct MerchantOrderCount {
    pub merchant: Pubkey,                   // 商家地址 (32字节)
    pub total_orders: u64,                  // 总订单数 (8字节)
    pub created_at: i64,                    // 创建时间 (8字节)
    pub updated_at: i64,                    // 更新时间 (8字节)
    pub bump: u8,                           // PDA bump (1字节)
}

impl MerchantOrderCount {
    /// 生成商家订单计数PDA种子
    pub fn seeds(merchant: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            b"merchant_order_count".to_vec(),
            merchant.to_bytes().to_vec(),
        ]
    }

    /// 初始化商家订单计数账户
    pub fn initialize(&mut self, merchant: Pubkey, bump: u8) -> Result<()> {
        self.merchant = merchant;
        self.total_orders = 0;
        self.created_at = Clock::get()?.unix_timestamp;
        self.updated_at = Clock::get()?.unix_timestamp;
        self.bump = bump;
        Ok(())
    }

    /// 递增订单总数并返回新的序列号
    pub fn increment_total_orders(&mut self) -> Result<u64> {
        self.total_orders = self
            .total_orders
            .checked_add(1)
            .ok_or(crate::error::ErrorCode::ArithmeticOverflow)?;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(self.total_orders)
    }

    /// 获取当前订单总数
    pub fn get_current_count(&self) -> u64 {
        self.total_orders
    }
}
