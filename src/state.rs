//! State transition types
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
};


/// Initializeing solana stream states
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct NftDetails{
    pub number_of_tokens: u64,
    pub price: u64,
    pub nft_mint: Pubkey,
    pub nft_escrow: Pubkey,
    pub token_mint: Pubkey,
    pub create_at: u64,
    pub days: f64,
    pub remaining_token:u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CoinFlip{
    pub won: u64,
    pub address: Pubkey,
    pub amount: u64,
}
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Auction{
    pub max_price: u64,
    pub max_payer: Pubkey,
    pub num_tokens: u64,
    pub day:u64,
}