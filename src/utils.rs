use solana_program::{
    pubkey::Pubkey,
    account_info::{AccountInfo},
    system_instruction,
    program::{invoke_signed,invoke},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack
};


use arrayref::array_ref;

pub fn create_account<'a>(
    payer: &AccountInfo<'a>,
    amount: u64,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
) -> ProgramResult {
        invoke(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                amount,
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                system_program.clone(),
            ],
        )
    }

pub fn create_account_signed<'a>(
    payer: &AccountInfo<'a>,
    amount: u64,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
    seeds: &[&[u8]],
) -> ProgramResult {
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                amount,
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                system_program.clone(),
            ],
            &[seeds],
        )
    }
 pub fn generate_pda_and_bump_seed(
        prefix: &str,
        sender: &Pubkey,
        pda: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                prefix.as_bytes(),
                &sender.to_bytes(),
                &pda.to_bytes()
            ],
            program_id,
        )
}

pub fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    amount: u64,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
) -> ProgramResult {
        invoke(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                amount,
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                system_program.clone(),
            ],
        )
    }
    pub fn check_data_len(data: &[u8], min_len: usize) -> Result<(), ProgramError> {
        if data.len() < min_len {
            Err(ProgramError::AccountDataTooSmall)
        } else {
            Ok(())
        }
    }
    pub fn get_token_balance(token_account: &AccountInfo) -> Result<u64, ProgramError> {
        let data = token_account.try_borrow_data()?;
        check_data_len(&data, spl_token::state::Account::get_packed_len())?;
        let amount = array_ref![data, 64, 8];
        Ok(u64::from_le_bytes(*amount))
    }
    