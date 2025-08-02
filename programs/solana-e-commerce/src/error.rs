use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // ID generator related errors
    #[msg("Missing keyword account")]
    MissingKeywordAccount,
    #[msg("Too many keywords")]
    TooManyKeywords,
    #[msg("Shard space insufficient")]
    ShardFull,
    #[msg("ID generation failed")]
    IdGenerationFailed,
    #[msg("Rent calculation failed")]
    RentCalculationFailed,
    #[msg("Merchant not registered")]
    MerchantNotRegistered,
    #[msg("ID already in use")]
    IdAlreadyInUse,
    #[msg("ID not found")]
    IdNotFound,
    #[msg("ID range overflow")]
    IdRangeOverflow,
    #[msg("No available ID")]
    NoAvailableId,
    #[msg("Invalid ID")]
    InvalidId,
    #[msg("Integer overflow")]
    IntegerOverflow,

    // Product related errors
    #[msg("Product not found")]
    ProductNotFound,
    #[msg("Invalid product")]
    InvalidProduct,
    #[msg("Invalid product account")]
    InvalidProductAccount,
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Purchase quantity must be greater than 0")]
    InvalidAmount,
    #[msg("Invalid product name")]
    InvalidProductName,
    #[msg("Invalid product name length")]
    InvalidProductNameLength,
    #[msg("Invalid product description")]
    InvalidProductDescription,
    #[msg("Invalid product description length")]
    InvalidProductDescriptionLength,
    #[msg("Too many image URLs")]
    TooManyImageUrls,
    #[msg("Too many sales regions")]
    TooManySalesRegions,
    #[msg("Too many logistics methods")]
    TooManyLogisticsMethods,

    // Merchant related errors
    #[msg("Invalid merchant")]
    InvalidMerchant,
    #[msg("Invalid merchant name length")]
    InvalidMerchantNameLength,
    #[msg("Invalid merchant description length")]
    InvalidMerchantDescriptionLength,
    #[msg("Unauthorized merchant operation")]
    UnauthorizedMerchant,

    // Keyword related errors
    #[msg("Invalid keyword")]
    InvalidKeyword,
    #[msg("Invalid keyword length")]
    InvalidKeywordLength,
    #[msg("Invalid keyword count")]
    InvalidKeywordCount,
    #[msg("Duplicate keyword")]
    DuplicateKeyword,

    // Index related errors
    #[msg("Shard is full")]
    ShardIsFull,
    #[msg("Invalid shard index")]
    InvalidShardIndex,
    #[msg("Price index node not found")]
    PriceIndexNodeNotFound,
    #[msg("Sales index node not found")]
    SalesIndexNodeNotFound,
    #[msg("Invalid price range")]
    InvalidPriceRange,
    #[msg("Invalid sales range")]
    InvalidSalesRange,
    #[msg("Bloom filter update failed")]
    BloomFilterUpdateFailed,
    #[msg("Keyword index not empty")]
    KeywordIndexNotEmpty,
    #[msg("Keyword shard not empty")]
    KeywordShardNotEmpty,
    #[msg("Merchant has active products")]
    MerchantHasActiveProducts,
    #[msg("ID chunk not empty")]
    IdChunkNotEmpty,
    #[msg("Merchant ID account not empty")]
    MerchantIdAccountNotEmpty,

    // Payment related errors
    #[msg("Unsupported token")]
    UnsupportedToken,
    #[msg("Insufficient token balance")]
    InsufficientTokenBalance,
    #[msg("Insufficient SOL balance")]
    InsufficientSolBalance,
    #[msg("Invalid token amount")]
    InvalidTokenAmount,
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    #[msg("Fee calculation error")]
    FeeCalculationError,
    #[msg("Payment config not found")]
    PaymentConfigNotFound,
    #[msg("Token not active")]
    TokenNotActive,
    #[msg("Below minimum amount")]
    BelowMinimumAmount,
    #[msg("Product creation failed")]
    ProductCreationFailed,
    #[msg("Atomic operation failed")]
    AtomicOperationFailed,
    #[msg("Invalid fee rate")]
    InvalidFeeRate,
    #[msg("Too many tokens")]
    TooManyTokens,
    #[msg("Invalid token symbol")]
    InvalidTokenSymbol,
    #[msg("Invalid token decimals")]
    InvalidTokenDecimals,
    #[msg("Invalid order status")]
    InvalidOrderStatus,
    #[msg("Invalid payment method")]
    InvalidPaymentMethod,

    // Order related errors
    #[msg("Order not found")]
    OrderNotFound,
    #[msg("Invalid order quantity")]
    InvalidOrderQuantity,
    #[msg("Invalid order price")]
    InvalidOrderPrice,
    #[msg("Invalid order total amount")]
    InvalidOrderTotalAmount,
    #[msg("Invalid order token price")]
    InvalidOrderTokenPrice,
    #[msg("Invalid order token total amount")]
    InvalidOrderTokenTotalAmount,
    #[msg("Invalid shipping address length")]
    InvalidShippingAddressLength,
    #[msg("Invalid order notes length")]
    InvalidOrderNotesLength,
    #[msg("Invalid transaction signature")]
    InvalidTransactionSignature,
    #[msg("Invalid order status transition")]
    InvalidOrderStatusTransition,
    #[msg("Order cannot be modified")]
    OrderCannotBeModified,
    #[msg("Order cannot be refunded")]
    OrderCannotBeRefunded,
    #[msg("Order already exists")]
    OrderAlreadyExists,

    // System related errors
    #[msg("Unauthorized operation")]
    Unauthorized,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Invalid account owner")]
    InvalidAccountOwner,
    #[msg("Invalid account data")]
    InvalidAccountData,
    #[msg("Invalid account size")]
    InvalidAccountSize,
    #[msg("Invalid PDA")]
    InvalidPda,
    #[msg("Invalid account seeds")]
    InvalidAccountSeeds,
    #[msg("Invalid account bump")]
    InvalidAccountBump,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Invalid active chunk")]
    InvalidActiveChunk,
    #[msg("Account discriminator mismatch")]
    AccountDiscriminatorMismatch,
    #[msg("Insufficient accounts")]
    InsufficientAccounts,

    // Deposit related errors
    #[msg("Insufficient deposit")]
    InsufficientDeposit,
    #[msg("Insufficient locked deposit")]
    InsufficientLockedDeposit,
    #[msg("Invalid deposit token")]
    InvalidDepositToken,
    #[msg("Invalid deposit amount")]
    InvalidDepositAmount,
    #[msg("Merchant deposit insufficient for transaction")]
    MerchantDepositInsufficient,
    #[msg("Deposit already locked")]
    DepositAlreadyLocked,
    #[msg("Deposit not locked")]
    DepositNotLocked,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,

    // Tracking number related errors
    #[msg("Tracking number required for shipping")]
    TrackingNumberRequired,
    #[msg("Invalid tracking number")]
    InvalidTrackingNumber,

    // Vault related errors
    #[msg("Invalid vault program")]
    InvalidVaultProgram,
    #[msg("Invalid vault account")]
    InvalidVaultAccount,
    #[msg("Invalid vault token account")]
    InvalidVaultTokenAccount,
    #[msg("Invalid platform token account")]
    InvalidPlatformTokenAccount,
}
