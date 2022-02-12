//! Instruction types
use solana_program::{
    program_error::ProgramError,
};
use {borsh::{BorshDeserialize}};

use crate::{
    error::TokenError,
};
use std::convert::TryInto;

pub struct ProcessDeposit{
    pub number_of_tokens: u64,
    pub price: u64,
}
pub struct ProcessBuy{
    pub token: u64,
}
pub enum TokenInstruction {
    ProcessDeposit(ProcessDeposit),
    ProcessBuy(ProcessBuy),
}
impl TokenInstruction {
    /// Unpacks a byte buffer into a [TokenInstruction](enum.TokenInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TokenError::InvalidInstruction;
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            // Initialize deposit NFT instruction 
            0 => {
                let (number_of_tokens, rest) = rest.split_at(8);
                let (price, _rest) = rest.split_at(8);
                let number_of_tokens = number_of_tokens.try_into().map(u64::from_le_bytes).or(Err(InvalidInstruction))?;
                let price = price.try_into().map(u64::from_le_bytes).or(Err(InvalidInstruction))?;
                Self::ProcessDeposit(ProcessDeposit{number_of_tokens,price})
            }
            1 => {
                let (number_of_tokens, rest) = rest.split_at(8);
                let token = number_of_tokens.try_into().map(u64::from_le_bytes).or(Err(InvalidInstruction))?;
                Self::ProcessBuy(ProcessBuy{token})
            }
            _ => return Err(TokenError::InvalidInstruction.into()),
        })
    }
}