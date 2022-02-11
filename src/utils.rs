use solana_program::{
    pubkey::Pubkey,
    account_info::{AccountInfo},
    system_instruction,
    program::{invoke_signed,invoke},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack
};
use super::error::TokenError;

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