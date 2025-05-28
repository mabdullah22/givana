use anchor_lang::prelude::*;
use crate::state::{UserAccount, GlobalState, NGOAccount};
use crate::error::ErrorCode; // Assuming ErrorCode might be needed for constraints

// Instruction to update a single user's reward accrual and credit their chosen NGO.
// This can be called by anyone (permissionless crank) to ensure rewards are settled
// even for inactive users, making donations available to NGOs.
#[derive(Accounts)]
#[instruction(user_account_authority: Pubkey)] // The authority of the UserAccount to crank, used for seeds.
pub struct CrankIndividualUserRewards<'info> {
    // The UserAccount to be cranked. Its rewards will be calculated and settled.
    #[account(
        mut,
        seeds = [b"user", user_account_authority.as_ref()],
        bump,
        // No owner check here, as this is a crank. Anyone can call it for any user.
        // We could add a constraint that user_account.authority == user_account_authority if we want to be explicit,
        // but the seeds already tie it.
    )]
    pub user_account_to_crank: Account<'info, UserAccount>,

    // Global state, needed for the current global reward accumulator value.
    // It's not mutated by the update_user_rewards_and_credit_ngo helper directly.
    #[account(
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    // The NGO account associated with the user_account_to_crank.
    // This account will be credited with the user's donated portion of rewards.
    // It needs to be mutable.
    // The seeds must use user_account_to_crank.ngo_address.
    // This requires careful handling in how accounts are passed or derived.
    // If user_account_to_crank.ngo_address is Pubkey::default(), this account might not be used
    // but still needs to be validly defined for the struct.
    // The constraint ensures that if an NGO is set, it's active and matches.
    #[account(
        mut,
        seeds = [b"ngo", user_account_to_crank.ngo_address.as_ref()],
        bump,
        constraint = (user_account_to_crank.ngo_address != Pubkey::default() && ngo_account.is_active && ngo_account.key() == user_account_to_crank.ngo_address) || user_account_to_crank.ngo_address == Pubkey::default() @ ErrorCode::NgoNotActiveOrMismatch
    )]
    pub ngo_account: Account<'info, NGOAccount>,

    // We don't need RewardPool here as update_user_rewards_and_credit_ngo doesn't directly use it.
}

// Implementation of the crank_individual_user_rewards function will go here in the next step.
```
