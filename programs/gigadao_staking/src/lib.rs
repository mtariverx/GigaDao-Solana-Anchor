mod utils;
use utils::*;
use std::str::FromStr;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::{
    program::invoke_signed,
    system_instruction::create_account,
};
use anchor_lang::solana_program::system_instruction::transfer;
use anchor_lang::solana_program::program::invoke;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

declare_id!("AGi7p8RritzUDX4sCYVfxApCH4By8FEpSV4ffL7bZ8Kp");

// fees
const FEE_RX_ADDRESS: &str = "5F1xSVrk8JuZj2qCqYupKjwzUFhYDJZoVZoJWR9JpxPB";
const FEE_MOD_ADDRESS: &str = "B5W8dHSLtLnbkeTHgKvmL5grNcFaGdZUitQ1p7NTMdtE";

// pda seeds
const TOKEN_POOL_PDA_SEED: &[u8] = b"token_pool_pda_seed";
const NFT_VAULT_PDA_SEED: &[u8] = b"nft_account_pda_seed";
const DAO_AUTH_PDA_SEED: &[u8] = b"dao_auth_pda_seed";
const STAKE_AUTH_PDA_SEED: &[u8] = b"stake_auth_pda_seed";
const STAKE_PDA_SEED: &[u8] = b"stake_pda_seed";
const CONNECTION_PDA_SEED: &[u8] = b"connection_pda_seed";
const METADATA_PREFIX: &[u8] = b"metadata";
const FEE_CONTROLLER_PDA_SEED: &[u8] = b"fee_controller";

// consts
const MAX_DECIMALS: u8 = 12;
const MAX_STREAM_RATE: u64 = 7e9 as u64; // assuming minimum 1 month runway and 10k connections
const MAX_CONNECTIONS_PER_STREAM: u64 = 2e4 as u64;

#[program]
pub mod gigadao_staking {
    use super::*;


    // system-wide config
    pub fn initialize_fee_controller(
        ctx: Context<InitializeFeeController>,
    ) -> ProgramResult {
        let fee_mod_address: Pubkey = Pubkey::from_str(FEE_MOD_ADDRESS).unwrap();
        if ctx.accounts.signer.key() != fee_mod_address{
            return Err(ErrorCode::InvalidFeeModAddress.into());
        }
        let fee_controller = &mut ctx.accounts.fee_controller;
        fee_controller.initialize_dao = 666;
        fee_controller.initialize_stream = 666;
        fee_controller.reactivate_stream = 666;
        fee_controller.propose_dao_command = 666;
        fee_controller.approve_dao_command = 666;
        fee_controller.execute_update_dao_multisig = 666;
        fee_controller.execute_deactivate_stream = 666;
        fee_controller.execute_withdraw_from_stream = 666;
        fee_controller.initialize_stake = 666;
        fee_controller.stake_nft = 666;
        fee_controller.unstake_nft = 666;
        fee_controller.initialize_connection = 666;
        fee_controller.connect_to_stream = 666;
        fee_controller.claim_from_stream = 666;
        fee_controller.disconnect_from_stream = 666;
        Ok(())
    }

    pub fn update_fee_controller(
        ctx: Context<UpdateFeeController>,
        instruction_name: String,
        new_fee_amount_lamports: u64,
    ) -> ProgramResult {
        msg!("Updating {:?} to {:?}", instruction_name, new_fee_amount_lamports);
        let fee_mod_address: Pubkey = Pubkey::from_str(FEE_MOD_ADDRESS).unwrap();
        if ctx.accounts.signer.key() != fee_mod_address{
            return Err(ErrorCode::InvalidFeeModAddress.into());
        }
        let fee_controller = &mut ctx.accounts.fee_controller;
        match instruction_name.as_str() {
            "initialize_dao" => {fee_controller.initialize_dao = new_fee_amount_lamports;}
            "initialize_stream" => {fee_controller.initialize_stream = new_fee_amount_lamports;}
            "reactivate_stream" => {fee_controller.reactivate_stream = new_fee_amount_lamports;}
            "propose_dao_command" => {fee_controller.propose_dao_command = new_fee_amount_lamports;}
            "approve_dao_command" => {fee_controller.approve_dao_command = new_fee_amount_lamports;}
            "execute_update_dao_multisig" => {fee_controller.execute_update_dao_multisig = new_fee_amount_lamports;}
            "execute_deactivate_stream" => {fee_controller.execute_deactivate_stream = new_fee_amount_lamports;}
            "execute_withdraw_from_stream" => {fee_controller.execute_withdraw_from_stream = new_fee_amount_lamports;}
            "initialize_stake" => {fee_controller.initialize_stake = new_fee_amount_lamports;}
            "stake_nft" => {fee_controller.stake_nft = new_fee_amount_lamports;}
            "unstake_nft" => {fee_controller.unstake_nft = new_fee_amount_lamports;}
            "initialize_connection" => {fee_controller.initialize_connection = new_fee_amount_lamports;}
            "connect_to_stream" => {fee_controller.connect_to_stream = new_fee_amount_lamports;}
            "claim_from_stream" => {fee_controller.claim_from_stream = new_fee_amount_lamports;}
            "disconnect_from_stream" => {fee_controller.disconnect_from_stream = new_fee_amount_lamports;}
            _ => return Err(ErrorCode::InvalidInstructionName.into())
        }
        msg!("Updated {:?} to {:?}", instruction_name, new_fee_amount_lamports);
        Ok(())
    }

    // dao instructions
    pub fn initialize_dao(
        ctx: Context<InitializeDao>,
        councillors: Vec<Pubkey>,
        approval_threshold: u64,
    ) -> ProgramResult {

        // validate inputs
        if councillors.len() > MAX_NUM_COUNCILLORS  || councillors.len() < 1 {
            return Err(ErrorCode::TooManyManagers.into());
        }
        if (approval_threshold as usize > councillors.len()) || (approval_threshold < 1) {
            return Err(ErrorCode::InvalidApprovalThreshold.into());
        }

        // initialize dao
        let dao = &mut ctx.accounts.dao;
        dao.councillors = councillors;
        dao.approval_threshold = approval_threshold;
        dao.num_streams = 0;

        // init multisig variables
        let mut signers = Vec::new();
        signers.resize(dao.councillors.len(), false);
        dao.proposal_signers = signers;
        dao.proposal_is_active = false;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.initialize_dao;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;

        Ok(())
    }

    pub fn initialize_stream(
        ctx: Context<InitializeStream>,
        verified_creator_addresses: Vec<Pubkey>,
        stream_rate: u64,
        is_simulation: bool,
    ) -> ProgramResult {

        // validate inputs
        if verified_creator_addresses.len() > MAX_NUM_VERIFIED_CREATOR_ADDRESSES {
            return Err(ErrorCode::TooManyVerifiedCreatorAddresses.into());
        }

        // check that signer is a dao councillor
        let _owner_index = ctx.accounts.dao.councillors
            .iter()
            .position(|a| a == ctx.accounts.signer.key)
            .ok_or(ErrorCode::InvalidCouncillor)?;

        // check decimals does not exceed max supported
        if ctx.accounts.token_mint.decimals > MAX_DECIMALS {
            return Err(ErrorCode::MaxSupportedDecimalsExceeded.into());
        }

        // check stream rate against maximum supported
        if stream_rate > MAX_STREAM_RATE {
            return Err(ErrorCode::MaxSupportedStreamRateExceeded.into());
        }

        // initialize stream
        let stream = &mut ctx.accounts.stream;
        stream.dao_address = ctx.accounts.dao.key();
        stream.token_mint_address = ctx.accounts.token_mint.key();
        stream.token_pool_address = ctx.accounts.token_pool.key();
        stream.verified_creator_addresses = verified_creator_addresses;
        stream.stream_rate = stream_rate;
        stream.is_simulation = is_simulation;

        stream.is_active = true;
        stream.num_connections = 0;
        stream.total_streamed = 0;
        stream.total_claimed = 0;
        stream.last_update_timestamp = Clock::get().unwrap().unix_timestamp as u64;

        // tx fee
        // let signer_handle = &ctx.accounts.signer;
        // let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        // let fee_lamports = ctx.accounts.fee_controller.initialize_stream;
        //
        // transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn reactivate_stream(
        ctx: Context<ReactivateStream>
    ) -> ProgramResult {

        // check that signer is a dao councillor
        let _owner_index = ctx.accounts.dao.councillors
            .iter()
            .position(|a| a == ctx.accounts.signer.key)
            .ok_or(ErrorCode::InvalidCouncillor)?;

        // calculate recent streamed
        let stream = &mut ctx.accounts.stream;
        let total_stream_rate = stream.stream_rate * stream.num_connections;
        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
        let seconds_since_last_update = current_timestamp - stream.last_update_timestamp;
        let recently_streamed = total_stream_rate * seconds_since_last_update;

        let total_unclaimed = stream.total_streamed - stream.total_claimed;
        let current_pool_balance = ctx.accounts.token_pool.amount;
        let current_pool_surplus = current_pool_balance - total_unclaimed;

        msg!("In reactivate stream got recently streamed: {:?} and surplus: {:?}", recently_streamed, current_pool_surplus);

        if recently_streamed > current_pool_surplus {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        stream.total_streamed += recently_streamed;
        stream.last_update_timestamp = current_timestamp;
        stream.is_active = true;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.reactivate_stream;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn propose_dao_command(
        ctx: Context<ProposeDaoCommand>,
        proposal_type_int: u8,
        proposed_councillors: Vec<Pubkey>,
        proposed_approval_threshold: u64,
        proposed_deactivation_stream: Pubkey,
        proposed_withdraw_amount: u64,
        proposed_withdrawal_receiver_owner: Pubkey,
        proposed_withdrawal_stream: Pubkey,
    ) -> ProgramResult {

        // convert proposal type
        let proposal_type: ProposalType = FromPrimitive::from_u8(proposal_type_int).ok_or(ErrorCode::InvalidProposalType)?;

        // check that signer is a dao councillor
        let dao = &ctx.accounts.dao;
        let councillor_index = dao.councillors
            .iter()
            .position(|a| a == ctx.accounts.signer.key)
            .ok_or(ErrorCode::InvalidCouncillor)?;

        let dao = &mut ctx.accounts.dao;
        match proposal_type {
            ProposalType::UpdateMultisig => {
                // validate input
                if proposed_councillors.len() > MAX_NUM_COUNCILLORS || proposed_councillors.len() < 1 {
                    return Err(ErrorCode::TooManyManagers.into());
                }
                if (proposed_approval_threshold as usize > proposed_councillors.len()) || (proposed_approval_threshold < 1) {
                    return Err(ErrorCode::InvalidApprovalThreshold.into());
                }
                dao.proposed_councillors = proposed_councillors;
                dao.proposed_approval_threshold = proposed_approval_threshold;
            },
            ProposalType::DeactivateStream => {
                dao.proposed_deactivation_stream = proposed_deactivation_stream;
            },
            ProposalType::WithdrawFromStream => {
                dao.proposed_withdrawal_amount = proposed_withdraw_amount;
                dao.proposed_withdrawal_receiver_owner = proposed_withdrawal_receiver_owner;
                dao.proposed_withdrawal_stream = proposed_withdrawal_stream
            }
        }

        // reset signers
        for i in 0..dao.councillors.len(){
            if i == councillor_index {
                dao.proposal_signers[i] = true;
            } else {
                dao.proposal_signers[i] = false;
            }
        }

        // finalize
        dao.proposal_type = proposal_type;
        dao.proposal_is_active = true;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.propose_dao_command;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn approve_dao_command(
        ctx: Context<ApproveDaoCommand>,
    ) -> ProgramResult {
        let dao = &ctx.accounts.dao;
        if !dao.proposal_is_active {
            return Err(ErrorCode::ProposalNotActive.into());
        }
        let councillor_index = dao.councillors
            .iter()
            .position(|a| a == ctx.accounts.signer.key)
            .ok_or(ErrorCode::InvalidCouncillor)?;
        ctx.accounts.dao.proposal_signers[councillor_index] = true;


        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.approve_dao_command;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;



        Ok(())
    }

    // dao commands
    pub fn execute_update_dao_multisig(
        ctx: Context<ExecuteUpdateDaoMultisig>,
    ) -> ProgramResult {

        // validate
        let dao = &mut ctx.accounts.dao;
        validate_proposal_approval(dao, ctx.accounts.signer.key)?;
        match dao.proposal_type {
            ProposalType::UpdateMultisig => (),
            _ => return Err(ErrorCode::MismatchProposalType.into())
        }

        // update multisig
        dao.councillors = dao.proposed_councillors.clone();
        dao.approval_threshold = dao.proposed_approval_threshold;
        let mut signers = Vec::new();
        signers.resize(dao.councillors.len(), false);
        dao.proposal_signers = signers;
        dao.proposal_is_active = false;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.execute_update_dao_multisig;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn execute_deactivate_stream(
        ctx: Context<ExecuteDeactivateStream>,
    ) -> ProgramResult {

        // validate
        let dao = &mut ctx.accounts.dao;
        validate_proposal_approval(dao, ctx.accounts.signer.key)?;
        match dao.proposal_type {
            ProposalType::DeactivateStream => (),
            _ => return Err(ErrorCode::MismatchProposalType.into())
        }

        // update stream
        let stream = &mut ctx.accounts.stream;
        let current_pool_balance = ctx.accounts.token_pool.amount;
        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
        let add_connection = false;
        update_stream_state(stream, current_pool_balance, current_timestamp, add_connection)?;

        // deactivate and finalize
        stream.is_active = false;
        dao.proposal_is_active = false;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.execute_deactivate_stream;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn execute_withdraw_from_stream(
        ctx: Context<ExecuteWithdrawFromStream>,
    ) -> ProgramResult {

        // validate
        let dao = &mut ctx.accounts.dao;
        validate_proposal_approval(dao, ctx.accounts.signer.key)?;
        match dao.proposal_type {
            ProposalType::WithdrawFromStream => (),
            _ => return Err(ErrorCode::MismatchProposalType.into())
        }

        // update stream
        let stream = &mut ctx.accounts.stream;
        let current_pool_balance = ctx.accounts.token_pool.amount;
        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
        let add_connection = false;
        update_stream_state(stream, current_pool_balance, current_timestamp, add_connection)?;

        // check proposed receiver match
        if ctx.accounts.receiver_token_account.owner != dao.proposed_withdrawal_receiver_owner {
            return Err(ErrorCode::InvalidProposedReceiverOwner.into());
        }

        // check proposed withdraw stream
        if dao.proposed_withdrawal_stream != stream.key() {
            return Err(ErrorCode::StreamMismatch.into());
        }

        // check proposed amount <= available amount
        let unclaimed_amount = stream.total_streamed - stream.total_claimed;
        let available_amount = current_pool_balance - unclaimed_amount;
        let proposed_withdrawal_amount = dao.proposed_withdrawal_amount;
        if proposed_withdrawal_amount > available_amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // get seeds to sign for auth_pda
        let dao_address = dao.key();
        let (dao_auth_pda, bump_seed) = Pubkey::find_program_address(&[dao_address.as_ref(), DAO_AUTH_PDA_SEED], ctx.program_id);
        let seeds = &[dao_address.as_ref(), &DAO_AUTH_PDA_SEED[..], &[bump_seed]];
        let signer = &[&seeds[..]];

        // check pda addy correct
        if dao_auth_pda != ctx.accounts.dao_auth_pda.key() {
            return Err(ErrorCode::InvalidAuthPda.into());
        }

        // transfer
        let cpi_accounts = Transfer {
            from: ctx.accounts.token_pool.to_account_info(),
            to: ctx.accounts.receiver_token_account.to_account_info(),
            authority: ctx.accounts.dao_auth_pda.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, proposed_withdrawal_amount)?;

        // finalize
        dao.proposal_is_active = false;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.execute_withdraw_from_stream;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    // Staker instructions
    pub fn initialize_stake(
        ctx: Context<InitializeStake>,
    ) -> ProgramResult {

        // initialize stake
        let stake = &mut ctx.accounts.stake;
        stake.owner_address = ctx.accounts.signer.key();
        stake.nft_mint_address = ctx.accounts.nft_mint.key();
        stake.nft_vault_address = ctx.accounts.nft_vault.key();

        stake.is_active = false;
        stake.num_connections = 0;
        stake.cumulative_seconds_staked = 0;
        stake.last_stake_timestamp = 0;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.initialize_stake;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn stake_nft(
        ctx: Context<StakeNft>,
    ) -> ProgramResult {

        // transfer nft between token accounts
        let cpi_accounts = Transfer {
            from: ctx.accounts.sender_nft_account.to_account_info(),
            to: ctx.accounts.nft_vault.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, 1)?;

        // update stake state
        ctx.accounts.stake.is_active = true;
        ctx.accounts.stake.last_stake_timestamp = Clock::get().unwrap().unix_timestamp as u64;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.stake_nft;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn unstake_nft(
        ctx: Context<UnstakeNft>,
    ) -> ProgramResult {

        // checks stake.num_connections == 0 in account constraints

        // init vault signer
        let stake_address = ctx.accounts.stake.key();
        let (stake_auth_pda, bump_seed) = Pubkey::find_program_address(&[stake_address.as_ref(), STAKE_AUTH_PDA_SEED], ctx.program_id);
        let seeds = &[stake_address.as_ref(), &STAKE_AUTH_PDA_SEED[..], &[bump_seed]];
        let signer = &[&seeds[..]];

        // check pda addy correct
        if stake_auth_pda != ctx.accounts.stake_auth_pda.key() {
            return Err(ErrorCode::InvalidAuthPda.into());
        }

        // transfer
        let cpi_accounts = Transfer {
            from: ctx.accounts.nft_vault.to_account_info(),
            to: ctx.accounts.receiver_nft_account.to_account_info(),
            authority: ctx.accounts.stake_auth_pda.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, 1)?;

        // update stake state
        ctx.accounts.stake.is_active = false;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.unstake_nft;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn initialize_connection(
        ctx: Context<InitializeConnection>,
    ) -> ProgramResult {
        ctx.accounts.connection.is_active = false;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.initialize_connection;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn connect_to_stream(
        ctx: Context<ConnectToStream>,
    ) -> ProgramResult {

        // manually lookup metaplex metadata pda and ensure match with address
        let metadata_program_id: Pubkey = if ctx.accounts.stream.is_simulation {
            ID
        } else {
            spl_token_metadata::ID
        };
        let nft_mint = &ctx.accounts.stake.nft_mint_address;
        let metadata_seeds = &[
            METADATA_PREFIX,
            metadata_program_id.as_ref(),
            nft_mint.as_ref(),
        ];
        let (metaplex_metadata_pda, _bump) = Pubkey::find_program_address(metadata_seeds, &metadata_program_id);
        if metaplex_metadata_pda != ctx.accounts.metaplex_metadata_pda.key() {
            return Err(ErrorCode::InvalidMetaplexMetadataPda.into());
        }

        // load creator vec from metadata and cross check it for any matches with stream verified addresses
        let stream_creator_pubkeys = &ctx.accounts.stream.verified_creator_addresses;
        let metadata = deser_metadata(&ctx.accounts.metaplex_metadata_pda, ctx.accounts.stream.is_simulation)?;
        let creators_vec = metadata.data.creators.as_ref().unwrap();
        let mut found_match = false;
        for creator_pubkey in stream_creator_pubkeys.iter() {
            if creators_vec.iter().any(|c| (c.address == *creator_pubkey) && c.verified) {
                found_match = true;
                break;
            }
        }
        if !found_match {
            return Err(ErrorCode::VerifiedCreatorAddressMismatch.into());
        }

        // proceed with connection logic
        let stream = &mut ctx.accounts.stream;
        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;

        let current_pool_balance = ctx.accounts.token_pool.amount;
        let add_connection = false;
        update_stream_state(stream, current_pool_balance, current_timestamp, add_connection)?;

        // check if is active
        if !stream.is_active {
            return Err(ErrorCode::StreamIsInactive.into());
        }

        // check if max connections will be exceeded
        if (stream.num_connections + 1) > MAX_CONNECTIONS_PER_STREAM {
            return Err(ErrorCode::MaxConnectionsPerStreamExceeded.into());
        }

        // initialize connection
        let connection = &mut ctx.accounts.connection;
        connection.owner_address = ctx.accounts.signer.key();
        connection.stake_address = ctx.accounts.stake.key();
        connection.stream_address = stream.key();
        connection.dao_address = stream.dao_address;
        connection.connection_timestamp = current_timestamp;

        connection.total_earned = 0;
        connection.total_claimed = 0;
        connection.last_update_timestamp = stream.last_update_timestamp;
        connection.is_active = true;

        // update stake
        ctx.accounts.stake.num_connections += 1;

        // update stream
        let add_connection = true;
        update_stream_state(stream, current_pool_balance, current_timestamp, add_connection)?;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.connect_to_stream;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn claim_from_stream(
        ctx: Context<ClaimFromStream>,
        claim_amount: u64,
        claim_max: bool,
    ) -> ProgramResult {

        // update stream
        let stream = &mut ctx.accounts.stream;
        let current_pool_balance = ctx.accounts.token_pool.amount;
        msg!("current pool balance: {:?}", current_pool_balance);
        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
        let add_connection = false;
        update_stream_state(stream, current_pool_balance, current_timestamp, add_connection)?;

        // update connection state (must always be atomically following update stream state)
        let connection = &mut ctx.accounts.connection;

        // use signed integer in case connection was updated more recently than the stream

        let connection_update_lag_seconds = stream.last_update_timestamp as i64 - connection.last_update_timestamp as i64;
        msg!("got connection_update_lag_seconds: {:?}", connection_update_lag_seconds);

        if stream.is_active || (!stream.is_active && (connection_update_lag_seconds > 0)) {
            let recently_earned = stream.stream_rate * connection_update_lag_seconds as u64; // should not be possible to overflow due to business logic
            msg!("calculated recently_earned: {:?}", recently_earned);
            connection.total_earned += recently_earned;
            connection.last_update_timestamp = stream.last_update_timestamp;
        }

        msg!("calculated total earned: {:?}", connection.total_earned);
        msg!("calculated total claimed: {:?}", connection.total_claimed);

        // calculate amount to transfer
        let available_to_claim = connection.total_earned - connection.total_claimed;

        msg!("available to claim from connection: {:?}", available_to_claim);

        let amount_to_transfer;
        if !claim_max {
            msg!("not claiming max...: {:?}", claim_max);
            if claim_amount > available_to_claim {
                return Err(ErrorCode::ClaimAmountExceedsAvailable.into());
            }
            amount_to_transfer = claim_amount;
        } else{
            amount_to_transfer = available_to_claim;
        }

        msg!("amount to transfer: {:?}", amount_to_transfer);

        // get seeds to sign for auth_pda
        let dao_address = ctx.accounts.dao.key();
        let (dao_auth_pda, bump_seed) = Pubkey::find_program_address(&[dao_address.as_ref(), DAO_AUTH_PDA_SEED], ctx.program_id);
        let seeds = &[dao_address.as_ref(), &DAO_AUTH_PDA_SEED[..], &[bump_seed]];
        let signer = &[&seeds[..]];

        // check pda addy correct
        if dao_auth_pda != ctx.accounts.dao_auth_pda.key() {
            return Err(ErrorCode::InvalidAuthPda.into());
        }

        // transfer
        let cpi_accounts = Transfer {
            from: ctx.accounts.token_pool.to_account_info(),
            to: ctx.accounts.receiver_token_account.to_account_info(),
            authority: ctx.accounts.dao_auth_pda.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount_to_transfer)?;

        // update connection
        connection.total_claimed += amount_to_transfer;

        // update stream
        stream.total_claimed += amount_to_transfer;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.claim_from_stream;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;

        Ok(())
    }

    pub fn disconnect_from_stream(
        ctx: Context<DisconnectFromStream>,
    ) -> ProgramResult {

        // TODO remember to atomically claim otherwise those funds are lost

        // update stream
        let stream = &mut ctx.accounts.stream;
        let current_pool_balance = ctx.accounts.token_pool.amount;
        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
        let add_connection = false;
        update_stream_state(stream, current_pool_balance, current_timestamp, add_connection)?;
        stream.num_connections -= 1;

        // update stake
        ctx.accounts.stake.num_connections -= 1;

        // update connection
        ctx.accounts.connection.is_active = false;

        // tx fee
        let signer_handle = &ctx.accounts.signer;
        let fee_rx_handle = &ctx.accounts.fee_receiver_address;
        let fee_lamports = ctx.accounts.fee_controller.disconnect_from_stream;

        transfer_fee(signer_handle, fee_rx_handle, fee_lamports)?;


        Ok(())
    }

    pub fn simulate_create_metadata(
        ctx: Context<SimulateCreateMetadata>,
        verified_creator_address: Pubkey,
    ) -> ProgramResult {

        let space = spl_token_metadata::state::MAX_METADATA_LEN;

        // lookup and verify pda info
        let nft_mint = &ctx.accounts.nft_mint.key();
        let metadata_program_id: Pubkey = ID;
        let metadata_seeds = &[METADATA_PREFIX, metadata_program_id.as_ref(),nft_mint.as_ref()];
        let (pda, bump_seed) = Pubkey::find_program_address(metadata_seeds, &metadata_program_id);
        if pda != ctx.accounts.metadata.key() {
            panic!("wrong pda addy");
        }
        let metadata_seeds = &[METADATA_PREFIX, metadata_program_id.as_ref(),nft_mint.as_ref(), &[bump_seed]];
        let signer = &[&metadata_seeds[..]];

        // create pda
        invoke_signed(
            &create_account(
                ctx.accounts.signer.key,
                &pda,
                1.max(Rent::get()?.minimum_balance(space)),
                space as u64,
                &ID
            ),
            &[
                ctx.accounts.signer.to_account_info().clone(),
                ctx.accounts.metadata.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            signer
        )?;

        let creator = spl_token_metadata::state::Creator {
            address: verified_creator_address,
            verified: true,
            share: 1,
        };
        let creators = vec![creator];
        let mut metadata = spl_token_metadata::state::Metadata::from_account_info(&ctx.accounts.metadata)?;
        let data = spl_token_metadata::state::Data {
            name: "".to_string(),
            symbol: "".to_string(),
            uri: "wutup gangsta mein".to_string(),
            seller_fee_basis_points: 666,
            creators: Some(creators),
        };
        metadata.mint = ctx.accounts.nft_mint.key();
        metadata.key = spl_token_metadata::state::Key::MetadataV1;
        metadata.data = data;
        metadata.is_mutable = true;
        metadata.update_authority = ctx.accounts.signer.key();

        spl_token_metadata::utils::puff_out_data_fields(&mut metadata);
        metadata.serialize(&mut *ctx.accounts.metadata.try_borrow_mut_data().unwrap())?;

        Ok(())
    }

    // pub fn tmp_override_stream(
    //     ctx: Context<ReactivateStream>,
    // ) -> ProgramResult {
    //
    //     // check that signer is a dao councillor
    //     let _owner_index = ctx.accounts.dao.councillors
    //         .iter()
    //         .position(|a| a == ctx.accounts.signer.key)
    //         .ok_or(ErrorCode::InvalidCouncillor)?;
    //
    //     // calculate recent streamed
    //     let stream = &mut ctx.accounts.stream;
    //
    //     let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
    //     stream.last_update_timestamp = current_timestamp;
    //     stream.total_streamed = 600_000_0000;
    //
    //     Ok(())
    // }

}

// Singleton system config
#[derive(Accounts)]
pub struct InitializeFeeController<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
    init,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump,
    payer = signer,
    space = MAX_FEE_CONTROLLER_ACCOUNT_LEN)]
    pub fee_controller: Account<'info, FeeController>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(instruction_name: String, new_fee_amount_lamports: u64)]
pub struct UpdateFeeController<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Account<'info, FeeController>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// DAO instruction contexts
#[derive(Accounts)]
#[instruction(councillors: Vec<Pubkey>, approval_threshold: u64)]
pub struct InitializeDao<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = MAX_DAO_ACCOUNT_LEN)]
    pub dao: Account<'info, Dao>,
    #[account(
        init,
        seeds = [dao.key().as_ref(), DAO_AUTH_PDA_SEED],
        bump,
        payer = signer,
        space = MIN_ACCOUNT_LEN)]
    pub dao_auth_pda: Account<'info, AuthAccount>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(verified_creator_addresses: Vec<Pubkey>, stream_rate: u64, is_simulation: bool)]
pub struct InitializeStream<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = MAX_STREAM_ACCOUNT_LEN)]
    pub stream: Account<'info, Stream>,
    #[account(mut)]
    pub dao: Account<'info, Dao>,
    pub token_mint: Account<'info, Mint>,
    #[account(
        init,
        token::mint = token_mint,
        token::authority = dao_auth_pda,
        seeds = [stream.key().as_ref(), TOKEN_POOL_PDA_SEED],
        bump,
        payer = signer)]
    pub token_pool: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [dao.key().as_ref(), DAO_AUTH_PDA_SEED],
        bump)]
    pub dao_auth_pda: Account<'info, AuthAccount>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction()]
pub struct ReactivateStream<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
    mut,
    constraint = stream.dao_address == dao.key(),
    constraint = stream.is_active == false,
    )]
    pub stream: Box<Account<'info, Stream>>,
    #[account(mut)]
    pub dao: Box<Account<'info, Dao>>,
    #[account(
    mut,
    seeds = [stream.key().as_ref(), TOKEN_POOL_PDA_SEED],
    bump
    )]
    pub token_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ProposeDaoCommand<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub dao: Box<Account<'info, Dao>>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ApproveDaoCommand<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub dao: Box<Account<'info, Dao>>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction()]
pub struct ExecuteUpdateDaoMultisig<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub dao: Box<Account<'info, Dao>>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExecuteDeactivateStream<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub dao: Box<Account<'info, Dao>>,
    #[account(mut, constraint = stream.dao_address == dao.key())]
    pub stream: Account<'info, Stream>,
    #[account(
        mut,
        seeds = [stream.key().as_ref(), TOKEN_POOL_PDA_SEED],
        bump
        )]
    pub token_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction()]
pub struct ExecuteWithdrawFromStream<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub dao: Box<Account<'info, Dao>>,
    #[account(mut, constraint = stream.dao_address == dao.key())]
    pub stream: Account<'info, Stream>,
    #[account(
        mut,
        seeds = [stream.key().as_ref(), TOKEN_POOL_PDA_SEED],
        bump
        )]
    pub token_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub receiver_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [dao.key().as_ref(), DAO_AUTH_PDA_SEED],
        bump)]
    pub dao_auth_pda: Account<'info, AuthAccount>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

// Staker instruction contexts
#[derive(Accounts)]
#[instruction()]
pub struct InitializeStake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        seeds = [signer.key().as_ref(), nft_mint.key().as_ref(), STAKE_PDA_SEED],
        bump,
        payer = signer,
        space = MAX_STAKE_ACCOUNT_LEN)]
    pub stake: Account<'info, Stake>,
    pub nft_mint: Account<'info, Mint>,
    #[account(
        init,
        token::mint = nft_mint,
        token::authority = stake_auth_pda,
        seeds = [stake.key().as_ref(), NFT_VAULT_PDA_SEED],
        bump,
        payer = signer)]
    pub nft_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        seeds = [stake.key().as_ref(), STAKE_AUTH_PDA_SEED],
        bump,
        payer = signer,
        space = MIN_ACCOUNT_LEN)]
    pub stake_auth_pda: Account<'info, AuthAccount>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction()]
pub struct StakeNft<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [signer.key().as_ref(), nft_mint.key().as_ref(), STAKE_PDA_SEED],
        bump,
        constraint = stake.owner_address == signer.key(),
        constraint = stake.is_active == false)]
    pub stake: Account<'info, Stake>,
    pub nft_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [stake.key().as_ref(), NFT_VAULT_PDA_SEED],
        bump,
        constraint = nft_vault.key() == stake.nft_vault_address)]
    pub nft_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = sender_nft_account.owner.key() == signer.key()
    )]
    pub sender_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction()]
pub struct UnstakeNft<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [signer.key().as_ref(), nft_mint.key().as_ref(), STAKE_PDA_SEED],
        bump,
        constraint = stake.owner_address == signer.key(),
        constraint = stake.is_active == true,
        constraint = stake.num_connections == 0,
    )]
    pub stake: Account<'info, Stake>,
    pub nft_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [stake.key().as_ref(), NFT_VAULT_PDA_SEED],
        bump,
        constraint = nft_vault.key() == stake.nft_vault_address)]
    pub nft_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = receiver_nft_account.owner.key() == signer.key()
        )]
    pub receiver_nft_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [stake.key().as_ref(), STAKE_AUTH_PDA_SEED],
        bump)]
    pub stake_auth_pda: Account<'info, AuthAccount>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction()]
pub struct InitializeConnection<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        seeds = [stake.key().as_ref(), stream.key().as_ref(), CONNECTION_PDA_SEED],
        bump,
        payer = signer,
        space = MAX_SUBSCRIPTION_ACCOUNT_LEN)]
    pub connection: Account<'info, Connection>,
    #[account(
        mut,
        seeds = [signer.key().as_ref(), stake.nft_mint_address.as_ref(), STAKE_PDA_SEED],
        bump,
        constraint = stake.owner_address == signer.key(),
        constraint = stake.is_active == true,
        )]
    pub stake: Account<'info, Stake>,
    #[account(mut)]
    pub stream: Account<'info, Stream>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction()]
pub struct ConnectToStream<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [stake.key().as_ref(), stream.key().as_ref(), CONNECTION_PDA_SEED],
        bump,
        constraint = connection.is_active == false,
        )]
    pub connection: Account<'info, Connection>,
    #[account(
        mut,
        seeds = [signer.key().as_ref(), stake.nft_mint_address.as_ref(), STAKE_PDA_SEED],
        bump,
        constraint = stake.owner_address == signer.key(),
        constraint = stake.is_active == true,
        )]
    pub stake: Account<'info, Stake>,
    #[account(mut)]
    pub stream: Account<'info, Stream>,
    pub metaplex_metadata_pda: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [stream.key().as_ref(), TOKEN_POOL_PDA_SEED],
        bump
        )]
    pub token_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimFromStream<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
    mut,
    seeds = [stake.key().as_ref(), stream.key().as_ref(), CONNECTION_PDA_SEED],
    bump,
    constraint = connection.stream_address == stream.key(),
    constraint = connection.owner_address == signer.key(),
    constraint = connection.stake_address == stake.key(),
    constraint = connection.is_active == true,
    )]
    pub connection: Box<Account<'info, Connection>>,
    #[account(
    mut,
    seeds = [signer.key().as_ref(), stake.nft_mint_address.as_ref(), STAKE_PDA_SEED],
    bump,
    constraint = stake.owner_address == signer.key(),
    constraint = stake.is_active == true,
    )]
    pub stake: Box<Account<'info, Stake>>,
    #[account(mut, constraint = stream.dao_address == dao.key())]
    pub stream: Box<Account<'info, Stream>>,
    #[account(
    mut,
    seeds = [stream.key().as_ref(), TOKEN_POOL_PDA_SEED],
    bump)]
    pub token_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub receiver_token_account: Account<'info, TokenAccount>,
    #[account(
    mut,
    seeds = [dao.key().as_ref(), DAO_AUTH_PDA_SEED],
    bump)]
    pub dao_auth_pda: Account<'info, AuthAccount>,
    #[account(mut)]
    pub dao: Box<Account<'info, Dao>>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DisconnectFromStream<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [stake.key().as_ref(), stream.key().as_ref(), CONNECTION_PDA_SEED],
        bump,
        constraint = connection.stream_address == stream.key(),
        constraint = connection.owner_address == signer.key(),
        constraint = connection.stake_address == stake.key(),
        constraint = connection.is_active == true,
        )]
    pub connection: Box<Account<'info, Connection>>,
    #[account(
        mut,
        seeds = [signer.key().as_ref(), stake.nft_mint_address.as_ref(), STAKE_PDA_SEED],
        bump,
        constraint = stake.owner_address == signer.key(),
        constraint = stake.is_active == true,
        )]
    pub stake: Box<Account<'info, Stake>>,
    #[account(mut, constraint = stream.dao_address == dao.key())]
    pub stream: Box<Account<'info, Stream>>,
    #[account(
        mut,
        seeds = [stream.key().as_ref(), TOKEN_POOL_PDA_SEED],
        bump)]
    pub token_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub receiver_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [dao.key().as_ref(), DAO_AUTH_PDA_SEED],
        bump)]
    pub dao_auth_pda: Account<'info, AuthAccount>,
    #[account(mut)]
    pub dao: Box<Account<'info, Dao>>,
    #[account(mut)]
    pub fee_receiver_address: AccountInfo<'info>,
    #[account(
    mut,
    seeds = [FEE_CONTROLLER_PDA_SEED],
    bump)]
    pub fee_controller: Box<Account<'info, FeeController>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SimulateCreateMetadata<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub metadata: AccountInfo<'info>,
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// DAO structs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, FromPrimitive)]
pub enum ProposalType {
    UpdateMultisig = 1,
    DeactivateStream = 2,
    WithdrawFromStream = 3,
}

impl Default for ProposalType {
    fn default() -> Self {
        ProposalType::DeactivateStream
    }
}

#[account]
#[derive(Default)]
pub struct Dao {
    // config
    pub councillors: Vec<Pubkey>,
    pub approval_threshold: u64,
    // proposal state
    pub proposal_signers: Vec<bool>,
    pub proposal_is_active: bool,
    pub proposal_type: ProposalType, // 4 bytes
    // update multisig proposal params
    pub proposed_councillors: Vec<Pubkey>,
    pub proposed_approval_threshold: u64,
    // deactivate stream proposal params
    pub proposed_deactivation_stream: Pubkey,
    // withdraw from  stream proposal params
    pub proposed_withdrawal_amount: u64,
    pub proposed_withdrawal_receiver_owner: Pubkey,
    pub proposed_withdrawal_stream: Pubkey,
    // stream state
    pub num_streams: u64,
}

#[account]
#[derive(Default)]
pub struct Stream {
    // config
    pub dao_address: Pubkey,
    pub token_mint_address: Pubkey,
    pub token_pool_address: Pubkey,
    pub verified_creator_addresses: Vec<Pubkey>,
    pub stream_rate: u64, // all amounts in absolute token units, rate is per connection per second
    pub is_simulation: bool,
    // state
    pub is_active: bool,
    pub num_connections: u64,
    pub total_streamed: u64,
    pub total_claimed: u64,
    pub last_update_timestamp: u64,
}

// Staker structs
#[account]
#[derive(Default)]
pub struct Stake {
    // config
    pub owner_address: Pubkey,
    pub nft_mint_address: Pubkey,
    pub nft_vault_address: Pubkey,
    // state
    pub is_active: bool,
    pub num_connections: u64,
    pub cumulative_seconds_staked: u64,
    pub last_stake_timestamp: u64,
}

#[account]
#[derive(Default)]
pub struct Connection {
    // config
    pub owner_address: Pubkey,
    pub stake_address: Pubkey,
    pub stream_address: Pubkey,
    pub dao_address: Pubkey,
    pub connection_timestamp: u64,
    // state
    pub total_earned: u64,
    pub total_claimed: u64,
    pub last_update_timestamp: u64,
    pub is_active: bool,
}

#[account]
#[derive(Default)]
pub struct FeeController {
    pub initialize_dao: u64,
    pub initialize_stream: u64,
    pub reactivate_stream: u64,
    pub propose_dao_command: u64,
    pub approve_dao_command: u64,
    pub execute_update_dao_multisig: u64,
    pub execute_deactivate_stream: u64,
    pub execute_withdraw_from_stream: u64,
    pub initialize_stake: u64,
    pub stake_nft: u64,
    pub unstake_nft: u64,
    pub initialize_connection: u64,
    pub connect_to_stream: u64,
    pub claim_from_stream: u64,
    pub disconnect_from_stream: u64,
}

#[account]
#[derive(Default)]
pub struct AuthAccount {}

// utils
fn update_stream_state(stream: &mut Account<Stream>,
                           current_pool_balance: u64,
                           current_timestamp: u64,
                           add_connection: bool) -> ProgramResult {

    if !stream.is_active {
        return Ok(())
    }

    // calculate amount streamed since last update
    let seconds_since_last_update = current_timestamp - stream.last_update_timestamp;
    let recent_amount_streamed: u128 = (seconds_since_last_update as u128 * stream.stream_rate as u128) * stream.num_connections as u128;

    // check if empty
    let total_streamed = stream.total_streamed;
    let total_claimed = stream.total_claimed;

    let new_total_streamed:u128 = total_streamed as u128 + recent_amount_streamed;
    let new_total_unclaimed = new_total_streamed - total_claimed as u128;

    if new_total_unclaimed > current_pool_balance as u128 {

        let total_unclaimed = total_streamed - total_claimed;
        let current_pool_surplus = current_pool_balance - total_unclaimed;
        let total_stream_rate = stream.stream_rate * stream.num_connections; // tokens per second
        let runway_since_last_update_seconds = current_pool_surplus / total_stream_rate;

        let new_reduced_total_streamed = total_stream_rate * runway_since_last_update_seconds;  // should not be possible for this to overflow given business logic
        msg!("Got current_surplus: {:?} and new_reduced_total_streamed: {:?}", current_pool_surplus, new_reduced_total_streamed);

        stream.total_streamed += new_reduced_total_streamed;
        stream.last_update_timestamp += runway_since_last_update_seconds;
        stream.is_active = false;

    } else {

        // proceed with update
        stream.total_streamed = new_total_streamed as u64;
        stream.last_update_timestamp = current_timestamp;
    }

    if add_connection {
        stream.num_connections += 1;
    }

    Ok(())
}

pub fn check_owner(info: &AccountInfo, is_simulation: bool) -> ProgramResult {
    let actual_owner = *info.owner;
    let expected_owner = if is_simulation {
        ID
    } else {
        spl_token_metadata::ID
    };
    if actual_owner != expected_owner {
        return Err(ErrorCode::InvalidAccountOwner.into());
    }
    Ok(())
}

pub fn deser_metadata(info: &AccountInfo, is_simulation: bool) -> core::result::Result<spl_token_metadata::state::Metadata, ProgramError> {
    check_owner(info, is_simulation)?;
    let data: &[u8] = &info.try_borrow_data()?;
    let md = spl_token_metadata::utils::try_from_slice_checked(
        data,
        spl_token_metadata::state::Key::MetadataV1,
        spl_token_metadata::state::MAX_METADATA_LEN)?;
    Ok(md)
}

pub fn validate_proposal_approval(dao: &mut Account<Dao>, signer_pubkey: &Pubkey) -> ProgramResult {
    let _councillor_index = dao.councillors
        .iter()
        .position(|a| a == signer_pubkey)
        .ok_or(ErrorCode::InvalidCouncillor)?;
    if !dao.proposal_is_active {
        return Err(ErrorCode::ProposalNotActive.into());
    }
    // calculate total signers and ensure meets threshold
    let mut num_signers = 0;
    for i in 0..dao.councillors.len() {
        if dao.proposal_signers[i] {
            num_signers += 1;
        }
    }
    if num_signers < dao.approval_threshold {
        return Err(ErrorCode::NotEnoughSignersApproved.into());
    }
    Ok(())
}

pub fn transfer_fee<'a>(signer: &Signer<'a>, fee_rx_acct_info: &AccountInfo<'a>, fee_lamports: u64) -> ProgramResult {

    // check sufficient balance to pay initialization fee
    let user_balance = signer.lamports();
    if user_balance < fee_lamports {
        return Err(ErrorCode::InsufficientFeeFunds.into());
    }

    // validate fee receiver ata
    let fee_rx_address: Pubkey = Pubkey::from_str(FEE_RX_ADDRESS).unwrap();
    if fee_rx_acct_info.key() != fee_rx_address {
    return Err(ErrorCode::InvalidFeeRxAddress.into());
    }

    let ix = transfer(signer.key, fee_rx_acct_info.key, fee_lamports);

    invoke(&ix,
           &[
               signer.to_account_info(),
               fee_rx_acct_info.to_account_info(),
           ])?;

    Ok(())
}

// custom errors
#[error]
pub enum ErrorCode {
    #[msg("Num councillors exceeds max.")]
    TooManyManagers,
    #[msg("Invalid approval threshold.")]
    InvalidApprovalThreshold,
    #[msg("Too many verified creator addresses.")]
    TooManyVerifiedCreatorAddresses,
    #[msg("Stream is not active or would be deactivated on update.")]
    StreamIsInactive,
    #[msg("Requested claim amount exceeds available funds.")]
    ClaimAmountExceedsAvailable,
    #[msg("Invalid Authorizer PDA.")]
    InvalidAuthPda,
    #[msg("Nft is not staked.")]
    NftNotStaked,
    #[msg("Invalid verified creator address.")]
    InvalidVerifiedCreatorAddress,
    #[msg("Invalid metaplex metadata pda.")]
    InvalidMetaplexMetadataPda,
    #[msg("Verified creator address mismatch.")]
    VerifiedCreatorAddressMismatch,
    #[msg("Invalid account owner.")]
    InvalidAccountOwner,
    #[msg("Invalid councillor.")]
    InvalidCouncillor,
    #[msg("Invalid proposal type.")]
    InvalidProposalType,
    #[msg("Proposal is not active.")]
    ProposalNotActive,
    #[msg("Not enough signers have approved.")]
    NotEnoughSignersApproved,
    #[msg("Mismatches proposal type.")]
    MismatchProposalType,
    #[msg("Invalid proposed receiver owner.")]
    InvalidProposedReceiverOwner,
    #[msg("Insufficient funds.")]
    InsufficientFunds,
    #[msg("Stream mismatch.")]
    StreamMismatch,
    #[msg("Max supported decimals for spl-token mint exceeded.")]
    MaxSupportedDecimalsExceeded,
    #[msg("Max supported stream rate exceeded.")]
    MaxSupportedStreamRateExceeded,
    #[msg("Max connections per stream exceeded.")]
    MaxConnectionsPerStreamExceeded,
    #[msg("Insufficient fee funds.")]
    InsufficientFeeFunds,
    #[msg("Invalid fee receiver address")]
    InvalidFeeRxAddress,
    #[msg("Invalid fee mod address")]
    InvalidFeeModAddress,
    #[msg("Invalid instruction name")]
    InvalidInstructionName,
}