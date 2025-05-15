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
    token_interface::{Mint as InterfaceMint, TokenAccount as InterfaceTokenAccount, TokenInterface,MintTo,mint_to as mint_interface},
    associated_token::AssociatedToken,
};

// use jito_vault_sdk::instruction::VaultInstruction::InitializeVault;
use jito_vault_sdk::sdk::{initialize_vault,mint_to,delegate_token_account,create_token_metadata};

use jito_vault_sdk::inline_mpl_token_metadata::pda::find_metadata_account;





// use jito_vault_client::{
//     instructions::{
//         InitializeVaultCpi,
//         InitializeVaultCpiAccounts,
//         InitializeVaultInstructionArgs,
//     },
// };

use crate::state::{GlobalState};
use crate::error::ErrorCode;

use crate::constants::{ADMIN_ADRESS,JITO_CONFIG};
use std::str::FromStr;
use crate::state::JitoVaultConfig;
pub static JITO_VAULT_PROGRAM_ID: Pubkey = pubkey!("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");

#[derive(Accounts)]

pub struct InitializeJitoVault<'info> {
    #[account(mut,
    constraint = admin.key() == Pubkey::from_str(ADMIN_ADRESS).unwrap()
    )]
    pub admin: Signer<'info>,

    #[account(mut,
    seeds = [b"global-state"],
    bump
    )]
    pub global_state: Account<'info, GlobalState>,
    /// CHECK: This is verified by the Jito vault program
    #[account(mut,
    constraint = config.key() == Pubkey::from_str(JITO_CONFIG).unwrap()
    )]
    pub config: AccountInfo<'info>,
    
    /// CHECK: This is verified by the Jito vault program
    #[account(mut,
        seeds = [b"vault", jito_manager.key().as_ref()],
        seeds::program = JITO_VAULT_PROGRAM_ID,
        bump
    )]
    pub vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub vrt_mint: Signer<'info>,
    
   
    // #[account(
    //     constraint = st_mint.key() == global_state.jito_vault_input_token_mint
    // )]
     /// CHECK: This is the nsol mint
    pub st_mint: AccountInfo<'info>,

    #[account(
        mut,
        constraint = st_mint.key() == global_state.jito_vault_input_token_mint
    )]
    pub nsol_mint: InterfaceAccount<'info, InterfaceMint>,
    
    /// CHECK: This is verified by the Jito vault program
    #[account(mut)]
    pub admin_st_token_account: AccountInfo<'info>,
    
    /// CHECK: This is verified by the Jito vault program
    #[account(mut)]
    pub vault_st_token_account: AccountInfo<'info>,
    
    /// CHECK: This is verified by the Jito vault program
    #[account(
       seeds = [b"burn_vault", jito_manager.key().as_ref()],
       seeds::program = JITO_VAULT_PROGRAM_ID,
       bump
   )]
    pub burn_vault: AccountInfo<'info>,
    
    /// CHECK: This is verified by the Jito vault program
    #[account(mut)]
    pub burn_vault_vrt_token_account: AccountInfo<'info>,
    
    /// CHECK: This is the PDA that will sign for the vault initialization. No checks needed as we verify the seeds and bump.
    #[account(
        mut,
        seeds = [b"jito_manager", admin.key().as_ref()],
        bump
    )]
    pub jito_manager: AccountInfo<'info>,
    
    /// CHECK: This is the PDA that will sign for the vault initialization. No checks needed as we verify the seeds and bump.
    #[account(
        seeds = [b"jito_manager", admin.key().as_ref()],
        bump
    )]
    pub jito_manager_for_jito: AccountInfo<'info>,
    
    /// CHECK: This is the Jito vault program that we're calling into. No checks needed as we verify the program ID.
    #[account(address = JITO_VAULT_PROGRAM_ID)]
    pub jito_vault_program: AccountInfo<'info>,
    
    /// CHECK: This is a PDA that serves as the protocol vault authority
    /// @audit  TODO! seeds = [b"protocol_vault_authority",jito_mint.key().to_bytes().as_ref(), gsol_mint.key().to_bytes().as_ref()],
    //     #[account(
    //         mut,
    //         seeds = [b"protocol_vault_authority",
    //         global_state.input_token_mint.key().to_bytes().as_ref(), 
    //         global_state.output_token_mint.key().to_bytes().as_ref()],
    //         bump
    //     )]
    // pub protocol_vault_authority: AccountInfo<'info>,
    
    #[account(
        init,
        payer = admin,
        space = 8 + JitoVaultConfig::INIT_SPACE,
        seeds = [b"jito_vault_config"],
        bump
    )]
    pub jito_vault_config: Account<'info, JitoVaultConfig>,
    // Will be passed from the client , need to import jito-type-script sdk
    /// CHECK: This is a metadata account passed from the client
    pub metadata_account: AccountInfo<'info>,
    /// CHECK: This is the token metadata program
    pub token_metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct DepositToJitoVault<'info> {
    /// CHECK: Verified by Jito Vault program
    #[account(
        mut,
        constraint = config.key() == Pubkey::from_str(JITO_CONFIG).unwrap()
    )]
    pub config: AccountInfo<'info>,
    
    /// CHECK: Vault account that will be updated
    #[account(
        mut,
        seeds = [b"vault", jito_manager.key().as_ref()],
        seeds::program = JITO_VAULT_PROGRAM_ID,
        bump
    )]
    pub vault: AccountInfo<'info>,

   
    // #[account(mut,
    //     constraint = vrt_mint.key() == jito_vault_config.jito_vault_vrt_mint
    // )]
    /// CHECK: VRT mint that will be updated
    #[account(mut)]
    pub vrt_mint: AccountInfo<'info>,
    
    /// CHECK: Protocol vault authority PDA that will be the depositor
    #[account(mut,
        seeds = [b"protocol_vault_authority"],
        bump
    )]
    pub depositor: AccountInfo<'info>,
    
    /// Protocol's nSOL token account
    #[account(
        mut,
        associated_token::mint = global_state.jito_vault_input_token_mint,
        associated_token::authority = jito_manager
    )]
    pub depositor_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,
    
    /// Vault's token account to receive nSOL
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,
    
    /// Protocol's VRT token account to receive minted tokens
    #[account(       
        init_if_needed,
        payer = admin,
        associated_token::mint = vrt_mint,
        associated_token::authority = jito_manager
    )]
    pub depositor_vrt_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,
    
    /// Vault's fee token account
    // #[account(
    //     associated_token::mint = vrt_mint,
    //     associated_token::authority = jito_manager
    // )]
    #[account(
    init_if_needed,
    payer=admin,
    associated_token::mint = vrt_mint,
    associated_token::authority = vault
    )]
    pub vault_fee_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,
    
    /// Optional mint authority signer
    // 
    /// CHECK: This is the mint signer
    pub mint_signer: Option<Signer<'info>>,

    #[account(
        seeds = [b"jito_vault_config"],
        bump
    )]
    pub jito_vault_config: Account<'info, JitoVaultConfig>,

    #[account(
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    /// CHECK: This is the PDA that will sign for the vault initialization
    #[account(
        mut,
        seeds = [b"jito_manager", admin.key().as_ref()],
        bump
    )]
    pub jito_manager: AccountInfo<'info>,

    #[account(mut,
            constraint = admin.key() == Pubkey::from_str(ADMIN_ADRESS).unwrap()
    )]
    pub admin: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is the metadata account
    pub metadata_account: AccountInfo<'info>,
    /// CHECK: This is the token metadata program
    pub token_metadata_program: AccountInfo<'info>,
    
}

#[derive(Accounts)]
pub struct DelegateJitoSolToRewardPool<'info> {
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
    
    /// CHECK: This is the PDA that will sign for the vault initialization. No checks needed as we verify the seeds and bump.
    #[account(
        seeds = [b"jito_manager",admin.key().as_ref()],
        bump
    )]
    pub jito_manager: AccountInfo<'info>,
    /// The jitoSOL mint
    #[account(
        constraint = jito_sol_mint.key() == global_state.input_token_mint
    )]
    pub jito_sol_mint: InterfaceAccount<'info, InterfaceMint>,

    /// Vault's jitoSOL token account
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,

    /// CHECK: This is the reward pool authority PDA
    #[account(
        seeds = [b"reward_pool_authority",jito_sol_mint.key().as_ref()],
        bump
    )]
    pub reward_pool_authority: AccountInfo<'info>,

    /// CHECK: This is the PDA that will sign for the delegate
    #[account(
        seeds = [b"jito_manager", admin.key().as_ref()],
        bump
    )]
    pub delegate_asset_admin: AccountInfo<'info>,

    #[account(mut,
        constraint = admin.key() == Pubkey::from_str(ADMIN_ADRESS).unwrap()
    )]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct TransferJitoSolToRewardPool<'info> {
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

    /// CHECK: Vault's token account that will be updated
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, InterfaceTokenAccount>,

    /// CHECK: Reward pool's token account that will be updated
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

    /// CHECK: This is the reward pool authority PDA
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

    pub token_program: Interface<'info, TokenInterface>,
}
// The Implementation Functions

impl<'info> DelegateJitoSolToRewardPool<'info> {
    pub fn delegate_jito_sol_rewards_to_reward_pool(&mut self) -> Result<()> {
        // Only admin can delegate token accounts
     

        let program_id = JITO_VAULT_PROGRAM_ID;

        // Create the instruction using the SDK's delegate_token_account function
        let instruction = delegate_token_account(
            &program_id,
            &self.config.key(),
            &self.vault.key(),
            &self.delegate_asset_admin.key(),
            &self.jito_sol_mint.key(),
            &self.vault_token_account.key(),
            &self.reward_pool_authority.key(),
            &self.token_program.key(),
        );

        // Get the bump for jito_manager PDA
        let bump = self.global_state.jito_manager_bump;
        let admin_key = self.admin.key();
        let authority_seeds = &[
            b"jito_manager",
            admin_key.as_ref(),
            &[bump]
        ];

        // Execute the instruction with jito_manager PDA as signer
        solana_program::program::invoke_signed(
            &instruction,
            &[
                self.config.to_account_info(),
                self.vault.to_account_info(),
                self.delegate_asset_admin.to_account_info(),
                self.jito_sol_mint.to_account_info(),
                self.vault_token_account.to_account_info(),
                self.reward_pool_authority.to_account_info(),
                self.token_program.to_account_info(),
            ],
            &[authority_seeds]
        )?;

        Ok(())
    }
}

impl<'info> DepositToJitoVault<'info> {
    pub fn deposit_to_jito_vault(&mut self, amount_to_deposit: u64, min_amount_out: u64) -> Result<()> {
        // check the global state
        if self.global_state.total_jitosol_deposited < amount_to_deposit {
            return Err(error!(ErrorCode::InsufficientJitosol));
        }
        // Only admin can delegate to the vault
        if self.admin.key() != Pubkey::from_str(ADMIN_ADRESS).unwrap() {
            return Err(error!(ErrorCode::Unauthorized));
        }
        if self.global_state.total_jitosol_deposited < amount_to_deposit {
            return Err(error!(ErrorCode::InsufficientJitosol));
        }

        let program_id = JITO_VAULT_PROGRAM_ID;
        // let mut accounts = vec![
        //     AccountMeta::new(*config, false),
        //     AccountMeta::new(*vault, false),
        //     AccountMeta::new(*vrt_mint, false),
        //     AccountMeta::new(*depositor, true),
        //     AccountMeta::new(*depositor_token_account, false),
        //     AccountMeta::new(*vault_token_account, false),
        //     AccountMeta::new(*depositor_vrt_token_account, false),
        //     AccountMeta::new(*vault_fee_token_account, false),
        //     AccountMeta::new_readonly(spl_token::id(), false),
        // ];
        // Create the instruction using the SDK's mint_to function
        let instruction = mint_to(
            &program_id,
            &self.config.key(),
            &self.vault.key(),
            &self.vrt_mint.key(),
            &self.jito_manager.key(),
            &self.depositor_token_account.key(),
            &self.vault_token_account.key(),
            &self.depositor_vrt_token_account.key(),
            &self.vault_fee_token_account.key(),
            None,
            amount_to_deposit,
            min_amount_out
        );

        // Get the bump for protocol vault authority PDA
        let bump = self.global_state.jito_manager_bump;
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"jito_manager",
            self.global_state.protocol_admin.as_ref(),
            &[bump]
        ]];
        msg!("Line 461");
        // Execute the instruction with protocol vault authority PDA as signer
        let accounts = vec![
            self.config.to_account_info(),
            self.vault.to_account_info(),
            self.vrt_mint.to_account_info(),
            self.jito_manager.to_account_info(),
            self.depositor_token_account.to_account_info(),
            self.vault_token_account.to_account_info(),
            self.depositor_vrt_token_account.to_account_info(),
            self.vault_fee_token_account.to_account_info(),
            self.mint_signer.as_ref().map(|s| s.to_account_info()).unwrap_or(self.admin.to_account_info()), // Mint_signer is optional 
            self.token_program.to_account_info(),
            
        ];

        // Always include mint_signer in accounts, even if None
       

        solana_program::program::invoke_signed(
            &instruction,
            &accounts,
            signer_seeds
        )?;


        // solana_program::program::invoke_signed(
        //     &instruction,
        //     &[
        //         self.config.to_account_info(),
        //         self.vault.to_account_info(),
        //         self.vrt_mint.to_account_info(),
        //         self.jito_manager.to_account_info(),
        //         self.depositor_token_account.to_account_info(),
        //         self.vault_token_account.to_account_info(),
        //         self.depositor_vrt_token_account.to_account_info(),
        //         self.vault_fee_token_account.to_account_info(),
        //         self.mint_signer.to_account_info(),
        //         self.token_program.to_account_info(),
        //         self.system_program.to_account_info(),
        //         self.token_program.to_account_info(),
        //         self.associated_token_program.to_account_info(),
        //     ],
        //     signer_seeds
        // )?;

        Ok(())
    }
}

impl<'info> InitializeJitoVault<'info> {
    pub fn initialize_jito_vault_config(
        &mut self,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
        decimals: u8,
        initialize_token_amount: u64,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        msg!("=== Program Debug Info ===");
        msg!("Input Token Mint: {}", self.global_state.input_token_mint.key());
        msg!("Output Token Mint: {}", self.global_state.output_token_mint.key());
       // msg!("Protocol Vault Authority: {}", self.protocol_vault_authority.key());
        // Only admin can initialize the vault
        if self.admin.key() != Pubkey::from_str(ADMIN_ADRESS).unwrap() {
            return Err(error!(ErrorCode::Unauthorized));
        }
        // Check if the vault is already initialized
        if self.jito_vault_config.initialized {
            return Err(error!(ErrorCode::VaultAlreadyInitialized));
        }

        self.mint_nsol_tokens(initialize_token_amount)?;
        // Get the bump for jito_manager PDA
        let bump = self.global_state.jito_manager_bump;
        let admin_key = self.admin.key();
        let jito_manager_seeds = &[
            b"jito_manager",
            admin_key.as_ref(),
            &[bump]
        ];

        let program_id = JITO_VAULT_PROGRAM_ID;
        
        // Create the instruction using the SDK
        let instruction = initialize_vault(
            &program_id,
            &self.config.key(),
            &self.vault.key(),
            &self.vrt_mint.key(),
            &self.st_mint.key(),
            &self.admin_st_token_account.key(),
            &self.vault_st_token_account.key(),
            &self.burn_vault.key(),
            &self.burn_vault_vrt_token_account.key(),
            &self.admin.key(),
            &self.jito_manager.key(),
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
            decimals,
            initialize_token_amount,
        );

        // Execute the instruction with jito_manager PDA as signer
        solana_program::program::invoke_signed(
            &instruction,
            &[
                self.config.to_account_info(),
                self.vault.to_account_info(),
                self.vrt_mint.to_account_info(),
                self.st_mint.to_account_info(),
                self.admin_st_token_account.to_account_info(),
                self.vault_st_token_account.to_account_info(),
                self.burn_vault.to_account_info(),
                self.burn_vault_vrt_token_account.to_account_info(),
                self.admin.to_account_info(),
                self.jito_manager_for_jito.to_account_info(),
                self.system_program.to_account_info(),
                self.token_program.to_account_info(),
                self.associated_token_program.to_account_info(),
                //
                self.token_metadata_program.to_account_info(),
            ],
            &[jito_manager_seeds]
        )?;




        // let (metadata_account, _) = find_metadata_account(&self.vrt_mint.key());
        // // Create token metadata after vault initialization
     
        // let metadata_instruction = create_token_metadata(
        //     &program_id,
        //     &self.vault.key(),
        //     &self.admin.key(),
        //     &self.vrt_mint.key(),
        //     &self.admin.key(), // payer is the admin
        //     // &metadata_account,
        //     &self.metadata_account.key(),
        //     name,
        //     symbol,
        //     uri,
        // );

        // // Execute the metadata instruction with jito_manager PDA as signer

        // solana_program::program::invoke_signed(
        //     &metadata_instruction,
        //     &[
        //         self.vault.to_account_info(),
        //         self.admin.to_account_info(),
        //         self.vrt_mint.to_account_info(),
        //         self.admin.to_account_info(),
        //         // Will be passed from the client , need to import jito-type-script sdk in the tests
        //         self.metadata_account.to_account_info(),
        //         self.token_program.to_account_info(),
        //         self.system_program.to_account_info(),
        //         self.token_metadata_program.to_account_info(),
        //     ],
        //     &[jito_manager_seeds]
        // )?;

        self.jito_vault_config.initialized = true;
        
        Ok(())
    }
    
    pub fn mint_nsol_tokens(&mut self, amount: u64) -> Result<()> {
        let admin_key = self.admin.key();
        let bump = self.global_state.jito_manager_bump;
        let signer_seeds: &[&[&[u8]]] = &[&[
        b"jito_manager", 
        admin_key.as_ref(),
        &[bump]     
    ]];
        msg!("Jito Manager Address: {}", self.jito_manager.key());
    
        let mint_to_accounts = MintTo {
            mint: self.nsol_mint.to_account_info(), // ST_MINT is the nsol mint
            to: self.admin_st_token_account.to_account_info(),
            authority: self.jito_manager.to_account_info(),
        };
    
        let mint_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            mint_to_accounts,
            signer_seeds,
        );
        mint_interface(mint_ctx, amount)?;
    
        // update the nsol vault state
        self.global_state.total_nsol_minted += amount;
    
    
        Ok(())
    }
}


impl<'info> TransferJitoSolToRewardPool<'info> {
    pub fn transfer_jito_sol_to_reward_pool(&mut self) -> Result<()> {
        // Only admin can transfer tokens
        if self.admin.key() != Pubkey::from_str(ADMIN_ADRESS).unwrap() {
            return Err(error!(ErrorCode::Unauthorized));
        }

        // Get the vault token account balance
        let vault_balance = self.vault_token_account.amount;
        if vault_balance == 0 {
            return Err(error!(ErrorCode::InsufficientBalance));
        }

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

        Ok(())
    }
}