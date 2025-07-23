use crate::error::ErrorCode;
use anchor_lang::prelude::*;

/// 系统级支付配置账户
#[account]
#[derive(InitSpace)]
pub struct PaymentConfig {
    pub authority: Pubkey, // 系统管理员
    #[max_len(10)]
    pub supported_tokens: Vec<SupportedToken>, // 支持的代币列表
    pub fee_rate: u16,     // 手续费率（基点，如100=1%）
    pub fee_recipient: Pubkey, // 手续费接收方
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

impl PaymentConfig {
    pub fn seeds() -> &'static [&'static [u8]] {
        &[b"payment_config"]
    }

    pub fn initialize(
        &mut self,
        authority: Pubkey,
        supported_tokens: Vec<SupportedToken>,
        fee_rate: u16,
        fee_recipient: Pubkey,
        bump: u8,
    ) -> Result<()> {
        require!(fee_rate <= 10000, ErrorCode::InvalidFeeRate); // 最大100%
        require!(supported_tokens.len() <= 10, ErrorCode::TooManyTokens);

        self.authority = authority;
        self.supported_tokens = supported_tokens;
        self.fee_rate = fee_rate;
        self.fee_recipient = fee_recipient;
        self.created_at = Clock::get()?.unix_timestamp;
        self.updated_at = Clock::get()?.unix_timestamp;
        self.bump = bump;

        Ok(())
    }

    pub fn is_token_supported(&self, mint: &Pubkey) -> bool {
        self.supported_tokens
            .iter()
            .any(|token| token.mint == *mint && token.is_active)
    }

    pub fn get_token_info(&self, mint: &Pubkey) -> Option<&SupportedToken> {
        self.supported_tokens
            .iter()
            .find(|token| token.mint == *mint && token.is_active)
    }

    pub fn update_tokens(&mut self, supported_tokens: Vec<SupportedToken>) -> Result<()> {
        require!(supported_tokens.len() <= 10, ErrorCode::TooManyTokens);
        self.supported_tokens = supported_tokens;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn update_fee_rate(&mut self, fee_rate: u16) -> Result<()> {
        require!(fee_rate <= 10000, ErrorCode::InvalidFeeRate);
        self.fee_rate = fee_rate;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

/// 支持的代币信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct SupportedToken {
    pub mint: Pubkey, // 代币mint地址
    #[max_len(10)]
    pub symbol: String, // 代币符号（如"USDC"）
    pub decimals: u8, // 代币精度
    pub is_active: bool, // 是否启用
    pub min_amount: u64, // 最小交易金额
}

impl SupportedToken {
    pub fn new(mint: Pubkey, symbol: String, decimals: u8, min_amount: u64) -> Result<Self> {
        require!(symbol.len() <= 10, ErrorCode::InvalidTokenSymbol);
        require!(decimals <= 18, ErrorCode::InvalidTokenDecimals);

        Ok(Self {
            mint,
            symbol,
            decimals,
            is_active: true,
            min_amount,
        })
    }

    pub fn validate_amount(&self, amount: u64) -> Result<()> {
        require!(amount >= self.min_amount, ErrorCode::BelowMinimumAmount);
        Ok(())
    }
}

/// 订单状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, InitSpace)]
pub enum OrderStatus {
    Pending,
    PendingConfirmation, // 待确认收货（托管支付状态）
    Completed,
    Cancelled,
    Failed,
}

/// 托管账户结构
#[account]
#[derive(InitSpace)]
pub struct EscrowAccount {
    pub order_id: u64,         // 订单ID（使用product_id + buyer的组合）
    pub buyer: Pubkey,         // 买家地址
    pub merchant: Pubkey,      // 商户地址
    pub product_id: u64,       // 产品ID
    pub payment_token: Pubkey, // 支付代币地址
    pub amount: u64,           // 购买数量
    pub total_price: u64,      // 总价格
    pub fee_amount: u64,       // 手续费金额
    pub merchant_amount: u64,  // 商户应收金额
    pub status: OrderStatus,   // 订单状态
    pub created_at: i64,       // 创建时间
    pub bump: u8,              // PDA bump
}

impl EscrowAccount {
    pub fn seeds(buyer: &Pubkey, product_id: u64) -> Vec<Vec<u8>> {
        vec![
            b"escrow".to_vec(),
            buyer.as_ref().to_vec(),
            product_id.to_le_bytes().to_vec(),
        ]
    }

    pub fn initialize(
        &mut self,
        order_id: u64,
        buyer: Pubkey,
        merchant: Pubkey,
        product_id: u64,
        payment_token: Pubkey,
        amount: u64,
        total_price: u64,
        fee_amount: u64,
        bump: u8,
    ) -> Result<()> {
        self.order_id = order_id;
        self.buyer = buyer;
        self.merchant = merchant;
        self.product_id = product_id;
        self.payment_token = payment_token;
        self.amount = amount;
        self.total_price = total_price;
        self.fee_amount = fee_amount;
        self.merchant_amount = total_price.saturating_sub(fee_amount);
        self.status = OrderStatus::PendingConfirmation;
        self.created_at = Clock::get()?.unix_timestamp;
        self.bump = bump;

        Ok(())
    }

    pub fn complete(&mut self) -> Result<()> {
        require!(
            self.status == OrderStatus::PendingConfirmation,
            ErrorCode::InvalidOrderStatus
        );
        self.status = OrderStatus::Completed;
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<()> {
        require!(
            self.status == OrderStatus::PendingConfirmation,
            ErrorCode::InvalidOrderStatus
        );
        self.status = OrderStatus::Cancelled;
        Ok(())
    }
}

/// 支付方式枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum PaymentMethod {
    Sol,
    SplToken { mint: Pubkey },
}

impl PaymentMethod {
    pub fn is_sol(&self) -> bool {
        matches!(self, PaymentMethod::Sol)
    }

    pub fn get_mint(&self) -> Option<Pubkey> {
        match self {
            PaymentMethod::Sol => None,
            PaymentMethod::SplToken { mint } => Some(*mint),
        }
    }
}

/// 支付信息结构
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PaymentInfo {
    pub method: PaymentMethod,
    pub amount: u64,
    pub fee_amount: u64,
    pub recipient: Pubkey,
}

impl PaymentInfo {
    pub fn new_sol_payment(amount: u64, fee_amount: u64, recipient: Pubkey) -> Self {
        Self {
            method: PaymentMethod::Sol,
            amount,
            fee_amount,
            recipient,
        }
    }

    pub fn new_token_payment(
        mint: Pubkey,
        amount: u64,
        fee_amount: u64,
        recipient: Pubkey,
    ) -> Self {
        Self {
            method: PaymentMethod::SplToken { mint },
            amount,
            fee_amount,
            recipient,
        }
    }

    pub fn total_amount(&self) -> u64 {
        self.amount.saturating_add(self.fee_amount)
    }
}

/// 商品支付配置（嵌入到Product结构中）
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProductPaymentConfig {
    pub payment_token: Option<Pubkey>, // 支付代币mint（None表示SOL）
    pub token_decimals: u8,            // 代币精度
    pub token_price: u64,              // 代币价格
    pub accept_sol: bool,              // 是否同时接受SOL
}

impl ProductPaymentConfig {
    pub fn new_sol_only(sol_price: u64) -> Self {
        Self {
            payment_token: None,
            token_decimals: 9, // SOL的精度
            token_price: sol_price,
            accept_sol: true,
        }
    }

    pub fn new_token_payment(
        token_mint: Pubkey,
        token_decimals: u8,
        token_price: u64,
        accept_sol: bool,
    ) -> Self {
        Self {
            payment_token: Some(token_mint),
            token_decimals,
            token_price,
            accept_sol,
        }
    }

    pub fn supports_payment_method(&self, method: &PaymentMethod) -> bool {
        match method {
            PaymentMethod::Sol => self.accept_sol,
            PaymentMethod::SplToken { mint } => self
                .payment_token
                .map_or(false, |token_mint| token_mint == *mint),
        }
    }

    pub fn get_price_for_method(&self, method: &PaymentMethod, sol_price: u64) -> Option<u64> {
        match method {
            PaymentMethod::Sol => {
                if self.accept_sol {
                    Some(sol_price)
                } else {
                    None
                }
            }
            PaymentMethod::SplToken { mint } => {
                if self
                    .payment_token
                    .map_or(false, |token_mint| token_mint == *mint)
                {
                    Some(self.token_price)
                } else {
                    None
                }
            }
        }
    }
}
