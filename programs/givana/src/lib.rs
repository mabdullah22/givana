#![allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use state::*;

// Import specific structs from instructions
use instructions::*;
// use instructions::ngo::RegisterNgo;
// use instructions::staker::Deposit;

declare_id!("HZmz2fQD7q7GK9A33k5jMu1VSHTEtDNu5Mu9ji5arh3m");

#[program]
pub mod givana {
    use super::*;

    pub fn initialize(ctx: Context<GlobalStateInitialize>) -> Result<()> {
        ctx.accounts.initialize_global_state(ctx.bumps)?;
        Ok(())
    }

    pub fn initialize_reward_pool(ctx: Context<InitializeRewardPool>) -> Result<()> {
       
        ctx.accounts.initialize_reward_pool(ctx.bumps)?;
        Ok(())
    }

    pub fn register_ngo(ctx: Context<RegisterNgo>) -> Result<()> {
        instructions::ngo::RegisterNgo::register_ngo(ctx)
    }

    pub fn activate_ngo(ctx: Context<ActivateNgo>) -> Result<()> {
        instructions::ngo::ActivateNgo::activate_ngo(ctx)
    }



    pub fn deposit(ctx: Context<Deposit>, ngo_address: Option<Pubkey>, amount: u64, donation_rate: u16) -> Result<()> {
        // Deposit the amount in Givana Vault
        ctx.accounts.deposit(amount,ngo_address)?;
        
        // Mint the Gsol tokens to the user
        msg!("line 47");
        let amount_to_mint = ctx.accounts.calculate_gsol_token_amount(amount)?;      
        ctx.accounts.mint_gsol_tokens(amount_to_mint)?;
        
        // Set the donation rate and NGO address if provided
        msg!("line 51");
        if let Some(ngo_addr) = ngo_address {
            ctx.accounts.set_donation_rate_and_address(donation_rate, ngo_addr)?;
            ctx.accounts.ngo_account.total_users_donating += 1;
            ctx.accounts.user_account.ngo_address = ngo_addr;
        }
        
        // Update the weighted donation rate
        msg!("line 53");
        ctx.accounts.update_weighted_donation_rate(amount)?;
        
        // Mint the nSOL tokens
        msg!("line 55");
        ctx.accounts.mint_nsol_tokens(amount)?;
    
        
        Ok(())
    }


    pub fn initialize_jito_vault(ctx: Context<InitializeJitoVault>,deposit_fee_bps: u16,withdrawal_fee_bps: u16,reward_fee_bps: u16,decimals: u8,initialize_token_amount: u64,name: String,symbol: String,uri: String) -> Result<()> {
       
       ctx.accounts.initialize_jito_vault_config(deposit_fee_bps,withdrawal_fee_bps,reward_fee_bps,decimals,initialize_token_amount,name,symbol,uri)?;
       Ok(())
    }



    pub fn admin_deposit_to_jito_vault(ctx: Context<DepositToJitoVault>,amount_to_deposit: u64,min_amount_out: u64) -> Result<()> {
       
        ctx.accounts.deposit_to_jito_vault(amount_to_deposit,min_amount_out)?;

        Ok(())
    }

    pub fn user_initiate_withdraw(ctx: Context<InitiateWithdraw>,withdraw_amount: u64) -> Result<()> {
        
        ctx.accounts.initiate_withdraw(withdraw_amount)?;
        Ok(())
    }


    pub fn admin_burn_withdraw_tokens(ctx: Context<BurnWithdrawTokens>) -> Result<()> {
        ctx.accounts.burn_tokens()?;
        Ok(())
    }

    pub fn user_claim_withdraw(ctx: Context<ClaimWithdraw>) -> Result<()> {
        ctx.accounts.claim_withdraw()?;
        Ok(())
    }

    pub fn admin_distribute_ngo_rewards(ctx: Context<DistributeNgoRewards>) -> Result<()> {
        ctx.accounts.distribute_ngo_rewards()?;
        Ok(())
    }

    pub fn user_claim_staker_rewards(ctx: Context<ClaimStakerRewards>) -> Result<()> {
        ctx.accounts.claim_staker_rewards()?;
        Ok(())
    }

    pub fn claim_ngo_rewards(ctx: Context<ClaimNgoRewards>) -> Result<()> {
        ctx.accounts.claim_ngo_rewards()?;
        Ok(())
    }

    pub fn admin_transfer_jito_vault_reward_to_reward_pool(ctx: Context<TransferJitoVaultRewardToRewardPool>) -> Result<()> {
        ctx.accounts.transfer_jito_vault_reward_to_reward_pool()?;
        Ok(())
    }

    pub fn process_rewards(ctx: Context<ProcessRewards>) -> Result<()> {
        ctx.accounts.process_rewards()?;
        Ok(())
    }



    


}
