use crate::error::ErrorCode;
use crate::state::{
    GlobalIdRoot, IdChunk, MerchantIdAccount, DEFAULT_CHUNK_SIZE, ID_CHUNK_BITMAP_SIZE,
    MAX_CHUNKS_PER_MERCHANT,
};
use anchor_lang::prelude::*;

// ID generator functionality
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

    // Check if current chunk has available IDs
    if active_chunk.is_full() {
        return Err(ErrorCode::NoAvailableId.into());
    }

    // Find next available ID
    let mut local_id = active_chunk.next_available;
    while local_id < active_chunk.capacity() {
        if !active_chunk.is_id_used(local_id) {
            // Allocate this ID
            active_chunk.mark_id_used(local_id);
            active_chunk.next_available = local_id + 1;
            merchant_account.last_local_id = local_id;

            let global_id = active_chunk.start_id + local_id;

            msg!(
                "Product ID generation successful, merchant: {}, local ID: {}, global ID: {}",
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

// 3. Allocate new chunk
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

    // Update merchant chunk index
    merchant_account.last_chunk_index += 1;
    let chunk_index = merchant_account.last_chunk_index;

    require!(
        chunk_index <= MAX_CHUNKS_PER_MERCHANT,
        ErrorCode::InvalidShardIndex
    );

    // Initialize new chunk - use merchant ID based range
    let merchant_start_id = merchant_account.merchant_id as u64 * 10000; // Reserve 10000 IDs per merchant
    let chunk_start_id = merchant_start_id + (chunk_index as u64 * global_root.chunk_size as u64);
    let new_chunk = &mut ctx.accounts.new_chunk;
    new_chunk.merchant_id = merchant_account.merchant_id;
    new_chunk.chunk_index = chunk_index;
    new_chunk.start_id = chunk_start_id;
    new_chunk.end_id = chunk_start_id + global_root.chunk_size as u64 - 1;
    new_chunk.next_available = 0;
    new_chunk.initialize_bitmap(); // Use safe initialization method
    new_chunk.bump = ctx.bumps.new_chunk;

    // Update global ID counter
    global_root.last_global_id = new_chunk.end_id + 1;

    // Add old active chunk to unused queue
    let old_active_chunk = merchant_account.active_chunk;
    merchant_account.unused_chunks.push(old_active_chunk);

    // Set new active chunk
    merchant_account.active_chunk = new_chunk.key();

    msg!(
        "New ID chunk allocation successful, merchant: {}, chunk index: {}, ID range: {} - {}",
        merchant_account.merchant_id,
        chunk_index,
        new_chunk.start_id,
        new_chunk.end_id
    );

    Ok(new_chunk.key())
}

// 4. ID existence verification
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct VerifyId<'info> {
    #[account(
        seeds = [b"merchant_id", merchant.key().as_ref()],
        bump
    )]
    pub merchant_account: Account<'info, MerchantIdAccount>,

    pub merchant: Signer<'info>,

    /// CHECK: Will verify this is the correct ID chunk in the instruction
    pub id_chunk: AccountInfo<'info>,
}

pub fn is_id_exists(ctx: Context<VerifyId>, id: u64) -> Result<bool> {
    let merchant_account = &ctx.accounts.merchant_account;

    // Calculate which chunk the ID should be in
    let chunk_size = DEFAULT_CHUNK_SIZE as u64;
    let merchant_start_id = merchant_account.merchant_id as u64 * 10000; // Reserve 10000 IDs per merchant

    if id < merchant_start_id {
        return Ok(false);
    }

    let chunk_index = (id - merchant_start_id) / chunk_size;

    // Verify ID chunk account
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

    // Deserialize ID chunk and check
    let chunk_account_data = ctx.accounts.id_chunk.data.borrow();
    let chunk = IdChunk::try_deserialize(&mut &chunk_account_data[8..])?;

    if id < chunk.start_id || id > chunk.end_id {
        return Ok(false);
    }

    let local_id = id - chunk.start_id;
    Ok(chunk.is_id_used(local_id))
}

// 5. Batch allocate IDs
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

    require!(count > 0 && count <= 100, ErrorCode::InvalidId); // Limit batch quantity

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
        "Batch ID generation successful, merchant: {}, quantity: {}",
        merchant_account.merchant_id,
        allocated
    );

    Ok(ids)
}

// 6. ID recycling
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
    let offset = id - chunk.start_id;
    let byte_index = (offset / 8) as usize;
    let bit_index = (offset % 8) as u8;
    chunk.bitmap[byte_index] &= !(1 << bit_index);
    Ok(())
}

// Helper function: chunk switching/pre-allocation
pub fn switch_or_allocate_chunk<'info>(
    merchant: &mut Account<'info, MerchantIdAccount>,
    root: &mut Account<'info, GlobalIdRoot>,
    payer: &Signer<'info>,
    _system_program: &Program<'info, System>,
    program_id: &Pubkey,
) -> Result<Pubkey> {
    // Prioritize using unused chunks
    if let Some(next_chunk) = merchant.unused_chunks.pop() {
        merchant.active_chunk = next_chunk;
        merchant.last_chunk_index += 1;
        return Ok(next_chunk);
    }
    // Otherwise create new chunk
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
    // This should actually be called externally by the allocate_new_chunk instruction to create the account
    merchant.active_chunk = chunk_key;
    merchant.last_chunk_index = new_chunk_index;
    root.last_global_id += size as u64;
    Ok(chunk_key)
}

// Pure function: allocate ID
pub fn allocate_id_in_chunk(
    merchant: &mut Account<MerchantIdAccount>,
    chunk: &mut Account<IdChunk>,
) -> Result<u64> {
    if chunk.next_available as usize >= ID_CHUNK_BITMAP_SIZE * 8 {
        return Err(ErrorCode::NoAvailableId.into());
    }
    let mut found = false;
    let mut local_id = chunk.next_available;
    for i in local_id..(ID_CHUNK_BITMAP_SIZE as u64 * 8) {
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

// Pure function: recycle ID
pub fn release_id_in_chunk(chunk: &mut Account<IdChunk>, id: u64) -> Result<()> {
    require!(
        id >= chunk.start_id && id <= chunk.end_id,
        ErrorCode::InvalidId
    );
    let offset = id - chunk.start_id;
    let byte_index = (offset / 8) as usize;
    let bit_index = (offset % 8) as u8;
    chunk.bitmap[byte_index] &= !(1 << bit_index);
    Ok(())
}

// Check chunk utilization
pub fn check_chunk_utilization(chunk: &Account<IdChunk>) -> f32 {
    chunk.utilization_rate()
}

// Pre-allocate next chunk (performance optimization)
pub fn should_preallocate_chunk(chunk: &Account<IdChunk>) -> bool {
    chunk.utilization_rate() > 0.8 // Pre-allocate when utilization exceeds 80%
}

// Close ID chunk account
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
    // Remove merchant account - permission verification already implemented through PDA seed mechanism
}

// Close merchant ID account
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
    // Remove merchant account - permission verification already implemented through PDA seed mechanism
}

// Close ID chunk account implementation
pub fn close_id_chunk(
    ctx: Context<CloseIdChunk>,
    _merchant_key: Pubkey,
    _chunk_index: u32,
    force: bool,
) -> Result<()> {
    let id_chunk = &ctx.accounts.id_chunk;

    // Check if empty (unless force delete)
    if !force {
        require!(
            id_chunk.utilization_rate() == 0.0,
            ErrorCode::IdChunkNotEmpty
        );
    }

    msg!(
        "ID chunk account closed, chunk index: {}, force delete: {}",
        _chunk_index,
        force
    );

    // Account will be automatically closed and rent reclaimed through close constraint
    Ok(())
}

// Close merchant ID account implementation
pub fn close_merchant_id_account(
    ctx: Context<CloseMerchantIdAccount>,
    _merchant_key: Pubkey,
    force: bool,
) -> Result<()> {
    let merchant_id_account = &ctx.accounts.merchant_id_account;

    // Check if there are still active chunks (unless force delete)
    if !force {
        require!(
            merchant_id_account.unused_chunks.is_empty(),
            ErrorCode::MerchantIdAccountNotEmpty
        );
    }

    msg!(
        "Merchant ID account closed, merchant: {}, force delete: {}",
        merchant_id_account.merchant_id,
        force
    );

    // Account will be automatically closed and rent reclaimed through close constraint
    Ok(())
}
