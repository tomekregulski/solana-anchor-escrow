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

// AccountInfo vs. Account:
//
// It seems proper to use Account over AccountInfo when you want Anchor to deserialize the data for convenience. 
// In that case, you can access the account data via a trivial method call. For example: ctx.accounts.vault_account.mint

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub initializer: AccountInfo<'info>, // Signer of InitialEscrow instruction. To be stored in EscrowAccount
    pub mint: Account<'info, Mint>, // The account of token account for token exchange. To be stored in EscrowAccount
    pub vault_account: Account<'info, TokenAccount>, // The account of token account for token exchange. To be stored in EscrowAccount
    pub initializer_deposit_token_account: Account<'info, TokenAccount>, // The account of TokenProgram
    pub initializer_receive_token_account: Account<'info, TokenAccount>, // The account of EscrowAccount
    pub escrow_account: Box<Account<'info, EscrowAccount>>, // The account of Vault, which is created by Anchor via constraints.
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    pub initializer: AccountInfo<'info>, // The initializer of EscrowAccount
    pub initializer_deposit_token_account: Account<'info, TokenAccount>, // The address of token account for token exchange
    pub vault_account: Account<'info, TokenAccount>, // The program derived address
    pub vault_authority: AccountInfo<'info>, // The program derived address
    pub escrow_account: Box<Account<'info, EscrowAccount>>, // The address of EscrowAccount. Have to check if the EscrowAccount follows certain constraints.
    pub token_program: AccountInfo<'info>, // The address of TokenProgram
}

#[derive(Accounts)]
pub struct Exchange<'info> {
    pub taker: AccountInfo<'info>, // Singer of Exchange instruction
    pub taker_deposit_token_account: Account<'info, TokenAccount>, // Token account for token exchange
    pub taker_receive_token_account: Account<'info, TokenAccount>, // Token account for token exchange
    pub initializer_deposit_token_account: Account<'info, TokenAccount>, // Token account for token exchange
    pub initializer_receive_token_account: Account<'info, TokenAccount>, // Token account for token exchange
    pub initializer: AccountInfo<'info>, // To be used in constraints
    pub escrow_account: Box<Account<'info, EscrowAccount>>, // The address of EscrowAccount. Have to check if the EscrowAccount follows certain constraints.
    pub vault_account: Account<'info, TokenAccount>, // The program derived address
    pub vault_authority: AccountInfo<'info>, // The program derived address
    pub token_program: AccountInfo<'info>, // The address of TokenProgram
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