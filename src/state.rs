//! State transition types
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
    program_error::{ProgramError},
    account_info:: AccountInfo,
    borsh::try_from_slice_unchecked,
};


/// Initializeing solana stream states
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct NftDetails{
    pub number_of_tokens: u64,
    pub price: u64,
   pub  nft_mint: Pubkey,
   pub  nft_escrow: Pubkey,
   pub token_mint: Pubkey,
   pub create_at: u64
}