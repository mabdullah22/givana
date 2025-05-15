use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";
pub const ADMIN_ADRESS: &str = "8ejBGYggnRDYMn9sbZ6XSDvH8zRn94foZpR2bzW9uFdM";
pub const JITO_PROGRAM: Pubkey = pubkey!("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");
pub const JITO_CONFIG: &str = "UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3";
pub const PRECISION_FACTOR: u64 = 1_000_000_000; // 10^9 for precision in calculations
// pub const GIVANA_JITO_VAULT: Pubkey = pubkey!("66666666666666666666666666666666");
// pub const GIVANA_JITO_VAULT_VRT_MINT: Pubkey = pubkey!("J2QiqUucufN7K3ev6xV4sUE4rTdQtJK8XB13HuwAosLd");
// pub const JITO_MINT: Pubkey = pubkey!("2G7ykGYLaxWk8SJfJbUR2CeYJjrH4WTjrdwVEGY24dDr");
pub const JITO_MINT: Pubkey = pubkey!("EBePA8f5ZdynjsrtcXsg8zuz1DZnqwkkwtUr3jtD2oB9");
pub const WITHDRAW_EPOCH_DURATION: i64 = 86400; // 24 hours


// Vault Account
    
//     ━━━ Basic Information ━━━
//       Base: 3WEYyPXFKavr6My4UiY3pBQRHN9G6Abg6QxfYY7Z1KUS
//       Vault Index: 241
//       Bump: 253
//       Is Paused: false
    
//     ━━━ Token Information ━━━
//       VRT Mint: J2QiqUucufN7K3ev6xV4sUE4rTdQtJK8XB13HuwAosLd
//       Supported Mint: 3jWYy9bfX39Q6ZPnMKgLYRofETKgFHfHdAkHgXp7WBk6
//       VRT Supply: 10
//       Tokens Deposited: 10
//       Deposit Capacity: 18446744073709551615
    
//     ━━━ Accounting ━━━
//       Staked Amount: 0
//       Cooling Down Amount: 0
//       Enqueued for Cooldown Amount: 0
//       Additional Assets Need Unstaking: 0
//       VRT Enqueued for Cooldown Amount: 0
//       VRT Cooling Down Amount: 0
//       VRT Ready To Claim Amount: 0
    
//     ━━━ Admin Authorities ━━━
//       Admin: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
//       Delegate Admin: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
//       Operator Admin: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
//       NCN Admin: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
//       Slasher Admin: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
//       Capacity Admin: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
//       Fee Admin: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
//       Delegate Asset Admin: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
//       Fee Wallet: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
//       Mint Burn Admin: 11111111111111111111111111111111
//       Metadata Admin: AvX4fjk8bEDuRpM4HC6voSJUq2vrgzNsQkHD8uMogRci
    
//     ━━━ Statistics ━━━
//       NCN Count: 0
//       Operator Count: 0
//       Slasher Count: 0
//       Last Fee Change Slot: 378828899
//       Last Start State Update Slot: 378828899
//       Last Full State Update Slot: 378828899
//       Deposit Fee BPS: 0
//       Withdrawal Fee BPS: 100
//       Next Withdrawal Fee BPS: 100
//       Reward Fee BPS: 100
//       Program Fee BPS: 10