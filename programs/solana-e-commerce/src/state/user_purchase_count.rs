use anchor_lang::prelude::*;

/// 用户购买计数账户
#[account]
#[derive(InitSpace)]
pub struct UserPurchaseCount {
    pub buyer: Pubkey,       // 买家地址
    pub purchase_count: u64, // 购买次数
    pub created_at: i64,     // 创建时间
    pub updated_at: i64,     // 更新时间
    pub bump: u8,            // PDA bump
}

impl UserPurchaseCount {
    pub fn seeds(buyer: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"user_purchase_count".to_vec(), buyer.to_bytes().to_vec()]
    }

    pub fn initialize(&mut self, buyer: Pubkey, bump: u8) -> Result<()> {
        self.buyer = buyer;
        self.purchase_count = 0;
        self.created_at = Clock::get()?.unix_timestamp;
        self.updated_at = Clock::get()?.unix_timestamp;
        self.bump = bump;
        Ok(())
    }

    pub fn increment_count(&mut self) -> Result<u64> {
        self.purchase_count = self
            .purchase_count
            .checked_add(1)
            .ok_or(crate::error::ErrorCode::ArithmeticOverflow)?;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(self.purchase_count)
    }

    pub fn get_current_count(&self) -> u64 {
        self.purchase_count
    }
}
