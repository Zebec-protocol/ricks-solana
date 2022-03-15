//! Instruction types
use solana_program::{
    program_error::ProgramError,
    msg
};


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
    pub price: u64,
}
pub struct ProcessBuy2{
    pub day:u64,
}
pub struct ProcessCoinFlip{
    pub token: u64,
}
pub struct ProcessClaimCoinFlip{
    pub token: u64,
}
pub struct ProcessAuction1{
    pub number_of_tokens: u64,
    pub price: u64,
}
pub enum TokenInstruction {
    ProcessDeposit(ProcessDeposit),
    ProcessBuy(ProcessBuy),
    ProcessBuy2(ProcessBuy2),
    ProcessCoinFlip(ProcessCoinFlip),
    ProcessClaimCoinFlip(ProcessClaimCoinFlip),
    ProcessAuction1(ProcessAuction1)
}
impl TokenInstruction {
    /// Unpacks a byte buffer into a [TokenInstruction](enum.TokenInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TokenError::InvalidInstruction;
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        msg!("{:?}",input);
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
                let (token, rest) = rest.split_at(8);
                let token = token.try_into().map(u64::from_le_bytes).or(Err(InvalidInstruction))?;
                let (price, _rest) = rest.split_at(8);
                let price = price.try_into().map(u64::from_le_bytes).or(Err(InvalidInstruction))?;
                Self::ProcessBuy(ProcessBuy{token,price})
            }
            2 => {
                let (day, _rest) = rest.split_at(8);
                let day= day.try_into().map(u64::from_le_bytes).or(Err(InvalidInstruction))?;
                Self::ProcessBuy2(ProcessBuy2{day})
            }
            3 => {
                msg!("{:?}",rest);
                let token = rest[0] as u64;
                Self::ProcessCoinFlip(ProcessCoinFlip{token})
            }
            4 => {
                let token = rest[0] as u64;
                Self::ProcessClaimCoinFlip(ProcessClaimCoinFlip{token})
            }
            5 => {
                let (number_of_tokens, rest) = rest.split_at(8);
                let (price, _rest) = rest.split_at(8);
                let number_of_tokens = number_of_tokens.try_into().map(u64::from_le_bytes).or(Err(InvalidInstruction))?;
                let price = price.try_into().map(u64::from_le_bytes).or(Err(InvalidInstruction))?;
                Self::ProcessAuction1(ProcessAuction1{number_of_tokens,price})
            }

            _ => return Err(TokenError::InvalidInstruction.into()),
        })
    }
}