use crate::{
    error::TokenError,
    instruction::{
        TokenInstruction,
        ProcessDeposit,
        ProcessBuy,
        ProcessBuy2,
        ProcessCoinFlip,
        ProcessClaimCoinFlip,
        ProcessAuction1,
    },
    utils::{generate_pda_and_bump_seed,create_pda_account,get_token_balance},
    SPLTOKENPREFIX,
    NFTPREFIX,
    AUCTIONPREFIX,
    state::{NftDetails,CoinFlip,Auction}
};
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{AccountInfo,next_account_info},
    program_error::{PrintProgramError,ProgramError},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    program::{invoke,invoke_signed},
    system_instruction,
    pubkey::Pubkey,
    sysvar::{rent::Rent,Sysvar,clock::Clock},
    msg,
};
use spl_associated_token_account::get_associated_token_address;
use num_traits::FromPrimitive;
/// Program state handler.
pub struct Processor {}
impl Processor {
    pub fn process_deposit_nft(program_id: &Pubkey,accounts: &[AccountInfo],number_of_tokens: u64, price:u64)-> ProgramResult {
        //depositing the NFT
        let account_info_iter = &mut accounts.iter();
        let nft_owner =  next_account_info(account_info_iter)?; // sender or signer
        let token_program_id = next_account_info(account_info_iter)?; // TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let pda = next_account_info(account_info_iter)?; // pda data 
        let spl_token_mint = next_account_info(account_info_iter)?;  // spl token address generated from SPLTOKENPREFIX, nft_owner, pda and program id
        let _spl_associated_token = next_account_info(account_info_iter)?; // nft owner associated of spl_token_mint 
        let nft_mint = next_account_info(account_info_iter)?;  // mint address of nft
        let nft_vault = next_account_info(account_info_iter)?;  // nft vault address from NFTPREFIX, nft_owner, pda and program id
        let nft_associated_address = next_account_info(account_info_iter)?; // address generated from nft_vault_address and nft mint address token account address
        let spl_vault_associated_address = next_account_info(account_info_iter)?; // address generated from nft_vault_address and spl token mint address
        let _nft_spl_owner_address = next_account_info(account_info_iter)?; // // nft/token vault address generated from spltoken mint, nft_owner, pda and program id
        let associated_token_info = next_account_info(account_info_iter)?; // Associated token master {ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL}
        let nft_owner_nft_associated = next_account_info(account_info_iter)?;  // nft owner nft id token account address
        let rent_info  = next_account_info(account_info_iter)?; // rent 
        let system_program = next_account_info(account_info_iter)?; //system program

       //checking if the owner is the signer or not
        if !nft_owner.is_signer
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        //finding nft token account
        let nft_token_address=get_associated_token_address(nft_owner.key,nft_mint.key);

        //verifying nft token address
        if nft_token_address!=*nft_owner_nft_associated.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        // new token mint for minting
        let (spl_token_address, bump_seed_spl) = generate_pda_and_bump_seed(
            SPLTOKENPREFIX,
            nft_owner.key,
            pda.key,
            program_id
        );
         // verifying
        if spl_token_address!=*spl_token_mint.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
         // signer seeds for spl_token_mint
        let spl_token_signer_seeds: &[&[_]] = &[
            SPLTOKENPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda.key.to_bytes(),
            &[bump_seed_spl],
        ];
        //verifying spl mint token account
        let spl_vault_associated_token =get_associated_token_address(nft_vault.key,spl_token_mint.key);
        if spl_vault_associated_token!=*spl_vault_associated_address.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        //nft_vault where nft is stored
        let (nft_vault_address, bump_seed) = generate_pda_and_bump_seed(
            NFTPREFIX,
            nft_owner.key,
            pda.key,
            program_id
        );
         //nft_vault where nft is stored
        if nft_vault_address!=*nft_vault.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        //signer seeds for nft_vault
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda.key.to_bytes(),
            &[bump_seed],
        ];
         //finding nft token account
         let vault_nft_token_address=get_associated_token_address(nft_vault.key,nft_mint.key);

         //verifying nft token address
         if vault_nft_token_address!=*nft_associated_address.key
         {
             return Err(ProgramError::MissingRequiredSignature);
         }
         
         //rent account
        let rent = Rent::get()?;
        let transfer_amount =  rent.minimum_balance(std::mem::size_of::<NftDetails>());
        create_pda_account( 
            nft_owner,
            transfer_amount,
            std::mem::size_of::<NftDetails>(),
            program_id,
            system_program,
            pda         //Data ACCOUNT 
        )?;
        invoke(
            &system_instruction::transfer(
                nft_owner.key,
                nft_vault.key,
                10000000 //calculate rent 
            ),
            &[
                nft_owner.clone(),
                nft_vault.clone(),
                system_program.clone()
            ],
        )?;
        invoke_signed(
            &system_instruction::create_account(
                nft_vault.key,
                spl_token_mint.key,
                1461600, //calculate rent 
                82,//size
                token_program_id.key,
            ),
            &[
                nft_vault.clone(),
                spl_token_mint.clone(),
                system_program.clone(),
            ],
            &[nft_vault_signer_seeds,spl_token_signer_seeds],
        )?;

        msg!("Initialize mint");
        invoke_signed(
            &spl_token::instruction::initialize_mint(
                token_program_id.key,
                spl_token_mint.key, 
                nft_vault.key,
                Some(nft_vault.key),
                9)?,
                &[
                    token_program_id.clone(),
                    nft_vault.clone(),
                    nft_vault.clone(),
                    spl_token_mint.clone(),
                    system_program.clone(),
                    rent_info.clone()
                ],
                &[nft_vault_signer_seeds,spl_token_signer_seeds]
            )?;
        // nft owner associated token using spl token mint
        msg!("Create associated token");
        invoke_signed(            
            &spl_associated_token_account::create_associated_token_account(
                nft_vault.key,
                nft_vault.key,////?????
                spl_token_mint.key,
            ),&[
                nft_vault.clone(),
                spl_vault_associated_address.clone(),
                nft_vault.clone(),
                spl_token_mint.clone(),
                token_program_id.clone(),
                rent_info.clone(),
                associated_token_info.clone(),
                system_program.clone()
            ],&[nft_vault_signer_seeds,spl_token_signer_seeds]
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
                9
            )?,&[
                token_program_id.clone(),
                spl_token_mint.clone(),
                spl_vault_associated_address.clone(),
                nft_vault.clone(),
                nft_vault.clone(),
                system_program.clone(),
                rent_info.clone()
            ],&[nft_vault_signer_seeds,spl_token_signer_seeds]
        )?;

        if nft_associated_address.data_is_empty(){
            invoke(            
                &spl_associated_token_account::create_associated_token_account(
                    nft_owner.key,
                    nft_vault.key,
                    nft_mint.key,
                ),&[
                    nft_owner.clone(),
                    nft_associated_address.clone(),
                    nft_vault.clone(),
                    nft_mint.clone(),
                    token_program_id.clone(),
                    rent_info.clone(),
                    associated_token_info.clone(),
                    system_program.clone()
                ]
            )?;
            msg!("Testing");
            msg!("vault {} spl token {} spl vault{}",nft_vault.key,spl_token_mint.key,spl_vault_associated_address.key);
        }
        msg!("transfer");
        invoke(
            &spl_token::instruction::transfer(
                token_program_id.key,
                nft_owner_nft_associated.key,
                nft_associated_address.key,
                nft_owner.key,
                &[nft_owner.key],
                1
            )?,
            &[
                token_program_id.clone(),
                nft_owner_nft_associated.clone(),
                nft_associated_address.clone(),
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
        escrow.nft_owner=*nft_owner.key;
        escrow.token_mint = *spl_token_mint.key;
        escrow.create_at = now;
        escrow.days = 0 as f64;
        escrow.remaining_token=number_of_tokens;
        escrow.serialize(&mut &mut pda.data.borrow_mut()[..])?;
        
        Ok(())
    }
    pub fn auction1(program_id: &Pubkey,accounts: &[AccountInfo], price:u64)->ProgramResult{   
        //Program to auction
        let account_info_iter = &mut accounts.iter();
        let bidder =  next_account_info(account_info_iter)?; // sender or signer
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let pda_data = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let nft_vault = next_account_info(account_info_iter)?; // nft vault which saves the amount 
        let auction_data = next_account_info(account_info_iter)?; //account made using Auction Prefix, Nft owner and Day
        let system_program = next_account_info(account_info_iter)?;//system_program
        let rent_info  = next_account_info(account_info_iter)?; // rent 

        if !bidder.is_signer
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if pda_data.owner!=program_id  
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let (nft_vault_address, bump_seed) = generate_pda_and_bump_seed(
            NFTPREFIX,
            nft_owner.key,
            pda_data.key,
            program_id
        );

        //check if the nft_vault is actual pda or not 
        if nft_vault_address != *nft_vault.key
        {
            return Err(ProgramError::MissingRequiredSignature);   
        }
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda_data.key.to_bytes(),
            &[bump_seed],
        ];
        let  mut pda_check = NftDetails::try_from_slice(&pda_data.data.borrow())?;
        let num_of_token=pda_check.number_of_tokens/100;
        if *nft_vault.key != pda_check.nft_escrow
        {
            return Err(ProgramError::MissingRequiredSignature);   
        }
        let now = Clock::get()?.unix_timestamp as u64; 

        if (now-pda_check.create_at) < 86400
        {
            
            msg!("The auction period has not started yet");
            return Err(TokenError::Notstarted.into());

        }
        let day:u64=1;
        let day_ip=day.to_string();
        let (auction_address,auction_bump)= Pubkey::find_program_address(
            &[
                AUCTIONPREFIX.as_bytes(),
                &nft_owner.key.to_bytes(),
                day_ip.as_bytes(),
            ],
            program_id,
        );
        
        let auction_signer_seeds: &[&[_]] = &[
            AUCTIONPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            day_ip.as_bytes(),
            &[auction_bump],
        ];
        msg!("The day is: {}",day);
        if *auction_data.key!=auction_address 
        {
            msg!("auction address don't match {}",auction_address);
            return Err(ProgramError::MissingRequiredSignature);  
        }
        let mut flag:u8=0;
        if auction_data.data_is_empty()
        {
            msg!("Inside account creation");
            let rent = Rent::get()?;
            let transfer_amount =  rent.minimum_balance(std::mem::size_of::<Auction>());
            invoke(
                &system_instruction::transfer(
                    bidder.key,
                    nft_vault.key,
                    transfer_amount,
                ),
                &[
                    bidder.clone(),
                    nft_vault.clone(),
                    system_program.clone()
                ],
            )?;
            msg!("Account Creation fees sent");
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
                &[nft_vault_signer_seeds,auction_signer_seeds],
            )?;
            msg!("Account created");
            let total_supply=pda_check.number_of_tokens/100+pda_check.number_of_tokens;
            pda_check.number_of_tokens=total_supply;
            flag =1;
        }
        else
        {
            msg!("Account already created so owner is checked");
            if auction_data.owner!=program_id
            {
                return Err(ProgramError::MissingRequiredSignature);   
            }
        }

       
        let mut auction_operation = Auction::try_from_slice(&auction_data.data.borrow())?;
        if flag ==1
        {
            msg!("Bid amount transfer for first time...");
            invoke(
                &system_instruction::transfer(
                    bidder.key,
                    nft_vault.key,
                    price,
                ),
                &[
                    bidder.clone(),
                    nft_vault.clone(),
                    system_program.clone()
                ],
            )?;
            msg!("Transfer completed");
            auction_operation.max_payer = *bidder.key;
            auction_operation.num_tokens = num_of_token;
            auction_operation.max_price=price;
            auction_operation.day=day;

        }
        else 
        {
            msg!("Bid after creation");
            if price > auction_operation.max_price 
            {

            let max_payer=next_account_info(account_info_iter)?; // previous maximum payer obtained by deserializing auction_data
            if *max_payer.key!=auction_operation.max_payer
            {
                return Err(ProgramError::MissingRequiredSignature);   
            }
            msg!("release amount of previous highest bidder");
            //release amount of previous highest bidder
            invoke_signed(  
                &system_instruction::transfer(
                nft_vault.key,
                max_payer.key,
                auction_operation.max_price,    
            ),
            &[
            nft_vault.clone(),
            max_payer.clone(),
            system_program.clone(),
            ],
            &[&nft_vault_signer_seeds],
            )?;
            msg!("amount released");

            auction_operation.max_payer = *bidder.key;
            auction_operation.num_tokens = num_of_token;
            auction_operation.max_price=price;
            msg!("bid amount to vault ..");
            invoke(
                &system_instruction::transfer(
                    bidder.key,
                    nft_vault.key,
                    price,
                ),
                &[
                    bidder.clone(),
                    nft_vault.clone(),
                    system_program.clone()
                ],
            )?;
            msg!("completed");
        }

        }
        auction_operation.serialize(&mut &mut auction_data.data.borrow_mut()[..])?;
        pda_check.serialize(&mut &mut pda_data.data.borrow_mut()[..])?;

       Ok(())

    }
    pub fn process_buy_nft_token(program_id: &Pubkey,accounts: &[AccountInfo],token:u64,price:u64)-> ProgramResult {
        //program to buy nft at the price set by the program initiator
        let account_info_iter = &mut accounts.iter();
        let buyer =  next_account_info(account_info_iter)?; // sender or signer
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let pda_data = next_account_info(account_info_iter)?; // pda data that consists number of tokens 
        let token_program_id = next_account_info(account_info_iter)?; //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let nft_vault = next_account_info(account_info_iter)?; // nft vault
        let spl_vault_associated_address = next_account_info(account_info_iter)?;  // find associated address from nft vault and spl token mint
        let buyer_spl_associated =  next_account_info(account_info_iter)?; // sender or signer
        let spl_token_mint = next_account_info(account_info_iter)?; 
        let rent_info = next_account_info(account_info_iter)?; 
        let associated_token_info = next_account_info(account_info_iter)?; 
        let system_program = next_account_info(account_info_iter)?;

        //verifying pda_data
        if pda_data.owner!=program_id
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if !buyer.is_signer
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut escrow = NftDetails::try_from_slice(&pda_data.data.borrow())?;
        //verifying owner and escrow
        if escrow.nft_escrow!=*nft_vault.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if escrow.nft_owner!=*nft_owner.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        msg!("spl: {}", spl_token_mint.key);
        msg!("token: {}", token);
        let now = Clock::get()?.unix_timestamp as u64; 
        let passed_time = now - escrow.create_at;
        if passed_time >= 86400 {
            msg!("The buying period has ended you can only auction now");
            return Err(TokenError::AuctionStarted.into());
        }
        if token > escrow.remaining_token
        {
            msg!("The remaining token is only {}",escrow.remaining_token);
            return Err(TokenError::TokenFinished.into());
        }
        if price < escrow.price
        {
            msg!("The price is lower then set");
            return Err(TokenError::PriceLower.into());
        }
        let (nft_vault_address, bump_seed) = generate_pda_and_bump_seed(
            NFTPREFIX,
            nft_owner.key,
            pda_data.key,
            program_id
        );
        if nft_vault_address != *nft_vault.key
        {
            return Err(ProgramError::MissingRequiredSignature);   
        }

        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda_data.key.to_bytes(),
            &[bump_seed],
        ];

         // new token mint for minting
         let (spl_token_address, _bump_seed_spl) = generate_pda_and_bump_seed(
            SPLTOKENPREFIX,
            nft_owner.key,
            pda_data.key,
            program_id
        );
         // verifying mint
        if spl_token_address!=*spl_token_mint.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        //verifying spl mint token account
        let spl_vault_associated_token =get_associated_token_address(nft_vault.key,spl_token_mint.key);

        if spl_vault_associated_token!=*spl_vault_associated_address.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        //verifying buyer token account
        let buyer_associated_address =get_associated_token_address(buyer.key,spl_token_mint.key);
        if buyer_associated_address!=*buyer_spl_associated.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if buyer_spl_associated.data_is_empty()
        {
        invoke(            
            &spl_associated_token_account::create_associated_token_account(
                buyer.key,
                buyer.key,
                spl_token_mint.key,
            ),&[
                buyer.clone(),
                buyer_spl_associated.clone(),
                buyer.clone(),
                spl_token_mint.clone(),
                token_program_id.clone(),
                rent_info.clone(),
                associated_token_info.clone(),
                system_program.clone()
            ]
        )?;
        }
        invoke(  
            &system_instruction::transfer(
            buyer.key,
            nft_owner.key,
            token*escrow.price,    
        ),
            &[
            nft_vault.clone(),
            nft_owner.clone(),
            buyer.clone(),
            system_program.clone()
            ],
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
                system_program.clone()
            ],&[&nft_vault_signer_seeds],
        )?;

       
        escrow.remaining_token-=token;
        escrow.serialize(&mut &mut pda_data.data.borrow_mut()[..])?;
        Ok(())
    }
    pub fn process_buy_nft_token2(program_id: &Pubkey,accounts: &[AccountInfo],day:u64)-> ProgramResult {
        //The winner of the auction can claim the tokens
        let account_info_iter = &mut accounts.iter();
        let buyer =  next_account_info(account_info_iter)?; // sender or signer
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let pda_data = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let token_program_id = next_account_info(account_info_iter)?; //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let nft_vault = next_account_info(account_info_iter)?; // nft vault
        let spl_vault_associated_address = next_account_info(account_info_iter)?;  // find associated address from nft vault and spl token mint
        let buyer_spl_associated =  next_account_info(account_info_iter)?; // sender or signer
        let spl_token_mint = next_account_info(account_info_iter)?; // spl token mint
        let system_program = next_account_info(account_info_iter)?; 
        let rent_info =next_account_info(account_info_iter)?; 
        let auction_data=next_account_info(account_info_iter)?;
        let associated_token_info= next_account_info(account_info_iter)?;

         //verifying pda_data
         if pda_data.owner!=program_id
         {
             return Err(ProgramError::MissingRequiredSignature);
         }
         if !buyer.is_signer
         {
             return Err(ProgramError::MissingRequiredSignature);
         }
 
        let escrow = NftDetails::try_from_slice(&pda_data.data.borrow())?;
        if escrow.nft_escrow!=*nft_vault.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if escrow.nft_owner!=*nft_owner.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let now = Clock::get()?.unix_timestamp as u64; 
        let days = (now - escrow.create_at)/86400;
        let (nft_vault_address, bump_seed) = generate_pda_and_bump_seed(
            NFTPREFIX,
            nft_owner.key,
            pda_data.key,
            program_id
        );
          //check if the nft_vault is actual pda or not 
          if nft_vault_address != *nft_vault.key
          {
              return Err(ProgramError::MissingRequiredSignature);   
          }
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda_data.key.to_bytes(),
            &[bump_seed],
        ];
        let (spl_token_address, bump_seed_spl) = generate_pda_and_bump_seed(
            SPLTOKENPREFIX,
            nft_owner.key,
            pda_data.key,
            program_id
        );
        // verifying mint
        if spl_token_address!=*spl_token_mint.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let spl_token_signer_seeds: &[&[_]] = &[
            SPLTOKENPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda_data.key.to_bytes(),
            &[bump_seed_spl],
        ];
         //verifying spl mint token account
         let spl_vault_associated_token =get_associated_token_address(nft_vault.key,spl_token_mint.key);

         if spl_vault_associated_token!=*spl_vault_associated_address.key
         {
             return Err(ProgramError::MissingRequiredSignature);
         }
 
         //verifying buyer token account
         let buyer_associated_address =get_associated_token_address(buyer.key,spl_token_mint.key);
         if buyer_associated_address!=*buyer_spl_associated.key
         {
             return Err(ProgramError::MissingRequiredSignature);
         }

        let day_ip=day.to_string();
        let (auction_address,_auction_bump)= Pubkey::find_program_address(
            &[
                AUCTIONPREFIX.as_bytes(),
                &nft_owner.key.to_bytes(),
                day_ip.as_bytes(),
            ],
            program_id,
        );
        if auction_data.data_is_empty()
        {
            msg!("The auction data is empty");
            return Err(TokenError::Notstarted.into());
        }
        if auction_address!=*auction_data.key && auction_data.owner !=program_id
        {
            return Err(ProgramError::MissingRequiredSignature);   
        }
              
        let mut auction_operation = Auction::try_from_slice(&auction_data.data.borrow())?;
        

        if day!=auction_operation.day && day>days
        {
            msg!("The day after auction doesn't match");
            return Err(TokenError::Notstarted.into());

        }
        if auction_operation.max_payer==*buyer.key&& auction_operation.max_price!=0{
            msg!("The maximum payer match");
            if buyer_spl_associated.data_is_empty()
            {
                invoke(            
                    &spl_associated_token_account::create_associated_token_account(
                        buyer.key,
                        buyer.key,
                        spl_token_mint.key,
                    ),&[
                        buyer.clone(),
                        buyer_spl_associated.clone(),
                        buyer.clone(),
                        spl_token_mint.clone(),
                        system_program.clone(),
                        token_program_id.clone(),
                        rent_info.clone(),
                        associated_token_info.clone(),
                    
                    ]
                )?;

            }
            msg!("Token account created or already exist");
                invoke_signed(
                &spl_token::instruction::mint_to_checked(
                    token_program_id.key,
                    spl_token_mint.key,
                    spl_vault_associated_address.key,
                    nft_vault.key,
                    &[&nft_vault.key],
                    auction_operation.num_tokens,
                    9
                )?,&[
                    token_program_id.clone(),
                    spl_token_mint.clone(),
                    spl_vault_associated_address.clone(),
                    nft_vault.clone(),
                    nft_vault.clone(),
                    system_program.clone(),
                    rent_info.clone()
                ],&[nft_vault_signer_seeds,spl_token_signer_seeds]
            )?;
            msg!("Token minted to nft_vault");

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
                system_program.clone()
            ],&[&nft_vault_signer_seeds],
        )?;
        msg!("Token transfered to winner");
        invoke_signed(  
            &system_instruction::transfer(
            nft_vault.key,
            nft_owner.key,
            auction_operation.max_price,    
        ),
            &[
            nft_vault.clone(),
            nft_owner.clone(),
            system_program.clone(),
            ],&[&nft_vault_signer_seeds],
            )?;
        msg!("Amount Released to nft owner");
        auction_operation.max_price=0;
        }
        else
        {
            msg!("You have already claimed or not the winner");
        }
        auction_operation.serialize(&mut &mut auction_data.data.borrow_mut()[..])?;
        escrow.serialize(&mut &mut pda_data.data.borrow_mut()[..])?;

        Ok(())
    }
    // need to improve security using recent blockhash or vrf
    pub fn process_coin_flip(program_id: &Pubkey,accounts: &[AccountInfo],_token:u64)-> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let player =  next_account_info(account_info_iter)?; // sender or signer
        let coinflip_pda = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let token_program_id = next_account_info(account_info_iter)?; //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let spl_vault_associated_address = next_account_info(account_info_iter)?;  // find associated address from nft vault and spl token mint
        let player_associated_token = next_account_info(account_info_iter)?; // spl token mint associate account
        let spl_token_mint = next_account_info(account_info_iter)?; // spl token mint
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let system_program = next_account_info(account_info_iter)?; 
        let pda =next_account_info(account_info_iter)?;  // main data account
        let nft_vault = next_account_info(account_info_iter)?; // nft vault

        if !player.is_signer
        {
            msg!("The player is not the signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
        let now = Clock::get()?.unix_timestamp as u64; 
        let rent = Rent::get()?;
        let transfer_amount =  rent.minimum_balance(std::mem::size_of::<CoinFlip>());
        let (spl_token_address, _bump_seed_spl) = generate_pda_and_bump_seed(
            SPLTOKENPREFIX,
            nft_owner.key,
            pda.key,
            program_id
        );
        // verifying mint
        if spl_token_address!=*spl_token_mint.key
        {
            msg!("SPL Token mint doesn't matches");
            return Err(ProgramError::MissingRequiredSignature);
        }
        let (nft_vault_address, _bump_seed) = generate_pda_and_bump_seed(
            NFTPREFIX,
            nft_owner.key,
            pda.key,
            program_id
        );
          //check if the nft_vault is actual pda or not 
          if nft_vault_address != *nft_vault.key
          {
            msg!("NFT vault doesn't match");
              return Err(ProgramError::MissingRequiredSignature);   
          }
        if pda.owner!=program_id
        {
            return Err(ProgramError::MissingRequiredSignature);   
        }
        let  pda_check = NftDetails::try_from_slice(&pda.data.borrow())?;
        let token_balance=get_token_balance(player_associated_token)?;
        //verifying mint token account
        let player_token_address= get_associated_token_address(player.key, spl_token_mint.key);
        if player_associated_token.data_is_empty() && player_token_address!=*player_associated_token.key
        {
            msg!("You don't have token at all");
            return Err(ProgramError::MissingRequiredSignature);
        }
        let spl_vault_address=get_associated_token_address(nft_vault.key,spl_token_mint.key);
        if  *spl_vault_associated_address.key!=spl_vault_address
        {
            msg!("SPL token account of the vault doesn't matches");
            return Err(ProgramError::MissingRequiredSignature);
        }
        if token_balance < (pda_check.number_of_tokens*2)/3
        {
            msg!("You don't have enough tokens");
            return Err(ProgramError::MissingRequiredSignature);   
        }
       //creating coinflip account
        create_pda_account( 
            player,
            transfer_amount,
            std::mem::size_of::<CoinFlip>(),
            program_id,
            system_program,
            coinflip_pda
        )?;

        let mut coinflip = CoinFlip::try_from_slice(&coinflip_pda.data.borrow())?;
        msg!("Transfering token ....");
        invoke(
            &spl_token::instruction::transfer(
                token_program_id.key,
                player_associated_token.key,
                spl_vault_associated_address.key,
                player.key,
                &[player.key],
                token_balance/1000,
            )?,
            &[
                token_program_id.clone(),
                player_associated_token.clone(),
                spl_vault_associated_address.clone(),
                player.clone(),
                system_program.clone()
            ],
        )?;

        msg!("Flipping the Coin");
        if now % 2 == 0 {
            msg!("You have lost");
            coinflip.won = 0
        }
        else 
        {
            msg!("You have won, claim the reward");
            coinflip.won = 1;
            coinflip.address=*player.key;
            coinflip.amount=pda_check.number_of_tokens/100; //1% of total tokens
        }
        coinflip.serialize(&mut &mut coinflip_pda.data.borrow_mut()[..])?;
        Ok(())
    }
    pub fn process_coin_flip_claim(program_id: &Pubkey,accounts: &[AccountInfo],_token:u64)-> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let player =  next_account_info(account_info_iter)?; // sender or signer
        let nft_owner = next_account_info(account_info_iter)?; // auction creator
        let pda = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let coinflip_pda = next_account_info(account_info_iter)?; // pda data that consists number of tokens , auction created
        let token_program_id = next_account_info(account_info_iter)?; //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let nft_vault = next_account_info(account_info_iter)?; // nft vault
        let spl_vault_associated_address = next_account_info(account_info_iter)?;  // find associated address from nft vault and spl token mint
        let buyer_spl_associated =  next_account_info(account_info_iter)?; // sender or signer
        let spl_token_mint=next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?; 
        let rent_info=next_account_info(account_info_iter)?; 


        let  mut pda_check = NftDetails::try_from_slice(&pda.data.borrow())?;
        if pda.owner !=program_id && coinflip_pda.owner!=program_id
        {
            return Err(ProgramError::MissingRequiredSignature);   
        }
        let (nft_vault_address, bump_seed) = generate_pda_and_bump_seed(
            NFTPREFIX,
            nft_owner.key,
            pda.key,
            program_id
        );
        if nft_vault_address != *nft_vault.key
        {
            return Err(ProgramError::MissingRequiredSignature);   
        }
        let nft_vault_signer_seeds: &[&[_]] = &[
            NFTPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda.key.to_bytes(),
            &[bump_seed],
        ];
        let (spl_token_address, bump_seed_spl) = generate_pda_and_bump_seed(
            SPLTOKENPREFIX,
            nft_owner.key,
            pda.key,
            program_id
        );
        if spl_token_address!=*spl_token_mint.key
        {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let spl_token_signer_seeds: &[&[_]] = &[
            SPLTOKENPREFIX.as_bytes(),
            &nft_owner.key.to_bytes(),
            &pda.key.to_bytes(),
            &[bump_seed_spl],
        ];
        let player_token_address= get_associated_token_address(player.key, spl_token_mint.key);
        if  player_token_address!=*buyer_spl_associated.key
        {
            msg!("Token Account doesn't matches");
            return Err(ProgramError::MissingRequiredSignature);
        }
        let spl_vault_address=get_associated_token_address(nft_vault.key,spl_token_mint.key);
        if  *spl_vault_associated_address.key!=spl_vault_address
        {
            msg!("SPL token account of the vault doesn't matches");
            return Err(ProgramError::MissingRequiredSignature);
        }
        let mut coinflip = CoinFlip::try_from_slice(&coinflip_pda.data.borrow())?;
        if coinflip.won == 1 && coinflip.address == *player.key
        {
            invoke_signed(
                &spl_token::instruction::mint_to_checked(
                    token_program_id.key,
                    spl_token_mint.key,
                    spl_vault_associated_address.key,
                    nft_vault.key,
                    &[&nft_vault.key],
                    coinflip.amount,
                    9
                )?,&[
                    token_program_id.clone(),
                    spl_token_mint.clone(),
                    spl_vault_associated_address.clone(),
                    nft_vault.clone(),
                    nft_vault.clone(),
                    system_program.clone(),
                    rent_info.clone()
                ],&[nft_vault_signer_seeds,spl_token_signer_seeds]
            )?;
        

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program_id.key,
                    spl_vault_associated_address.key,
                    buyer_spl_associated.key,
                    nft_vault.key,
                    &[nft_vault.key],
                    coinflip.amount,
                )?,
                &[
                    token_program_id.clone(),
                    spl_vault_associated_address.clone(),
                    buyer_spl_associated.clone(),
                    nft_vault.clone(),
                    system_program.clone()
                ],&[&nft_vault_signer_seeds],
            )?;
            invoke(
                &system_instruction::transfer
                (
                    player.key, 
                    nft_owner.key, 
                    coinflip.amount*pda_check.price,
                ),
                   &[
                       player.clone(),
                       nft_owner.clone(),
                       system_program.clone(),
                   ])?;
        pda_check.number_of_tokens+=coinflip.amount;
        coinflip.won=0;
        }
        else
        {
            msg!("You haven't won or you have already claimed");
        }

        coinflip.serialize(&mut &mut coinflip_pda.data.borrow_mut()[..])?;
        pda_check.serialize(&mut &mut pda.data.borrow_mut()[..])?;

        Ok(())
    }
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TokenInstruction::unpack(input)?;
        match instruction {
            TokenInstruction::ProcessDeposit (ProcessDeposit{
                number_of_tokens,
                price,
            }) => {
                msg!("Instruction: Fractionalizing NFT");
                Self::process_deposit_nft(program_id,accounts,number_of_tokens, price)
            }
            TokenInstruction::ProcessBuy(ProcessBuy{token,price}) => {
                msg!("Instruction: Buy token");
                Self::process_buy_nft_token(program_id,accounts,token,price)
            }
            TokenInstruction::ProcessBuy2(ProcessBuy2{day}) => {
                msg!("Instruction:  Buy token");
                Self::process_buy_nft_token2(program_id,accounts,day)
            }
            TokenInstruction::ProcessCoinFlip(ProcessCoinFlip{token}) => {
                msg!("Instruction:  Flip Coin");
                Self::process_coin_flip(program_id,accounts,token)
            }
            TokenInstruction::ProcessClaimCoinFlip(ProcessClaimCoinFlip{token}) => {
                msg!("Instruction:  Claim Token");
                Self::process_coin_flip_claim(program_id,accounts,token)
            }
            TokenInstruction::ProcessAuction1(ProcessAuction1{price}) => {
                msg!("Instruction:  Auction");
                Self::auction1(program_id, accounts, price)
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
            TokenError::AuctionStarted => msg!("Error: Buy Period Ended"),
            TokenError::Overflow => msg!("Error: Token Overflow"),
            TokenError::Notstarted =>msg!("Error: Not started"),
            TokenError::TokenFinished =>msg!("Error: Token Finished"),
            TokenError::PriceLower =>msg!("Error: Price is Lower"),
        }
    }
}