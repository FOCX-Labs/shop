use super::ProductSales;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct SalesIndexNode {
    pub sales_range_start: u32,
    pub sales_range_end: u32,
    #[max_len(500)]
    pub product_ids: Vec<u64>,
    #[max_len(10)]
    pub top_items: Vec<ProductSales>, // 缓存热销商品
    pub left_child: Option<Pubkey>,
    pub right_child: Option<Pubkey>,
    pub parent: Option<Pubkey>,
    pub height: u8,
    pub bump: u8,
}

impl SalesIndexNode {
    pub fn seeds(sales_range_start: u32, sales_range_end: u32) -> Vec<Vec<u8>> {
        vec![
            b"sales_index".to_vec(),
            sales_range_start.to_le_bytes().to_vec(),
            sales_range_end.to_le_bytes().to_vec(),
        ]
    }

    pub fn initialize(
        &mut self,
        sales_range_start: u32,
        sales_range_end: u32,
        bump: u8,
    ) -> Result<()> {
        self.sales_range_start = sales_range_start;
        self.sales_range_end = sales_range_end;
        self.product_ids = Vec::new();
        self.top_items = Vec::new();
        self.left_child = None;
        self.right_child = None;
        self.parent = None;
        self.height = 1;
        self.bump = bump;

        Ok(())
    }

    pub fn add_product(&mut self, product_id: u64, sales: u32) -> Result<()> {
        require!(
            sales >= self.sales_range_start && sales <= self.sales_range_end,
            crate::error::ErrorCode::InvalidSalesRange
        );

        if !self.product_ids.contains(&product_id) {
            self.product_ids.push(product_id);
            self.update_top_items(product_id, sales)?;
        }

        Ok(())
    }

    pub fn remove_product(&mut self, product_id: u64) -> Result<bool> {
        if let Some(index) = self.product_ids.iter().position(|&x| x == product_id) {
            self.product_ids.remove(index);
            self.remove_from_top_items(product_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn update_product_sales(&mut self, product_id: u64, new_sales: u32) -> Result<()> {
        if self.product_ids.contains(&product_id) {
            self.update_top_items(product_id, new_sales)?;
        }
        Ok(())
    }

    pub fn update_top_items(&mut self, product_id: u64, sales: u32) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let product_sales = ProductSales {
            product_id,
            merchant: Pubkey::default(), // TODO: 从产品账户获取实际商户信息
            name: String::new(),         // TODO: 从产品账户获取实际产品名称
            price: 0,                    // TODO: 从产品账户获取实际价格
            sales,
            last_update: now,
        };

        // 移除旧记录（如果存在）
        self.remove_from_top_items(product_id);

        // 插入新记录，保持按销量排序
        let insert_pos = self
            .top_items
            .binary_search_by(|item| item.sales.cmp(&sales).reverse())
            .unwrap_or_else(|pos| pos);

        self.top_items.insert(insert_pos, product_sales);

        // 限制top_items数量为20个
        if self.top_items.len() > 20 {
            self.top_items.truncate(20);
        }

        Ok(())
    }

    pub fn remove_from_top_items(&mut self, product_id: u64) {
        if let Some(index) = self
            .top_items
            .iter()
            .position(|item| item.product_id == product_id)
        {
            self.top_items.remove(index);
        }
    }

    pub fn contains_sales(&self, sales: u32) -> bool {
        sales >= self.sales_range_start && sales <= self.sales_range_end
    }

    pub fn is_leaf(&self) -> bool {
        self.left_child.is_none() && self.right_child.is_none()
    }

    pub fn needs_split(&self) -> bool {
        self.product_ids.len() > super::MAX_PRODUCTS_PER_SHARD
    }

    pub fn needs_merge(&self) -> bool {
        self.product_ids.len() < super::MAX_PRODUCTS_PER_SHARD / 4
    }

    pub fn balance_factor(&self) -> i8 {
        let left_height = if self.left_child.is_some() {
            self.height
        } else {
            0
        };
        let right_height = if self.right_child.is_some() {
            self.height
        } else {
            0
        };
        right_height as i8 - left_height as i8
    }

    pub fn update_height(&mut self, left_height: u8, right_height: u8) {
        self.height = 1 + left_height.max(right_height);
    }

    pub fn get_products_in_range(&self, min_sales: u32, max_sales: u32) -> Vec<u64> {
        if min_sales <= self.sales_range_end && max_sales >= self.sales_range_start {
            self.product_ids.clone()
        } else {
            Vec::new()
        }
    }

    pub fn get_top_products(&self, limit: usize) -> Vec<ProductSales> {
        self.top_items.iter().take(limit).cloned().collect()
    }
}
