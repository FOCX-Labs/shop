use crate::error::ErrorCode;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum OrderManagementStatus {
    Pending,   // 待处理
    Confirmed, // 已确认
    Shipped,   // 已发货
    Delivered, // 已送达
    Refunded,  // 已退款
}

impl Default for OrderManagementStatus {
    fn default() -> Self {
        OrderManagementStatus::Pending
    }
}

#[account]
pub struct Order {
    pub id: u64,                       // 订单ID
    pub buyer: Pubkey,                 // 买家地址
    pub merchant: Pubkey,              // 商户地址
    pub product_id: u64,               // 商品ID
    pub quantity: u32,                 // 购买数量
    pub unit_price: u64,               // 单价（SOL lamports）
    pub total_amount: u64,             // 总金额（SOL lamports）
    pub payment_token: Pubkey,         // 支付代币mint
    pub token_decimals: u8,            // 代币精度
    pub token_unit_price: u64,         // 代币单价
    pub token_total_amount: u64,       // 代币总金额
    pub status: OrderManagementStatus, // 订单状态
    pub shipping_address: String,      // 收货地址
    pub notes: String,                 // 订单备注
    pub created_at: i64,               // 创建时间
    pub updated_at: i64,               // 更新时间
    pub confirmed_at: Option<i64>,     // 确认时间
    pub shipped_at: Option<i64>,       // 发货时间
    pub delivered_at: Option<i64>,     // 送达时间
    pub refunded_at: Option<i64>,      // 退款时间
    pub transaction_signature: String, // 支付交易签名
    pub bump: u8,                      // PDA bump
}

impl Order {
    // 计算账户大小
    pub const LEN: usize = 8        // discriminator
        + 8                         // id
        + 32                        // buyer
        + 32                        // merchant
        + 8                         // product_id
        + 4                         // quantity
        + 8                         // unit_price
        + 8                         // total_amount
        + 32                        // payment_token
        + 1                         // token_decimals
        + 8                         // token_unit_price
        + 8                         // token_total_amount
        + 1                         // status (enum)
        + (4 + 200)                 // shipping_address (max 200 chars)
        + (4 + 500)                 // notes (max 500 chars)
        + 8                         // created_at
        + 8                         // updated_at
        + (1 + 8)                   // confirmed_at (Option<i64>)
        + (1 + 8)                   // shipped_at (Option<i64>)
        + (1 + 8)                   // delivered_at (Option<i64>)
        + (1 + 8)                   // refunded_at (Option<i64>)
        + (4 + 88)                  // transaction_signature (max 88 chars for base58)
        + 1; // bump

    // PDA种子
    pub fn seeds(order_id: u64) -> Vec<Vec<u8>> {
        vec![b"order".to_vec(), order_id.to_le_bytes().to_vec()]
    }

    // 验证订单数据
    pub fn validate(&self) -> Result<()> {
        require!(self.quantity > 0, ErrorCode::InvalidOrderQuantity);
        require!(self.unit_price > 0, ErrorCode::InvalidOrderPrice);
        require!(
            self.total_amount == self.unit_price.checked_mul(self.quantity as u64).unwrap(),
            ErrorCode::InvalidOrderTotalAmount
        );
        require!(self.token_unit_price > 0, ErrorCode::InvalidOrderTokenPrice);
        require!(
            self.token_total_amount
                == self
                    .token_unit_price
                    .checked_mul(self.quantity as u64)
                    .unwrap(),
            ErrorCode::InvalidOrderTokenTotalAmount
        );
        require!(
            self.shipping_address.len() <= 200,
            ErrorCode::InvalidShippingAddressLength
        );
        require!(self.notes.len() <= 500, ErrorCode::InvalidOrderNotesLength);
        // 注意：transaction_signature在订单创建时可以为空，稍后通过专门的指令设置
        // require!(
        //     !self.transaction_signature.is_empty(),
        //     ErrorCode::InvalidTransactionSignature
        // );
        Ok(())
    }

    // 检查订单是否可以修改
    pub fn can_modify(&self) -> bool {
        matches!(
            self.status,
            OrderManagementStatus::Pending | OrderManagementStatus::Confirmed
        )
    }

    // 检查订单是否可以退款
    pub fn can_refund(&self) -> bool {
        matches!(
            self.status,
            OrderManagementStatus::Delivered | OrderManagementStatus::Shipped
        )
    }

    // 更新订单状态
    pub fn update_status(
        &mut self,
        new_status: OrderManagementStatus,
        timestamp: i64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        match new_status {
            OrderManagementStatus::Confirmed => {
                require!(
                    self.status == OrderManagementStatus::Pending,
                    ErrorCode::InvalidOrderStatusTransition
                );
                self.confirmed_at = Some(timestamp);
            }
            OrderManagementStatus::Shipped => {
                require!(
                    self.status == OrderManagementStatus::Confirmed,
                    ErrorCode::InvalidOrderStatusTransition
                );
                self.shipped_at = Some(timestamp);
            }
            OrderManagementStatus::Delivered => {
                require!(
                    self.status == OrderManagementStatus::Shipped,
                    ErrorCode::InvalidOrderStatusTransition
                );
                self.delivered_at = Some(timestamp);
            }
            OrderManagementStatus::Refunded => {
                require!(self.can_refund(), ErrorCode::InvalidOrderStatusTransition);
                self.refunded_at = Some(timestamp);
            }
            _ => {
                return Err(ErrorCode::InvalidOrderStatusTransition.into());
            }
        }

        self.status = new_status;
        self.updated_at = current_time;
        Ok(())
    }
}

// 订单统计信息
#[account]
pub struct OrderStats {
    pub total_orders: u64,     // 总订单数
    pub pending_orders: u64,   // 待处理订单数
    pub confirmed_orders: u64, // 已确认订单数
    pub shipped_orders: u64,   // 已发货订单数
    pub delivered_orders: u64, // 已送达订单数
    pub refunded_orders: u64,  // 已退款订单数
    pub total_revenue: u64,    // 总收入（SOL lamports）
    pub bump: u8,
}

impl OrderStats {
    pub const LEN: usize = 8        // discriminator
        + 8                         // total_orders
        + 8                         // pending_orders
        + 8                         // confirmed_orders
        + 8                         // shipped_orders
        + 8                         // delivered_orders
        + 8                         // refunded_orders
        + 8                         // total_revenue
        + 1; // bump

    pub fn seeds() -> Vec<Vec<u8>> {
        vec![b"order_stats".to_vec()]
    }

    // 更新订单统计
    pub fn update_for_new_order(&mut self, order: &Order) {
        self.total_orders += 1;
        match order.status {
            OrderManagementStatus::Pending => self.pending_orders += 1,
            OrderManagementStatus::Confirmed => self.confirmed_orders += 1,
            OrderManagementStatus::Shipped => self.shipped_orders += 1,
            OrderManagementStatus::Delivered => {
                self.delivered_orders += 1;
                self.total_revenue += order.total_amount;
            }
            OrderManagementStatus::Refunded => self.refunded_orders += 1,
        }
    }

    // 更新订单状态变化的统计
    pub fn update_for_status_change(
        &mut self,
        old_status: &OrderManagementStatus,
        new_status: &OrderManagementStatus,
        order_amount: u64,
    ) {
        // 减少旧状态计数
        match old_status {
            OrderManagementStatus::Pending => self.pending_orders -= 1,
            OrderManagementStatus::Confirmed => self.confirmed_orders -= 1,
            OrderManagementStatus::Shipped => self.shipped_orders -= 1,
            OrderManagementStatus::Delivered => {
                self.delivered_orders -= 1;
                self.total_revenue -= order_amount;
            }
            OrderManagementStatus::Refunded => self.refunded_orders -= 1,
        }

        // 增加新状态计数
        match new_status {
            OrderManagementStatus::Pending => self.pending_orders += 1,
            OrderManagementStatus::Confirmed => self.confirmed_orders += 1,
            OrderManagementStatus::Shipped => self.shipped_orders += 1,
            OrderManagementStatus::Delivered => {
                self.delivered_orders += 1;
                self.total_revenue += order_amount;
            }
            OrderManagementStatus::Refunded => self.refunded_orders += 1,
        }
    }
}
