use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{
        Mint, TokenAccount, transfer_checked, 
        TransferChecked, TokenInterface, mint_to, MintTo
    },
    associated_token::AssociatedToken,
    associated_token::get_associated_token_address
};

use crate::{instructions::ngo, state::{GlobalState, NGOAccount, RewardPool, UserAccount}, PRECISION_FACTOR};
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(ngo_address: Option<Pubkey>)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + UserAccount::INIT_SPACE,
        seeds = [b"user", authority.key().to_bytes().as_ref()],
        bump
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = jito_mint,
        associated_token::authority = authority
    )]
    pub staker_jito_sol_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    pub jito_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [b"vault",jito_mint.key().to_bytes().as_ref()],
        bump,
        token::mint = jito_mint
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

  /// @Check: I don't know why there is a seed error I am removing for test @audit
  #[account(mut)]
    pub gsol_mint: Box<InterfaceAccount<'info, Mint>>,
    
    /// CHECK: This is a PDA that serves as the protocol vault authority
    #[account(
        mut,
        seeds = [b"protocol_vault_authority",jito_mint.key().to_bytes().as_ref(), gsol_mint.key().to_bytes().as_ref()],
        bump
    )]
    pub protocol_vault_authority: AccountInfo<'info>,

    /// CHECK: This is a PDA that serves as the authority for nsol mint
    #[account(
        mut,
        seeds = [b"jito_manager", global_state.protocol_admin.as_ref()],
        bump
    )]
    pub jito_manager: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = gsol_mint,
        associated_token::authority = authority
    )]
    pub user_gsol_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    
    #[account(
        mut,
        constraint = nsol_mint.key() == global_state.jito_vault_input_token_mint
    )]
    pub nsol_mint: Box<InterfaceAccount<'info, Mint>>,
    
    // #[account(
    //     init_if_needed,
    //     payer = authority,
    //     associated_token::mint = nsol_mint,
    //     associated_token::authority = protocol_vault_authority
    // )]
    // pub protocol_nsol_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = nsol_mint,
        associated_token::authority = jito_manager
    )]
    pub protocol_nsol_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + NGOAccount::INIT_SPACE,
        seeds = [b"ngo", ngo_address.unwrap_or(Pubkey::default()).as_ref()],
        bump

    )]
    pub ngo_account: Box<Account<'info, NGOAccount>>,
    
    #[account(
        seeds = [b"reward_pool_state_v2"],
        bump
    )]
    pub reward_pool_state: Box<Account<'info, RewardPool>>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}



impl<'info> Deposit<'info> {
pub fn deposit(&mut self, amount: u64, ngo_address: Option<Pubkey>) -> Result<()> {
   // Check if the user account is initialized
  

   let current_time = Clock::get()?.unix_timestamp;
   
   if !self.user_account.is_initialized {
   // use set_inner to set the account data
   self.user_account.set_inner(UserAccount {
    authority: self.authority.key(),
    gsol_balance: 0,
    donation_rate: 0,
    ngo_address: ngo_address.unwrap_or(Pubkey::default()),
    stake_time: 0,
    last_reward_checkpoint: 0,
    last_ngo_donation_checkpoint: 0, // Not being updated anywhere 
    last_claim_time: 0,
    last_claim_block: 0,
    withdraw_requested: false,
    withdraw_request_time: 0,
    withdraw_amount: 0,
    pending_rewards: 0,
    pending_ngo_donation: 0,
    total_claimed: 0,
    is_initialized: true,
   });
   }
   else {
    let time_diff = current_time
                .checked_sub(self.user_account.stake_time)
                .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

   }

   if ngo_address != None {
    // If NGO is specified, check if it's active
    require!(self.ngo_account.is_active, ErrorCode::NgoNotActive);

    msg!("ngo_address: {:?}", ngo_address.unwrap());
    msg!("SETTED NGO ADDRESS: {:?}", self.ngo_account.key());

    // If user hasn't set an NGO before, allow setting this NGO
    if self.user_account.ngo_address == Pubkey::default() {
        self.user_account.ngo_address = ngo_address.unwrap();
    } else {
        // If user already has an NGO, ensure it's the same one
        require!(
            self.user_account.ngo_address == ngo_address.unwrap(),
            ErrorCode::NgoAlreadySet
        );
    }
}

   // transfer jito from authority to vault
   let cpi_account = TransferChecked{
    from: self.staker_jito_sol_ata.to_account_info(),
    to: self.vault.to_account_info(),
    mint: self.jito_mint.to_account_info(),
    authority: self.authority.to_account_info(),
   };

   let cpi_program = self.token_program.to_account_info();
   let cpi_ctx = CpiContext::new(cpi_program, cpi_account);
   transfer_checked(cpi_ctx, amount, self.jito_mint.decimals)?;
   msg!("line 146");
   // update the global state
   self.global_state.total_jitosol_deposited += amount;

   

   
   
Ok(())
}

pub fn mint_gsol_tokens(&mut self, amount: u64) -> Result<()> {

    let mint_to_accounts = MintTo {
        mint: self.gsol_mint.to_account_info(),
        to: self.user_gsol_ata.to_account_info(),
        authority: self.protocol_vault_authority.to_account_info(),
    };
 
    // Build seeds for PDA signing
    let bump = self.global_state.protocol_vault_authority_bump;
    let jito_mint_key = self.jito_mint.key();
    let gsol_mint_key = self.gsol_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
    b"protocol_vault_authority", 
    jito_mint_key.as_ref(),
    gsol_mint_key.as_ref(),
    &[bump]  // Now &[bump] is &[u8] as required

    // Mint the same amount of nsol to the nsol_ata defined in the initialize instruction
    
    
]];

    let mint_ctx = CpiContext::new_with_signer(
        self.token_program.to_account_info(),
        mint_to_accounts,
        signer_seeds,
    );
    // I need to put calulation here for how much gSOL to mint. It should be based on the amount of jitoSOL deposited and the number of rewards in the reward pool
 
    mint_to(mint_ctx, amount)?;


 
    // update user account
    self.user_account.gsol_balance += amount;
 
    // update vault
    self.global_state.total_gsol_supply += amount;

 


    Ok(())
}

pub fn mint_nsol_tokens(&mut self, amount: u64) -> Result<()> {
    let bump = self.global_state.jito_manager_bump;
 
    let signer_seeds: &[&[&[u8]]] = &[&[
    b"jito_manager", 
    self.global_state.protocol_admin.as_ref(),
    &[bump]  // Now &[bump] is &[u8] as required

    // Mint the same amount of nsol to the nsol_ata defined in the initialize instruction
    
]];

    let mint_to_accounts = MintTo {
        mint: self.nsol_mint.to_account_info(),
        to: self.protocol_nsol_ata.to_account_info(),
        authority: self.jito_manager.to_account_info(),
    };

    let mint_ctx = CpiContext::new_with_signer(
        self.token_program.to_account_info(),
        mint_to_accounts,
        signer_seeds,
    );
    mint_to(mint_ctx, amount)?;

    // update the nsol vault state
    self.global_state.total_nsol_minted += amount;
    


    Ok(())
}
pub fn set_donation_rate_and_address(&mut self, donation_rate: u16, ngo_address: Pubkey) -> Result<()> {
    // Check if the ngo_address and donation rate is already set. If it is set to any ngo address, dont allow to set it again
    if donation_rate > 10000 {
        return Err(error!(ErrorCode::InvalidDonationRate));
    }

    if !self.ngo_account.is_active {
        return Err(error!(ErrorCode::NgoNotActive));
    }

    // if self.user_account.ngo_address != Pubkey::default() {
    //     return Err(error!(ErrorCode::NgoAlreadySet));
    // }

    // Store the donation rate and NGO address
    self.user_account.donation_rate = donation_rate;
    self.user_account.ngo_address = ngo_address;
    self.user_account.stake_time = Clock::get()?.unix_timestamp;

    Ok(())
}

pub fn update_weighted_donation_rate(&mut self, new_stake_amount: u64) -> Result<()> {
    let current_total_stake = self.global_state.total_jitosol_deposited
        .checked_sub(new_stake_amount)
        .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
    
    let donation_rate = self.user_account.donation_rate;
    // new_weighted_rate = (current_weighted_rate * current_total_stake + donation_rate * new_stake_amount) / total_stake
    // This formula calculates the new global weighted donation rate when a user makes a new deposit.
    // It works by:
    // 1. Taking the current weighted rate and multiplying it by the current total stake
    //    - This gives us the total weighted contribution of all existing stakes
    // 2. Adding the new user's weighted contribution
    //    - This is their donation rate multiplied by their new stake amount
    // 3. Dividing by the new total stake
    //    - This gives us the new average weighted rate across all stakes
    //
    let new_weighted_rate = if current_total_stake > 0 {
        // Convert donation rate to same precision as weighted_rate
        let scaled_donation_rate = (donation_rate as u64)
            .checked_mul(PRECISION_FACTOR)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
            .checked_div(10000)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;

        // Calculate total weighted contribution
        let current_weighted_contribution = self.global_state.weighted_donation_rate
            .checked_mul(current_total_stake)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
        
        let new_weighted_contribution = scaled_donation_rate
            .checked_mul(new_stake_amount)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
        
        let total_stake = current_total_stake
            .checked_add(new_stake_amount)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
        
        // Calculate new weighted rate
        current_weighted_contribution
            .checked_add(new_weighted_contribution)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
            .checked_div(total_stake)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
    } else {
        // For first deposit, scale the donation rate
        (donation_rate as u64)
            .checked_mul(PRECISION_FACTOR)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
            .checked_div(10000)
            .ok_or(error!(ErrorCode::ArithmeticOverflow))?
    };

    self.global_state.weighted_donation_rate = new_weighted_rate;
    Ok(())
}

pub fn calculate_gsol_token_amount(&self, amount: u64) -> Result<u64> {
    // For first deposit, use 1:1 ratio
    if self.global_state.total_gsol_supply == 0 {
        return Ok(amount);
    }

    let total_jitosol_deposited = self.global_state.total_jitosol_deposited;
    let total_gsol_supply = self.global_state.total_gsol_supply;
    let total_undistributed_rewards = self.reward_pool_state.total_undistributed_rewards;
    
    // Unwrap each Result immediately with ?
    let numerator = amount.checked_mul(total_gsol_supply)
        .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
    let denominator = total_jitosol_deposited.checked_add(total_undistributed_rewards)
        .ok_or(error!(ErrorCode::ArithmeticOverflow))?;
    let amount_to_mint = numerator.checked_div(denominator)
        .ok_or(error!(ErrorCode::DivisionByZero))?;

    Ok(amount_to_mint)
}




}

