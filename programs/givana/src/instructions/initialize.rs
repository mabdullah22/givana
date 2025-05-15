    use anchor_lang::prelude::*;
    use anchor_spl::{
        token_interface::{
            Mint, TokenAccount, TokenInterface,
        },

        associated_token::AssociatedToken,
    };

    use crate::state::GlobalState;
    use crate::state::RewardPool;
    use crate::constants::{ADMIN_ADRESS,JITO_MINT};
    use std::str::FromStr;



    #[derive(Accounts)]

    pub struct GlobalStateInitialize<'info> {
        
        #[account(mut,
            constraint = signer.key() == Pubkey::from_str(ADMIN_ADRESS).unwrap()
        )]
        pub signer: Signer<'info>,

        #[account(
            init,
            payer = signer,
            space = 8 + GlobalState::INIT_SPACE,
            seeds = [b"global-state"],
            bump
        )]
        pub global_state: Account<'info, GlobalState>,
        // @audit I think I should add a constraint to check that the jito_mint is matched with the constant.

        #[account(
            mint::token_program = token_program,
            constraint = jito_mint.key() == JITO_MINT
        )]
        pub jito_mint: InterfaceAccount<'info, Mint>,
        // The vault is a PDA that will receive the JitoSOL from the users

        #[account(
            init,
            payer = signer,
            seeds = [b"vault",jito_mint.key().to_bytes().as_ref()],
            token::mint = jito_mint,
            token::authority = protocol_vault_authority,
            bump
        )]
        pub vault: InterfaceAccount<'info, TokenAccount>,

    
        /// CHECK: This is a PDA that serves as the authority for the vault and doesn't need to be checked
        #[account(
            seeds = [b"protocol_vault_authority",
            jito_mint.key().to_bytes().as_ref(), 
            gsol_mint.key().to_bytes().as_ref()],
            bump
        )]
        pub protocol_vault_authority: AccountInfo<'info>,
        
        /// CHECK: This is a PDA that serves as the authority for the vault and doesn't need to be checked
        #[account(
            seeds = [b"jito_manager",signer.key().as_ref()],
            bump
        )]
        pub jito_manager: AccountInfo<'info>,
        
        // gsol_mint is what the user will receive
        #[account(
            init,
            payer = signer,
            mint::decimals = jito_mint.decimals,
            mint::authority = protocol_vault_authority,
            mint::freeze_authority = protocol_vault_authority,
            mint::token_program = token_program,
            constraint = gsol_mint.supply == 0
        )]
        pub gsol_mint: InterfaceAccount<'info, Mint>,

        #[account(
            init,
            payer = signer,
            mint::decimals = jito_mint.decimals,
            mint::token_program = token_program,
            mint::freeze_authority = jito_manager,
            mint::authority = jito_manager,
           
        )]
        pub nsol_mint: InterfaceAccount<'info, Mint>,
        
        #[account(
            init_if_needed,
            payer = signer,
            associated_token::mint = nsol_mint,
            associated_token::authority = jito_manager,
        )]
        pub nsol_ata: InterfaceAccount<'info, TokenAccount>,
        pub system_program: Program<'info, System>,
        pub token_program: Interface<'info, TokenInterface>,
        pub rent: Sysvar<'info, Rent>,
        pub associated_token_program: Program<'info, AssociatedToken>,
    }


    #[derive(Accounts)]
    pub struct InitializeRewardPool<'info> {

        #[account(mut)]
        #[account(
            constraint = signer.key() == Pubkey::from_str(ADMIN_ADRESS).unwrap()
        )]
        pub signer: Signer<'info>,
        
        #[account(
            init,
            payer = signer,
            seeds = [b"reward_pool_state_v2"],
            space = 8 + RewardPool::INIT_SPACE,
            bump
        )]
        pub reward_pool_state: Account<'info,RewardPool>,
        /// CHECK: This is a PDA that serves as the authority for the vault and doesn't need to be checked
        #[account(
            seeds = [b"reward_pool_authority",jito_mint.key().to_bytes().as_ref()],
            bump
        )]
        pub reward_pool_authority: AccountInfo<'info>,
        
        #[account(
            init,
            payer = signer,
            seeds = [b"reward_pool",jito_mint.key().to_bytes().as_ref()],
            token::mint = jito_mint,
            token::authority = reward_pool_authority,
            bump,
            token::token_program = token_program,
        )]
        pub reward_pool: InterfaceAccount<'info,TokenAccount>,

        

        pub jito_mint: InterfaceAccount<'info,Mint>,
        pub token_program: Interface<'info,TokenInterface>,

        pub system_program: Program<'info, System>,
        
    }



    impl<'info> GlobalStateInitialize<'info> {
    pub fn initialize_global_state(&mut self, bumps: GlobalStateInitializeBumps ) -> Result<()> {
        let global_state = &mut self.global_state;
        
        global_state.total_jitosol_deposited = 0;
        global_state.total_gsol_supply = 0;
        global_state.acc_reward_per_share = 0;
        global_state.acc_ngo_donation_per_share = 0;
        global_state.weighted_donation_rate = 0;
        global_state.current_block_index = 0;
        global_state.last_update_time = Clock::get()?.unix_timestamp;
        global_state.input_token_mint = self.jito_mint.key();
        global_state.output_token_mint = self.gsol_mint.key();
        global_state.protocol_admin = self.signer.key();
        global_state.jito_vault_input_token_mint = self.nsol_mint.key();

        global_state.bump = bumps.global_state;
        global_state.vault_bump = bumps.vault;
        global_state.protocol_vault_authority_bump = bumps.protocol_vault_authority;
        global_state.jito_manager_bump = bumps.jito_manager;



        
        Ok(())
    }
    }   

    impl<'info> InitializeRewardPool<'info> {
    pub fn initialize_reward_pool(&mut self, bumps: InitializeRewardPoolBumps) -> Result<()> {
        let reward_pool = &mut self.reward_pool_state;
        if !reward_pool.initialized {
            reward_pool.total_undistributed_rewards = 0;
            reward_pool.initialized = true;
        }

        reward_pool.bump = bumps.reward_pool_state;

        Ok(())
    }
    }

