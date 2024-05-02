use anchor_lang::prelude::*;

#[program]
mod nftraffle {
    use super::*;

    #[state]
    pub struct NFTRaffle {
        pub owner: Pubkey,
        pub entry_count: HashMap<Pubkey, u64>,
        pub players: Vec<Pubkey>,
        pub player_selector: Vec<Pubkey>,
        pub raffle_status: bool,
        pub entry_cost: u64,
        pub nft_address: Pubkey,
        pub nft_id: u64,
        pub total_entries: u64,
    }

    impl NFTRaffle {
        pub fn new(ctx: Context<Initialize>, entry_cost: u64) -> ProgramResult {
            let raffle = &mut ctx.accounts.raffle;
            raffle.owner = *ctx.accounts.owner.key;
            raffle.entry_cost = entry_cost;
            raffle.raffle_status = false;
            raffle.total_entries = 0;
            Ok(())
        }

        pub fn initialize_raffle(
            ctx: Context<InitializeRaffle>,
            nft_address: Pubkey,
            nft_id: u64,
        ) -> ProgramResult {
            let raffle = &mut ctx.accounts.raffle;
            let owner = &ctx.accounts.owner;
            if raffle.raffle_status {
                return Err(ErrorCode::RaffleAlreadyStarted.into());
            }
            if raffle.nft_address != Pubkey::default() {
                return Err(ErrorCode::NFTPrizeAlreadySet.into());
            }
            if owner.key != &nft_address {
                return Err(ErrorCode::OwnerDoesNotOwnNFT.into());
            }

            raffle.nft_address = nft_address;
            raffle.nft_id = nft_id;
            raffle.raffle_status = true;
            Ok(())
        }

        pub fn buy_entry(ctx: Context<BuyEntry>, number_of_entries: u64) -> ProgramResult {
            let raffle = &mut ctx.accounts.raffle;
            let payer = &ctx.accounts.payer;
    
            if !raffle.raffle_status {
                return Err(ErrorCode::RaffleNotStarted.into());
            }
    
            let required_amount = raffle.entry_cost * number_of_entries;
            if payer.lamports() < required_amount {
                return Err(ErrorCode::InsufficientFunds.into());
            }
    
            let player = *ctx.accounts.player.key;
            raffle.entry_count.entry(player).or_insert(0);
            *raffle.entry_count.get_mut(&player).unwrap() += number_of_entries;
    
            raffle.total_entries += number_of_entries;
    
            if !raffle.players.contains(&player) {
                raffle.players.push(player);
            }
    
            for _ in 0..number_of_entries {
                raffle.player_selector.push(player);
            }
    
            Ok(())
        }
    
        pub fn end_raffle(ctx: Context<EndRaffle>) -> ProgramResult {
            let raffle = &mut ctx.accounts.raffle;
    
            if !raffle.raffle_status {
                return Err(ErrorCode::RaffleNotStarted.into());
            }
    
            raffle.raffle_status = false;
            Ok(())
        }
    
        pub fn select_winner(ctx: Context<SelectWinner>) -> ProgramResult {
            let raffle = &mut ctx.accounts.raffle;
    
            if raffle.raffle_status {
                return Err(ErrorCode::RaffleStillRunning.into());
            }
    
            if raffle.player_selector.is_empty() {
                return Err(ErrorCode::NoPlayerInRaffle.into());
            }
    
            if raffle.nft_address == Pubkey::default() {
                return Err(ErrorCode::NFTPrizeNotSet.into());
            }
    
            let winner_index = (rand::random::<usize>()) % raffle.player_selector.len();
            let winner = raffle.player_selector[winner_index];
    
            let winner_account = &ctx.accounts.winner_account;
            raffle.nft_address.transfer(&raffle.owner, winner, raffle.nft_id)?;
    
            raffle.entry_count.clear();
            raffle.players.clear();
            raffle.player_selector.clear();
            raffle.nft_address = Pubkey::default();
            raffle.nft_id = 0;
            raffle.total_entries = 0;
    
            Ok(())
        }

        pub fn change_entry_cost(ctx: Context<ChangeEntryCost>, new_entry_cost: u64) -> ProgramResult {
            let raffle = &mut ctx.accounts.raffle;
    
            if raffle.raffle_status {
                return Err(ErrorCode::RaffleInProgress.into());
            }
    
            raffle.entry_cost = new_entry_cost;
            Ok(())
        }
    
        pub fn withdraw_balance(ctx: Context<WithdrawBalance>) -> ProgramResult {
            let raffle = &mut ctx.accounts.raffle;
            let owner = &ctx.accounts.owner;
    
            if raffle.balance() == 0 {
                return Err(ErrorCode::NoBalanceToWithdraw.into());
            }
    
            owner.try_account_ref_mut()?.lamports += raffle.balance();
            Ok(())
        }
    
        pub fn reset_contract(ctx: Context<ResetContract>) -> ProgramResult {
            let raffle = &mut ctx.accounts.raffle;
    
            raffle.entry_count.clear();
            raffle.players.clear();
            raffle.player_selector.clear();
            raffle.raffle_status = false;
            raffle.nft_address = Pubkey::default();
            raffle.nft_id = 0;
            raffle.entry_cost = 0;
            raffle.total_entries = 0;
    
            Ok(())
        }

        // Other methods omitted for brevity
    }

    #[derive(Accounts)]
    pub struct Initialize<'info> {
        #[account(init, payer = owner, space = 8 + 8 + 32 + 8)]
        pub raffle: ProgramAccount<'info, NFTRaffle>,
        pub owner: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    pub struct InitializeRaffle<'info> {
        #[account(mut)]
        pub raffle: ProgramAccount<'info, NFTRaffle>,
        pub owner: AccountInfo<'info>,
        pub system_program: Program<'info, System>,
    }

    #[error]
    pub enum ErrorCode {
        #[msg("Raffle is already started")]
        RaffleAlreadyStarted,
        #[msg("NFT prize is already set")]
        NFTPrizeAlreadySet,
        #[msg("Owner does not own the NFT")]
        OwnerDoesNotOwnNFT,
    }

}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    nftraffle::process_instruction(program_id, accounts, instruction_data)
}