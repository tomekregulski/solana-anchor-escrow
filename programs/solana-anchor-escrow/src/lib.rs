use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

declare_id!("BgBM8rneuxRfido2bBwapiJYPVddQszTJhcjJkaHQV83");

#[program]
pub mod solana_anchor_escrow {
    use super::*;
    // Initialize can be considered as a wrapper for instructions. This wrapper is enhanced by Anchor via derived macro (#[derive(account)]). 
    pub fn initialize(
        ctx: Context<Initialize>, 
        _vault_account_bump: u8, 
        initializer_amount: u64, 
        taker_amount: u64
    ) -> ProgramResult {
        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> ProgramResult {
        // TODO
        Ok(())
    }

    pub fn exchange(ctx: Context<Exchange>) -> ProgramResult {
        // TODO
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // TODO
}

#[derive(Accounts)]
pub struct Exchange<'info> {
    // TODO
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    // TODO
}

// we design an account that stores the minimum information to validate the escrow state and keep the integrity of the program:
#[account]
pub struct EscrowAccount {
    pub initializer_key: Pubkey, // To authorize the actions properly
    pub initializer_deposit_token_account: Pubkey, // To record the deposit account of initialzer
    pub initializer_receive_token_account: Pubkey, // To record the receiving account of initializer
    pub initializer_amount: u64, // To record how much token should the initializer transfer to taker
    pub taker_amount: u64, // To record how much token should the initializer receive from the taker
}