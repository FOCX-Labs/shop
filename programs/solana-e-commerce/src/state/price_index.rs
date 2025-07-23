use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PriceIndexNode {
    pub price_range_start: u64,
    pub price_range_end: u64,
    #[max_len(1000)]
    pub product_ids: Vec<u64>,
    pub left_child: Option<Pubkey>,
    pub right_child: Option<Pubkey>,
    pub parent: Option<Pubkey>,
    pub height: u8,
    pub bump: u8,
}

impl PriceIndexNode {
    pub fn seeds(price_range_start: u64, price_range_end: u64) -> Vec<Vec<u8>> {
        vec![
            b"price_index".to_vec(),
            price_range_start.to_le_bytes().to_vec(),
            price_range_end.to_le_bytes().to_vec(),
        ]
    }

    pub fn initialize(
        &mut self,
        price_range_start: u64,
        price_range_end: u64,
        bump: u8,
    ) -> Result<()> {
        self.price_range_start = price_range_start;
        self.price_range_end = price_range_end;
        self.product_ids = Vec::new();
        self.left_child = None;
        self.right_child = None;
        self.parent = None;
        self.height = 1;
        self.bump = bump;

        Ok(())
    }

    pub fn add_product(&mut self, product_id: u64, price: u64) -> Result<()> {
        require!(
            price >= self.price_range_start && price <= self.price_range_end,
            crate::error::ErrorCode::InvalidPriceRange
        );

        if !self.product_ids.contains(&product_id) {
            self.product_ids.push(product_id);
        }

        Ok(())
    }

    pub fn remove_product(&mut self, product_id: u64) -> Result<bool> {
        if let Some(index) = self.product_ids.iter().position(|&x| x == product_id) {
            self.product_ids.remove(index);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn contains_price(&self, price: u64) -> bool {
        price >= self.price_range_start && price <= self.price_range_end
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

    pub fn get_products_in_range(&self, min_price: u64, max_price: u64) -> Vec<u64> {
        if min_price <= self.price_range_end && max_price >= self.price_range_start {
            self.product_ids.clone()
        } else {
            Vec::new()
        }
    }
}
