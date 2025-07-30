use crate::error::ErrorCode;
use crate::state::merchant::Merchant;
use crate::SystemConfig;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

/// Merchant deposit/supplement deposit (unified instruction)
#[derive(Accounts)]
pub struct ManageDeposit<'info> {
    #[account(mut)]
    pub merchant_owner: Signer<'info>,

    // Merchant information account
    #[account(
        mut,
        seeds = [b"merchant_info", merchant_owner.key().as_ref()],
        bump,
        constraint = merchant.owner == merchant_owner.key() @ ErrorCode::InvalidMerchant
    )]
    pub merchant: Account<'info, Merchant>,

    // System configuration account
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, SystemConfig>,

    // Merchant's token account (pay deposit)
    #[account(
        mut,
        constraint = merchant_token_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub merchant_token_account: Account<'info, TokenAccount>,

    // Deposit token mint account
    #[account(
        constraint = deposit_token_mint.key() == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_token_mint: Account<'info, Mint>,

    // System deposit escrow account
    #[account(
        init_if_needed,
        payer = merchant_owner,
        seeds = [b"deposit_escrow", deposit_token_mint.key().as_ref()],
        bump,
        token::mint = deposit_token_mint,
        token::authority = deposit_escrow_account,
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Merchant deposit/supplement deposit (unified processing)
pub fn manage_deposit(ctx: Context<ManageDeposit>, amount: u64) -> Result<()> {
    let merchant = &mut ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;

    // Validate deposit amount
    require!(amount > 0, ErrorCode::InvalidDepositAmount);

    // Validate merchant token account balance
    require!(
        ctx.accounts.merchant_token_account.amount >= amount,
        ErrorCode::InsufficientFunds
    );

    // Validate deposit token type
    require!(
        merchant.is_valid_deposit_token(&system_config.deposit_token_mint),
        ErrorCode::InvalidDepositToken
    );

    // Record deposit status before operation
    let old_deposit = merchant.deposit_amount;
    let is_initial_deposit = old_deposit == 0;

    // Execute token transfer to system escrow account
    let cpi_accounts = Transfer {
        from: ctx.accounts.merchant_token_account.to_account_info(),
        to: ctx.accounts.deposit_escrow_account.to_account_info(),
        authority: ctx.accounts.merchant_owner.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, amount)?;

    // Update merchant deposit balance
    merchant.add_deposit(amount)?;

    // Output different logs based on operation type
    if is_initial_deposit {
        msg!(
            "Merchant {} initial deposit {} tokens, current deposit balance: {}",
            merchant.owner,
            amount,
            merchant.deposit_amount
        );
    } else {
        msg!(
            "Merchant {} supplement deposit {} tokens, deposit balance: {} -> {}",
            merchant.owner,
            amount,
            old_deposit,
            merchant.deposit_amount
        );
    }

    // Force validate if minimum deposit requirement is met
    // Get Token precision
    let token_decimals = ctx.accounts.deposit_token_mint.decimals;
    let required_deposit = system_config.get_deposit_requirement(token_decimals);
    require!(
        merchant.deposit_amount >= required_deposit,
        ErrorCode::MerchantDepositInsufficient
    );

    msg!(
        "Merchant deposit validation passed: {} >= {} (required), Token precision: {}",
        merchant.deposit_amount,
        required_deposit,
        token_decimals
    );

    Ok(())
}

/// Merchant withdraw deposit (supports dual permissions for merchant and administrator)
#[derive(Accounts)]
pub struct WithdrawMerchantDeposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // Merchant information account
    #[account(
        mut,
        seeds = [b"merchant_info", merchant_owner.key().as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,

    /// Merchant owner (signer)
    pub merchant_owner: Signer<'info>,

    // System configuration account
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, SystemConfig>,

    // Token account to receive withdrawn deposit
    #[account(
        mut,
        constraint = recipient_token_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    // Deposit Token mint account (for getting precision)
    #[account(
        constraint = deposit_token_mint.key() == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_token_mint: Account<'info, Mint>,

    // System deposit escrow account
    #[account(
        mut,
        seeds = [b"deposit_escrow", deposit_token_mint.key().as_ref()],
        bump,
        constraint = deposit_escrow_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// Merchant withdraw deposit (only merchant owner can operate)
pub fn withdraw_merchant_deposit(ctx: Context<WithdrawMerchantDeposit>, amount: u64) -> Result<()> {
    let merchant = &mut ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;
    let merchant_owner = &ctx.accounts.merchant_owner;

    // Validate withdrawal amount
    require!(amount > 0, ErrorCode::InvalidDepositAmount);

    // Validate merchant available deposit balance
    require!(
        merchant.get_available_deposit() >= amount,
        ErrorCode::InsufficientDeposit
    );

    // Permission validation: only merchant owner
    require!(
        merchant.owner == merchant_owner.key(),
        ErrorCode::Unauthorized
    );

    // Check if minimum deposit limit is met after withdrawal
    let remaining_deposit = merchant.get_available_deposit().saturating_sub(amount);
    // Get Token precision
    let token_decimals = ctx.accounts.deposit_token_mint.decimals;
    let required_deposit = system_config.get_deposit_requirement(token_decimals);
    require!(
        remaining_deposit >= required_deposit,
        ErrorCode::MerchantDepositInsufficient
    );

    msg!(
        "Merchant {} withdraw deposit, remaining after withdrawal: {} tokens, minimum requirement: {} tokens, Token precision: {}",
        merchant.owner,
        remaining_deposit,
        required_deposit,
        token_decimals
    );

    // Validate escrow account balance
    require!(
        ctx.accounts.deposit_escrow_account.amount >= amount,
        ErrorCode::InsufficientFunds
    );

    // Execute token transfer from system escrow account to recipient account
    let deposit_escrow_bump = ctx.bumps.deposit_escrow_account;
    let token_mint_key = ctx.accounts.deposit_token_mint.key();
    let seeds = &[
        b"deposit_escrow".as_ref(),
        token_mint_key.as_ref(),
        &[deposit_escrow_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.deposit_escrow_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.deposit_escrow_account.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    token::transfer(cpi_ctx, amount)?;

    // Update merchant deposit balance
    merchant.deduct_deposit(amount)?;

    msg!(
        "Deposit withdrawal successful: merchant {}, withdrawal amount: {} tokens, current deposit balance: {} tokens, operator: {}",
        merchant.owner,
        amount,
        merchant.deposit_amount,
        merchant_owner.key()
    );

    Ok(())
}

/// Query merchant deposit information
#[derive(Accounts)]
pub struct GetMerchantDepositInfo<'info> {
    // Merchant information account
    #[account(
        seeds = [b"merchant_info", merchant_owner.key().as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,

    pub merchant_owner: Signer<'info>,

    // System configuration account
    #[account(
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, SystemConfig>,

    // Deposit Token mint account (for getting precision)
    #[account(
        constraint = deposit_token_mint.key() == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_token_mint: Account<'info, Mint>,
}

/// Query merchant deposit information
pub fn get_merchant_deposit_info(
    ctx: Context<GetMerchantDepositInfo>,
) -> Result<MerchantDepositInfo> {
    let merchant = &ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;
    let token_decimals = ctx.accounts.deposit_token_mint.decimals;
    let required_deposit = system_config.get_deposit_requirement(token_decimals);

    Ok(MerchantDepositInfo {
        total_deposit: merchant.deposit_amount,
        locked_deposit: merchant.deposit_locked,
        available_deposit: merchant.get_available_deposit(),
        required_deposit,
        is_sufficient: merchant.deposit_amount >= required_deposit,
        deposit_token_mint: merchant.deposit_token_mint,
        last_updated: merchant.deposit_updated_at,
    })
}

/// Merchant deposit information structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MerchantDepositInfo {
    pub total_deposit: u64,         // Total deposit
    pub locked_deposit: u64,        // Locked deposit
    pub available_deposit: u64,     // Available deposit
    pub required_deposit: u64,      // Required deposit
    pub is_sufficient: bool,        // Whether requirement is met
    pub deposit_token_mint: Pubkey, // Deposit token mint
    pub last_updated: i64,          // Last updated time
}

/// System administrator update deposit requirement
#[derive(Accounts)]
pub struct UpdateDepositRequirement<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    // System configuration account
    #[account(
        mut,
        seeds = [b"system_config"],
        bump
    )]
    pub system_config: Account<'info, SystemConfig>,
}

/// Update deposit requirement
pub fn update_deposit_requirement(
    ctx: Context<UpdateDepositRequirement>,
    new_requirement: u64,
) -> Result<()> {
    let system_config = &mut ctx.accounts.system_config;

    require!(new_requirement > 0, ErrorCode::InvalidDepositAmount);

    let old_requirement = system_config.merchant_deposit_required;
    system_config.merchant_deposit_required = new_requirement;

    msg!(
        "System deposit requirement updated: {} -> {} tokens",
        old_requirement,
        new_requirement
    );

    Ok(())
}

/// Administrator deduct merchant deposit (for violation penalties, etc.)
#[derive(Accounts)]
pub struct DeductMerchantDeposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    // Merchant information account
    #[account(
        mut,
        seeds = [b"merchant_info", merchant_owner.key().as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,

    /// Merchant owner public key (for PDA calculation)
    /// CHECK: This is the merchant's public key, used for PDA calculation
    pub merchant_owner: UncheckedAccount<'info>,

    // System configuration account
    #[account(
        seeds = [b"system_config"],
        bump,
        constraint = system_config.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub system_config: Account<'info, SystemConfig>,

    // Deposit Token mint account (for getting precision)
    #[account(
        constraint = deposit_token_mint.key() == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_token_mint: Account<'info, Mint>,

    // System deposit escrow account
    #[account(
        mut,
        seeds = [b"deposit_escrow", deposit_token_mint.key().as_ref()],
        bump,
        constraint = deposit_escrow_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub deposit_escrow_account: Account<'info, TokenAccount>,

    // Administrator token account to receive deducted deposit
    #[account(
        mut,
        constraint = admin_token_account.mint == system_config.deposit_token_mint @ ErrorCode::InvalidDepositToken
    )]
    pub admin_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// Administrator deduct merchant deposit
pub fn deduct_merchant_deposit(
    ctx: Context<DeductMerchantDeposit>,
    amount: u64,
    reason: String,
) -> Result<()> {
    let merchant = &mut ctx.accounts.merchant;
    let system_config = &ctx.accounts.system_config;

    // Validate deduction amount
    require!(amount > 0, ErrorCode::InvalidDepositAmount);
    require!(!reason.is_empty(), ErrorCode::InvalidDepositAmount);

    // Validate merchant deposit balance
    require!(
        merchant.deposit_amount >= amount,
        ErrorCode::InsufficientDeposit
    );

    // Validate escrow account balance
    require!(
        ctx.accounts.deposit_escrow_account.amount >= amount,
        ErrorCode::InsufficientFunds
    );

    // Execute token transfer from system escrow account to administrator account
    let deposit_escrow_bump = ctx.bumps.deposit_escrow_account;
    let seeds = &[b"deposit_escrow".as_ref(), &[deposit_escrow_bump]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.deposit_escrow_account.to_account_info(),
        to: ctx.accounts.admin_token_account.to_account_info(),
        authority: ctx.accounts.deposit_escrow_account.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    token::transfer(cpi_ctx, amount)?;

    // Update merchant deposit balance
    merchant.deduct_deposit(amount)?;

    msg!(
        "Administrator {} deducted merchant {} deposit {} tokens, reason: {}, remaining deposit: {}",
        ctx.accounts.authority.key(),
        merchant.owner,
        amount,
        reason,
        merchant.deposit_amount
    );

    // Check if deposit is below requirement
    let token_decimals = ctx.accounts.deposit_token_mint.decimals;
    let required_deposit = system_config.get_deposit_requirement(token_decimals);
    if merchant.deposit_amount < required_deposit {
        msg!(
            "Warning: Merchant deposit insufficient, current: {} < required: {}, Token precision: {}",
            merchant.deposit_amount,
            required_deposit,
            token_decimals
        );
    }

    Ok(())
}
