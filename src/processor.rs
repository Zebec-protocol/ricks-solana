use crate::{
    error::TokenError,
    instruction::{
        ProcessAuction1, ProcessBuy, ProcessBuy2, ProcessClaimCoinFlip, ProcessCoinFlip,
        ProcessDeposit, TokenInstruction,
    },
    state::{Auction, CoinFlip, NftDetails},
    utils::{create_pda_account, generate_pda_and_bump_seed},
    AUCTIONPREFIX, NFTPREFIX, SPLTOKENPREFIX,
};
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::{PrintProgramError, ProgramError},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};

use num_traits::FromPrimitive;
/// Program state handler.
pub struct Processor {}
impl Processor {
    pub fn process_deposit_nft(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        number_of_tokens: u64,
        price: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let nft_owner = next_account_info(account_info_iter)?; // sender or signer
        let token_program_id = next_account_info(account_info_iter)?; // TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let pda = next_account_info(account_info_iter)?; // pda data
        let spl_token_mint = next_account_info(account_info_iter)?; // spl token address generated from SPLTOKENPREFIX, nft_owner, pda and program id
        let spl_associated_token = next_account_info(account_info_iter)?; // nft owner associated of spl_token_mint
        let nft_mint = next_account_info(account_info_iter)?; // mint address of nft
        let nft_vault = next_account_info(account_info_iter)?; // nft vault address from NFTPREFIX, nft_owner, pda and program id
        let nft_associated_address = next_account_info(account_info_iter)?; // address generated from nft_vault_address and nft mint address
        let spl_vault_associated_address = next_account_info(account_info_iter)?; // address generated from nft_vault_address and spl token mint address
        let _nft_spl_owner_address = next_account_info(account_info_iter)?; // // nft/token vault address generated from NFTPREFIX, nft_owner, pda and program id
        let associated_token_info = next_account_info(account_info_iter)?; // Associated token master {ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL}
        let nft_owner_nft_associated = next_account_info(account_info_iter)?; // nft owner nft id
        let rent_info = next_account_info(account_info_iter)?; // rent
        let system_program = next_account_info(account_info_iter)?;

        let (spl_token_address, bump_seed_spl) =
            generate_pda_and_bump_seed(SPLTOKENPREFIX, nft_owner.key, pda.key, program_id);

        // Verify that token mint is as expected
        if spl_token_address != *spl_token_mint.key {
            return Err(TokenError::InvalidTokenMintAddress.into());
        }

        let spl_token_signer_seeds: &[&[_]] = &[
            SPLTOKENPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda.key.to_bytes(),
            &[bump_seed_spl],
        ];
        let (nft_vault_address, bump_seed) =
            generate_pda_and_bump_seed(NFTPREFIX, nft_owner.key, pda.key, program_id);
        if nft_vault_address != *spl_associated_token.key {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda.key.to_bytes(),
            &[bump_seed],
        ];

        let rent = Rent::get()?;
        let transfer_amount = rent.minimum_balance(std::mem::size_of::<NftDetails>());
        create_pda_account(
            nft_owner,
            transfer_amount,
            std::mem::size_of::<NftDetails>(),
            program_id,
            system_program,
            pda, //Data ACCOUNT
        )?;

        invoke(
            &system_instruction::transfer(
                nft_owner.key,
                nft_vault.key,
                10000000, //calculate rent
            ),
            &[nft_owner.clone(), nft_vault.clone(), system_program.clone()],
        )?;

        invoke_signed(
            &system_instruction::create_account(
                nft_vault.key,
                spl_token_mint.key,
                1461600, //calculate rent
                82,
                token_program_id.key,
            ),
            &[
                nft_vault.clone(),
                spl_token_mint.clone(),
                system_program.clone(),
            ],
            &[nft_vault_signer_seeds, spl_token_signer_seeds],
        )?;

        msg!("Initialize mint");
        invoke_signed(
            &spl_token::instruction::initialize_mint(
                token_program_id.key,
                spl_token_mint.key,
                nft_vault.key,
                Some(nft_vault.key),
                9,
            )?,
            &[
                token_program_id.clone(),
                nft_vault.clone(),
                nft_vault.clone(),
                spl_token_mint.clone(),
                system_program.clone(),
                rent_info.clone(),
            ],
            &[nft_vault_signer_seeds, spl_token_signer_seeds],
        )?;

        // nft owner associated token using spl token mint
        msg!("Create associated token");
        invoke_signed(
            &spl_associated_token_account::create_associated_token_account(
                nft_vault.key,
                nft_vault.key, ////?????
                spl_token_mint.key,
            ),
            &[
                nft_vault.clone(),
                spl_vault_associated_address.clone(),
                nft_vault.clone(),
                spl_token_mint.clone(),
                token_program_id.clone(),
                rent_info.clone(),
                associated_token_info.clone(),
                system_program.clone(),
            ],
            &[nft_vault_signer_seeds, spl_token_signer_seeds],
        )?;

        msg!("minting token");
        invoke_signed(
            &spl_token::instruction::mint_to_checked(
                token_program_id.key,
                spl_token_mint.key,
                spl_vault_associated_address.key,
                nft_vault.key,
                &[&nft_vault.key],
                number_of_tokens,
                9,
            )?,
            &[
                token_program_id.clone(),
                spl_token_mint.clone(),
                spl_vault_associated_address.clone(),
                nft_vault.clone(),
                nft_vault.clone(),
                system_program.clone(),
                rent_info.clone(),
            ],
            &[nft_vault_signer_seeds, spl_token_signer_seeds],
        )?;

        if nft_associated_address.data_is_empty() {
            invoke(
                &spl_associated_token_account::create_associated_token_account(
                    nft_owner.key,
                    nft_vault.key,
                    nft_mint.key,
                ),
                &[
                    nft_owner.clone(),
                    nft_associated_address.clone(),
                    nft_vault.clone(),
                    nft_mint.clone(),
                    token_program_id.clone(),
                    rent_info.clone(),
                    associated_token_info.clone(),
                    system_program.clone(),
                ],
            )?;
            msg!("Testing");
            msg!(
                "vault {} spl token {} spl vault{}",
                nft_vault.key,
                spl_token_mint.key,
                spl_vault_associated_address.key
            );
        }

        msg!("transfer");
        invoke(
            &spl_token::instruction::transfer(
                token_program_id.key,
                nft_owner_nft_associated.key,
                nft_associated_address.key,
                nft_owner.key,
                &[nft_owner.key],
                1,
            )?,
            &[
                token_program_id.clone(),
                nft_owner_nft_associated.clone(),
                nft_associated_address.clone(),
                nft_owner.clone(),
                system_program.clone(),
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
        escrow.days = 0 as f64;
        escrow.remaining_token = number_of_tokens;
        escrow.serialize(&mut &mut pda.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn auction1(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        number_of_tokens: u64,
        price: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let bidder = next_account_info(account_info_iter)?; // sender or signer
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let pda_data = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let nft_vault = next_account_info(account_info_iter)?; // nft vault which saves the amount
        let auction_data = next_account_info(account_info_iter)?; //account made using
        let system_program = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?; // rent

        if !bidder.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if pda_data.owner != program_id {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let (nft_vault_address, bump_seed) =
            generate_pda_and_bump_seed(NFTPREFIX, nft_owner.key, pda_data.key, program_id);

        //check if the nft_vault is actual pda or not
        if nft_vault_address != *nft_vault.key {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda_data.key.to_bytes(),
            &[bump_seed],
        ];
        let mut pda_check = NftDetails::try_from_slice(&pda_data.data.borrow())?;
        if *nft_vault.key != pda_check.nft_escrow {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let now = Clock::get()?.unix_timestamp as u64;

        if (now - pda_check.create_at) < 86400 {
            return Err(TokenError::Notstarted.into());
        }
        let day: u64 = (now - pda_check.create_at) % 86400;
        if number_of_tokens <= pda_check.number_of_tokens / 100 {
            return Err(TokenError::Overflow.into());
        }
        let (auction_address, auction_bump) = Pubkey::find_program_address(
            &[
                AUCTIONPREFIX.as_bytes(),
                &nft_owner.key.to_bytes(),
                &[day as u8],
            ],
            program_id,
        );
        let auction_signer_seeds: &[&[_]] = &[
            AUCTIONPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &[day as u8],
            &[auction_bump],
        ];
        let mut flag: u8 = 0;
        if auction_data.data_is_empty() {
            let rent = Rent::get()?;
            let transfer_amount = rent.minimum_balance(std::mem::size_of::<Auction>());
            invoke(
                &system_instruction::transfer(bidder.key, nft_vault.key, transfer_amount),
                &[nft_owner.clone(), nft_vault.clone(), system_program.clone()],
            )?;
            invoke_signed(
                &system_instruction::create_account(
                    nft_vault.key,
                    auction_data.key,
                    transfer_amount,
                    std::mem::size_of::<Auction>() as u64,
                    program_id,
                ),
                &[
                    nft_vault.clone(),
                    auction_data.clone(),
                    system_program.clone(),
                    rent_info.clone(),
                ],
                &[nft_vault_signer_seeds, auction_signer_seeds],
            )?;
            let total_supply = pda_check.number_of_tokens / 100 + pda_check.number_of_tokens;
            pda_check.number_of_tokens = total_supply;
            flag = 1;
        } else {
            if *auction_data.key != auction_address && auction_data.owner != program_id {
                return Err(ProgramError::MissingRequiredSignature);
            }
        }

        let mut auction_operation = Auction::try_from_slice(&auction_data.data.borrow())?;
        if flag == 1 {
            invoke(
                &system_instruction::transfer(bidder.key, nft_vault.key, number_of_tokens * price),
                &[bidder.clone(), nft_vault.clone(), system_program.clone()],
            )?;
            auction_operation.max_payer = *bidder.key;
            auction_operation.num_tokens = number_of_tokens;
            auction_operation.day = day;
        } else if price > auction_operation.max_price {
            //release amount of previous highest bidder
            invoke_signed(
                &system_instruction::transfer(
                    nft_vault.key,
                    &auction_operation.max_payer,
                    auction_operation.max_price * auction_operation.num_tokens,
                ),
                &[nft_vault.clone(), system_program.clone()],
                &[&nft_vault_signer_seeds],
            )?;

            auction_operation.max_payer = *bidder.key;
            auction_operation.num_tokens = number_of_tokens;
            invoke(
                &system_instruction::transfer(bidder.key, nft_vault.key, number_of_tokens * price),
                &[bidder.clone(), nft_vault.clone(), system_program.clone()],
            )?;
        }
        auction_operation.serialize(&mut &mut auction_data.data.borrow_mut()[..])?;
        pda_check.serialize(&mut &mut pda_data.data.borrow_mut()[..])?;

        Ok(())
    }
    pub fn process_buy_nft_token(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        token: u64,
        price: u64,
    ) -> ProgramResult {
        //program to buy nft at the price set by the program initiator
        let account_info_iter = &mut accounts.iter();
        let buyer = next_account_info(account_info_iter)?; // sender or signer
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let pda_data = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let token_program_id = next_account_info(account_info_iter)?; //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let nft_vault = next_account_info(account_info_iter)?; // nft vault
        let spl_vault_associated_address = next_account_info(account_info_iter)?; // find associated address from nft vault and spl token mint
        let buyer_spl_associated = next_account_info(account_info_iter)?; // sender or signer
        let spl_token_mint = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let associated_token_info = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        let mut escrow = NftDetails::try_from_slice(&pda_data.data.borrow())?;

        msg!("spl: {}", spl_token_mint.key);
        msg!("token: {}", token);
        let now = Clock::get()?.unix_timestamp as u64;
        let passed_time = now - escrow.create_at;
        if passed_time >= 86400 {
            return Err(TokenError::AuctionStarted.into());
        }
        if token > escrow.remaining_token {
            return Err(TokenError::TokenFinished.into());
        }
        if price < escrow.price {
            return Err(TokenError::PriceLower.into());
        }
        let (nft_vault_address, bump_seed) =
            generate_pda_and_bump_seed(NFTPREFIX, nft_owner.key, pda_data.key, program_id);
        if nft_vault_address != *nft_vault.key {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda_data.key.to_bytes(),
            &[bump_seed],
        ];

        invoke(
            &spl_associated_token_account::create_associated_token_account(
                buyer.key,
                buyer.key,
                spl_token_mint.key,
            ),
            &[
                buyer.clone(),
                buyer_spl_associated.clone(),
                buyer.clone(),
                spl_token_mint.clone(),
                token_program_id.clone(),
                rent_info.clone(),
                associated_token_info.clone(),
                system_program.clone(),
            ],
        )?;
        invoke(
            &system_instruction::transfer(buyer.key, nft_owner.key, token * escrow.price),
            &[nft_vault.clone(), buyer.clone(), system_program.clone()],
        )?;
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program_id.key,
                spl_vault_associated_address.key,
                buyer_spl_associated.key,
                nft_vault.key,
                &[nft_vault.key],
                token,
            )?,
            &[
                token_program_id.clone(),
                spl_vault_associated_address.clone(),
                buyer_spl_associated.clone(),
                nft_vault.clone(),
                system_program.clone(),
            ],
            &[&nft_vault_signer_seeds],
        )?;

        escrow.remaining_token -= token;
        escrow.serialize(&mut &mut pda_data.data.borrow_mut()[..])?;
        Ok(())
    }
    pub fn process_buy_nft_token2(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        day: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let buyer = next_account_info(account_info_iter)?; // sender or signer
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let pda_data = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let token_program_id = next_account_info(account_info_iter)?; //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let nft_vault = next_account_info(account_info_iter)?; // nft vault
        let spl_vault_associated_address = next_account_info(account_info_iter)?; // find associated address from nft vault and spl token mint
        let buyer_spl_associated = next_account_info(account_info_iter)?; // sender or signer
        let spl_token_mint = next_account_info(account_info_iter)?; // spl token mint
        let system_program = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let auction_data = next_account_info(account_info_iter)?;

        let escrow = NftDetails::try_from_slice(&pda_data.data.borrow())?;
        let now = Clock::get()?.unix_timestamp as u64;
        let days = (now - escrow.create_at) % 86400;
        let (nft_vault_address, bump_seed) =
            generate_pda_and_bump_seed(NFTPREFIX, nft_owner.key, pda_data.key, program_id);
        //check if the nft_vault is actual pda or not
        if nft_vault_address != *nft_vault.key {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda_data.key.to_bytes(),
            &[bump_seed],
        ];
        let (_spl_token_address, bump_seed_spl) =
            generate_pda_and_bump_seed(SPLTOKENPREFIX, nft_owner.key, pda_data.key, program_id);
        let spl_token_signer_seeds: &[&[_]] = &[
            SPLTOKENPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda_data.key.to_bytes(),
            &[bump_seed_spl],
        ];
        let (auction_address, _auction_bump) = Pubkey::find_program_address(
            &[
                AUCTIONPREFIX.as_bytes(),
                &nft_owner.key.to_bytes(),
                &[day as u8],
            ],
            program_id,
        );
        if auction_data.data_is_empty() {
            return Err(TokenError::Notstarted.into());
        }
        if auction_address != *auction_data.key && auction_data.owner != program_id {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut auction_operation = Auction::try_from_slice(&auction_data.data.borrow())?;
        if day != auction_operation.day && day < days {
            return Err(TokenError::Notstarted.into());
        }
        if auction_operation.max_payer == *buyer.key && auction_operation.max_price != 0 {
            invoke_signed(
                &spl_token::instruction::mint_to_checked(
                    token_program_id.key,
                    spl_token_mint.key,
                    spl_vault_associated_address.key,
                    nft_vault.key,
                    &[&nft_vault.key],
                    auction_operation.num_tokens,
                    9,
                )?,
                &[
                    token_program_id.clone(),
                    spl_token_mint.clone(),
                    spl_vault_associated_address.clone(),
                    nft_vault.clone(),
                    nft_vault.clone(),
                    system_program.clone(),
                    rent_info.clone(),
                ],
                &[nft_vault_signer_seeds, spl_token_signer_seeds],
            )?;
        }

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program_id.key,
                spl_vault_associated_address.key,
                buyer_spl_associated.key,
                nft_vault.key,
                &[nft_vault.key],
                auction_operation.num_tokens,
            )?,
            &[
                token_program_id.clone(),
                spl_vault_associated_address.clone(),
                buyer_spl_associated.clone(),
                nft_vault.clone(),
                system_program.clone(),
            ],
            &[&nft_vault_signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::transfer(
                nft_vault.key,
                nft_owner.key,
                auction_operation.max_price * auction_operation.num_tokens,
            ),
            &[nft_vault.clone(), nft_owner.clone(), system_program.clone()],
            &[&nft_vault_signer_seeds],
        )?;
        auction_operation.max_price = 0;

        Ok(())
    }
    // need to improve security using recent blockhash or vrf
    pub fn process_coin_flip(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        token: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let player = next_account_info(account_info_iter)?; // sender or signer
        let coinflip_pda = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let token_program_id = next_account_info(account_info_iter)?; //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let spl_vault_associated_address = next_account_info(account_info_iter)?; // find associated address from nft vault and spl token mint
        let player_associated_token = next_account_info(account_info_iter)?; // spl token mint
        let system_program = next_account_info(account_info_iter)?;

        msg!("{}", token);
        let now = Clock::get()?.unix_timestamp as u64;
        let rent = Rent::get()?;
        let transfer_amount = rent.minimum_balance(std::mem::size_of::<CoinFlip>());
        create_pda_account(
            player,
            transfer_amount,
            std::mem::size_of::<CoinFlip>(),
            program_id,
            system_program,
            coinflip_pda,
        )?;
        let mut coinflip = CoinFlip::try_from_slice(&coinflip_pda.data.borrow())?;
        invoke(
            &spl_token::instruction::transfer(
                token_program_id.key,
                player_associated_token.key,
                spl_vault_associated_address.key,
                player.key,
                &[player.key],
                token,
            )?,
            &[
                token_program_id.clone(),
                player_associated_token.clone(),
                spl_vault_associated_address.clone(),
                player.clone(),
                system_program.clone(),
            ],
        )?;

        //use recent blockhash
        if now % 2 == 0 {
            coinflip.won = 0
        } else {
            coinflip.won = 1
        }
        coinflip.serialize(&mut &mut coinflip_pda.data.borrow_mut()[..])?;
        Ok(())
    }
    pub fn process_coin_flip_claim(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        token: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let player = next_account_info(account_info_iter)?; // sender or signer
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let pda = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let coinflip_pda = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let token_program_id = next_account_info(account_info_iter)?; //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let nft_vault = next_account_info(account_info_iter)?; // nft vault
        let spl_vault_associated_address = next_account_info(account_info_iter)?; // find associated address from nft vault and spl token mint
        let buyer_spl_associated = next_account_info(account_info_iter)?; // sender or signer
        let system_program = next_account_info(account_info_iter)?;

        let (nft_vault_address, bump_seed) =
            generate_pda_and_bump_seed(NFTPREFIX, nft_owner.key, pda.key, program_id);
        if nft_vault_address != *nft_vault.key {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda.key.to_bytes(),
            &[bump_seed],
        ];

        let coinflip = CoinFlip::try_from_slice(&coinflip_pda.data.borrow())?;
        if coinflip.won == 1 {
            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program_id.key,
                    spl_vault_associated_address.key,
                    buyer_spl_associated.key,
                    nft_vault.key,
                    &[nft_vault.key],
                    token,
                )?,
                &[
                    token_program_id.clone(),
                    spl_vault_associated_address.clone(),
                    buyer_spl_associated.clone(),
                    nft_vault.clone(),
                    system_program.clone(),
                ],
                &[&nft_vault_signer_seeds],
            )?;
        }
        coinflip.serialize(&mut &mut coinflip_pda.data.borrow_mut()[..])?;
        let dest_starting_lamports = player.lamports();
        **player.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(coinflip_pda.lamports())
            .ok_or(TokenError::Overflow)?;
        **coinflip_pda.lamports.borrow_mut() = 0;
        Ok(())
    }
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TokenInstruction::unpack(input)?;
        match instruction {
            TokenInstruction::ProcessDeposit(ProcessDeposit {
                number_of_tokens,
                price,
            }) => {
                msg!("Instruction: Fractionalizing NFT");
                Self::process_deposit_nft(program_id, accounts, number_of_tokens, price)
            }
            TokenInstruction::ProcessBuy(ProcessBuy { token, price }) => {
                msg!("Instruction: Buy token");
                Self::process_buy_nft_token(program_id, accounts, token, price)
            }
            TokenInstruction::ProcessBuy2(ProcessBuy2 { day }) => {
                msg!("Instruction:  Buy token");
                Self::process_buy_nft_token2(program_id, accounts, day)
            }
            TokenInstruction::ProcessCoinFlip(ProcessCoinFlip { token }) => {
                msg!("Instruction:  Flip Coin");
                Self::process_coin_flip(program_id, accounts, token)
            }
            TokenInstruction::ProcessClaimCoinFlip(ProcessClaimCoinFlip { token }) => {
                msg!("Instruction:  Claim Token");
                Self::process_coin_flip_claim(program_id, accounts, token)
            }
            TokenInstruction::ProcessAuction1(ProcessAuction1 {
                number_of_tokens,
                price,
            }) => {
                msg!("Instruction:  Claim Token");
                Self::auction1(program_id, accounts, number_of_tokens, price)
            }
        }
    }
}

impl PrintProgramError for TokenError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        msg!("Error {}", self);
    }
}
