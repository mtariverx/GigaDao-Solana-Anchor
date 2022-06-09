pub const MIN_ACCOUNT_LEN: usize = 9;
pub const MAX_NUM_COUNCILLORS: usize = 11;
pub const MAX_NUM_VERIFIED_CREATOR_ADDRESSES: usize = 50;
pub const MAX_INSTRUCTIONS: usize = 30;

pub const MAX_SUBSCRIPTION_ACCOUNT_LEN: usize = MIN_ACCOUNT_LEN
    + 32 // owner_address
    + 32 // stake_address
    + 32 // stream_address
    + 32 // dao_address
    + 8 // subscription_timestamp
    + 16 // total_earned
    + 16 // total_claimed
    + 8; // last_update_timestamp

pub const MAX_DAO_ACCOUNT_LEN: usize = MIN_ACCOUNT_LEN
    + (32 * MAX_NUM_COUNCILLORS) // owners
    + 8 // approval_threshold
    + (1 * MAX_NUM_COUNCILLORS) // proposal_signers
    + 1 // proposal_is_active
    + 4 // proposal_type
    + (32 * MAX_NUM_COUNCILLORS) // proposed_councillors
    + 8 // proposed_approval_threshold
    + 32 // proposed_deactivation_stream
    + 8 // proposed_withdrawal_amount
    + 32 // proposed_withdrawal_receiver_owner
    + 32 // proposed_withdrawal_stream
    + 8; // num_streams

pub const MAX_FEE_CONTROLLER_ACCOUNT_LEN: usize = MIN_ACCOUNT_LEN
        + (8 * MAX_INSTRUCTIONS); // proposal_signers

pub const MAX_STREAM_ACCOUNT_LEN: usize = MIN_ACCOUNT_LEN
        + 32 // dao_address
        + 32 // token_mint_address
        + 32 // token_pool_address
        + (32 * MAX_NUM_VERIFIED_CREATOR_ADDRESSES)  // verified_creator_addresses
        + 1 // is_active
        + 1 // is_simulation
        + 8 // num_subscribers
        + 16 // total_streamed
        + 16 // total_claimed
        + 8; // last_update_timestamp

pub const MAX_STAKE_ACCOUNT_LEN: usize = MIN_ACCOUNT_LEN
    + 32 // owner_address
    + 32 // nft_mint_address
    + 32 // token_vault_address
    + 1 // is_staked
    + 8 // num_subscriptions
    + 8 // cumulative_seconds_staked
    + 8; // last_staked_timestamp