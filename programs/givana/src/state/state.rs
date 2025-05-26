use anchor_lang::prelude::*;
use anchor_lang::InitSpace;

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub total_jitosol_deposited: u64,
    pub total_gsol_supply: u64,
    pub acc_total_yield_per_gsol: u64,
    pub current_block_index: u64,
    pub total_nsol_minted: u64,
    pub last_update_time: i64,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub protocol_admin: Pubkey,
    pub jito_vault_input_token_mint: Pubkey,
    pub bump: u8,
    pub vault_bump: u8,
    pub protocol_vault_authority_bump: u8,
    pub jito_manager_bump: u8,
}



#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub authority: Pubkey,
    pub gsol_balance: u64,
    pub donation_rate: u16,
    pub ngo_address: Pubkey,
    pub stake_time: i64,
    pub last_total_yield_checkpoint: u64,
    pub last_claim_time: i64,
    pub last_claim_block: u64,  // Track which block the user last claimed rewards
    pub total_claimed: u64,
    pub is_initialized: bool,
    pub withdraw_requested: bool,
    pub withdraw_request_time: i64,
    pub withdraw_amount: u64,
    pub pending_rewards: u64,
}

#[account]
#[derive(InitSpace)]
pub struct NGOAccount {
    pub authority: Pubkey,
    pub is_active: bool,
    pub pending_rewards: u64,        // Rewards waiting to be claimed
    pub last_claim_time: i64,        // Last time rewards were claimed
    pub total_donations_received: u64, // Total donations received (in jitoSOL)
    pub pending_claimable_donations: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct SettlementBlock {
    pub id: u64,                             // Block identifier
    pub start_time: i64,                     // Block start timestamp
    pub end_time: i64,                       // Block end timestamp
    pub reward_amount: u64,                  // Rewards distributed in this block
    pub acc_reward_snapshot: u64,            // Reward accumulator at block creation
    pub acc_ngo_donation_snapshot: u64,      // NGO donation accumulator at block creation
}



#[account]
#[derive(InitSpace)]
pub struct RewardPool {
    pub initialized: bool,
    pub total_undistributed_rewards: u64,
    pub vault_token_balance: u64,  // Track vault token balance here
    pub ngo_pending_rewards: u64,    // Total rewards allocated for NGOs
    pub bump: u8,
    pub last_update_time: i64,
}



#[account]
#[derive(InitSpace)]
pub struct JitoVaultConfig {
    pub bump: u8,
    pub jito_vault_address: Pubkey,
    pub jito_vault_program_id: Pubkey,
    pub jito_vault_vrt_mint: Pubkey,
    pub initialized: bool,
}

