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

#[derive(Accounts)]
pub struct ProcessRewards<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: This is the user's pubkey
    pub user: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

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

#[derive(Accounts)]
pub struct DistributeNgoRewards<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"reward_pool_state_v2"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    #[account(
        mut,
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    /// The NGO account to distribute rewards to
    #[account(
        mut,
        seeds = [b"ngo", ngo_account.authority.as_ref()],
        bump,
        constraint = ngo_account.is_active @ ErrorCode::NgoNotActive
    )]
    pub ngo_account: Account<'info, NGOAccount>,

    pub system_program: Program<'info, System>,
}
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
        self.reward_pool.vault_token_balance = self.reward_pool.vault_token_balance.checked_add(transfer_amount).ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Calculate NGO and user portions using the transfer amount
        let ngo_portion = (transfer_amount * self.global_state.weighted_donation_rate)
            .checked_div(PRECISION_FACTOR)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        let user_portion = transfer_amount
            .checked_sub(ngo_portion)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Update reward pool with NGO portion
        // !TODO: I need to check if there is some kind of double counting here??
        self.reward_pool.ngo_pending_rewards = self.reward_pool.ngo_pending_rewards
            .checked_add(ngo_portion)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Total undistributed contains both the user and ngo portions
        self.reward_pool.total_undistributed_rewards = self.reward_pool.total_undistributed_rewards.checked_add(transfer_amount).ok_or(error!(ErrorCode::ArithmeticOverflow))?;
        
        // Update global accumulators if there are staked tokens
        if self.global_state.total_gsol_supply > 0 {
            self.global_state.acc_reward_per_share = self.global_state.acc_reward_per_share
                .checked_add((user_portion * PRECISION_FACTOR) / self.global_state.total_gsol_supply)
                .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

            self.global_state.acc_ngo_donation_per_share = self.global_state.acc_ngo_donation_per_share
                .checked_add((ngo_portion * PRECISION_FACTOR) / self.global_state.total_gsol_supply)
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

impl<'info> ProcessRewards<'info> {
    pub fn process_rewards(&mut self) -> Result<()> {
        // Validate that the user's last claim was from a previous block
        if self.user_account.last_claim_time >= Clock::get()?.unix_timestamp {
            return Err(error!(ErrorCode::RewardAlreadyClaimed));
        }

        // Calculate reward using accumulator difference
        let reward_delta = self.global_state.acc_reward_per_share
            .checked_sub(self.user_account.last_reward_checkpoint)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        let gross_pending = (self.user_account.gsol_balance * reward_delta)
            .checked_div(PRECISION_FACTOR)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Calculate NGO donation portion
        let ngo_donation_delta = self.global_state.acc_ngo_donation_per_share
            .checked_sub(self.user_account.last_ngo_donation_checkpoint)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        let ngo_donation = (self.user_account.gsol_balance * ngo_donation_delta)
            .checked_div(PRECISION_FACTOR)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Update user's checkpoints
        self.user_account.last_reward_checkpoint = self.global_state.acc_reward_per_share;
        self.user_account.last_ngo_donation_checkpoint = self.global_state.acc_ngo_donation_per_share;
        self.user_account.last_claim_time = Clock::get()?.unix_timestamp;
        self.user_account.last_claim_block = self.global_state.current_block_index;

        // Update total claimed amount
        self.user_account.total_claimed = self.user_account.total_claimed
            .checked_add(gross_pending)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        Ok(())
    }
}

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
        // Check if NGO has pending rewards
        if self.ngo_account.pending_rewards == 0 {
            return Err(error!(ErrorCode::NoPendingRewards));
        }

        // Check if enough time has passed since last claim
        let now = Clock::get()?.unix_timestamp;
        if now <= self.ngo_account.last_claim_time {
            return Err(error!(ErrorCode::RewardAlreadyClaimed));
        }

        // Transfer rewards to NGO
        let transfer_amount = self.ngo_account.pending_rewards;
        
        anchor_spl::token_interface::transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: self.reward_pool_token_account.to_account_info(),
                    mint: self.jito_sol_mint.to_account_info(),
                    to: self.ngo_token_account.to_account_info(),
                    authority: self.reward_pool_authority.to_account_info(),
                },
            ),
            transfer_amount,
            self.jito_sol_mint.decimals,
        )?;

        // Update NGO account state
        self.ngo_account.pending_rewards = 0;
        self.ngo_account.last_claim_time = now;
        self.ngo_account.total_donations_received = self.ngo_account.total_donations_received
            .checked_add(transfer_amount)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Update checkpoint to current accumulator value
        self.ngo_account.last_ngo_checkpoint = self.global_state.acc_ngo_donation_per_share;

        Ok(())
    }
}

impl<'info> DistributeNgoRewards<'info> {
    pub fn distribute_ngo_rewards(&mut self) -> Result<()> {
        // Only admin can distribute rewards
        if self.admin.key() != Pubkey::from_str(ADMIN_ADRESS).unwrap() {
            return Err(error!(ErrorCode::Unauthorized));
        }

        // Calculate NGO's weight (W_n) - sum of (G_i × D_i) for all users donating to this NGO
        let ngo_weight = self.ngo_account.total_users_donating
            .checked_mul(self.global_state.weighted_donation_rate)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Calculate pending donation using checkpoint system
        // Pending_donation_n = W_n × (A_N - Checkpoint_N,n)
        let ngo_share = if ngo_weight > 0 {
            let acc_difference = self.global_state.acc_ngo_donation_per_share
                .checked_sub(self.ngo_account.last_ngo_checkpoint)
                .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

            ngo_weight
                .checked_mul(acc_difference)
                .ok_or(error!(ErrorCode::ArithmeticOverflow))?
                .checked_div(PRECISION_FACTOR)
                .ok_or(error!(ErrorCode::DivisionByZero))?
        } else {
            0
        };

        // Update NGO's pending rewards
        self.ngo_account.pending_rewards = self.ngo_account.pending_rewards
            .checked_add(ngo_share)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Update NGO's checkpoint
        self.ngo_account.last_ngo_checkpoint = self.global_state.acc_ngo_donation_per_share;

        Ok(())
    }
}


impl<'info> ClaimStakerRewards<'info> {
    pub fn claim_staker_rewards(&mut self) -> Result<()> {

        let current_time = Clock::get()?.unix_timestamp;

        let staking_duration = current_time.checked_sub(self.user_account.stake_time).ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        if staking_duration < 86400 {
            return Err(error!(ErrorCode::StakingPeriodTooShort));
        }
        let time_weighted_balance = (self.user_account.gsol_balance as u128)
            .checked_mul(staking_duration as u128)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        let gross_pending = (time_weighted_balance * self.global_state.acc_reward_per_share as u128)
            .checked_div(PRECISION_FACTOR as u128)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

     


        Ok(())
    }
}