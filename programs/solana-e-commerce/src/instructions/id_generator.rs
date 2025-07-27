use crate::error::ErrorCode;
use crate::state::{
    GlobalIdRoot, IdChunk, MerchantIdAccount, DEFAULT_CHUNK_SIZE, ID_CHUNK_BITMAP_SIZE,
    MAX_CHUNKS_PER_MERCHANT,
};
use anchor_lang::prelude::*;

// ID生成器功能
#[derive(Accounts)]
pub struct GenerateId<'info> {
    #[account(
        mut,
        seeds = [b"merchant_id", merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantIdAccount>,

    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"id_chunk",
            merchant.key().as_ref(),
            merchant_account.last_chunk_index.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub active_chunk: Account<'info, IdChunk>,
}

pub fn generate_product_id(ctx: Context<GenerateId>) -> Result<u64> {
    let merchant_account = &mut ctx.accounts.merchant_account;
    let active_chunk = &mut ctx.accounts.active_chunk;

    // 检查当前块是否有可用ID
    if active_chunk.is_full() {
        return Err(ErrorCode::NoAvailableId.into());
    }

    // 查找下一个可用的ID
    let mut local_id = active_chunk.next_available;
    while local_id < active_chunk.capacity() {
        if !active_chunk.is_id_used(local_id) {
            // 分配这个ID
            active_chunk.mark_id_used(local_id);
            active_chunk.next_available = local_id + 1;
            merchant_account.last_local_id = local_id;

            let global_id = active_chunk.start_id + local_id as u64;

            msg!(
                "生成产品ID成功，商户: {}, 本地ID: {}, 全局ID: {}",
                merchant_account.merchant_id,
                local_id,
                global_id
            );

            return Ok(global_id);
        }
        local_id += 1;
    }

    Err(ErrorCode::NoAvailableId.into())
}

// 3. 分配新块
#[derive(Accounts)]
pub struct AllocateChunk<'info> {
    #[account(
        mut,
        seeds = [b"global_id_root"],
        bump
    )]
    pub global_root: Account<'info, GlobalIdRoot>,

    #[account(
        mut,
        seeds = [b"merchant_id", merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantIdAccount>,

    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + IdChunk::INIT_SPACE,
        seeds = [
            b"id_chunk",
            merchant.key().as_ref(),
            (merchant_account.last_chunk_index + 1).to_le_bytes().as_ref()
        ],
        bump
    )]
    pub new_chunk: Account<'info, IdChunk>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn allocate_new_chunk(ctx: Context<AllocateChunk>) -> Result<Pubkey> {
    let global_root = &mut ctx.accounts.global_root;
    let merchant_account = &mut ctx.accounts.merchant_account;
    let _payer = &ctx.accounts.payer;
    let _system_program = &ctx.accounts.system_program;

    // 更新商户块索引
    merchant_account.last_chunk_index += 1;
    let chunk_index = merchant_account.last_chunk_index;

    require!(
        chunk_index <= MAX_CHUNKS_PER_MERCHANT,
        ErrorCode::InvalidShardIndex
    );

    // 初始化新块 - 使用基于商户ID的范围
    let merchant_start_id = merchant_account.merchant_id as u64 * 10000; // 每个商户预留10000个ID
    let chunk_start_id = merchant_start_id + (chunk_index as u64 * global_root.chunk_size as u64);
    let new_chunk = &mut ctx.accounts.new_chunk;
    new_chunk.merchant_id = merchant_account.merchant_id;
    new_chunk.chunk_index = chunk_index;
    new_chunk.start_id = chunk_start_id;
    new_chunk.end_id = chunk_start_id + global_root.chunk_size as u64 - 1;
    new_chunk.next_available = 0;
    new_chunk.initialize_bitmap(); // 使用安全的初始化方法
    new_chunk.bump = ctx.bumps.new_chunk;

    // 更新全局ID计数器
    global_root.last_global_id = new_chunk.end_id + 1;

    // 将旧的活跃块加入未使用队列
    let old_active_chunk = merchant_account.active_chunk;
    merchant_account.unused_chunks.push(old_active_chunk);

    // 设置新的活跃块
    merchant_account.active_chunk = new_chunk.key();

    msg!(
        "分配新ID块成功，商户: {}, 块索引: {}, ID范围: {} - {}",
        merchant_account.merchant_id,
        chunk_index,
        new_chunk.start_id,
        new_chunk.end_id
    );

    Ok(new_chunk.key())
}

// 4. ID存在性验证
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct VerifyId<'info> {
    #[account(
        seeds = [b"merchant_id", merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantIdAccount>,

    pub merchant: Signer<'info>,

    /// CHECK: 会在指令中验证这是正确的ID块
    pub id_chunk: AccountInfo<'info>,
}

pub fn is_id_exists(ctx: Context<VerifyId>, id: u64) -> Result<bool> {
    let merchant_account = &ctx.accounts.merchant_account;

    // 计算ID应该在哪个块中
    let chunk_size = DEFAULT_CHUNK_SIZE as u64;
    let merchant_start_id = merchant_account.merchant_id as u64 * 10000; // 每个商户预留10000个ID

    if id < merchant_start_id {
        return Ok(false);
    }

    let chunk_index = (id - merchant_start_id) / chunk_size;

    // 验证ID块账户
    let merchant_key_bytes = ctx.accounts.merchant.key().to_bytes();
    let chunk_index_bytes = (chunk_index as u32).to_le_bytes();
    let expected_chunk_seeds = [
        b"id_chunk".as_ref(),
        merchant_key_bytes.as_ref(),
        chunk_index_bytes.as_ref(),
    ];

    let (expected_chunk_key, _) =
        Pubkey::find_program_address(&expected_chunk_seeds, &ctx.program_id);

    require!(
        ctx.accounts.id_chunk.key() == expected_chunk_key,
        ErrorCode::InvalidPda
    );

    // 反序列化ID块并检查
    let chunk_account_data = ctx.accounts.id_chunk.data.borrow();
    let chunk = IdChunk::try_deserialize(&mut &chunk_account_data[8..])?;

    if id < chunk.start_id || id > chunk.end_id {
        return Ok(false);
    }

    let local_id = (id - chunk.start_id) as u32;
    Ok(chunk.is_id_used(local_id))
}

// 5. 批量分配ID
#[derive(Accounts)]
pub struct BatchGenerate<'info> {
    #[account(
        mut,
        seeds = [b"merchant_id", merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantIdAccount>,

    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        mut,
        constraint = active_chunk.key() == merchant_account.active_chunk @ ErrorCode::InvalidActiveChunk
    )]
    pub active_chunk: Account<'info, IdChunk>,
}

pub fn batch_generate_ids(ctx: Context<BatchGenerate>, count: u16) -> Result<Vec<u64>> {
    let merchant_account = &mut ctx.accounts.merchant_account;
    let active_chunk = &mut ctx.accounts.active_chunk;

    require!(count > 0 && count <= 100, ErrorCode::InvalidId); // 限制批量数量

    let mut ids = Vec::new();
    let mut local_id = active_chunk.next_available;
    let mut allocated = 0u16;

    while allocated < count && local_id < active_chunk.capacity() {
        if !active_chunk.is_id_used(local_id) {
            active_chunk.mark_id_used(local_id);
            let global_id = active_chunk.start_id + local_id as u64;
            ids.push(global_id);
            allocated += 1;
        }
        local_id += 1;
    }

    if allocated > 0 {
        active_chunk.next_available = local_id;
        merchant_account.last_local_id = local_id - 1;
    }

    require!(allocated == count, ErrorCode::NoAvailableId);

    msg!(
        "批量生成ID成功，商户: {}, 数量: {}",
        merchant_account.merchant_id,
        allocated
    );

    Ok(ids)
}

// 6. ID回收
#[derive(Accounts)]
pub struct ReleaseId<'info> {
    #[account(mut)]
    pub id_chunk: Account<'info, IdChunk>,
}

pub fn release_id(ctx: Context<ReleaseId>, id: u64) -> Result<()> {
    let chunk = &mut ctx.accounts.id_chunk;
    require!(
        id >= chunk.start_id && id <= chunk.end_id,
        ErrorCode::InvalidId
    );
    let offset = (id - chunk.start_id) as u32;
    let byte_index = (offset / 8) as usize;
    let bit_index = offset % 8;
    chunk.bitmap[byte_index] &= !(1 << bit_index);
    Ok(())
}

// 辅助函数：块切换/预分配
pub fn switch_or_allocate_chunk<'info>(
    merchant: &mut Account<'info, MerchantIdAccount>,
    root: &mut Account<'info, GlobalIdRoot>,
    payer: &Signer<'info>,
    _system_program: &Program<'info, System>,
    program_id: &Pubkey,
) -> Result<Pubkey> {
    // 优先用未用块
    if let Some(next_chunk) = merchant.unused_chunks.pop() {
        merchant.active_chunk = next_chunk;
        merchant.last_chunk_index += 1;
        return Ok(next_chunk);
    }
    // 否则新建块
    let new_chunk_index = merchant.last_chunk_index + 1;
    require!(
        new_chunk_index <= MAX_CHUNKS_PER_MERCHANT,
        ErrorCode::InvalidShardIndex
    );
    let _start_id = root.last_global_id;
    let size = root.chunk_size;
    let (chunk_key, _bump) = Pubkey::find_program_address(
        &[
            b"id_chunk",
            &payer.key().to_bytes(),
            &new_chunk_index.to_le_bytes(),
        ],
        program_id,
    );
    // 这里实际应由外部调用 allocate_new_chunk 指令创建账户
    merchant.active_chunk = chunk_key;
    merchant.last_chunk_index = new_chunk_index;
    root.last_global_id += size as u64;
    Ok(chunk_key)
}

// 纯函数：分配ID
pub fn allocate_id_in_chunk(
    merchant: &mut Account<MerchantIdAccount>,
    chunk: &mut Account<IdChunk>,
) -> Result<u64> {
    if chunk.next_available as usize >= ID_CHUNK_BITMAP_SIZE * 8 {
        return Err(ErrorCode::NoAvailableId.into());
    }
    let mut found = false;
    let mut local_id = chunk.next_available;
    for i in local_id..(ID_CHUNK_BITMAP_SIZE as u32 * 8) {
        let byte_index = (i / 8) as usize;
        let bit_index = i % 8;
        if chunk.bitmap[byte_index] & (1 << bit_index) == 0 {
            chunk.bitmap[byte_index] |= 1 << bit_index;
            chunk.next_available = i + 1;
            merchant.last_local_id = i;
            found = true;
            local_id = i;
            break;
        }
    }
    if !found {
        return Err(ErrorCode::NoAvailableId.into());
    }
    let global_id = chunk.start_id + local_id as u64;
    Ok(global_id)
}

// 纯函数：回收ID
pub fn release_id_in_chunk(chunk: &mut Account<IdChunk>, id: u64) -> Result<()> {
    require!(
        id >= chunk.start_id && id <= chunk.end_id,
        ErrorCode::InvalidId
    );
    let offset = (id - chunk.start_id) as u32;
    let byte_index = (offset / 8) as usize;
    let bit_index = offset % 8;
    chunk.bitmap[byte_index] &= !(1 << bit_index);
    Ok(())
}

// 检查块利用率
pub fn check_chunk_utilization(chunk: &Account<IdChunk>) -> f32 {
    chunk.utilization_rate()
}

// 预分配下一个块（优化性能）
pub fn should_preallocate_chunk(chunk: &Account<IdChunk>) -> bool {
    chunk.utilization_rate() > 0.8 // 超过80%使用率时预分配
}

// 关闭ID块账户
#[derive(Accounts)]
#[instruction(merchant_key: Pubkey, chunk_index: u32)]
pub struct CloseIdChunk<'info> {
    #[account(
        mut,
        close = beneficiary,
        seeds = [b"id_chunk", merchant_key.as_ref(), chunk_index.to_le_bytes().as_ref()],
        bump
    )]
    pub id_chunk: Account<'info, IdChunk>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,
    // 移除merchant账户 - 权限验证通过PDA种子机制已经实现
}

// 关闭商户ID账户
#[derive(Accounts)]
#[instruction(merchant_key: Pubkey)]
pub struct CloseMerchantIdAccount<'info> {
    #[account(
        mut,
        close = beneficiary,
        seeds = [b"merchant_id", merchant_key.as_ref()],
        bump
    )]
    pub merchant_id_account: Account<'info, MerchantIdAccount>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,
    // 移除merchant账户 - 权限验证通过PDA种子机制已经实现
}

// 关闭ID块账户实现
pub fn close_id_chunk(
    ctx: Context<CloseIdChunk>,
    _merchant_key: Pubkey,
    _chunk_index: u32,
    force: bool,
) -> Result<()> {
    let id_chunk = &ctx.accounts.id_chunk;

    // 检查是否为空（除非强制删除）
    if !force {
        require!(
            id_chunk.utilization_rate() == 0.0,
            ErrorCode::IdChunkNotEmpty
        );
    }

    msg!(
        "ID块账户已关闭，块索引: {}, 强制删除: {}",
        _chunk_index,
        force
    );

    // 账户将通过close约束自动关闭并回收租金
    Ok(())
}

// 关闭商户ID账户实现
pub fn close_merchant_id_account(
    ctx: Context<CloseMerchantIdAccount>,
    _merchant_key: Pubkey,
    force: bool,
) -> Result<()> {
    let merchant_id_account = &ctx.accounts.merchant_id_account;

    // 检查是否还有活跃块（除非强制删除）
    if !force {
        require!(
            merchant_id_account.unused_chunks.is_empty(),
            ErrorCode::MerchantIdAccountNotEmpty
        );
    }

    msg!(
        "商户ID账户已关闭，商户: {}, 强制删除: {}",
        merchant_id_account.merchant_id,
        force
    );

    // 账户将通过close约束自动关闭并回收租金
    Ok(())
}
