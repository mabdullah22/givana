use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{
        Mint, TokenAccount, TokenInterface, transfer_checked, 
        TransferChecked, burn, Burn
    },
    associated_token::AssociatedToken,
};

use crate::state::{UserAccount, GlobalState, NGOAccount, RewardPool};
use crate::error::ErrorCode;
use crate::constants::{ADMIN_ADRESS, WITHDRAW_EPOCH_DURATION, PRECISION_FACTOR};

use std::str::FromStr;

// Step 1: Initiate Withdraw
#[derive(Accounts)]
pub struct InitiateWithdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().to_bytes().as_ref()],
        bump,
        constraint = !user_account.withdraw_requested @ ErrorCode::WithdrawAlreadyRequested
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
        seeds = [b"ngo", user_account.ngo_address.as_ref()],
        bump,
        constraint = ngo_account.is_active @ ErrorCode::NgoNotActive
    )]
    pub ngo_account: Account<'info, NGOAccount>,

    #[account(
        mut,
        seeds = [b"reward_pool_state_v2"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    #[account(
        mut,
        address = global_state.output_token_mint
    )]
    pub gsol_mint: InterfaceAccount<'info, Mint>,

    #[account(
       mut,
        
        associated_token::mint = gsol_mint,
        associated_token::authority = user
    )]
    pub user_gsol_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: This is the protocol vault authority PDA
    #[account(
        mut,
        seeds = [b"protocol_vault_authority",global_state.input_token_mint.key().to_bytes().as_ref(), gsol_mint.key().to_bytes().as_ref()],
        bump
    )]
    pub protocol_vault_authority: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = gsol_mint,
        associated_token::authority = protocol_vault_authority
    )]
    pub protocol_gsol_ata: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// Step 2: Burn Withdraw Tokens
#[derive(Accounts)]
pub struct BurnWithdrawTokens<'info> {
    #[account(mut,
        constraint = admin.key() == Pubkey::from_str(ADMIN_ADRESS).unwrap() @ ErrorCode::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user_account.authority.as_ref()],
        bump,
        constraint = user_account.withdraw_requested @ ErrorCode::NoWithdrawRequested,
        // constraint = Clock::get()?.unix_timestamp >= user_account.withdraw_request_time + WITHDRAW_EPOCH_DURATION @ ErrorCode::EpochDurationNotCompleted
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
        address = global_state.output_token_mint
    )]
    pub gsol_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: This is the protocol vault authority PDA
    #[account(
        mut,
        seeds = [b"protocol_vault_authority",global_state.input_token_mint.key().to_bytes().as_ref(), gsol_mint.key().to_bytes().as_ref()],
        bump
    )]
    pub protocol_vault_authority: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = gsol_mint,
        associated_token::authority = protocol_vault_authority
    )]
    pub protocol_gsol_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

// Step 3: Claim Withdraw
#[derive(Accounts)]
pub struct ClaimWithdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().to_bytes().as_ref()],
        bump,
        constraint = user_account.withdraw_requested @ ErrorCode::NoWithdrawRequested,
        constraint = user_account.withdraw_amount > 0 @ ErrorCode::NoWithdrawRequested
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

    #[account(
        mut,
        seeds = [b"vault",global_state.input_token_mint.key().to_bytes().as_ref()],
        bump,
        token::mint = global_state.input_token_mint
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: This is the protocol vault authority PDA
    #[account(
        mut,
        seeds = [b"protocol_vault_authority",global_state.input_token_mint.key().to_bytes().as_ref(), 
        global_state.output_token_mint.key().to_bytes().as_ref()],
        bump
    )]
    pub protocol_vault_authority: AccountInfo<'info>,



    #[account(
        mut,
        seeds = [b"reward_pool",global_state.input_token_mint.key().to_bytes().as_ref()],
        bump
    )]
    pub reward_pool_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = vault.mint,
        associated_token::authority = user
    )]
    pub user_jitosol_ata: InterfaceAccount<'info, TokenAccount>,
    pub jito_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> InitiateWithdraw<'info> {
    pub fn initiate_withdraw(&mut self, withdraw_amount: u64) -> Result<()> {
        // Validate withdrawal amount
        if withdraw_amount == 0 {
            return Err(error!(ErrorCode::InvalidWithdrawAmount));
        }
       
        
        if withdraw_amount > self.user_account.gsol_balance {
            return Err(error!(ErrorCode::InsufficientBalance));
        }

        // Calculate withdrawal ratio
        let withdrawal_ratio = (withdraw_amount as u128)
            .checked_mul(PRECISION_FACTOR as u128)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
            .checked_div(self.user_account.gsol_balance as u128)
            .ok_or(error!(ErrorCode::DivisionByZero))?;

        // Calculate pending rewards
        let pending_rewards = self.calculate_pending_rewards(withdrawal_ratio)?;
        
        // Calculate NGO donation
        let pending_ngo_donation = self.calculate_ngo_donation(withdrawal_ratio)?;

        // Transfer gSOL from user to protocol
        let transfer_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.user_gsol_ata.to_account_info(),
                mint: self.gsol_mint.to_account_info(),
                to: self.protocol_gsol_ata.to_account_info(),
                authority: self.user.to_account_info(),
            },
        );
        transfer_checked(transfer_ctx, withdraw_amount, self.gsol_mint.decimals)?;

        // Update user account
        self.user_account.withdraw_requested = true;
        self.user_account.withdraw_request_time = Clock::get()?.unix_timestamp;
        self.user_account.withdraw_amount = withdraw_amount;
        self.user_account.pending_rewards = pending_rewards;
        self.user_account.pending_ngo_donation = pending_ngo_donation;

        // Update NGO stats if full withdrawal
        if withdraw_amount == self.user_account.gsol_balance {
            self.ngo_account.total_donations_received = self.ngo_account.total_donations_received
                .checked_add(pending_ngo_donation)
                .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
        }

        Ok(())
    }

    fn calculate_pending_rewards(&self, withdrawal_ratio: u128) -> Result<u64> {
        let reward_per_share = self.global_state.acc_reward_per_share
            .checked_sub(self.user_account.last_reward_checkpoint)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        let pending_rewards = (reward_per_share as u128)
            .checked_mul(self.user_account.gsol_balance as u128)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
            .checked_mul(withdrawal_ratio)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
            .checked_div(PRECISION_FACTOR as u128)
            .ok_or(error!(ErrorCode::DivisionByZero))?
            .checked_div(PRECISION_FACTOR as u128)
            .ok_or(error!(ErrorCode::DivisionByZero))?;

        Ok(pending_rewards as u64)
    }

    fn calculate_ngo_donation(&self, withdrawal_ratio: u128) -> Result<u64> {
        let donation_per_share = self.global_state.acc_ngo_donation_per_share
            .checked_sub(self.user_account.last_ngo_donation_checkpoint)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        let pending_donation = (donation_per_share as u128)
            .checked_mul(self.user_account.gsol_balance as u128)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
            .checked_mul(withdrawal_ratio)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
            .checked_div(PRECISION_FACTOR as u128)
            .ok_or(error!(ErrorCode::DivisionByZero))?
            .checked_div(PRECISION_FACTOR as u128)
            .ok_or(error!(ErrorCode::DivisionByZero))?;

        Ok(pending_donation as u64)
    }
}

impl<'info> BurnWithdrawTokens<'info> {
    pub fn burn_tokens(&mut self) -> Result<()> {
        let withdraw_amount = self.user_account.withdraw_amount;

        // Burn gSOL tokens
        let burn_gsol_accounts = Burn {
            mint: self.gsol_mint.to_account_info(),
            from: self.protocol_gsol_ata.to_account_info(),
            authority: self.protocol_vault_authority.to_account_info(),
        };

        let input_token_mint = self.global_state.input_token_mint.key().to_bytes().clone();
        let gsol_mint = self.gsol_mint.key().to_bytes().clone();

        // let bump = self.global_state.protocol_vault_authority_bump;
        // let signer_seeds: &[&[&[u8]]] = &[&[
        //     b"protocol_vault_authority",
        //     input_token_mint,
        //     gsol_mint,
        //     &[bump]
        // ]];

        let bump = self.global_state.protocol_vault_authority_bump;
        let jito_mint_key = self.global_state.input_token_mint.key();
        let gsol_mint_key = self.gsol_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
        b"protocol_vault_authority", 
        jito_mint_key.as_ref(),
        gsol_mint_key.as_ref(),
        &[bump]  // Now &[bump] is &[u8] as required
    
        // Mint the same amount of nsol to the nsol_ata defined in the initialize instruction
        
        
    ]];

        let burn_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            burn_gsol_accounts,
            signer_seeds,
        );
        burn(burn_ctx, withdraw_amount)?;

        // Update global state
        self.global_state.total_gsol_supply = self.global_state.total_gsol_supply
            .checked_sub(withdraw_amount)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        Ok(())
    }
}

impl<'info> ClaimWithdraw<'info> {
    pub fn claim_withdraw(&mut self) -> Result<()> {
        let withdraw_amount = self.user_account.withdraw_amount;
        let pending_ngo_donation = self.user_account.pending_ngo_donation;

        // Calculate amount to transfer (stake - donation)
        let total_amount = withdraw_amount
            .checked_sub(pending_ngo_donation)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Verify vault has sufficient balance
        if self.vault.amount < total_amount {
            return Err(error!(ErrorCode::InsufficientVaultBalance));
        }

        // Transfer jitoSOL from vault to user
        let bump = self.global_state.protocol_vault_authority_bump;
        let input_token_mint = self.global_state.input_token_mint.key();
        let output_token_mint = self.global_state.output_token_mint.key();
        
        // Create signer seeds for protocol vault authority
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"protocol_vault_authority",
            input_token_mint.as_ref(),
            output_token_mint.as_ref(),
            &[bump]
        ]];

        // Transfer jitoSOL from vault to user with PDA signing
        let transfer_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.vault.to_account_info(),
                to: self.user_jitosol_ata.to_account_info(),
                authority: self.protocol_vault_authority.to_account_info(),
                mint: self.jito_mint.to_account_info(),
            },
            signer_seeds,
        );
        transfer_checked(transfer_ctx, total_amount, self.jito_mint.decimals)?;

        // Reset user account withdrawal state
        self.user_account.withdraw_requested = false;
        self.user_account.withdraw_amount = 0;
        self.user_account.pending_ngo_donation = 0;
        self.user_account.gsol_balance = self.user_account.gsol_balance
            .checked_sub(withdraw_amount)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Update global state
        self.global_state.total_jitosol_deposited = self.global_state.total_jitosol_deposited
            .checked_sub(total_amount)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        Ok(())
    }
}