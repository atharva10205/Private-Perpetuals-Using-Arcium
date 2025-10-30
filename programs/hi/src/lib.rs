use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use arcium_anchor::prelude::*; 
const COMP_DEF_OFFSET_OPEN_POSITION: u32 = comp_def_offset("open_position_v1");
const COMP_DEF_OFFSET_CLOSE_POSITION: u32 = comp_def_offset("close_position_v1");
declare_id!("HVFgsYknF4UZuTTeHBpQFZYGvHjYK347mtxSfhseJ2ir");

#[program]
pub mod private_perps {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_bump: u8,
        fee_bps: u64,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.authority = ctx.accounts.authority.key();
        market.bump = market_bump;
        market.fee_bps = fee_bps;
        market.total_open_interest = 0;
        Ok(())
    }

       pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let user = &mut ctx.accounts.user_account;
        user.collateral = user
            .collateral
            .checked_add(amount)
            .ok_or(PerpError::Overflow)?;
        Ok(())
    }

        pub fn submit_encrypted_order(
        ctx: Context<SubmitEncryptedOrder>,
        arcium_computation_id: [u8; 32],
        commitment: [u8; 32],
        circuit_type: u8, 
    ) -> Result<()> {
        let req = &mut ctx.accounts.computation_request;
        req.owner = ctx.accounts.user.key();
        req.market = ctx.accounts.market.key();
        req.arcium_computation_id = arcium_computation_id;
        req.commitment = commitment;
        req.status = ComputationStatus::Pending;
        req.applied_nonce = 0;
        req.arcium_receipt = Vec::new();

        req.circuit_offset = match circuit_type {
            0 => COMP_DEF_OFFSET_OPEN_POSITION,
            1 => COMP_DEF_OFFSET_CLOSE_POSITION,
            _ => return err!(PerpError::InvalidCircuitType),
        };

        Ok(())
    }

   
    pub fn apply_encrypted_result(
        ctx: Context<ApplyEncryptedResult>,
        arcium_computation_id: [u8; 32],
        result_fill_amount: u64,
        result_price: u64,
        result_side: u8, 
        result_nonce: u64,
        arcium_receipt: Vec<u8>,
    ) -> Result<()> {
        let req = &mut ctx.accounts.computation_request;

        require!(
            req.status == ComputationStatus::Pending,
            PerpError::ComputationNotPending
        );
        require!(
            req.arcium_computation_id == arcium_computation_id,
            PerpError::ComputationIdMismatch
        );


        let market = &mut ctx.accounts.market;
        market.total_open_interest = market
            .total_open_interest
            .checked_add(result_fill_amount)
            .ok_or(PerpError::Overflow)?;

        let position = &mut ctx.accounts.position;
        let old_size = position.size;
        let new_size = old_size
            .checked_add(result_fill_amount)
            .ok_or(PerpError::Overflow)?;

        let new_avg_price = if old_size == 0 {
            result_price
        } else {
            let left = (old_size as u128)
                .checked_mul(position.avg_price as u128)
                .ok_or(PerpError::Overflow)?;
            let right = (result_fill_amount as u128)
                .checked_mul(result_price as u128)
                .ok_or(PerpError::Overflow)?;
            let sum = left.checked_add(right).ok_or(PerpError::Overflow)?;
            (sum / (new_size as u128)) as u64
        };

        position.owner = ctx.accounts.user.key();
        position.size = new_size;
        position.avg_price = new_avg_price;

        req.status = ComputationStatus::Applied;
        req.applied_nonce = result_nonce;
        req.arcium_receipt = arcium_receipt;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ComputationStatus {
    Pending,
    Applied,
    Cancelled,
}

#[account]
pub struct Market {
    pub authority: Pubkey,
    pub bump: u8,
    pub fee_bps: u64,
    pub total_open_interest: u64,
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub collateral: u64,
}

#[account]
pub struct Position {
    pub owner: Pubkey,
    pub size: u64,
    pub avg_price: u64,
}

#[account]
pub struct ComputationRequest {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub arcium_computation_id: [u8; 32],
    pub commitment: [u8; 32],
    pub status: ComputationStatus,
    pub applied_nonce: u64,
    pub arcium_receipt: Vec<u8>,
    pub circuit_offset: u32, 
}

const PUBKEY_SIZE: usize = 32;
const U8_SIZE: usize = 1;
const U64_SIZE: usize = 8;
const U32_SIZE: usize = 4;
const ENUM_SIZE: usize = 1;
const VEC_PREFIX: usize = 4;
const MAX_RECEIPT_LEN: usize = 1024;

fn market_space() -> usize {
    8 + PUBKEY_SIZE + U8_SIZE + U64_SIZE + U64_SIZE
}

fn user_account_space() -> usize {
    8 + PUBKEY_SIZE + U64_SIZE
}

fn position_space() -> usize {
    8 + PUBKEY_SIZE + U64_SIZE + U64_SIZE
}

fn computation_request_space() -> usize {
    8 + PUBKEY_SIZE + PUBKEY_SIZE + 32 + 32 + ENUM_SIZE + U64_SIZE + VEC_PREFIX + MAX_RECEIPT_LEN + U32_SIZE
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(init, payer = authority, space = market_space())]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, has_one = owner)]
    pub user_account: Account<'info, UserAccount>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitEncryptedOrder<'info> {
    #[account(init_if_needed, payer = user, space = computation_request_space())]
    pub computation_request: Account<'info, ComputationRequest>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApplyEncryptedResult<'info> {
    #[account(mut, has_one = market)]
    pub computation_request: Account<'info, ComputationRequest>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(init_if_needed, payer = user, space = position_space())]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[error_code]
pub enum PerpError {
    #[msg("Computation request not pending")]
    ComputationNotPending,
    #[msg("Computation ID mismatch")]
    ComputationIdMismatch,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid circuit type")]
    InvalidCircuitType,
}
