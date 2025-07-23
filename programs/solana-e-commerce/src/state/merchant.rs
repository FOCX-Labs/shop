use crate::error::ErrorCode;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Merchant {
    pub owner: Pubkey,
    #[max_len(100)]
    pub name: String,
    #[max_len(500)]
    pub description: String,
    pub product_count: u64,
    pub total_sales: u64,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    // 保证金相关字段
    pub deposit_amount: u64,        // 当前保证金余额
    pub deposit_token_mint: Pubkey, // 保证金代币mint
    pub deposit_locked: u64,        // 锁定的保证金金额（用于处理中的订单）
    pub deposit_updated_at: i64,    // 保证金最后更新时间
    pub bump: u8,
}

impl Merchant {
    pub fn seeds(owner: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"merchant_info".to_vec(), owner.as_ref().to_vec()]
    }

    pub fn initialize(
        &mut self,
        owner: Pubkey,
        name: String,
        description: String,
        deposit_token_mint: Pubkey,
        bump: u8,
    ) -> Result<()> {
        require!(
            name.len() <= super::MAX_MERCHANT_NAME_LENGTH,
            ErrorCode::InvalidMerchantNameLength
        );
        require!(
            description.len() <= super::MAX_MERCHANT_DESCRIPTION_LENGTH,
            ErrorCode::InvalidMerchantDescriptionLength
        );

        let current_time = Clock::get()?.unix_timestamp;

        self.owner = owner;
        self.name = name;
        self.description = description;
        self.product_count = 0;
        self.total_sales = 0;
        self.is_active = true;
        self.created_at = current_time;
        self.updated_at = current_time;
        // 初始化保证金相关字段
        self.deposit_amount = 0;
        self.deposit_token_mint = deposit_token_mint;
        self.deposit_locked = 0;
        self.deposit_updated_at = current_time;
        self.bump = bump;

        Ok(())
    }

    pub fn update_info(&mut self, name: Option<String>, description: Option<String>) -> Result<()> {
        if let Some(new_name) = name {
            require!(
                new_name.len() <= super::MAX_MERCHANT_NAME_LENGTH,
                ErrorCode::InvalidMerchantNameLength
            );
            self.name = new_name;
        }

        if let Some(new_description) = description {
            require!(
                new_description.len() <= super::MAX_MERCHANT_DESCRIPTION_LENGTH,
                ErrorCode::InvalidMerchantDescriptionLength
            );
            self.description = new_description;
        }

        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn increment_product_count(&mut self) -> Result<()> {
        self.product_count = self.product_count.saturating_add(1);
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn decrement_product_count(&mut self) -> Result<()> {
        self.product_count = self.product_count.saturating_sub(1);
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn add_sales(&mut self, sales: u64) -> Result<()> {
        self.total_sales = self.total_sales.saturating_add(sales);
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn set_active(&mut self, active: bool) -> Result<()> {
        self.is_active = active;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        require!(
            self.name.len() <= super::MAX_MERCHANT_NAME_LENGTH,
            ErrorCode::InvalidMerchantNameLength
        );
        require!(
            self.description.len() <= super::MAX_MERCHANT_DESCRIPTION_LENGTH,
            ErrorCode::InvalidMerchantDescriptionLength
        );
        Ok(())
    }

    /// 添加保证金
    pub fn add_deposit(&mut self, amount: u64) -> Result<()> {
        self.deposit_amount = self
            .deposit_amount
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        self.deposit_updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// 扣除保证金
    pub fn deduct_deposit(&mut self, amount: u64) -> Result<()> {
        require!(
            self.deposit_amount >= amount,
            ErrorCode::InsufficientDeposit
        );
        self.deposit_amount = self
            .deposit_amount
            .checked_sub(amount)
            .ok_or(ErrorCode::ArithmeticUnderflow)?;
        self.deposit_updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// 锁定保证金（用于处理中的订单）
    pub fn lock_deposit(&mut self, amount: u64) -> Result<()> {
        require!(
            self.deposit_amount >= self.deposit_locked + amount,
            ErrorCode::InsufficientDeposit
        );
        self.deposit_locked = self
            .deposit_locked
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        self.deposit_updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// 解锁保证金
    pub fn unlock_deposit(&mut self, amount: u64) -> Result<()> {
        require!(
            self.deposit_locked >= amount,
            ErrorCode::InsufficientLockedDeposit
        );
        self.deposit_locked = self
            .deposit_locked
            .checked_sub(amount)
            .ok_or(ErrorCode::ArithmeticUnderflow)?;
        self.deposit_updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// 获取可用保证金余额
    pub fn get_available_deposit(&self) -> u64 {
        self.deposit_amount.saturating_sub(self.deposit_locked)
    }

    /// 检查保证金是否满足要求
    pub fn has_sufficient_deposit(&self, required_amount: u64) -> bool {
        self.get_available_deposit() >= required_amount
    }

    /// 验证保证金代币
    pub fn is_valid_deposit_token(&self, token_mint: &Pubkey) -> bool {
        self.deposit_token_mint == *token_mint
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MerchantInfo {
    pub name: String,
    pub description: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MerchantStats {
    pub product_count: u64,
    pub total_sales: u64,
    pub active_products: u64,
    pub total_keywords: u64,
    pub avg_product_price: u64,
    pub last_updated: i64,
}
