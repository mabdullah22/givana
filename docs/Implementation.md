# Givana Reward System Specification

## 1. Terminology and Variables

### Core Variables
| Symbol | Description |
|--------|-------------|
| $T_c$ | Current timestamp (in seconds) |
| $T_s(u)$ | Stake timestamp for user $u$ |
| $T_{l,u}$ | Last contribution update timestamp for user $u$ |
| $T_{l,n}$ | Last claim timestamp for NGO $n$ |
| $G_u$ | Amount of gSOL held by user $u$ |
| $D_u$ | Donation rate chosen by user $u$ (e.g., 0.05 for 5%) |
| $R_i$ | Total rewards added in settlement block $i$ |
| $B_i$ | Settlement block $i$ (defined by start time $B_{i,start}$ and end time $B_{i,end}$) |

### Contribution Metrics
| Symbol | Description |
|--------|-------------|
| $C_u(i)$ | User $u$'s contribution during settlement block $i$ |
| $C_{n,u}(i)$ | NGO contribution from user $u$ during settlement block $i$ |
| $C_{total}(i)$ | Total user contributions during settlement block $i$ |
| $C_{n,total}(i)$ | Total NGO contributions during settlement block $i$ |
| $\hat{C}_u$ | User $u$'s ongoing contribution (not yet in a settlement block) |
| $\hat{C}_{n,u}$ | NGO contribution from user $u$ (not yet in a settlement block) |

### Rewards and Distributions
| Symbol | Description |
|--------|-------------|
| $ER_t$ | Exchange rate at time $t$ |
| $URP_u(i)$ | User $u$'s reward portion from block $i$ |
| $NRP_{n,u}(i)$ | NGO $n$'s reward portion from user $u$ in block $i$ |
| $NRP_n(i)$ | Total NGO $n$'s reward portion from block $i$ |
| $UC_u$ | User $u$'s claimable rewards (across all settled blocks) |
| $NC_n$ | NGO $n$'s claimable rewards (across all settled blocks) |

## 2. Mathematical Model

### 2.1 gSOL Exchange Rate

The exchange rate determines how much gSOL is minted for a given JitoSOL deposit:

$$ER_t = \frac{Total\_JitoSOL\_Deposited + Total\_Undistributed\_Rewards}{Total\_gSOL\_Supply}$$

When user deposits $X$ JitoSOL:

$$gSOL\_to\_mint = \frac{X}{ER_t}$$

### 2.2 Contribution Calculation

Contribution is the time-integrated product of gSOL amount and applicable rate:

For user contributions (applying user's keep rate):
$$C_u(t_1, t_2) = G_u \times (1-D_u) \times (t_2 - t_1)$$

For NGO contributions (applying user's donation rate):
$$C_{n,u}(t_1, t_2) = G_u \times D_u \times (t_2 - t_1)$$

### 2.3 Settlement Block Creation

When new rewards $R_{new}$ arrive at time $T_c$:

1. Create new settlement block $B_i$ where:
   - $B_{i,start} = B_{i-1,end}$ (end of previous block)
   - $B_{i,end} = T_c$ (current time)
   - $R_i = R_{new}$ (rewards to distribute in this block)

2. For each user $u$, calculate and record their contribution for this block:
   $$C_u(i) = G_u \times (1-D_u) \times (B_{i,end} - max(B_{i,start}, T_{l,u}))$$

3. For each NGO $n$ and user $u$ donating to $n$, calculate and record NGO contribution:
   $$C_{n,u}(i) = G_u \times D_u \times (B_{i,end} - max(B_{i,start}, T_{l,u}))$$

4. Calculate total contributions for this block:
   $$C_{total}(i) = \sum_{all\ users} C_u(i)$$
   $$C_{n,total}(i) = \sum_{all\ users\ donating\ to\ n} C_{n,u}(i)$$

### 2.4 Reward Distribution Calculation

For each user $u$ in settlement block $i$:
$$URP_u(i) = \frac{C_u(i)}{C_{total}(i)} \times R_i \times (1-D_u)$$

For each NGO $n$ in settlement block $i$:
$$NRP_n(i) = \sum_{users\ donating\ to\ n} \frac{C_{n,u}(i)}{C_{n,total}(i)} \times R_i \times D_u$$

### 2.5 Claim Calculation

User's claimable rewards (across all settled blocks they haven't claimed yet):
$$UC_u = \sum_{i: B_{i,end} > T_{l,u}} URP_u(i)$$

NGO's claimable rewards (across all settled blocks they haven't claimed yet):
$$NC_n = \sum_{i: B_{i,end} > T_{l,n}} NRP_n(i)$$

## 3. Protocol Flows

### 3.1 Deposit Flow

When user deposits $X$ JitoSOL:

1. Calculate current exchange rate $ER_{T_c}$
2. Mint $gSOL\_to\_mint = \frac{X}{ER_{T_c}}$ to user
3. Record initial user state:
   - $T_s(u) = T_c$ (stake time)
   - $T_{l,u} = T_c$ (last update time)
   - $G_u = gSOL\_to\_mint$ (gSOL balance)
   - $D_u =$ user-specified donation rate
   - Record NGO selection

### 3.2 Token Transfer Hook

When gSOL is transferred between wallets:

1. For sender:
   - Calculate contribution until current time:
     $\hat{C}_{sender} += G_{sender} \times (1-D_{sender}) \times (T_c - T_{l,sender})$
     $\hat{C}_{n,sender} += G_{sender} \times D_{sender} \times (T_c - T_{l,sender})$
   - Update sender's last update time: $T_{l,sender} = T_c$
   - Reduce sender's gSOL balance: $G_{sender} -= transfer\_amount$

2. For receiver:
   - If receiver is new, initialize their account
   - Update receiver's last update time: $T_{l,receiver} = T_c$
   - Increase receiver's gSOL balance: $G_{receiver} += transfer\_amount$

### 3.3 Reward Addition Flow

When rewards $R_{new}$ arrive:

1. Create new settlement block as described in 2.3
2. Update all users' ongoing contributions up to current time
3. Update contribution tracking for all users and NGOs
4. Record block details and reward amount
5. Update exchange rate to reflect new rewards

### 3.4 User Claim Flow

When user $u$ claims rewards:

1. Calculate all claimable rewards $UC_u$ across settlement blocks
2. Transfer rewards to user
3. Update user's last claim time: $T_{l,u} = T_c$
4. Reset user's unclaimed rewards counter

### 3.5 NGO Claim Flow

When NGO $n$ claims rewards:

1. Calculate all claimable rewards $NC_n$ across settlement blocks
2. Transfer rewards to NGO
3. Update NGO's last claim time: $T_{l,n} = T_c$
4. Reset NGO's unclaimed rewards counter

## 4. Practical Considerations

### 4.1 Gas Optimization

1. For large user bases, calculate contributions only when needed:
   - When a user transfers gSOL
   - When a user claims rewards
   - When an NGO claims rewards

2. Batch updates during settlement block creation

### 4.2 Anti-Exploitation Measures

1. Minimum staking period before first claim eligibility (e.g., 24 hours)
2. Rate-limiting NGO claims (e.g., once per day)
3. Minimum contribution thresholds to prevent dust attacks
4. Time-weighted averaging for sudden large deposits

### 4.3 Precision Management

1. Use fixed-point arithmetic with 9 decimal places for all calculations
2. Implement consistent rounding strategy (floor for conservative estimates)
3. Set minimum deposit and claim thresholds

### 4.4 Edge Cases

1. Handle zero contribution periods appropriately
2. Properly account for users who stake after a settlement block starts
3. Handle the case where an NGO has no donations in a specific block
