//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TokenError {
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt = 0,

    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Auction ended")]
    AuctionEnded,

    #[error("Token overflowed")]
    Overflow,

    #[error("Not started")]
    Notstarted,

    #[error("Token Finished")]
    TokenFinished,

    #[error("Buy Period Ended")]
    AuctionStarted,

    #[error("Price is Lower")]
    PriceLower,

    #[error("Token mint address is not as expected")]
    InvalidTokenMintAddress,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TokenError {
    fn type_of() -> &'static str {
        "TokenError"
    }
}
