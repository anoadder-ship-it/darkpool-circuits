use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_macros::circuit_hash;
use arcium_client::idl::arcium::types::{OffChainCircuitSource, CircuitSource, CallbackAccount, CallbackInstruction};

const COMP_DEF_OFFSET_PLACE_ORDER:  u32 = comp_def_offset("place_order");
const COMP_DEF_OFFSET_MATCH_ORDERS: u32 = comp_def_offset("match_orders");
const COMP_DEF_OFFSET_CANCEL_ORDER: u32 = comp_def_offset("cancel_order");
const COMP_DEF_OFFSET_GET_STATS:    u32 = comp_def_offset("get_stats");
const COMP_DEF_OFFSET_INIT_ORDER_BOOK: u32 = comp_def_offset("init_order_book");
const COMP_DEF_OFFSET_UPDATE_REPUTATION: u32 = comp_def_offset("update_reputation");
const COMP_DEF_OFFSET_CHECK_THRESHOLD:   u32 = comp_def_offset("check_threshold");

declare_id!("FiNnbmFNn9BfRNKarLSqVtnKvQ9JwcuUvHxZjvMbDVDL");

// PoolStatus enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PoolStatus {
    Active,
    Moeras,
}

// PoolState account
#[account]
pub struct PoolState {
    pub guardian: Pubkey,
    pub status: PoolStatus,
    pub last_heartbeat_slot: u64,
}

// SendHeartbeat accounts context
#[derive(Accounts)]
pub struct SendHeartbeat<'info> {
    #[account(mut)]
    pub pool: Account<'info, PoolState>,
    pub signer: Signer<'info>,
}

// TriggerMoeras accounts context
#[derive(Accounts)]
pub struct TriggerMoeras<'info> {
    #[account(mut)]
    pub pool: Account<'info, PoolState>,
    pub signer: Signer<'info>,
}

// ReactivatePool accounts context
#[derive(Accounts)]
pub struct ReactivatePool<'info> {
    #[account(mut)]
    pub pool: Account<'info, PoolState>,
    pub signer: Signer<'info>,
}
// InitializePool accounts context
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = user, space = 8 + 32 + 2 + 8)]
    pub pool: Account<'info, PoolState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}


const PLACE_ORDER_URL:  &str = "https://github.com/anoadder-ship-it/darkpool-circuits/releases/download/v0.11.4/place_order.arcis";
const MATCH_ORDERS_URL: &str = "https://github.com/anoadder-ship-it/darkpool-circuits/releases/download/v0.11.4/match_orders.arcis";
const CANCEL_ORDER_URL: &str = "https://github.com/anoadder-ship-it/darkpool-circuits/releases/download/v0.11.4/cancel_order.arcis";
const GET_STATS_URL:    &str = "https://github.com/anoadder-ship-it/darkpool-circuits/releases/download/v0.11.4/get_stats.arcis";
const INIT_ORDER_BOOK_URL: &str = "https://github.com/anoadder-ship-it/darkpool-circuits/releases/download/v0.12.0/init_order_book.arcis";
const UPDATE_REPUTATION_URL: &str = "https://github.com/anoadder-ship-it/darkpool-circuits/releases/download/v0.11.5/update_reputation.arcis";
const CHECK_THRESHOLD_URL:   &str = "https://github.com/anoadder-ship-it/darkpool-circuits/releases/download/v0.11.5/check_threshold.arcis";

#[arcium_program]
pub mod solana_darkpool {
    use super::*;

    pub fn init_place_order_comp_def(ctx: Context<InitPlaceOrderCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: PLACE_ORDER_URL.to_string(),
            hash: circuit_hash!("place_order"),
        })))?;
        Ok(())
    }

    pub fn init_match_orders_comp_def(ctx: Context<InitMatchOrdersCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: MATCH_ORDERS_URL.to_string(),
            hash: circuit_hash!("match_orders"),
        })))?;
        Ok(())
    }

    pub fn init_cancel_order_comp_def(ctx: Context<InitCancelOrderCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: CANCEL_ORDER_URL.to_string(),
            hash: circuit_hash!("cancel_order"),
        })))?;
        Ok(())
    }

    pub fn init_get_stats_comp_def(ctx: Context<InitGetStatsCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: GET_STATS_URL.to_string(),
            hash: circuit_hash!("get_stats"),
        })))?;
        Ok(())
    }

    pub fn place_order(
        ctx: Context<PlaceOrder>,
        computation_offset: u64,
        encrypted_bid:    [u8; 32],
        encrypted_size:   [u8; 32],
        encrypted_is_buy: [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        require!(ctx.accounts.moeras_pool.status == PoolStatus::Active, ErrorCode::MoerasModeActive);
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(encrypted_bid)
            .encrypted_u64(encrypted_size)
            .encrypted_u64(encrypted_is_buy)
            .build();
        queue_computation(
            ctx.accounts, computation_offset, args,
            vec![PlaceOrderCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "place_order")]
    pub fn place_order_callback(
        ctx: Context<PlaceOrderCallback>,
        output: SignedComputationOutputs<PlaceOrderOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(PlaceOrderOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(OrderPlacedEvent { result: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
        Ok(())
    }

    pub fn match_orders(
        ctx: Context<MatchOrders>,
        computation_offset: u64,
        encrypted_buy_bid:  [u8; 32],
        encrypted_sell_bid: [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        require!(ctx.accounts.moeras_pool.status == PoolStatus::Active, ErrorCode::MoerasModeActive);
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(encrypted_buy_bid)
            .encrypted_u64(encrypted_sell_bid)
            .build();
        queue_computation(
            ctx.accounts, computation_offset, args,
            vec![MatchOrdersCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "match_orders")]
    pub fn match_orders_callback(
        ctx: Context<MatchOrdersCallback>,
        output: SignedComputationOutputs<MatchOrdersOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(MatchOrdersOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(MatchEvent { result: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
        Ok(())
    }

    pub fn cancel_order(
        ctx: Context<CancelOrder>,
        computation_offset: u64,
        encrypted_order_id: [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        require!(ctx.accounts.moeras_pool.status == PoolStatus::Active, ErrorCode::MoerasModeActive);
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(encrypted_order_id)
            .build();
        queue_computation(
            ctx.accounts, computation_offset, args,
            vec![CancelOrderCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "cancel_order")]
    pub fn cancel_order_callback(
        ctx: Context<CancelOrderCallback>,
        output: SignedComputationOutputs<CancelOrderOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(CancelOrderOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(OrderCancelledEvent { result: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
        Ok(())
    }


    // Pool-initialisatie: maakt het PoolState-account aan, initialisator wordt guardian
    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.guardian = ctx.accounts.user.key();
        pool.status = PoolStatus::Active;
        pool.last_heartbeat_slot = Clock::get()?.slot;
        msg!("Pool geinitialiseerd, guardian: {}", pool.guardian);
        Ok(())
    }
    // Herstelfunctie: zet Moeras-modus terug naar Active, alleen door de guardian
    pub fn reactivate_pool(ctx: Context<ReactivatePool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(ctx.accounts.signer.key() == pool.guardian, ErrorCode::UnauthorizedGuardian);
        pool.status = PoolStatus::Active;
        pool.last_heartbeat_slot = Clock::get()?.slot;
        msg!("Pool gereactiveerd, trading hervat.");
        Ok(())
    }
    // Heartbeat functie
    pub fn send_heartbeat(ctx: Context<SendHeartbeat>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(ctx.accounts.signer.key() == pool.guardian, ErrorCode::UnauthorizedGuardian);
        pool.last_heartbeat_slot = Clock::get()?.slot;
        Ok(())
    }

    // Moeras trigger functie
    pub fn trigger_moeras(ctx: Context<TriggerMoeras>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(ctx.accounts.signer.key() == pool.guardian, ErrorCode::UnauthorizedGuardian);
        pool.status = PoolStatus::Moeras;
        msg!("🚨 Moeras-modus geactiveerd!");
        Ok(())
    }
    pub fn get_stats(
        ctx: Context<GetStats>,
        computation_offset: u64,
        encrypted_buy_vol:  [u8; 32],
        encrypted_sell_vol: [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(encrypted_buy_vol)
            .encrypted_u64(encrypted_sell_vol)
            .build();
        queue_computation(
            ctx.accounts, computation_offset, args,
            vec![GetStatsCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "get_stats")]
    pub fn get_stats_callback(
        ctx: Context<GetStatsCallback>,
        output: SignedComputationOutputs<GetStatsOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(GetStatsOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(StatsEvent { result: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
        Ok(())
    }

    // ============================================================
    // Escrow-mechanisme voor Trading Darkpool
    // Koper stort SOL bij een trade; verkoper claimt na levering, of
    // koper disput binnen de termijn. Disputes worden beslecht via de
    // Squads-multisig (vault-PDA) via resolve_dispute. Zonder dispute
    // binnen de termijn mag de verkoper zelf claimen.
    // ============================================================

    pub fn init_update_reputation_comp_def(ctx: Context<InitUpdateReputationCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: UPDATE_REPUTATION_URL.to_string(),
            hash: circuit_hash!("update_reputation"),
        })))?;
        Ok(())
    }

    pub fn init_check_threshold_comp_def(ctx: Context<InitCheckThresholdCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: CHECK_THRESHOLD_URL.to_string(),
            hash: circuit_hash!("check_threshold"),
        })))?;
        Ok(())
    }

    pub fn update_reputation(
        ctx: Context<UpdateReputation>,
        computation_offset: u64,
        enc_completed_trades: [u8; 32],
        enc_disputes_lost:    [u8; 32],
        enc_score:            [u8; 32],
        enc_is_completion:    [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_completed_trades)
            .encrypted_u64(enc_disputes_lost)
            .encrypted_u64(enc_score)
            .encrypted_u64(enc_is_completion)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![UpdateReputationCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "update_reputation")]
    pub fn update_reputation_callback(
        ctx: Context<UpdateReputationCallback>,
        output: SignedComputationOutputs<UpdateReputationOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(UpdateReputationOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(ReputationUpdatedEvent {
            completed_trades: o.ciphertexts[0],
            disputes_lost:    o.ciphertexts[1],
            score:             o.ciphertexts[2],
            nonce:             o.nonce.to_le_bytes(),
        });
        Ok(())
    }

    pub fn check_threshold(
        ctx: Context<CheckThreshold>,
        computation_offset: u64,
        enc_score:     [u8; 32],
        enc_min_score: [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_score)
            .encrypted_u64(enc_min_score)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![CheckThresholdCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "check_threshold")]
    pub fn check_threshold_callback(
        ctx: Context<CheckThresholdCallback>,
        output: SignedComputationOutputs<CheckThresholdOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(CheckThresholdOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(ThresholdCheckedEvent { passes: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
        Ok(())
    }

    pub fn init_init_order_book_comp_def(ctx: Context<InitInitOrderBookCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: INIT_ORDER_BOOK_URL.to_string(),
            hash: circuit_hash!("init_order_book"),
        })))?;
        Ok(())
    }

    pub fn initialize_order_book(
        ctx: Context<InitializeOrderBook>,
        computation_offset: u64,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new().build();
        let callback_ix = CallbackInstruction {
            program_id: ID_CONST,
            discriminator: instruction::InitOrderBookCallback::DISCRIMINATOR.to_vec(),
            accounts: vec![
                CallbackAccount { pubkey: ctx.accounts.arcium_program.key(), is_writable: false },
                CallbackAccount { pubkey: derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_ORDER_BOOK), is_writable: false },
                CallbackAccount { pubkey: derive_mxe_pda!(), is_writable: false },
                CallbackAccount { pubkey: derive_cluster_pda!(ctx.accounts.mxe_account), is_writable: false },
                CallbackAccount { pubkey: ::arcium_anchor::solana_instructions_sysvar::ID, is_writable: false },
                CallbackAccount { pubkey: ctx.accounts.order_book_state.key(), is_writable: true },
            ],
        };
        queue_computation(
            ctx.accounts, computation_offset, args,
            vec![callback_ix],
            1, 0, 0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "init_order_book")]
    pub fn init_order_book_callback(
        ctx: Context<InitOrderBookCallback>,
        output: SignedComputationOutputs<InitOrderBookOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(InitOrderBookOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        let data = ctx.accounts.order_book_state.to_account_info();
        let mut bytes = data.try_borrow_mut_data()?;
        for (i, ct) in o.ciphertexts.iter().enumerate() {
            let start = 8 + i * 32;
            bytes[start..start + 32].copy_from_slice(ct);
        }
        Ok(())
    }

    pub fn create_escrow(ctx: Context<CreateEscrow>, amount: u64, seller: Pubkey, seed_id: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidEscrowAmount);

        let clock = Clock::get()?;
        let buyer_key = ctx.accounts.buyer.key();
        let escrow_key = ctx.accounts.escrow_account.key();

        {
            let escrow = &mut ctx.accounts.escrow_account;
            escrow.buyer = buyer_key;
            escrow.seller = seller;
            escrow.amount = amount;
            escrow.status = EscrowStatus::Pending;
            escrow.seed_id = seed_id;
            escrow.created_at = clock.unix_timestamp;
            escrow.dispute_deadline = clock.unix_timestamp + ESCROW_DISPUTE_PERIOD_SECONDS;
            escrow.bump = ctx.bumps.escrow_account;
        }

        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &buyer_key,
            &escrow_key,
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        emit!(EscrowCreatedEvent {
            escrow: escrow_key,
            buyer: buyer_key,
            seller,
            amount,
        });
        Ok(())
    }

    pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        require!(escrow.status == EscrowStatus::Pending, ErrorCode::InvalidEscrowStatus);

        let amount = escrow.amount;
        escrow.status = EscrowStatus::Released;

        **ctx.accounts.escrow_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += amount;

        emit!(EscrowReleasedEvent { escrow: ctx.accounts.escrow_account.key(), amount });
        Ok(())
    }

    pub fn dispute_escrow(ctx: Context<DisputeEscrow>) -> Result<()> {
        let clock = Clock::get()?;
        let escrow = &mut ctx.accounts.escrow_account;
        require!(escrow.status == EscrowStatus::Pending, ErrorCode::InvalidEscrowStatus);
        require!(clock.unix_timestamp < escrow.dispute_deadline, ErrorCode::DisputeWindowClosed);

        escrow.status = EscrowStatus::Disputed;
        emit!(EscrowDisputedEvent { escrow: ctx.accounts.escrow_account.key() });
        Ok(())
    }

    pub fn resolve_dispute(ctx: Context<ResolveDispute>, release_to_seller: bool) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        require!(escrow.status == EscrowStatus::Disputed, ErrorCode::InvalidEscrowStatus);

        let amount = escrow.amount;
        escrow.status = if release_to_seller { EscrowStatus::Released } else { EscrowStatus::Refunded };

        let recipient_info = if release_to_seller {
            ctx.accounts.seller.to_account_info()
        } else {
            ctx.accounts.buyer.to_account_info()
        };
        **ctx.accounts.escrow_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        **recipient_info.try_borrow_mut_lamports()? += amount;

        emit!(EscrowDisputeResolvedEvent {
            escrow: ctx.accounts.escrow_account.key(),
            released_to_seller: release_to_seller,
            amount,
        });
        Ok(())
    }

    pub fn claim_after_timeout(ctx: Context<ClaimAfterTimeout>) -> Result<()> {
        let clock = Clock::get()?;
        let escrow = &mut ctx.accounts.escrow_account;
        require!(escrow.status == EscrowStatus::Pending, ErrorCode::InvalidEscrowStatus);
        require!(clock.unix_timestamp >= escrow.dispute_deadline, ErrorCode::DisputeWindowStillOpen);

        let amount = escrow.amount;
        escrow.status = EscrowStatus::Released;

        **ctx.accounts.escrow_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += amount;

        emit!(EscrowReleasedEvent { escrow: ctx.accounts.escrow_account.key(), amount });
        Ok(())
    }
}

#[queue_computation_accounts("place_order", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct PlaceOrder<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))]
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PLACE_ORDER))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    pub moeras_pool: Account<'info, PoolState>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("place_order")]
#[derive(Accounts)]
pub struct PlaceOrderCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PLACE_ORDER))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("place_order", payer)]
#[derive(Accounts)]
pub struct InitPlaceOrderCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: arcium.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("match_orders", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct MatchOrders<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))]
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_ORDERS))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    pub moeras_pool: Account<'info, PoolState>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("match_orders")]
#[derive(Accounts)]
pub struct MatchOrdersCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_ORDERS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("match_orders", payer)]
#[derive(Accounts)]
pub struct InitMatchOrdersCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: arcium.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("cancel_order", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))]
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CANCEL_ORDER))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    pub moeras_pool: Account<'info, PoolState>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("cancel_order")]
#[derive(Accounts)]
pub struct CancelOrderCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CANCEL_ORDER))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("cancel_order", payer)]
#[derive(Accounts)]
pub struct InitCancelOrderCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: arcium.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("get_stats", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct GetStats<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))]
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_GET_STATS))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("get_stats")]
#[derive(Accounts)]
pub struct GetStatsCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_GET_STATS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("get_stats", payer)]
#[derive(Accounts)]
pub struct InitGetStatsCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: arcium.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("update_reputation", payer)]
#[derive(Accounts)]
pub struct InitUpdateReputationCompDef<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: not yet initialized.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: arcium.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("check_threshold", payer)]
#[derive(Accounts)]
pub struct InitCheckThresholdCompDef<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: not yet initialized.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: arcium.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("update_reputation", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct UpdateReputation<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))]
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_UPDATE_REPUTATION))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("update_reputation")]
#[derive(Accounts)]
pub struct UpdateReputationCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_UPDATE_REPUTATION))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[queue_computation_accounts("check_threshold", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CheckThreshold<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))]
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CHECK_THRESHOLD))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("check_threshold")]
#[derive(Accounts)]
pub struct CheckThresholdCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CHECK_THRESHOLD))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

pub const ORDER_BOOK_CT_LEN: usize = 6001;

#[account]
pub struct OrderBookState {
    pub ciphertexts: [[u8; 32]; ORDER_BOOK_CT_LEN],
}
impl OrderBookState {
    pub const SPACE: usize = 8 + ORDER_BOOK_CT_LEN * 32;
}

#[init_computation_definition_accounts("init_order_book", payer)]
#[derive(Accounts)]
pub struct InitInitOrderBookCompDef<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: not yet initialized.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: arcium.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("init_order_book", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct InitializeOrderBook<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))]
    /// CHECK: arcium.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))]
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_ORDER_BOOK))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    #[account(init, payer = payer, space = OrderBookState::SPACE, seeds = [b"order_book"], bump)]
    pub order_book_state: Box<Account<'info, OrderBookState>>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("init_order_book")]
#[derive(Accounts)]
pub struct InitOrderBookCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_ORDER_BOOK))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"order_book"], bump)]
    pub order_book_state: Box<Account<'info, OrderBookState>>,
}

#[event] pub struct OrderPlacedEvent    { pub result: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct MatchEvent          { pub result: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct OrderCancelledEvent { pub result: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct StatsEvent          { pub result: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct ReputationUpdatedEvent { pub completed_trades: [u8; 32], pub disputes_lost: [u8; 32], pub score: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct ThresholdCheckedEvent  { pub passes: [u8; 32], pub nonce: [u8; 16] }

#[error_code]
pub enum ErrorCode {
    #[msg("Onbevoegde aanroep. Alleen de DGX Spark Guardian mag dit doen.")]
    UnauthorizedGuardian,
    #[msg("The computation was aborted")]
    AbortedComputation,

    #[msg("Moeras-modus is actief: trading is tijdelijk bevroren voor beveiligingsonderzoek")]
    MoerasModeActive,
    #[msg("Escrow amount must be greater than zero")]
    InvalidEscrowAmount,
    #[msg("Escrow is not in the required status for this action")]
    InvalidEscrowStatus,
    #[msg("This account is not authorized to perform this escrow action")]
    UnauthorizedEscrowAction,
    #[msg("The dispute window for this escrow has closed")]
    DisputeWindowClosed,
    #[msg("The dispute window for this escrow is still open")]
    DisputeWindowStillOpen,
    #[msg("Only the vault PDA (multisig) may resolve a dispute")]
    UnauthorizedArbiter,
}


// ============================================================
// Escrow-mechanisme: types, accounts en events
// ============================================================
pub const ESCROW_DISPUTE_PERIOD_SECONDS: i64 = 7 * 24 * 60 * 60; // 7 dagen
pub const VAULT_PDA: Pubkey = anchor_lang::prelude::pubkey!("EmYvQBX7WPmLDnYEhSGRPv9wWf9whAEgLnZviSc4xWqY");
#[account]
pub struct EscrowAccount {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub amount: u64,
    pub status: EscrowStatus,
    pub seed_id: u64,
    pub created_at: i64,
    pub dispute_deadline: i64,
    pub bump: u8,
}
impl EscrowAccount {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 1 + 8 + 8 + 8 + 1;
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum EscrowStatus {
    Pending,
    Released,
    Disputed,
    Refunded,
}
#[derive(Accounts)]
#[instruction(amount: u64, seller: Pubkey, seed_id: u64)]
pub struct CreateEscrow<'info> {
    #[account(mut)] pub buyer: Signer<'info>,
    #[account(
        init,
        payer = buyer,
        space = EscrowAccount::SPACE,
        seeds = [b"escrow", buyer.key().as_ref(), seller.as_ref(), &seed_id.to_le_bytes()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct ReleaseEscrow<'info> {
    #[account(constraint = buyer.key() == escrow_account.buyer @ ErrorCode::UnauthorizedEscrowAction)]
    pub buyer: Signer<'info>,
    #[account(mut, seeds = [b"escrow", escrow_account.buyer.as_ref(), escrow_account.seller.as_ref(), &escrow_account.seed_id.to_le_bytes()], bump = escrow_account.bump)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut, address = escrow_account.seller)]
    /// CHECK: adres wordt geverifieerd tegen escrow_account.seller
    pub seller: UncheckedAccount<'info>,
}
#[derive(Accounts)]
pub struct DisputeEscrow<'info> {
    #[account(constraint = disputer.key() == escrow_account.buyer || disputer.key() == escrow_account.seller @ ErrorCode::UnauthorizedEscrowAction)]
    pub disputer: Signer<'info>,
    #[account(mut, seeds = [b"escrow", escrow_account.buyer.as_ref(), escrow_account.seller.as_ref(), &escrow_account.seed_id.to_le_bytes()], bump = escrow_account.bump)]
    pub escrow_account: Account<'info, EscrowAccount>,
}
#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    #[account(constraint = authority.key() == VAULT_PDA @ ErrorCode::UnauthorizedArbiter)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"escrow", escrow_account.buyer.as_ref(), escrow_account.seller.as_ref(), &escrow_account.seed_id.to_le_bytes()], bump = escrow_account.bump)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut, address = escrow_account.buyer)]
    /// CHECK: adres wordt geverifieerd tegen escrow_account.buyer
    pub buyer: UncheckedAccount<'info>,
    #[account(mut, address = escrow_account.seller)]
    /// CHECK: adres wordt geverifieerd tegen escrow_account.seller
    pub seller: UncheckedAccount<'info>,
}
#[derive(Accounts)]
pub struct ClaimAfterTimeout<'info> {
    #[account(constraint = seller.key() == escrow_account.seller @ ErrorCode::UnauthorizedEscrowAction)]
    pub seller: Signer<'info>,
    #[account(mut, seeds = [b"escrow", escrow_account.buyer.as_ref(), escrow_account.seller.as_ref(), &escrow_account.seed_id.to_le_bytes()], bump = escrow_account.bump)]
    pub escrow_account: Account<'info, EscrowAccount>,
}
#[event]
pub struct EscrowCreatedEvent {
    pub escrow: Pubkey,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub amount: u64,
}
#[event]
pub struct EscrowReleasedEvent {
    pub escrow: Pubkey,
    pub amount: u64,
}
#[event]
pub struct EscrowDisputedEvent {
    pub escrow: Pubkey,
}
#[event]
pub struct EscrowDisputeResolvedEvent {
    pub escrow: Pubkey,
    pub released_to_seller: bool,
    pub amount: u64,
}
