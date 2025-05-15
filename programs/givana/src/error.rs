use anchor_lang::prelude::*;


#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid donation rate")]
    InvalidDonationRate,
    #[msg("NGO not active")]
    NgoNotActive,
    #[msg("NGO already set")]
    NgoAlreadySet,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Invalid vault balance")]
    InvalidVaultBalance,
    #[msg("Transfer failed")]
    TransferFailed,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("No pending rewards")]
    NoPendingRewards,
    #[msg("Reward already claimed")]
    RewardAlreadyClaimed,
    #[msg("Invalid NGO authority")]
    InvalidNgoAuthority,
    #[msg("Invalid user authority")]
    InvalidUserAuthority,
    #[msg("No withdraw requested")]
    NoWithdrawRequested,
    #[msg("Withdraw not ready")]
    WithdrawNotReady,
    #[msg("Withdraw already requested")]
    WithdrawAlreadyRequested,
    #[msg("No staked tokens")]
    NoStakedTokens,
    #[msg("Withdrawal amount is too small")]
    WithdrawAmountTooSmall,
    #[msg("Epoch duration is not completed")]
    EpochDurationNotCompleted,
    #[msg("Withdrawal request expired")]
    WithdrawRequestExpired,
    #[msg("Invalid withdrawal amount (total amount would be zero)")]
    InvalidWithdrawAmount,
    #[msg("Insufficient vault balance for withdrawal")]
    InsufficientVaultBalance,
    #[msg("Insufficient token balance in user's account")]
    InsufficientTokenBalance,
    #[msg("Insufficient reward balance in reward pool")]
    InsufficientRewardBalance,
    #[msg("Invalid Jito vault config")]
    InvalidJitoVaultConfig,
    #[msg("Insufficient JitoSOL balance in vault")]
    InsufficientJitosol,
    #[msg("Vault already initialized")]
    VaultAlreadyInitialized,
    #[msg("Staking period is too short")]
    StakingPeriodTooShort,
    #[msg("NGO already active")]
    NgoAlreadyActive,
}