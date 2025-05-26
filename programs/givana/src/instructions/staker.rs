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
pub fn deposit(&mut self, amount: u64, ngo_address_param: Option<Pubkey>) -> Result<()> {
    let user_account = &mut self.user_account;
    let global_state = &self.global_state;
    // self.ngo_account is loaded based on the input `ngo_address_param` from the instruction arguments

    if user_account.is_initialized && user_account.gsol_balance > 0 {
        let mut ngo_ref_option = None;
        if user_account.ngo_address != Pubkey::default() {
            // User has an existing NGO.
            // self.ngo_account is loaded based on ngo_address_param.
            // It must match the user's current NGO for rewards to be settled for that NGO.
            if self.ngo_account.key() == user_account.ngo_address {
                ngo_ref_option = Some(&mut self.ngo_account);
            } else {
                msg!("User's current NGO {} does not match the provided ngo_address_param (or param was None). NGO rewards for current NGO will not be updated in this transaction.", user_account.ngo_address);
            }
        }
        // If user_account.ngo_address is Pubkey::default(), no NGO to pass, ngo_ref_option remains None.
        
        update_user_rewards_and_credit_ngo(
            user_account,
            global_state,
            ngo_ref_option,
        )?;
    }
   
   let current_time = Clock::get()?.unix_timestamp;
   
   if !user_account.is_initialized {
       user_account.set_inner(UserAccount {
        authority: self.authority.key(),
        gsol_balance: 0, // gSOL balance is updated by mint_gsol_tokens instruction
        donation_rate: 0, // Default, can be set by set_donation_rate_and_address
        ngo_address: ngo_address_param.unwrap_or(Pubkey::default()), // Set if provided with first deposit
        stake_time: current_time, // Set when user account is first initialized
        last_total_yield_checkpoint: global_state.acc_total_yield_per_gsol, // Initialize checkpoint
        last_claim_time: 0,
        last_claim_block: 0,
        total_claimed: 0,
        is_initialized: true,
        withdraw_requested: false,
        withdraw_request_time: 0,
        withdraw_amount: 0,
        pending_rewards: 0,
        // Obsolete fields (last_reward_checkpoint, last_ngo_donation_checkpoint, pending_ngo_donation) are removed
       });
   }
   // For existing users, stake_time is not reset here.
   // gsol_balance is managed by mint_gsol_tokens / burn_gsol_tokens (latter not in this file).

   // Handle NGO address logic for the deposit
   if ngo_address_param.is_some() {
       let new_ngo_addr_val = ngo_address_param.unwrap();
       // self.ngo_account is loaded based on new_ngo_addr_val.
       // Ensure it's active and the key matches.
       require!(self.ngo_account.is_active, ErrorCode::NgoNotActive);
       require!(self.ngo_account.key() == new_ngo_addr_val, ErrorCode::InvalidNgoAuthority); // Sanity check

       if user_account.ngo_address == Pubkey::default() {
           // User is setting NGO for the first time along with deposit.
           user_account.ngo_address = new_ngo_addr_val;
       } else {
           // User already has an NGO. It must match the one provided in deposit.
           // To change NGO, they must use set_donation_rate_and_address.
           require!(
               user_account.ngo_address == new_ngo_addr_val,
               ErrorCode::NgoAlreadySet // Or a more specific error like "CannotChangeNgoViaDeposit"
           );
       }
   }
   // If ngo_address_param is None, user_account.ngo_address remains unchanged.

   // Transfer JitoSOL from user to vault
   let cpi_accounts_transfer = TransferChecked {
    from: self.staker_jito_sol_ata.to_account_info(),
    to: self.vault.to_account_info(),
    mint: self.jito_mint.to_account_info(),
    authority: self.authority.to_account_info(),
   };
   let cpi_program_transfer = self.token_program.to_account_info();
   let cpi_ctx_transfer = CpiContext::new(cpi_program_transfer, cpi_accounts_transfer);
   transfer_checked(cpi_ctx_transfer, amount, self.jito_mint.decimals)?;
   
   // Update global state for total JitoSOL deposited
   self.global_state.total_jitosol_deposited = self.global_state.total_jitosol_deposited
    .checked_add(amount)
    .ok_or(ErrorCode::ArithmeticOverflow)?;
   
   // Note: The actual minting of gSOL (which increases user_account.gsol_balance) 
   // and nSOL happens via separate instructions (mint_gsol_tokens, mint_nsol_tokens),
   // presumably called by the client after this deposit instruction.
   // The update_weighted_donation_rate function is removed as it's obsolete.
   
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
pub fn set_donation_rate_and_address(&mut self, new_donation_rate: u16, new_ngo_address: Pubkey) -> Result<()> {
    let user_account = &mut self.user_account;
    let global_state = &self.global_state;
    // self.ngo_account is loaded based on the `new_ngo_address` parameter.

    if new_donation_rate > 10000 {
        return Err(error!(ErrorCode::InvalidDonationRate));
    }

    // Ensure the new NGO account is active if a new NGO address is being set.
    // self.ngo_account is loaded based on new_ngo_address. So, if new_ngo_address is default(),
    // self.ngo_account might be for the default pubkey which might not be 'active'.
    // This check should only apply if new_ngo_address is not Pubkey::default().
    if new_ngo_address != Pubkey::default() {
        require!(self.ngo_account.is_active, ErrorCode::NgoNotActive);
        // Also ensure that the loaded self.ngo_account corresponds to new_ngo_address.
        // This is implicitly handled by how seeds are defined for self.ngo_account if new_ngo_address is used in seeds.
        // However, a direct key check is good for safety if applicable.
        // For this structure: seeds = [b"ngo", ngo_address.unwrap_or(Pubkey::default()).as_ref()]
        // if new_ngo_address is the one from instruction, this should hold.
        require!(self.ngo_account.key() == new_ngo_address, ErrorCode::InvalidNgoAuthority);
    }


    // Settle rewards with the current/old NGO before changing settings.
    let mut called_update_rewards = false;
    if user_account.is_initialized && user_account.gsol_balance > 0 {
        let mut old_ngo_ref_mut_option = None;

        if user_account.ngo_address != Pubkey::default() {
            // User has an existing NGO.
            if user_account.ngo_address == new_ngo_address {
                // The new NGO is the same as the old one (user is likely just changing the rate).
                // We can use self.ngo_account because it's loaded for new_ngo_address which is same as old.
                old_ngo_ref_mut_option = Some(&mut self.ngo_account);
            } else {
                // User is changing from an old NGO to a new NGO.
                // This instruction, as defined, loads self.ngo_account based on new_ngo_address.
                // It does not have the old NGO's account if it's different.
                return Err(error!(ErrorCode::ChangingNgoNotSupportedDirectly));
            }
        }
        // If user_account.ngo_address was Pubkey::default(), then no old NGO to settle with;
        // old_ngo_ref_mut_option remains None, which is correct.

        update_user_rewards_and_credit_ngo(
            user_account,
            global_state,
            old_ngo_ref_mut_option,
        )?;
        called_update_rewards = true;
    }

    // Update user account with new settings
    user_account.donation_rate = new_donation_rate;
    user_account.ngo_address = new_ngo_address;
    
    // As per original logic, stake_time is updated. This might reset staking duration calculations depending on its usage.
    user_account.stake_time = Clock::get()?.unix_timestamp;

    // If update_user_rewards_and_credit_ngo was not called (e.g., gsol_balance is 0),
    // ensure the checkpoint is updated to the current global accumulator value.
    if !called_update_rewards { 
        user_account.last_total_yield_checkpoint = global_state.acc_total_yield_per_gsol;
    }
    // If update_user_rewards_and_credit_ngo was called, it already updated the checkpoint.

    Ok(())
}

// Obsolete update_weighted_donation_rate and the incorrect calculate_gsol_token_amount removed.
// The correct calculate_gsol_token_amount method remains.
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

