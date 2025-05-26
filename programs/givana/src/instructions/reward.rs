use anchor_lang::prelude::*;
use anchor_lang::solana_program;
// use anchor_spl::{
//     token_interface::{
//         Mint, TokenAccount, transfer_checked, 
//         TransferChecked, TokenInterface, mint_to, MintTo
//     },
//     associated_token::AssociatedToken
// };

use anchor_spl::{
    token::{Mint as TokenMint, TokenAccount as TokenAccount2, Token},
    token_interface::{Mint as InterfaceMint, TokenAccount as InterfaceTokenAccount, TokenInterface},
    associated_token::AssociatedToken,
};

// use jito_vault_sdk::instruction::VaultInstruction::InitializeVault;
use jito_vault_sdk::sdk::{initialize_vault,mint_to,delegate_token_account,create_token_metadata};
use jito_vault_sdk::instruction::VaultInstruction::InitializeVault;

use jito_vault_sdk::inline_mpl_token_metadata::pda::find_metadata_account;
use crate::state::{GlobalState, UserAccount,RewardPool,NGOAccount};
use crate::error::ErrorCode;

use crate::constants::{ADMIN_ADRESS,JITO_CONFIG, PRECISION_FACTOR};
use std::str::FromStr;
use crate::state::JitoVaultConfig;
pub static JITO_VAULT_PROGRAM_ID: Pubkey = pubkey!("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");


#[derive(Accounts)]
pub struct TransferJitoVaultRewardToRewardPool<'info> {
    /// CHECK: Verified by Jito Vault program
    #[account(
        constraint = config.key() == Pubkey::from_str(JITO_CONFIG).unwrap()
    )]
    pub config: AccountInfo<'info>,
    
    /// CHECK: Vault account that will be updated
    #[account(mut,
        seeds = [b"vault", jito_manager.key().as_ref()],
        seeds::program = JITO_VAULT_PROGRAM_ID,
        bump
    )]
    pub vault: AccountInfo<'info>,

    /// The jitoSOL mint
    #[account(
        constraint = jito_sol_mint.key() == global_state.input_token_mint
    )]
    pub jito_sol_mint: InterfaceAccount<'info, InterfaceMint>,

    /// Vault's jitoSOL token account (source)
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,

    /// Reward pool's jitoSOL token account (destination)
    #[account(mut,
        token::mint = jito_sol_mint,
        token::authority = reward_pool_authority
    )]
    pub reward_pool_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,

    /// CHECK: This is the PDA that will sign for the transfer
    #[account(
        seeds = [b"jito_manager", admin.key().as_ref()],
        bump
    )]
    pub jito_manager: AccountInfo<'info>,

    /// CHECK: This is the reward pool authority
    #[account(
        seeds = [b"reward_pool_authority", jito_sol_mint.key().as_ref()],
        bump
    )]
    pub reward_pool_authority: AccountInfo<'info>,

    #[account(mut,
        constraint = admin.key() == Pubkey::from_str(ADMIN_ADRESS).unwrap()
    )]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [b"reward_pool_state_v2"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    pub token_program: Interface<'info, TokenInterface>,
}

// ProcessRewards struct definition is removed.

// #[derive(Accounts)]
// pub struct UpdateRewardAccumulators<'info> {
//     #[account(mut)]
//     pub admin: Signer<'info>,

//     #[account(
//         mut,
//         seeds = [b"global-state"],
//         bump
//     )]
//     pub global_state: Account<'info, GlobalState>,

//     pub system_program: Program<'info, System>,
// }

#[derive(Accounts)]
pub struct ClaimNgoRewards<'info> {
    #[account(mut)]
    pub ngo_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"ngo", ngo_authority.key().as_ref()],
        bump,
        constraint = ngo_account.authority == ngo_authority.key() @ ErrorCode::InvalidNgoAuthority,
        constraint = ngo_account.is_active @ ErrorCode::NgoNotActive
    )]
    pub ngo_account: Account<'info, NGOAccount>,

    #[account(
        mut,
        seeds = [b"reward_pool_state_v2"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// The jitoSOL mint
    #[account(
        constraint = jito_sol_mint.key() == global_state.input_token_mint
    )]
    pub jito_sol_mint: InterfaceAccount<'info, InterfaceMint>,

    /// NGO's jitoSOL token account
    #[account(mut,
        token::mint = jito_sol_mint,
        token::authority = ngo_authority
    )]
    pub ngo_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,

    /// Reward pool's jitoSOL token account
    #[account(mut,
        token::mint = jito_sol_mint,
        token::authority = reward_pool_authority
    )]
    pub reward_pool_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,

    /// CHECK: This is the reward pool authority
    #[account(
        seeds = [b"reward_pool_authority", jito_sol_mint.key().as_ref()],
        bump
    )]
    pub reward_pool_authority: AccountInfo<'info>,

    #[account(
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    pub token_program: Interface<'info, TokenInterface>,
}

// DistributeNgoRewards struct is removed.

#[derive(Accounts)]
pub struct ClaimStakerRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [b"reward_pool_state_v2"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// The jitoSOL mint
    #[account(
        constraint = jito_sol_mint.key() == global_state.input_token_mint
    )]
    pub jito_sol_mint: InterfaceAccount<'info, InterfaceMint>,

    /// User's jitoSOL token account
    #[account(
        mut,
        associated_token::mint = jito_sol_mint,
        associated_token::authority = user
    )]
    pub user_jitosol_ata: InterfaceAccount<'info, InterfaceTokenAccount>,

    /// Reward pool's jitoSOL token account
    #[account(
        mut,
        token::mint = jito_sol_mint,
        token::authority = reward_pool_authority
    )]
    pub reward_pool_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,

    /// CHECK: This is the reward pool authority
    #[account(
        seeds = [b"reward_pool_authority", jito_sol_mint.key().as_ref()],
        bump
    )]
    pub reward_pool_authority: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// Internal helper function to update user rewards and credit NGO
pub(crate) fn update_user_rewards_and_credit_ngo<'info>(
    user_account: &mut Account<'info, UserAccount>,
    global_state: &Account<'info, GlobalState>,
    ngo_account_option: Option<&mut Account<'info, NGOAccount>>,
) -> Result<()> {
    let current_total_yield_acc = global_state.acc_total_yield_per_gsol;
    let last_checkpoint = user_account.last_total_yield_checkpoint;

    // If gSOL balance is zero, no rewards can be generated.
    // Also, if yield hasn't changed, no new rewards.
    // Update checkpoint only if current_total_yield_acc > last_checkpoint.
    // If current_total_yield_acc == last_checkpoint, nothing changes, including checkpoint.
    if user_account.gsol_balance == 0 {
        if current_total_yield_acc > last_checkpoint {
            user_account.last_total_yield_checkpoint = current_total_yield_acc;
        }
        return Ok(()); // No rewards to process as gsol_balance is zero
    }

    if current_total_yield_acc <= last_checkpoint {
        // No new yield accumulated or an erroneous state (should not happen if time moves forward)
        // No need to update checkpoint if it's already current or ahead.
        return Ok(()); 
    }

    let delta_yield = current_total_yield_acc
        .checked_sub(last_checkpoint) // Already checked current_total_yield_acc > last_checkpoint
        .ok_or(ErrorCode::ArithmeticOverflow)?; 

    let total_rewards_generated_by_stake_u128 = (user_account.gsol_balance as u128)
        .checked_mul(delta_yield as u128)
        .ok_or(ErrorCode::ArithmeticOverflow)?
        .checked_div(PRECISION_FACTOR as u128)
        .ok_or(ErrorCode::DivisionByZero)?; // PRECISION_FACTOR is non-zero
    
    if total_rewards_generated_by_stake_u128 == 0 {
        // If no rewards were generated (e.g. due to truncation in division),
        // still update the checkpoint and exit.
        user_account.last_total_yield_checkpoint = current_total_yield_acc;
        return Ok(());
    }
    
    // Ensure it fits in u64
    if total_rewards_generated_by_stake_u128 > u64::MAX as u128 {
        return Err(error!(ErrorCode::ArithmeticOverflow));
    }
    let total_rewards_generated_by_stake_u64 = total_rewards_generated_by_stake_u128 as u64;

    // This check is technically redundant now due to the u128 check above, 
    // but kept for logical clarity if total_rewards_generated_by_stake_u64 could be 0 from other paths.
    // if total_rewards_generated_by_stake_u64 > 0 { // Covered by total_rewards_generated_by_stake_u128 == 0 check
        let ngo_donation_amount_u64: u64;
        let user_net_reward_amount: u64;

        if user_account.donation_rate > 0 && user_account.ngo_address != Pubkey::default() {
            let ngo_donation_amount_u128 = (total_rewards_generated_by_stake_u64 as u128)
                .checked_mul(user_account.donation_rate as u128) // donation_rate is u16 (0-10000)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_div(10000 as u128) // Basis points division
                .ok_or(ErrorCode::DivisionByZero)?; // 10000 is non-zero
            
            if ngo_donation_amount_u128 > u64::MAX as u128 {
                 return Err(error!(ErrorCode::ArithmeticOverflow));
            }
            ngo_donation_amount_u64 = ngo_donation_amount_u128 as u64;

            user_net_reward_amount = total_rewards_generated_by_stake_u64
                .checked_sub(ngo_donation_amount_u64)
                .ok_or(ErrorCode::ArithmeticOverflow)?;

            if let Some(ngo_account_ref_mut) = ngo_account_option {
                // Ensure the provided NGO account matches the user's selected NGO
                // This check is crucial: user_account.ngo_address must be the authority of the ngo_account_ref_mut
                // For PDAs, the authority is often part of the seeds or a field in the account.
                // Assuming ngo_account_ref_mut.authority field stores the NGO's identity key.
                if ngo_account_ref_mut.key() == user_account.ngo_address {
                     ngo_account_ref_mut.pending_claimable_donations = ngo_account_ref_mut.pending_claimable_donations
                        .checked_add(ngo_donation_amount_u64)
                        .ok_or(ErrorCode::ArithmeticOverflow)?;
                } else {
                    // msg!("Warning: Provided NGO account does not match user's selected NGO address.");
                    // This case implies a mismatch in how accounts are loaded by the caller.
                    // The donation for this user, for this transaction, will not be credited to the (wrong) NGO account.
                    // It's also not being added to user_net_reward_amount, so it's effectively burned if this path is taken.
                    // This should be an error or handled more gracefully by ensuring correct account loading.
                }
            } else {
                // msg!("Warning: User has donation settings but no NGO account was provided to credit.");
                // Similar to above, donation is effectively burned.
            }
        } else {
            // No donation (rate is 0 or no NGO address)
            ngo_donation_amount_u64 = 0;
            user_net_reward_amount = total_rewards_generated_by_stake_u64;
        }

        if user_net_reward_amount > 0 {
            user_account.pending_rewards = user_account.pending_rewards
                .checked_add(user_net_reward_amount)
                .ok_or(ErrorCode::ArithmeticOverflow)?;
        }
    // } // End of if total_rewards_generated_by_stake_u64 > 0

    user_account.last_total_yield_checkpoint = current_total_yield_acc;

    Ok(())
}

// The Implementation Functions

impl<'info> TransferJitoVaultRewardToRewardPool<'info> {
    pub fn transfer_jito_vault_reward_to_reward_pool(&mut self) -> Result<()> {
        // Only admin can transfer tokens
        if self.admin.key() != Pubkey::from_str(ADMIN_ADRESS).unwrap() {
            return Err(error!(ErrorCode::Unauthorized));
        }

        // Get the vault token account balance
        let vault_balance = self.vault_token_account.amount;
        if vault_balance == 0 {
            return Err(error!(ErrorCode::InsufficientBalance));
        }

        // Verify the vault balance matches our internal tracking
        if vault_balance != self.reward_pool.vault_token_balance {
            return Err(error!(ErrorCode::InvalidVaultBalance));
        }

        // Store the transfer amount before the transfer
        let transfer_amount = vault_balance;

        // Get the bump for jito_manager PDA
        let bump = self.global_state.jito_manager_bump;
        let admin_key = self.admin.key();
        let authority_seeds = &[
            b"jito_manager",
            admin_key.as_ref(),
            &[bump]
        ];

        // Create the transfer instruction
        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: self.vault_token_account.to_account_info(),
                    mint: self.jito_sol_mint.to_account_info(),
                    to: self.reward_pool_token_account.to_account_info(),
                    authority: self.jito_manager.to_account_info(),
                },
                &[authority_seeds],
            ),
            vault_balance,
            self.jito_sol_mint.decimals,
        )?;

        // Reload accounts after CPI
        self.vault_token_account.reload()?;
        self.reward_pool_token_account.reload()?;
        self.reward_pool.reload()?;

        // Verify the transfer was successful and update internal balance
        if self.vault_token_account.amount != 0 {
            return Err(error!(ErrorCode::TransferFailed));
        }
        // vault_token_balance is basically the total amount of jitoSOL in the reward pool
        // self.reward_pool.vault_token_balance = self.reward_pool.vault_token_balance.checked_add(transfer_amount).ok_or(error!(ErrorCode::ArithmeticOverflow))?;
        // The above line is redundant as vault_token_balance is set to reward_pool_token_account.amount after transfer which is effectively the new total.
        // However, the total_undistributed_rewards should be incremented by the transfer_amount.
        self.reward_pool.vault_token_balance = self.reward_pool_token_account.amount;


        // Total undistributed contains the sum of all rewards that have come into the pool but not yet been "assigned" or "claimed" by users/NGOs.
        // It is incremented by the full transfer_amount.
        self.reward_pool.total_undistributed_rewards = self.reward_pool.total_undistributed_rewards
            .checked_add(transfer_amount)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
        
        // Update global yield accumulator if there is gSOL supply
        if self.global_state.total_gsol_supply > 0 {
            let reward_increment = transfer_amount
                .checked_mul(PRECISION_FACTOR)
                .ok_or(error!(ErrorCode::ArithmeticOverflow))?
                .checked_div(self.global_state.total_gsol_supply)
                .ok_or(error!(ErrorCode::DivisionByZero))?; // Ensure DivisionByZero is handled if total_gsol_supply is 0, though guarded by if

            self.global_state.acc_total_yield_per_gsol = self.global_state.acc_total_yield_per_gsol
                .checked_add(reward_increment)
                .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
        }

        // Update global state timestamps
        let now = Clock::get()?.unix_timestamp;
        // Update the last update time for the reward pool
        self.reward_pool.last_update_time = now;
        self.global_state.current_block_index = self.global_state.current_block_index
            .checked_add(1)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        Ok(())
    }
}

// impl ProcessRewards block is removed.

// impl<'info> UpdateRewardAccumulators<'info> {
//     pub fn update_accumulators(&mut self, reward_amount: u64) -> Result<()> {
//         // Only admin can update accumulators
//         if self.admin.key() != Pubkey::from_str(ADMIN_ADRESS).unwrap() {
//             return Err(error!(ErrorCode::Unauthorized));
//         }

//         // Calculate NGO and user portions
//         let ngo_portion = (reward_amount * self.global_state.weighted_donation_rate)
//             .checked_div(PRECISION_FACTOR)
//             .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

//         let user_portion = reward_amount
//             .checked_sub(ngo_portion)
//             .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

//         // Update global accumulators if there are staked tokens
//         if self.global_state.total_gsol_supply > 0 {
//             self.global_state.acc_reward_per_share = self.global_state.acc_reward_per_share
//                 .checked_add((user_portion * PRECISION_FACTOR) / self.global_state.total_gsol_supply)
//                 .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

//             self.global_state.acc_ngo_donation_per_share = self.global_state.acc_ngo_donation_per_share
//                 .checked_add((ngo_portion * PRECISION_FACTOR) / self.global_state.total_gsol_supply)
//                 .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
//         }

//         // Update global state timestamps
//         let now = Clock::get()?.unix_timestamp;
//         self.global_state.last_update_time = now;
//         self.global_state.current_block_index = self.global_state.current_block_index
//             .checked_add(1)
//             .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

//         Ok(())
//     }
// }

impl<'info> ClaimNgoRewards<'info> {
    pub fn claim_ngo_rewards(&mut self) -> Result<()> {
        let ngo_account = &mut self.ngo_account;
        let reward_pool = &mut self.reward_pool;

        // Check if NGO has pending rewards
        if ngo_account.pending_claimable_donations == 0 {
            return Err(error!(ErrorCode::NoPendingRewards));
        }

        // Check if enough time has passed since last claim
        let now = Clock::get()?.unix_timestamp;
        if now <= ngo_account.last_claim_time {
            return Err(error!(ErrorCode::RewardAlreadyClaimed));
        }

        let transfer_amount = ngo_account.pending_claimable_donations;

        // Check if reward pool has enough balance (important!)
        if self.reward_pool_token_account.amount < transfer_amount {
            return Err(error!(ErrorCode::InsufficientRewardBalance));
        }
        
        // Prepare seeds for CPI
        let seeds = &[
            b"reward_pool_authority",
            self.jito_sol_mint.to_account_info().key.as_ref(),
            &[reward_pool.bump], 
        ];
        let signer_seeds = &[&seeds[..]];

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: self.reward_pool_token_account.to_account_info(),
                    mint: self.jito_sol_mint.to_account_info(),
                    to: self.ngo_token_account.to_account_info(),
                    authority: self.reward_pool_authority.to_account_info(),
                },
                signer_seeds,
            ),
            transfer_amount,
            self.jito_sol_mint.decimals,
        )?;

        // Update NGO account state
        ngo_account.pending_claimable_donations = 0;
        ngo_account.last_claim_time = now;
        ngo_account.total_donations_received = ngo_account.total_donations_received
            .checked_add(transfer_amount)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
        // ngo_account.last_ngo_checkpoint is removed.

        // Update reward pool state
        reward_pool.reload()?; // Reload to get latest state if CPI changed it (though transfer_checked doesn't modify reward_pool directly)
        reward_pool.total_undistributed_rewards = reward_pool.total_undistributed_rewards
            .checked_sub(transfer_amount)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        Ok(())
    }
}

// impl DistributeNgoRewards is removed.


impl<'info> ClaimStakerRewards<'info> {
    pub fn claim_staker_rewards(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let user_account = &mut self.user_account;
        let global_state = &self.global_state;
        let user_ngo_account = &mut self.user_ngo_account; // Made mutable for the option

        // Optional: Keep staking duration check as a business rule for eligibility
        let staking_duration = current_time
            .checked_sub(user_account.stake_time)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        if staking_duration < 86400 { // 86400 seconds = 1 day
            return Err(error!(ErrorCode::StakingPeriodTooShort));
        }

        // Prepare ngo_account_option for the helper function
        let ngo_opt: Option<&mut Account<NGOAccount>> = if user_account.ngo_address != Pubkey::default() {
            // Ensure the provided user_ngo_account is the one selected by the user.
            if user_ngo_account.key() != user_account.ngo_address {
                return err!(ErrorCode::InvalidNgoAccountForClaim);
            }
            Some(user_ngo_account)
        } else {
            None
        };

        // Call the helper function to update pending rewards and credit NGO
        update_user_rewards_and_credit_ngo(
            user_account,
            global_state,
            ngo_opt,
        )?;

        let amount_to_claim = user_account.pending_rewards;

        if amount_to_claim == 0 {
            return Err(error!(ErrorCode::NoPendingRewards));
        }

        // Check if reward pool has enough balance
        if self.reward_pool_token_account.amount < amount_to_claim {
            return Err(error!(ErrorCode::InsufficientRewardBalance));
        }
        
        // Transfer rewards from reward pool to user
        let seeds = &[
            b"reward_pool_authority",
            self.jito_sol_mint.to_account_info().key.as_ref(),
            &[self.reward_pool.bump], // Assuming reward_pool struct has bump field for its authority
        ];
        let signer_seeds = &[&seeds[..]];

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: self.reward_pool_token_account.to_account_info(),
                    mint: self.jito_sol_mint.to_account_info(),
                    to: self.user_jitosol_ata.to_account_info(),
                    authority: self.reward_pool_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount_to_claim,
            self.jito_sol_mint.decimals,
        )?;

        // Update user account state
        user_account.total_claimed = user_account.total_claimed
            .checked_add(amount_to_claim)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        user_account.pending_rewards = 0;
        user_account.last_claim_time = current_time;
        user_account.last_claim_block = global_state.current_block_index;
        
        // Reload reward pool account to reflect transferred amount
        self.reward_pool.reload()?;
        self.reward_pool.total_undistributed_rewards = self.reward_pool.total_undistributed_rewards
            .checked_sub(amount_to_claim)
            .ok_or(ErrorCode::ArithmeticOverflow)?;


        Ok(())
    }
}