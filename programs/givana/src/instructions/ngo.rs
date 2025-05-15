use anchor_lang::prelude::*;

use crate::state::NGOAccount;
use crate::state::GlobalState;

use anchor_spl::token_interface::TokenAccount;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct RegisterNgo<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + NGOAccount::INIT_SPACE,
        seeds = [b"ngo", authority.key().to_bytes().as_ref()],
        bump
    )]
    pub ngo_account: Account<'info, NGOAccount>,

    #[account(
        mut,
        seeds = [b"global-state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,

    

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ActivateNgo<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"ngo", ngo_address.key().to_bytes().as_ref()],
        bump = ngo_account.bump
    )]
    pub ngo_account: Account<'info, NGOAccount>,
    
    #[account(
        mut,
        seeds = [b"global-state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    ///CHECK: This is just an address
    pub ngo_address: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl <'info> RegisterNgo<'info> {
    pub fn register_ngo(ctx: Context<RegisterNgo>) -> Result<()> {
        let ngo_account = &mut ctx.accounts.ngo_account;
        ngo_account.set_inner(NGOAccount {
            authority: ctx.accounts.authority.key(),
            is_active: false,
            pending_rewards: 0,
            last_claim_time: 0,
            total_users_donating: 0,
            total_donations_received: 0,
            last_ngo_checkpoint: 0,
            bump: ctx.bumps.ngo_account,
        });
        Ok(())
    }

   

    

}

impl<'info> ActivateNgo<'info> {
    pub fn activate_ngo(ctx: Context<ActivateNgo>) -> Result<()> {
        assert!(ctx.accounts.authority.key() == ctx.accounts.global_state.protocol_admin);
        

        let ngo_account = &mut ctx.accounts.ngo_account;
        if(ngo_account.is_active == false){
            ngo_account.is_active = true;
        }
        else{
            return Err(ErrorCode::NgoAlreadyActive.into());
        }

        Ok(())
    }
}