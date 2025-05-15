import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { Givana } from "../target/types/givana";
import wallet from "../program_admin.json";
import ngo from "../ngo.json";
import ngo2 from "../ngo2.json";
import depositor from "../depositor.json";
import depositor2 from "../depositor2.json";
import { ComputeBudgetProgram } from "@solana/web3.js";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  clusterApiUrl,
  Connection,
  sendAndConfirmTransaction,
  Transaction,
  TransactionInstruction,
  
} from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getOrCreateAssociatedTokenAccount,
  createInitializeMintInstruction,
  createMintToInstruction,
  getAssociatedTokenAddress,
  
  MINT_SIZE,
  createMint,
} from "@solana/spl-token";
import { assert } from "chai";
import {} from "@jito-foundation/vault-sdk";
import { none } from "@solana/kit";

describe("givana", () => {
  // Configure the client to use devnet.
  const connection = new Connection("https://api.devnet.solana.com", {
    commitment: "confirmed"
  });
  
  // Create a wallet from the imported keypair
  const walletKeypair = Keypair.fromSecretKey(new Uint8Array(wallet));
  
  const ngoKeypair = Keypair.fromSecretKey(new Uint8Array(ngo));
  const ngoKeypair2 = Keypair.fromSecretKey(new Uint8Array(ngo2));
  
  const depositorKeypair = Keypair.fromSecretKey(new Uint8Array(depositor));
  const depositorKeypair2 = Keypair.fromSecretKey(new Uint8Array(depositor2));
  
  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(walletKeypair),
    { commitment: "confirmed" }
  );
  
  anchor.setProvider(provider);
  const program = anchor.workspace.Givana as Program<Givana>;
  const JITO_VAULT_PROGRAM_ID = new PublicKey("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");

  // Use the existing Jito mint
  const jitoMint = new PublicKey("EBePA8f5ZdynjsrtcXsg8zuz1DZnqwkkwtUr3jtD2oB9");
  const jitoAta = new PublicKey("6EB1aCdgFaT3gDvVZJHvNXXAHj7bskUAD3yzCVufMMiz");
  const jitoConfig = new PublicKey("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3");
  
  let gsolMintaddress: PublicKey;
  let nsolMintaddress: PublicKey;
  let depositorJitoAta: PublicKey;
  let depositorJitoAta2: PublicKey;
  let st_mint: PublicKey;
  let admin_st_ata: PublicKey;
  let protocolVaultAuthority: PublicKey;
  let protocol_vault_for_jito_sol: PublicKey;
  let globalState: PublicKey;
  let rewardPoolState: PublicKey;
  let jitoManager: PublicKey;
  let protocolNsolAta: PublicKey;
  let vrtMint: PublicKey;

  // Add NGO keypair
  let metadataAccount: PublicKey;
  let ngoAccount: PublicKey;
  let ngoAccount2: PublicKey;
  before(async () => {
    try {
      // Check if devnet is accessible
      console.log("admin's public key:", walletKeypair.publicKey.toBase58());

      // 1. First get global state and mints
      [globalState] = PublicKey.findProgramAddressSync(
        [Buffer.from("global-state")],
        program.programId
      );

      const globalStateAccount = await program.account.globalState.fetch(globalState);
      gsolMintaddress = globalStateAccount.outputTokenMint;

      nsolMintaddress = globalStateAccount.jitoVaultInputTokenMint;
      st_mint = globalStateAccount.jitoVaultInputTokenMint;

      // 2. Get jito manager
      [jitoManager] = PublicKey.findProgramAddressSync(
        [Buffer.from("jito_manager"), walletKeypair.publicKey.toBuffer()],
        program.programId
      );

      // 3. Get protocol vault authority
      [protocolVaultAuthority] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("protocol_vault_authority"),
          globalStateAccount.inputTokenMint.toBuffer(),
          globalStateAccount.outputTokenMint.toBuffer()
        ],
        program.programId
      );

      // 4. Now we can get protocol nsol ATA since we have protocolVaultAuthority
      protocolNsolAta = await getAssociatedTokenAddress(
        nsolMintaddress,
        protocolVaultAuthority,
        true
      );

      // Rest of the initialization...
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), jitoManager.toBuffer()],
        JITO_VAULT_PROGRAM_ID
      );

      [protocol_vault_for_jito_sol] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), jitoMint.toBuffer()],
        program.programId
      );

      depositorJitoAta = await getAssociatedTokenAddress(jitoMint, depositorKeypair.publicKey);
      depositorJitoAta2 = await getAssociatedTokenAddress(jitoMint, depositorKeypair2.publicKey);
      admin_st_ata = await getAssociatedTokenAddress(st_mint, walletKeypair.publicKey);

      // Airdrop SOL if needed
      const balance = await connection.getBalance(walletKeypair.publicKey);
      if (balance < 1 * LAMPORTS_PER_SOL) {
        console.log("Airdropping 2 SOL to wallet...");
        const sig = await connection.requestAirdrop(walletKeypair.publicKey, 2 * LAMPORTS_PER_SOL);
        await connection.confirmTransaction(sig, "confirmed");
      }

      const depositorBalance = await connection.getBalance(depositorKeypair.publicKey);
      if (depositorBalance < 1 * LAMPORTS_PER_SOL) {
        console.log("Airdropping 2 SOL to depositor wallet...");
        const sig = await connection.requestAirdrop(depositorKeypair.publicKey, 2 * LAMPORTS_PER_SOL);
        await connection.confirmTransaction(sig, "confirmed");
      }

      const ngoBalance = await connection.getBalance(ngoKeypair.publicKey);
      if (ngoBalance < 1 * LAMPORTS_PER_SOL) {
        console.log("Airdropping 2 SOL to NGO wallet...");
        const sig = await connection.requestAirdrop(ngoKeypair.publicKey, 2 * LAMPORTS_PER_SOL);
        await connection.confirmTransaction(sig, "confirmed");
      }

      // Derive NGO account PDA
      [ngoAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from("ngo"), ngoKeypair.publicKey.toBuffer()],
        program.programId
      );

      [ngoAccount2] = PublicKey.findProgramAddressSync(
        [Buffer.from("ngo"), ngoKeypair2.publicKey.toBuffer()],
        program.programId
      );

      console.log("GSOL Mint:", gsolMintaddress.toBase58());
      console.log("NSOL Mint:", nsolMintaddress.toBase58());
      console.log("Jito Manager:", jitoManager.toBase58());
      console.log("Vault PDA for jito restaking vault :", vaultPda.toBase58());
      console.log("Protocol Vault Authority:", protocolVaultAuthority.toBase58());

      console.log("NGO Keypair:", ngoKeypair.publicKey.toBase58());
      console.log("NGO Account:", ngoAccount.toBase58());
    } catch (error) {
      console.error("Error in setup:", error);
      throw error;
    }
  });

  // it("Initialize the Givana Program", async () => {
  //   // Get PDAs
  //   const [globalState] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("global-state")],
  //     program.programId
  //   );

  //   const [vault] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("vault"), jitoMint.toBuffer()],
  //     program.programId
  //   );

   

  //   const [jitoManager] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("jito_manager"), walletKeypair.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   console.log("Jito Manager:", jitoManager.toBase58());

  //   // const [nsolAta] = PublicKey.findProgramAddressSync(
  //   //   [
  //   //     jitoManager.toBuffer(),
  //   //     Buffer.from([6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172, 28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169]),
  //   //     nsolMintaddress.toBuffer()
  //   //   ],
  //   //   ASSOCIATED_TOKEN_PROGRAM_ID
  //   // );

  //   try {
  //     // Check if the program is already initialized by fetching the global state
  //     const globalStateAccount = await program.account.globalState.fetch(globalState);
  //     console.log("Program already initialized. Global State:", globalStateAccount.totalJitosolDeposited.toString());
  //     return;
  //   } catch (error) {
  //     console.log("Program not initialized. Proceeding with initialization...");
  //   }

  //   console.log("gsol mint:", gsolMintaddress);

  //   // Initialize the program
  //   const tx = await program.methods
  //     .initialize()
  //     .accounts({
  //       signer: walletKeypair.publicKey,
  //       jitoMint: jitoMint,
  //       gsolMint: gsolMintaddress,
  //       nsolMint: nsolMintaddress,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     })
  //     .signers([walletKeypair])
  //     .rpc();

  //   console.log("Initialize transaction signature:", tx);
    
  //   // Fetch the global state account to verify initialization
  //   const globalStateAccount = await program.account.globalState.fetch(globalState);
  //   console.log("Global State Account 238:", globalStateAccount.totalJitosolDeposited.toString());
  // });

  // it("Test the Jito Initialization", async () => {
   

   
   
   
  //   const tx = await program.methods.initializeJitoVault(100, 100, 0, 9, new BN(1000000000), "jitoVRT", "jVRT", "https://givana.com").accounts({
  //   admin: walletKeypair.publicKey,
  //   config: jitoConfig,
  
    

    
  //  }).signers([walletKeypair]).rpc();
  // });

  xit("Initialize the Givana Program", async () => {
    // Get PDAs
    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from("global-state")],
      program.programId
    );

    const [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), jitoMint.toBuffer()],
      program.programId
    );

    const [jitoManager] = PublicKey.findProgramAddressSync(
      [Buffer.from("jito_manager"), walletKeypair.publicKey.toBuffer()],
      program.programId
    );

    console.log("Jito Manager:", jitoManager.toBase58());

    try {
      // Check if the program is already initialized
      const globalStateAccount = await program.account.globalState.fetch(globalState);
      console.log("Program already initialized. Global State:", globalStateAccount.totalJitosolDeposited.toString());
      return;
    } catch (error) {
      console.log("Program not initialized. Proceeding with initialization...");
    }

    // Create a new keypair for the gsol mint
    const gsolMintKeypair = Keypair.generate();
    gsolMintaddress = gsolMintKeypair.publicKey;

    // Create a new keypair for the nsol mint
    const nsolMintKeypair = Keypair.generate();
    nsolMintaddress = nsolMintKeypair.publicKey;

    // Initialize the program
    const tx = await program.methods
      .initialize()
      .accounts({
        signer: walletKeypair.publicKey,
        jitoMint: jitoMint,
        gsolMint: gsolMintaddress,
        nsolMint: nsolMintaddress,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([walletKeypair, gsolMintKeypair, nsolMintKeypair])
      .rpc();

    console.log("Initialize transaction signature:", tx);
    
    // Fetch the global state account to verify initialization
    const globalStateAccount = await program.account.globalState.fetch(globalState);
    console.log("Global State Account:", globalStateAccount.totalJitosolDeposited.toString());
  });

  xit("Initialize the Reward Pool", async () => {
    // Get PDAs for reward pool
   [rewardPoolState] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_pool_state_v2")],
      program.programId
    );

    const [rewardPool] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_pool"), jitoMint.toBuffer()],
      program.programId
    );

    const [rewardPoolAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_pool_authority"), jitoMint.toBuffer()],
      program.programId
    );

    try {
      // Check if the reward pool is already initialized
      const rewardPoolStateAccount = await program.account.rewardPool.fetch(rewardPoolState);
      console.log("Reward Pool already initialized:", rewardPoolStateAccount);
      return;
    } catch (error) {
      console.log("Reward Pool not initialized. Proceeding with initialization...");
    }

    const tx = await program.methods
      .initializeRewardPool()
      .accountsPartial({
        signer: walletKeypair.publicKey,
        rewardPoolState: rewardPoolState,
        rewardPool: rewardPool,
        rewardPoolAuthority: rewardPoolAuthority,
        jitoMint: jitoMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([walletKeypair])
      .rpc({skipPreflight: false});

    console.log("Initialize Reward Pool transaction signature:", tx);
    
    // Verify initialization
    const rewardPoolStateAccount = await program.account.rewardPool.fetch(rewardPoolState);
    console.log("Reward Pool State:", rewardPoolStateAccount);
  });

  xit("Test the Jito Initialization", async () => {
    // Generate necessary keypairs
    // Fetch the jito_vault_config and check the initialized flag , if its true, skip the test
    const [jitoVaultConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("jito_vault_config")],
      program.programId
    );
    const jitoVaultConfigAccount = await program.account.jitoVaultConfig.fetch(jitoVaultConfig);
    console.log("Jito Vault Config:", jitoVaultConfigAccount);
    if (jitoVaultConfigAccount.initialized) {
      console.log("Jito Vault already initialized. Skipping test...");
      return;
    }
   
    const base = Keypair.generate();
    const vrtMint = Keypair.generate();
    
    const globalStateAccount = await program.account.globalState.fetch(globalState);
    
    // Debug logs
    console.log("=== Debug Info ===");
    console.log("Global State Account:", globalState);
    console.log("Input Token Mint:", globalStateAccount.inputTokenMint.toBase58());
    console.log("Output Token Mint:", globalStateAccount.outputTokenMint.toBase58());
    console.log("Jito Mint:", jitoMint.toBase58());
    console.log("GSOL Mint:", gsolMintaddress.toBase58());
    // Get metadata account after vrtMint is created
    const [metadataAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
        vrtMint.publicKey.toBuffer(),
      ],
      new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
    );

    // Find PDAs
    const [jitoManager] = PublicKey.findProgramAddressSync(
      [Buffer.from("jito_manager"), walletKeypair.publicKey.toBuffer()],
      program.programId
    );

    const [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), jitoManager.toBuffer()],
      JITO_VAULT_PROGRAM_ID
    );

    // Get global state to ensure we're using the correct mints
    const seeds = [
      Buffer.from("protocol_vault_authority"),
      globalStateAccount.inputTokenMint.toBuffer(),
      globalStateAccount.outputTokenMint.toBuffer()
  ];
  console.log("Seeds used:", seeds.map(s => s.toString('hex')));

  const [protocolVaultAuthorityy] = PublicKey.findProgramAddressSync(
      seeds,
      program.programId
  );
  console.log("Protocol Vault Authority:", protocolVaultAuthorityy.toBase58());

    // Fix: Derive protocol_vault_authority with correct seeds using global state values
    const [protocolVaultAuthority] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("protocol_vault_authority"),
        globalStateAccount.inputTokenMint.toBuffer(),  // Use from global state
        globalStateAccount.outputTokenMint.toBuffer()  // Use from global state
      ],
      program.programId
    );
    console.log("Protocol Vault Authority:", protocolVaultAuthority.toBase58());

    console.log("Vault PDA:", vaultPda.toBase58());
    
    const [burnVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("burn_vault"), jitoManager.toBuffer()],
      JITO_VAULT_PROGRAM_ID
    );

    // Get token accounts
    const adminStTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection, 
      walletKeypair,
      nsolMintaddress,
      walletKeypair.publicKey,
      true,
      null,
      null,
      TOKEN_PROGRAM_ID
    );

    const vaultStTokenAccount = await getAssociatedTokenAddress(
      nsolMintaddress,
      vaultPda,
      true
    );

    const burnVaultVrtTokenAccount = await getAssociatedTokenAddress(
      vrtMint.publicKey,
      burnVaultPda,
      true
    );

    // 2. Initialize the vault
    try {
       // Create compute budget instruction to request more CUs and set priority fee
  const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({ 
    units: 1000000 // Request 1M CUs (max is 1.4M)
  });
  
  const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 50000 // Priority fee of 0.00005 SOL per million CUs
  });
      const tx = await program.methods
        .initializeJitoVault(
          100, // deposit_fee_bps
          100, // withdrawal_fee_bps
          0,   // reward_fee_bps
          9,   // decimals
          new BN(1000000000), // initialize_token_amount
          "jitoVRT", // name
          "jVRT",    // symbol
          "https://givana.com" // uri
        )
        .accounts({
          admin: walletKeypair.publicKey,
          config: new PublicKey("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3"),
          vrtMint: vrtMint.publicKey,
          stMint: nsolMintaddress,
          nsolMint: nsolMintaddress,
          adminStTokenAccount: adminStTokenAccount.address,
          vaultStTokenAccount: vaultStTokenAccount,
          burnVaultVrtTokenAccount: burnVaultVrtTokenAccount,
          metadataAccount: metadataAccount,
          tokenMetadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
        }).preInstructions([modifyComputeUnits, addPriorityFee])
        .signers([walletKeypair, vrtMint])
        .rpc({ skipPreflight: true});

      console.log("Vault initialization tx:", tx);

      // 3. Check results
      const vaultAccount = await connection.getAccountInfo(vaultPda);
      console.log("Vault created:", !!vaultAccount);
    } catch (error) {
      console.error("Vault initialization failed:", error);
      throw error;
    }
  });

  it("Register an NGO", async () => {
    try {
      // Check if NGO is already registered
      const [defaultNgoAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from("ngo"), PublicKey.default.toBuffer()],
        program.programId
    );

      const ngoAccountInfo = await program.account.ngoAccount.fetch(ngoAccount2);
      console.log("NGO already registered:", ngoAccountInfo);
      return;
    } catch (error) {
      console.log("NGO not registered. Proceeding with registration...");
    }

    const tx = await program.methods
      .registerNgo()
      .accounts({
        authority: ngoKeypair2.publicKey
      })
      .signers([ngoKeypair2])
      .rpc();

    console.log("Register NGO transaction signature:", tx);
    
    // Verify registration
    const ngoAccountInfo = await program.account.ngoAccount.fetch(ngoAccount);
    console.log("NGO Account Info:", ngoAccountInfo);
    assert.equal(ngoAccountInfo.authority.toBase58(), ngoKeypair2.publicKey.toBase58());
    
  });

  it("Activate an NGO", async () => {

    // Check if NGO is already activated
    const ngoAccountInfoo = await program.account.ngoAccount.fetch(ngoAccount2);
    console.log("NGO already activated:", ngoAccountInfoo);
    if (ngoAccountInfoo.isActive) {
      console.log("NGO already activated. Skipping test...");
      return;
    }
    
    const tx = await program.methods.activateNgo().accounts({
      authority: walletKeypair.publicKey,
      ngoAddress: ngoKeypair2.publicKey,
      }).signers([walletKeypair]).rpc();

    const ngoAccountInfo = await program.account.ngoAccount.fetch(ngoAccount2);
    console.log("NGO Account Info:", ngoAccountInfo);
    assert.equal(ngoAccountInfo.isActive, true);
    
    console.log("Activate NGO transaction signature:", tx);
  });
  

  xit("Deposit", async () => {
    const [defaultNgoAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("ngo"), PublicKey.default.toBuffer()],
      program.programId
  );
  console.log("defaultNgoAccount:", defaultNgoAccount);
  
   

    const usergSolAta = await getAssociatedTokenAddress(
        gsolMintaddress,
        depositorKeypair.publicKey,
        true
    );

    const usergSolAta2 = await getAssociatedTokenAddress(
        gsolMintaddress,
        depositorKeypair2.publicKey,
        true
    );
    const protocolNsolAta = await getOrCreateAssociatedTokenAccount(
      connection,
      walletKeypair,
      nsolMintaddress,
      jitoManager,
      true
    );

    // Console.log all the accounts being passed to the deposit function
    // console.log("protocolNsolAta:", protocolNsolAta);
    // console.log("usergSolAta:", usergSolAta);
    // console.log("gsolMintaddress:", gsolMintaddress);
    // console.log("vault:", protocol_vault_for_jito_sol);
    // console.log("globalState:", globalState);
    // console.log("protocolVaultAuthority:", protocolVaultAuthority);
    // console.log("ngoAccount:", ngoAccount);
    // console.log("rewardPoolState:", rewardPoolState);
    // console.log("depositorJitoAta:", depositorJitoAta);
    // console.log("jitoMint:", jitoMint);
    // console.log("tokenProgram:", TOKEN_PROGRAM_ID);
    // console.log("depositorKeypair:", depositorKeypair.publicKey);

   // Test deposit with NGO
    const tx = await program.methods
        .deposit(ngoKeypair2.publicKey, new BN(100000000), 1000) // NGO address, amount, donation rate
        .accountsPartial({
            authority: depositorKeypair2.publicKey,
            stakerJitoSolAta: depositorJitoAta2,
            jitoMint: jitoMint,
            protocolNsolAta: protocolNsolAta.address,
            nsolMint: nsolMintaddress,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            userGsolAta: usergSolAta2,
            gsolMint: gsolMintaddress,
            vault: protocol_vault_for_jito_sol,
            globalState: globalState,
            protocolVaultAuthority: protocolVaultAuthority,
            ngoAccount: ngoAccount2,
            rewardPoolState: rewardPoolState,
            jitoManager: jitoManager,
        })
        .signers([depositorKeypair2])
        .rpc({skipPreflight: true});

    // // Test deposit without NGO
    // const tx2 = await program.methods
    //     .deposit(null, new BN(100000000), 0) // null for NGO, amount, 0 donation rate
    //     .accountsPartial({
    //         authority: depositorKeypair.publicKey,
    //         stakerJitoSolAta: depositorJitoAta,
    //         jitoMint: jitoMint,
    //         protocolNsolAta: protocolNsolAta,
    //         nsolMint: nsolMintaddress,
    //         tokenProgram: TOKEN_PROGRAM_ID,
    //         systemProgram: SystemProgram.programId,
    //         userGsolAta: usergSolAta,
    //         gsolMint: gsolMintaddress,
    //         vault: protocol_vault_for_jito_sol,
    //         globalState: globalState,
    //         protocolVaultAuthority: protocolVaultAuthority,
    //         ngoAccount: defaultNgoAccount,
    //         rewardPoolState: rewardPoolState,
    //         jitoManager: jitoManager,
    //     })
    //     .signers([depositorKeypair])
    //     .rpc({skipPreflight: false});

    // Verify balances after deposit
    const jitoBalance_after = await connection.getTokenAccountBalance(depositorJitoAta);
    console.log("Jito Balance after deposit:", jitoBalance_after);

    // Check global state
    const globalStateAccount = await program.account.globalState.fetch(globalState);
    console.log("Global State Account:", globalStateAccount.totalJitosolDeposited.toString());
});

  xit("Withdraw", async () => {
    // const tx = await program.methods.withdraw()
  });

  // This test is failing , I need to look into it. Jito expect an option account to be passed into the instructions. 
xit("Admin deposit to Jito vault", async () => {
    // Get necessary PDAs and accounts

    const vrt_mint_from_vault = new PublicKey("3HJYMpT3icK3duS5mVRXVC8HYbGQRobhsrmbBRtZ7Trr");
    
    const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), jitoManager.toBuffer()],
        JITO_VAULT_PROGRAM_ID
    );

    const [jitoVaultConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("jito_vault_config"), jitoManager.toBuffer()],
      program.programId
    );

    
    const [protocolVaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("protocol_vault_authority")],
      program.programId
  );
  const protocol_nsol_ata = await getAssociatedTokenAddress(
    nsolMintaddress,
    jitoManager,
    true
  );


 


    // const [vrtMint] = PublicKey.findProgramAddressSync(
    //     [Buffer.from("vrt_mint")],
    //     program.programId
    // );

    const globalStateAccount = await program.account.globalState.fetch(globalState);
    const vrtMint = globalStateAccount.jitoVaultInputTokenMint;

    const protocol_vrt_ata = await getAssociatedTokenAddress(
      vrt_mint_from_vault,
      jitoManager,
      true
    );

    // Get token accounts
    const protocolVrtAta = await getAssociatedTokenAddress(
      vrt_mint_from_vault,
        protocolVaultAuthority,
        true
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
        nsolMintaddress,
        vaultPda,
        true
    );

    const vaultFeeTokenAccount = await getAssociatedTokenAddress(
        vrt_mint_from_vault,
        vaultPda,
        true
    );

    // Execute admin deposit
    const tx = await program.methods
        .adminDepositToJitoVault(
            new BN(10000000), // amount_to_deposit
            new BN(9000000)   // min_amount_out
        )
        .accounts({
            config: jitoConfig,
            // vault: vaultPda,
            vrtMint: vrt_mint_from_vault,
            // depositor: protocolVaultAuthority,
            // depositorTokenAccount: protocol_nsol_ata,
            vaultTokenAccount: vaultTokenAccount,
            // depositorVrtTokenAccount: protocol_vrt_ata,
            // vaultFeeTokenAccount: vaultFeeTokenAccount,
            admin: walletKeypair.publicKey,
            // systemProgram: SystemProgram.programId,
            // associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataAccount: metadataAccount,
            tokenMetadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),

        })
        .signers([walletKeypair])
        .rpc({ skipPreflight: true });

    console.log("Admin deposit transaction signature:", tx);

    // Verify the deposit
    const vaultBalance = await connection.getTokenAccountBalance(vaultTokenAccount);
    console.log("Vault balance after deposit:", vaultBalance.value.amount);
});

  // Move the Withdraw Flow inside the main describe block
  describe("Withdraw Flow", () => {
    xit("Initiate Withdraw", async () => {
      // Check if withdraw is already initiated,if yes, skip the test
      const [stakerAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), depositorKeypair2.publicKey.toBuffer()],
        program.programId
      );
      const userAccount = await program.account.userAccount.fetch(stakerAccount);
      if (userAccount.withdrawRequested) {
        console.log("Withdraw already initiated. Skipping test.");
        return;
      }

      const user_gsol_ata = await getAssociatedTokenAddress(
        gsolMintaddress,
        depositorKeypair2.publicKey,
        true
      );

      const protocol_gsol_ata = await getAssociatedTokenAddress(
        gsolMintaddress,
        protocolVaultAuthority,
        true
      );
      const [rewardPool] = PublicKey.findProgramAddressSync(
        [Buffer.from("reward_pool_state_v2")],
        program.programId
      );

    
      const withdrawAmount = new BN(100000);

      const tx = await program.methods
        .userInitiateWithdraw(withdrawAmount)
        .accountsPartial({
          user: depositorKeypair2.publicKey,
          userAccount: stakerAccount,
          globalState: globalState,
          rewardPool: rewardPool,
          protocolVaultAuthority: protocolVaultAuthority,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          userGsolAta: user_gsol_ata,
          protocolGsolAta: protocol_gsol_ata,
          gsolMint: gsolMintaddress,
        })
        .signers([depositorKeypair2])
        .rpc({skipPreflight: true, commitment: "confirmed"});

      console.log("InitiateWithdraw tx:", tx);

      
      assert.isTrue(userAccount.withdrawRequested, "Withdraw should be requested");
      assert.equal(userAccount.withdrawAmount.toString(), withdrawAmount.toString(), "Withdraw amount should match");
      
    });

    xit("Admin Burn Withdraw Tokens", async () => {
      
      
      const [stakerAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), depositorKeypair2.publicKey.toBuffer()],
        program.programId
      );

      const protocol_gsol_ata = await getAssociatedTokenAddress(
        gsolMintaddress,
        protocolVaultAuthority,
        true
      );

      const tx = await program.methods
        .adminBurnWithdrawTokens()
        .accountsPartial({
          admin: walletKeypair.publicKey,
          userAccount: stakerAccount,
          globalState: globalState,
          gsolMint: gsolMintaddress,
          protocolVaultAuthority: protocolVaultAuthority,
          protocolGsolAta: protocol_gsol_ata,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([walletKeypair])
        .rpc({ skipPreflight: true });

      console.log("Burn Withdraw Tokens tx:", tx);

      const userAccount = await program.account.userAccount.fetch(stakerAccount);
      const globalStateAccount = await program.account.globalState.fetch(globalState);
      
   
    });

    it("User Claim Withdraw", async () => {
      const [stakerAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), depositorKeypair2.publicKey.toBuffer()],
        program.programId
      );

      const user_jitosol_ata = await getAssociatedTokenAddress(
        jitoMint,
        depositorKeypair2.publicKey,
        true
      );

      const [vault] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), jitoMint.toBuffer()],
        program.programId
      );

      const [rewardPool] = PublicKey.findProgramAddressSync(
        [Buffer.from("reward_pool_state_v2")],
        program.programId
      );

      const [rewardPoolTokenAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from("reward_pool"), jitoMint.toBuffer()],
        program.programId
      );

      const tx = await program.methods
        .userClaimWithdraw()
        .accountsPartial({
          user: depositorKeypair2.publicKey,
          userAccount: stakerAccount,
          globalState: globalState,
          rewardPool: rewardPool,
          vault: vault,
          rewardPoolTokenAccount: rewardPoolTokenAccount,
          userJitosolAta: user_jitosol_ata,
          jitoMint: jitoMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([depositorKeypair2])
        .rpc({ skipPreflight: true });

      console.log("Claim Withdraw tx:", tx);

      const userAccount = await program.account.userAccount.fetch(stakerAccount);
      assert.isFalse(userAccount.withdrawRequested, "Withdraw should be completed");
      assert.equal(userAccount.withdrawAmount.toString(), "0", "Withdraw amount should be reset");
    });
  });
});
