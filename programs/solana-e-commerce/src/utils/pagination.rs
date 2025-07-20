use anchor_lang::prelude::*;

pub const DEFAULT_PAGE_SIZE: u32 = 20;
pub const MAX_PAGE_SIZE: u32 = 100;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }
}

impl PaginationParams {
    pub fn new(page: u32, page_size: u32) -> Self {
        Self {
            page,
            page_size: page_size.min(MAX_PAGE_SIZE).max(1),
        }
    }

    pub fn offset(&self) -> u32 {
        self.page * self.page_size
    }

    pub fn limit(&self) -> u32 {
        self.page_size
    }

    pub fn validate(&self) -> Result<()> {
        require!(self.page_size > 0, ErrorCode::InvalidPageSize);
        require!(self.page_size <= MAX_PAGE_SIZE, ErrorCode::PageSizeTooLarge);
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PaginationResult<T> {
    pub items: Vec<T>,
    pub total_count: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> PaginationResult<T> {
    pub fn new(items: Vec<T>, total_count: u32, params: PaginationParams) -> Self {
        let total_pages = if total_count == 0 {
            0
        } else {
            (total_count + params.page_size - 1) / params.page_size
        };

        let has_next = params.page + 1 < total_pages;
        let has_prev = params.page > 0;

        Self {
            items,
            total_count,
            page: params.page,
            page_size: params.page_size,
            total_pages,
            has_next,
            has_prev,
        }
    }

    pub fn empty(params: PaginationParams) -> Self {
        Self::new(Vec::new(), 0, params)
    }
}

// 分页辅助函数
pub fn paginate_slice<T: Clone>(data: &[T], params: PaginationParams) -> PaginationResult<T> {
    let total_count = data.len() as u32;
    let offset = params.offset() as usize;
    let limit = params.limit() as usize;

    let items = if offset >= data.len() {
        Vec::new()
    } else {
        let end = (offset + limit).min(data.len());
        data[offset..end].to_vec()
    };

    PaginationResult::new(items, total_count, params)
}

// 计算分页范围
pub fn calculate_range(params: PaginationParams) -> (usize, usize) {
    let start = params.offset() as usize;
    let end = start + params.limit() as usize;
    (start, end)
}

// 验证分页参数
pub fn validate_pagination(params: &PaginationParams) -> Result<()> {
    params.validate()
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid page size")]
    InvalidPageSize,
    #[msg("Page size too large")]
    PageSizeTooLarge,
}
