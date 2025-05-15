// use anchor_lang::prelude::*;
// // use anchor_spl::token_interface::{
// //     Mint, TokenAccount, TokenInterface,
// // };
// use anchor_spl::associated_token::AssociatedToken;
// use crate::state::GlobalState;
// use crate::state::RewardPool;
// use crate::constants::*;

// use anchor_spl::token_interface::Mint;
// use anchor_spl::token_interface::TokenAccount;
// use anchor_spl::token_2022::*;


// #[derive(Accounts)]
// pub struct Delegate<'info> {
//     /// Vault configuration account
//     #[account(
//         constraint = config.key() == JITO_CONFIG
//     )]
//     pub config: AccountInfo<'info>,
    
//     /// The vault account
//     #[account(mut)]
//     pub vault: AccountInfo<'info>,
    
//     /// VRT mint account
//     #[account(mut)]
//     pub vrt_mint: InterfaceAccount<'info, Mint>,
    
//     /// Depositor account (protocol)
//     #[account(mut)]
//     pub depositor: Signer<'info>,
    
//     /// Protocol's nSOL token account
//     #[account(mut)]
//     pub depositor_token_account: InterfaceAccount<'info, TokenAccount>,
    
//     /// Vault's token account to receive nSOL
//     #[account(mut)]
//     pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
    
//     /// Protocol's VRT token account
//     #[account(mut)]
//     pub depositor_vrt_token_account: InterfaceAccount<'info, TokenAccount>,
    
//     /// Vault's fee token account
//     #[account(mut)]
//     pub vault_fee_token_account: InterfaceAccount<'info, TokenAccount>,
    
//     pub token_program: Interface<'info, TokenInterface>,
// }

// impl<'info> Delegate<'info> {
//     pub fn delegate(&mut self, amount: u64) -> Result<()> {
//         Ok(())
//     }
// }

