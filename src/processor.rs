use crate::{
    error::TokenError,
    instruction::{
        TokenInstruction,
        ProcessDeposit
    },
    utils::{create_account,generate_pda_and_bump_seed,create_account_signed,create_pda_account},
    SPLTOKENPREFIX,
    NFTPREFIX,
    state::{NftDetails}
};
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{AccountInfo,next_account_info},
    program_error::{PrintProgramError},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    program::{invoke,invoke_signed},
    system_instruction,
    pubkey::Pubkey,
    sysvar::{rent::Rent,Sysvar,clock::Clock},
    msg,
    system_program,
};
use num_traits::FromPrimitive;
/// Program state handler.
pub struct Processor {}
impl Processor {
    pub fn process_deposit_nft(program_id: &Pubkey,accounts: &[AccountInfo],number_of_tokens: u64, price:u64)-> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let nft_owner =  next_account_info(account_info_iter)?; // sender or signer
        let token_program_id = next_account_info(account_info_iter)?; // TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let pda = next_account_info(account_info_iter)?; // token mint address 
        let spl_token_mint = next_account_info(account_info_iter)?;  // spl token address generated from SPLTOKENPREFIX, nft_owner, pda and program id
        let spl_associated_token = next_account_info(account_info_iter)?; // pda associated of spl_token_mint 
        let nft_mint = next_account_info(account_info_iter)?;  // mint address of nft
        let nft_associated_address = next_account_info(account_info_iter)?; // address generated from nft_vault_address and nft mint address
        let spl_vault_associated_address = next_account_info(account_info_iter)?; // address generated from nft_vault_address and spl token mint address
        let nft_spl_owner_address = next_account_info(account_info_iter)?; // // nft/token vault address generated from NFTPREFIX, nft_owner, pda and program id
        let associated_token_info = next_account_info(account_info_iter)?; // Associated token master {ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL}
        let rent_info  = next_account_info(account_info_iter)?; // rent 
        let system_program = next_account_info(account_info_iter)?;

        let (spl_token_address, bump_seed) = generate_pda_and_bump_seed(
            SPLTOKENPREFIX,
            nft_owner.key,
            pda.key,
            program_id
        );
        let spl_token_signer_seeds: &[&[_]] = &[
            SPLTOKENPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda.key.to_bytes(),
            &[bump_seed],
        ];
        let (nft_vault_address, bump_seed) = generate_pda_and_bump_seed(
            NFTPREFIX,
            nft_owner.key,
            pda.key,
            program_id
        );
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda.key.to_bytes(),
            &[bump_seed],
        ];
        let rent = Rent::get()?;
        let transfer_amount =  rent.minimum_balance(std::mem::size_of::<NftDetails>()+355);
        create_pda_account( 
            nft_owner,
            transfer_amount,
            std::mem::size_of::<NftDetails>(),
            program_id,
            system_program,
            pda
        )?;
        create_account_signed(nft_owner, 0.0014616 as u64, 82, token_program_id.key,system_program, spl_token_mint,spl_token_signer_seeds)?;
        invoke_signed(
            &spl_token::instruction::initialize_mint(
                token_program_id.key,
                spl_token_mint.key,
                nft_owner.key,
                Some(nft_owner.key),
                0)?,
                &[
                    token_program_id.clone(),
                    nft_owner.clone(),
                    nft_owner.clone(),
                    spl_token_mint.clone(),
                    system_program.clone()
                ],&[spl_token_signer_seeds]
            )?;
        // nft owner associated token using spl token mint
        invoke_signed(            
            &spl_associated_token_account::create_associated_token_account(
                nft_owner.key,
                nft_owner.key,
                spl_token_mint.key,
            ),&[
                nft_owner.clone(),
                spl_associated_token.clone(),
                nft_owner.clone(),
                spl_token_mint.clone(),
                token_program_id.clone(),
                rent_info.clone(),
                associated_token_info.clone(),
                system_program.clone()
            ],&[spl_token_signer_seeds]
        )?;
        invoke_signed(
            &spl_token::instruction::mint_to_checked(
                token_program_id.key,
                spl_token_mint.key,
                spl_associated_token.key,
                nft_owner.key,
                &[&nft_owner.key],
                number_of_tokens,
                9
            )?,&[
                token_program_id.clone(),
                spl_token_mint.clone(),
                spl_associated_token.clone(),
                nft_owner.clone(),
                nft_owner.clone(),
                system_program.clone()
            ],&[spl_token_signer_seeds]
        )?;
        if nft_associated_address.data_is_empty(){
            invoke(            
                &spl_associated_token_account::create_associated_token_account(
                    nft_owner.key,
                    nft_owner.key,
                    nft_mint.key,
                ),&[
                    nft_owner.clone(),
                    nft_associated_address.clone(),
                    nft_owner.clone(),
                    pda.clone(),
                    token_program_id.clone(),
                    rent_info.clone(),
                    associated_token_info.clone(),
                    system_program.clone()
                ]
            )?;
            invoke(            
                &spl_associated_token_account::create_associated_token_account(
                    nft_owner.key,
                    nft_owner.key,
                    spl_token_mint.key,
                ),&[
                    nft_owner.clone(),
                    nft_associated_address.clone(),
                    nft_owner.clone(),
                    pda.clone(),
                    token_program_id.clone(),
                    rent_info.clone(),
                    associated_token_info.clone(),
                    system_program.clone()
                ]
            )?;
        }
        invoke(
            &spl_token::instruction::transfer(
                token_program_id.key,
                nft_spl_owner_address.key,
                nft_associated_address.key,
                nft_owner.key,
                &[nft_owner.key],
                1
            )?,
            &[
                token_program_id.clone(),
                nft_spl_owner_address.clone(),
                nft_associated_address.clone(),
                nft_owner.clone(),
                system_program.clone()
            ],
        )?;
        invoke(
            &spl_token::instruction::transfer(
                token_program_id.key,
                nft_spl_owner_address.key,
                spl_vault_associated_address.key,
                nft_owner.key,
                &[nft_owner.key],
                1
            )?,
            &[
                token_program_id.clone(),
                nft_spl_owner_address.clone(),
                spl_vault_associated_address.clone(),
                nft_owner.clone(),
                system_program.clone()
            ],
        )?;
        let now = Clock::get()?.unix_timestamp as u64; 
        let mut escrow = NftDetails::try_from_slice(&pda.data.borrow())?;
        escrow.number_of_tokens = number_of_tokens;
        escrow.price = price;
        escrow.nft_mint = *nft_mint.key;
        escrow.nft_escrow = nft_vault_address;
        escrow.token_mint = *spl_token_mint.key;
        escrow.create_at = now;
        escrow.serialize(&mut &mut pda.data.borrow_mut()[..])?;
        Ok(())
    }
    pub fn process_buy_nft_token(program_id: &Pubkey,accounts: &[AccountInfo],token:u64)-> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let buyer =  next_account_info(account_info_iter)?; // sender or signer
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let token_mint_info = next_account_info(account_info_iter)?; // token mint 
        let nft_mint_address = next_account_info(account_info_iter)?; // token mint 
        let pda_data = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let token_program_id = next_account_info(account_info_iter)?; //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let nft_spl_owner_address = next_account_info(account_info_iter)?; // // nft/token vault address generated from NFTPREFIX, nft_owner, pda and program id
        let spl_vault_associated_address = next_account_info(account_info_iter)?;  // associated token of spl_vault_associated_address
       let system_program = next_account_info(account_info_iter)?;

        let escrow = NftDetails::try_from_slice(&pda_data.data.borrow())?;
        let now = Clock::get()?.unix_timestamp as u64; 
        let passed_time = now - escrow.create_at;
        if passed_time <= 86400 {
            return Err(TokenError::AuctionEnded.into());
        }
        let (nft_vault_address, bump_seed) = generate_pda_and_bump_seed(
            NFTPREFIX,
            nft_owner.key,
            pda_data.key,
            program_id
        );
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda_data.key.to_bytes(),
            &[bump_seed],
        ];
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program_id.key,
                spl_vault_associated_address.key,
                buyer.key,
                nft_spl_owner_address.key,
                &[nft_spl_owner_address.key],
                token
            )?,
            &[
                token_program_id.clone(),
                spl_vault_associated_address.clone(),
                buyer.clone(),
                nft_spl_owner_address.clone(),
                system_program.clone()
            ],&[&nft_vault_signer_seeds],
        )?;
        invoke(
            &system_instruction::transfer(
                buyer.key,
                nft_owner.key,
                token
            ),
            &[
                buyer.clone(),
                nft_owner.clone(),
                system_program.clone()
            ],
        )?;
        Ok(())
    }
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TokenInstruction::unpack(input)?;
        match instruction {
            TokenInstruction::ProcessDeposit (ProcessDeposit{
                number_of_tokens,
                price,
            }) => {
                msg!("Instruction: Sol Stream");
                Self::process_deposit_nft(program_id,accounts,number_of_tokens, price)
            }
        }
    }
}

impl PrintProgramError for TokenError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            TokenError::NotRentExempt => msg!("Error: Lamport balance below rent-exempt threshold"),
            TokenError::InvalidInstruction => msg!("Error: Invalid instruction"),
            TokenError::AuctionEnded => msg!("Error: Auction Ended"),
        }
    }
}