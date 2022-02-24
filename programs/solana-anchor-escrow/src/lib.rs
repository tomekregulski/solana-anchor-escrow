use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

declare_id!("BgBM8rneuxRfido2bBwapiJYPVddQszTJhcjJkaHQV83");

#[program]
pub mod solana_anchor_escrow {
    use super::*;
    const ESCROW_PDA_SEED: &[u8] = b"escrow";
    // Initialize can be considered as a wrapper for instructions. This wrapper is enhanced by Anchor via derived macro (#[derive(account)]). 
    
    // In initialize, what happens is that the input accounts are assigned to EscrowAccount fields one by one. Then, a program derived address, or PDA, is derived to be going to become new authority of initializer_deposit_token_account.
    pub fn initialize(
        ctx: Context<Initialize>, 
        _vault_account_bump: u8, 
        initializer_amount: u64, 
        taker_amount: u64
    ) -> ProgramResult {
        ctx.accounts.escrow_account.initializer_key = *ctx.accounts.initializer.key;
        ctx.accounts
            .escrow_account
            .initializer_deposit_token_account = *ctx
            .accounts
            .initializer_deposit_token_account
            .to_account_info()
            .key;
        ctx.accounts
            .escrow_account
            .initializer_receive_token_account = *ctx
            .accounts
            .initializer_receive_token_account
            .to_account_info()
            .key;
        ctx.accounts.escrow_account.initializer_amount = initializer_amount;
        ctx.accounts.escrow_account.taker_amount = taker_amount;

        let (vault_authority, _vault_authority_bump) =
            Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        token::set_authority(
            ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(vault_authority),
        )?;

        token::transfer(
            ctx.accounts.into_transfer_to_pda_context(),
            ctx.accounts.escrow_account.initializer_amount,
        )?;

        Ok(())
    }

    // In cancel, it just simply reset the authority from PDA back to the initializer.
    pub fn cancel(ctx: Context<Cancel>) -> ProgramResult {
        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        let authority_seeds = &[&ESCROW_PDA_SEED[..], &[vault_authority_bump]];

        token::transfer(
            ctx.accounts
                .into_transfer_to_initializer_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_account.initializer_amount,
        )?;

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        Ok(())
    }

    // In exchange, 3 things happen:
    //      First, token A gets transfered from pda_deposit_token_account to taker_receive_token_account.
    //      Next, token B gets transfered from taker_deposit_token_account to initializer_receive_token_account.
    //      Finally, the authority of pda_deposit_token_account gets set back to the initializer.
    //
    pub fn exchange(ctx: Context<Exchange>) -> ProgramResult {
        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        let authority_seeds = &[&ESCROW_PDA_SEED[..], &[vault_authority_bump]];

        token::transfer(
            ctx.accounts.into_transfer_to_initializer_context(),
            ctx.accounts.escrow_account.taker_amount,
        )?;

        token::transfer(
            ctx.accounts
                .into_transfer_to_taker_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_account.initializer_amount,
        )?;

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        Ok(())
    }
}

// AccountInfo vs. Account:
//
// It seems proper to use Account over AccountInfo when you want Anchor to deserialize the data for convenience. 
// In that case, you can access the account data via a trivial method call. For example: ctx.accounts.vault_account.mint

#[derive(Accounts)]
#[instruction(vault_account_bump: u8, initializer_amount: u64)]
pub struct Initialize<'info> {
    #[account(mut, signer)] // Checks the given account signed the transaction
    pub initializer: AccountInfo<'info>, // Signer of InitialEscrow instruction. To be stored in EscrowAccount
    pub mint: Account<'info, Mint>, // The account of token account for token exchange. To be stored in EscrowAccount
    #[account(
        init,
        seeds = [b"token-seed".as_ref()],
        bump = vault_account_bump,
        payer = initializer,
        token::mint = mint,
        token::authority = initializer,
    )]  // Notice that we used a rather complex constraint to create an token account that has a PDA key
    pub vault_account: Account<'info, TokenAccount>, // The account of token account for token exchange. To be stored in EscrowAccount
    #[account(
        mut,
        constraint = initializer_deposit_token_account.amount >= initializer_amount
    )] // Executes the given code as a constraint. The expression should evaluate to a boolean
    pub initializer_deposit_token_account: Account<'info, TokenAccount>, // The account of TokenProgram
    pub initializer_receive_token_account: Account<'info, TokenAccount>, // The account of EscrowAccount
    #[account(zero)]
    pub escrow_account: Box<Account<'info, EscrowAccount>>, // The account of Vault, which is created by Anchor via constraints.
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    #[account(mut, signer)] 
    pub initializer: AccountInfo<'info>, // The initializer of EscrowAccount
    #[account(mut)] // Marks the account as mutable and persists the state transition
    pub vault_account: Account<'info, TokenAccount>, // The program derived address
    pub vault_authority: AccountInfo<'info>, // The program derived address
    #[account(mut)]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>, // The address of token account for token exchange
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key,
        constraint = escrow_account.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        close = initializer
    )] // close = <target\> // Marks the account as being closed at the end of the instruction’s execution, sending the rent exemption lamports to the specified
    pub escrow_account: Box<Account<'info, EscrowAccount>>, // The address of EscrowAccount. Have to check if the EscrowAccount follows certain constraints.
    pub token_program: AccountInfo<'info>, // The address of TokenProgram
}

#[derive(Accounts)]
pub struct Exchange<'info> {
    #[account(signer)] 
    pub taker: AccountInfo<'info>, // Singer of Exchange instruction
    #[account(mut)]
    pub taker_deposit_token_account: Account<'info, TokenAccount>, // Token account for token exchange
    #[account(mut)]
    pub taker_receive_token_account: Account<'info, TokenAccount>, // Token account for token exchange
    #[account(mut)]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>, // Token account for token exchange
    #[account(mut)]
    pub initializer_receive_token_account: Account<'info, TokenAccount>, // Token account for token exchange
    #[account(mut)]
    pub initializer: AccountInfo<'info>, // To be used in constraints
    #[account(
        mut,
        constraint = escrow_account.taker_amount <= taker_deposit_token_account.amount,
        constraint = escrow_account.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        constraint = escrow_account.initializer_receive_token_account == *initializer_receive_token_account.to_account_info().key,
        constraint = escrow_account.initializer_key == *initializer.key,
        close = initializer
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>, // The address of EscrowAccount. Have to check if the EscrowAccount follows certain constraints.
    #[account(mut)]
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


impl<'info> Initialize<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .initializer_deposit_token_account
                .to_account_info()
                .clone(),
            to: self.vault_account.to_account_info().clone(),
            authority: self.initializer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.vault_account.to_account_info().clone(),
            current_authority: self.initializer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

// Utils
//
impl<'info> Cancel<'info> {
    fn into_transfer_to_initializer_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault_account.to_account_info().clone(),
            to: self
                .initializer_deposit_token_account
                .to_account_info()
                .clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.vault_account.to_account_info().clone(),
            destination: self.initializer.clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

impl<'info> Exchange<'info> {
    fn into_transfer_to_initializer_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.taker_deposit_token_account.to_account_info().clone(),
            to: self
                .initializer_receive_token_account
                .to_account_info()
                .clone(),
            authority: self.taker.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_taker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault_account.to_account_info().clone(),
            to: self.taker_receive_token_account.to_account_info().clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.vault_account.to_account_info().clone(),
            destination: self.initializer.clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}