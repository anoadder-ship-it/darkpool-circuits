use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_macros::circuit_hash;
use arcium_client::idl::arcium::types::{OffChainCircuitSource, CircuitSource, CallbackAccount, CallbackInstruction};

const COMP_DEF_OFFSET_REGISTER: u32 = comp_def_offset("register_chip");
const COMP_DEF_OFFSET_MATCH:    u32 = comp_def_offset("match_chip");
const COMP_DEF_OFFSET_AGGREGATE:  u32 = comp_def_offset("aggregate_volume");
const COMP_DEF_OFFSET_INIT_BOOK:  u32 = comp_def_offset("init_chip_book");
const COMP_DEF_OFFSET_SETTLE:     u32 = comp_def_offset("settle_chip");
const COMP_DEF_OFFSET_CANCEL:     u32 = comp_def_offset("cancel_chip");
const COMP_DEF_OFFSET_UPDATE_REPUTATION: u32 = comp_def_offset("update_reputation");
const COMP_DEF_OFFSET_CHECK_THRESHOLD:   u32 = comp_def_offset("check_threshold");

declare_id!("6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o");

const REGISTER_CHIP_URL: &str = "https://github.com/anoadder-ship-it/chip-circuits/releases/download/v0.3.0/register_chip.arcis";
const MATCH_CHIP_URL:    &str = "https://github.com/anoadder-ship-it/chip-circuits/releases/download/v0.3.0/match_chip.arcis";
const AGGREGATE_URL:  &str = "https://github.com/anoadder-ship-it/chip-circuits/releases/download/v0.3.0/aggregate_volume.arcis";
const INIT_BOOK_URL:  &str = "https://github.com/anoadder-ship-it/chip-circuits/releases/download/v0.3.0/init_chip_book.arcis";
const SETTLE_URL:     &str = "https://github.com/anoadder-ship-it/chip-circuits/releases/download/v0.3.0/settle_chip.arcis";
const CANCEL_URL:     &str = "https://github.com/anoadder-ship-it/chip-circuits/releases/download/v0.3.0/cancel_chip.arcis";
const UPDATE_REPUTATION_URL: &str = "https://github.com/anoadder-ship-it/chip-circuits/releases/download/v0.2.0/update_reputation.arcis";
const CHECK_THRESHOLD_URL:   &str = "https://github.com/anoadder-ship-it/chip-circuits/releases/download/v0.2.0/check_threshold.arcis";

#[arcium_program]
pub mod chip_darkpool {
    use super::*;

    pub fn init_register_chip_comp_def(ctx: Context<InitRegisterChipCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: REGISTER_CHIP_URL.to_string(),
            hash: circuit_hash!("register_chip"),
        })))?;
        Ok(())
    }

    pub fn init_match_chip_comp_def(ctx: Context<InitMatchChipCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: MATCH_CHIP_URL.to_string(),
            hash: circuit_hash!("match_chip"),
        })))?;
        Ok(())
    }

    pub fn init_aggregate_volume_comp_def(ctx: Context<InitAggregateVolumeCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: AGGREGATE_URL.to_string(),
            hash: circuit_hash!("aggregate_volume"),
        })))?;
        Ok(())
    }

    /// Eenmalig: maakt het (grote, versleutelde, MXE-eigendom)
    /// chip-boek-account aan en vult het met een leeg boek.
    pub fn initialize_chip_book(
        ctx: Context<InitializeChipBook>,
        computation_offset: u64,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new().build();
        let callback_ix = CallbackInstruction {
            program_id: ID_CONST,
            discriminator: instruction::InitChipBookCallback::DISCRIMINATOR.to_vec(),
            accounts: vec![
                CallbackAccount { pubkey: ctx.accounts.arcium_program.key(), is_writable: false },
                CallbackAccount { pubkey: derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_BOOK), is_writable: false },
                CallbackAccount { pubkey: derive_mxe_pda!(), is_writable: false },
                CallbackAccount { pubkey: derive_cluster_pda!(ctx.accounts.mxe_account), is_writable: false },
                CallbackAccount { pubkey: ::arcium_anchor::solana_instructions_sysvar::ID, is_writable: false },
                CallbackAccount { pubkey: ctx.accounts.chip_book_state.key(), is_writable: true },
            ],
        };
        queue_computation(ctx.accounts, computation_offset, args, vec![callback_ix], 1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "init_chip_book")]
    pub fn init_chip_book_callback(
        ctx: Context<InitChipBookCallback>,
        output: SignedComputationOutputs<InitChipBookOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(InitChipBookOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        let data = ctx.accounts.chip_book_state.to_account_info();
        let mut bytes = data.try_borrow_mut_data()?;
        for (i, ct) in o.ciphertexts.iter().enumerate() {
            let start = 8 + i * 32;
            bytes[start..start + 32].copy_from_slice(ct);
        }
        Ok(())
    }

    pub fn register_chip(
        ctx: Context<RegisterChip>,
        computation_offset: u64,
        enc_chip_type:  [u8; 32],
        enc_volume:     [u8; 32],
        enc_price:      [u8; 32],
        enc_is_supply:  [u8; 32],
        enc_expires_at: [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        require!(ctx.accounts.moeras_pool.status == PoolStatus::Active, ErrorCode::MoerasModeActive);
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .account(ctx.accounts.chip_book_state.key(), 8, (CHIP_BOOK_CT_LEN * 32) as u32)
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_chip_type)
            .encrypted_u64(enc_volume)
            .encrypted_u64(enc_price)
            .encrypted_u64(enc_is_supply)
            .encrypted_u64(enc_expires_at)
            .build();
        let callback_ix = CallbackInstruction {
            program_id: ID_CONST,
            discriminator: instruction::RegisterChipCallback::DISCRIMINATOR.to_vec(),
            accounts: vec![
                CallbackAccount { pubkey: ctx.accounts.arcium_program.key(), is_writable: false },
                CallbackAccount { pubkey: derive_comp_def_pda!(COMP_DEF_OFFSET_REGISTER), is_writable: false },
                CallbackAccount { pubkey: derive_mxe_pda!(), is_writable: false },
                CallbackAccount { pubkey: derive_cluster_pda!(ctx.accounts.mxe_account), is_writable: false },
                CallbackAccount { pubkey: ::arcium_anchor::solana_instructions_sysvar::ID, is_writable: false },
                CallbackAccount { pubkey: ctx.accounts.chip_book_state.key(), is_writable: true },
                CallbackAccount { pubkey: ctx.accounts.payer.key(), is_writable: false },
            ],
        };
        queue_computation(ctx.accounts, computation_offset, args, vec![callback_ix], 1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "register_chip")]
    pub fn register_chip_callback(
        ctx: Context<RegisterChipCallback>,
        output: SignedComputationOutputs<RegisterChipOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(RegisterChipOutput { field_0 }) => (field_0.field_0, field_0.field_1),
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        let (book, placed_index) = o;
        {
            let data = ctx.accounts.chip_book_state.to_account_info();
            let mut bytes = data.try_borrow_mut_data()?;
            for (i, ct) in book.ciphertexts.iter().enumerate() {
                let start = 8 + i * 32;
                bytes[start..start + 32].copy_from_slice(ct);
            }
            if placed_index < CHIP_BOOK_MAX_OFFERS as u64 {
                let owners_start = 8 + CHIP_BOOK_CT_LEN * 32 + (placed_index as usize) * 32;
                bytes[owners_start..owners_start + 32].copy_from_slice(&ctx.accounts.owner.key().to_bytes());
            }
        }
        emit!(ChipRegisteredEvent { placed_index, nonce: book.nonce.to_le_bytes() });
        Ok(())
    }

    /// Vindt beste aanbod (laagste prijs) en beste vraag (hoogste prijs)
    /// voor dit chiptype. Onthult de twee indices.
    pub fn match_chip(
        ctx: Context<MatchChip>,
        computation_offset: u64,
        enc_chip_type: [u8; 32],
        current_time: u64,
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        require!(ctx.accounts.moeras_pool.status == PoolStatus::Active, ErrorCode::MoerasModeActive);
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .account(ctx.accounts.chip_book_state.key(), 8, (CHIP_BOOK_CT_LEN * 32) as u32)
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_chip_type)
            .plaintext_u64(current_time)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![MatchChipCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "match_chip")]
    pub fn match_chip_callback(
        ctx: Context<MatchChipCallback>,
        output: SignedComputationOutputs<MatchChipOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(MatchChipOutput { field_0 }) => (field_0.field_0, field_0.field_1, field_0.field_2),
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        let (result, supply_idx, demand_idx) = o;
        emit!(ChipMatchEvent {
            result: result.ciphertexts[0],
            nonce:  result.nonce.to_le_bytes(),
            supply_idx,
            demand_idx,
        });
        Ok(())
    }

    /// Partial fills op volumes van een gevonden match.
    pub fn settle_chip(
        ctx: Context<SettleChip>,
        computation_offset: u64,
        supply_idx: u64,
        demand_idx: u64,
    ) -> Result<()> {
        require!(ctx.accounts.moeras_pool.status == PoolStatus::Active, ErrorCode::MoerasModeActive);
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .account(ctx.accounts.chip_book_state.key(), 8, (CHIP_BOOK_CT_LEN * 32) as u32)
            .plaintext_u64(supply_idx)
            .plaintext_u64(demand_idx)
            .build();
        let callback_ix = CallbackInstruction {
            program_id: ID_CONST,
            discriminator: instruction::SettleChipCallback::DISCRIMINATOR.to_vec(),
            accounts: vec![
                CallbackAccount { pubkey: ctx.accounts.arcium_program.key(), is_writable: false },
                CallbackAccount { pubkey: derive_comp_def_pda!(COMP_DEF_OFFSET_SETTLE), is_writable: false },
                CallbackAccount { pubkey: derive_mxe_pda!(), is_writable: false },
                CallbackAccount { pubkey: derive_cluster_pda!(ctx.accounts.mxe_account), is_writable: false },
                CallbackAccount { pubkey: ::arcium_anchor::solana_instructions_sysvar::ID, is_writable: false },
                CallbackAccount { pubkey: ctx.accounts.chip_book_state.key(), is_writable: true },
            ],
        };
        queue_computation(ctx.accounts, computation_offset, args, vec![callback_ix], 1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "settle_chip")]
    pub fn settle_chip_callback(
        ctx: Context<SettleChipCallback>,
        output: SignedComputationOutputs<SettleChipOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(SettleChipOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        let data = ctx.accounts.chip_book_state.to_account_info();
        let mut bytes = data.try_borrow_mut_data()?;
        for (i, ct) in o.ciphertexts.iter().enumerate() {
            let start = 8 + i * 32;
            bytes[start..start + 32].copy_from_slice(ct);
        }
        Ok(())
    }

    /// Annuleert het aanbod op de gegeven index (eigendom on-chain gecheckt).
    pub fn cancel_chip(
        ctx: Context<CancelChip>,
        computation_offset: u64,
        index: u64,
    ) -> Result<()> {
        require!(ctx.accounts.moeras_pool.status == PoolStatus::Active, ErrorCode::MoerasModeActive);
        require!(index < CHIP_BOOK_MAX_OFFERS as u64, ErrorCode::InvalidOfferIndex);
        require!(
            ctx.accounts.chip_book_state.owners[index as usize] == ctx.accounts.payer.key(),
            ErrorCode::NotOfferOwner
        );
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .account(ctx.accounts.chip_book_state.key(), 8, (CHIP_BOOK_CT_LEN * 32) as u32)
            .plaintext_u64(index)
            .build();
        let callback_ix = CallbackInstruction {
            program_id: ID_CONST,
            discriminator: instruction::CancelChipCallback::DISCRIMINATOR.to_vec(),
            accounts: vec![
                CallbackAccount { pubkey: ctx.accounts.arcium_program.key(), is_writable: false },
                CallbackAccount { pubkey: derive_comp_def_pda!(COMP_DEF_OFFSET_CANCEL), is_writable: false },
                CallbackAccount { pubkey: derive_mxe_pda!(), is_writable: false },
                CallbackAccount { pubkey: derive_cluster_pda!(ctx.accounts.mxe_account), is_writable: false },
                CallbackAccount { pubkey: ::arcium_anchor::solana_instructions_sysvar::ID, is_writable: false },
                CallbackAccount { pubkey: ctx.accounts.chip_book_state.key(), is_writable: true },
            ],
        };
        queue_computation(ctx.accounts, computation_offset, args, vec![callback_ix], 1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "cancel_chip")]
    pub fn cancel_chip_callback(
        ctx: Context<CancelChipCallback>,
        output: SignedComputationOutputs<CancelChipOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(CancelChipOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        let data = ctx.accounts.chip_book_state.to_account_info();
        let mut bytes = data.try_borrow_mut_data()?;
        for (i, ct) in o.ciphertexts.iter().enumerate() {
            let start = 8 + i * 32;
            bytes[start..start + 32].copy_from_slice(ct);
        }
        Ok(())
    }

    /// Statistieken over het hele boek (aantal aanbiedingen,
    /// aanbod-/vraagvolume), versleuteld als MXE-resultaat.
    pub fn aggregate_volume(
        ctx: Context<AggregateVolume>,
        computation_offset: u64,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .account(ctx.accounts.chip_book_state.key(), 8, (CHIP_BOOK_CT_LEN * 32) as u32)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![AggregateVolumeCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "aggregate_volume")]
    pub fn aggregate_volume_callback(
        ctx: Context<AggregateVolumeCallback>,
        output: SignedComputationOutputs<AggregateVolumeOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(AggregateVolumeOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(VolumeAggregatedEvent {
            total_offers:  o.ciphertexts[0],
            supply_volume: o.ciphertexts[1],
            demand_volume: o.ciphertexts[2],
            nonce: o.nonce.to_le_bytes(),
        });
        Ok(())
    }

    pub fn init_init_chip_book_comp_def(ctx: Context<InitInitChipBookCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: INIT_BOOK_URL.to_string(),
            hash: circuit_hash!("init_chip_book"),
        })))?;
        Ok(())
    }

    pub fn init_settle_chip_comp_def(ctx: Context<InitSettleChipCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: SETTLE_URL.to_string(),
            hash: circuit_hash!("settle_chip"),
        })))?;
        Ok(())
    }

    pub fn init_cancel_chip_comp_def(ctx: Context<InitCancelChipCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: CANCEL_URL.to_string(),
            hash: circuit_hash!("cancel_chip"),
        })))?;
        Ok(())
    }

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

    /// Verwerkt een reputatie-event (voltooide trade of verloren dispute) en
    /// geeft de nieuwe versleutelde reputatiestand terug. De aanroeper (client)
    /// bewaart deze ciphertext zelf en geeft 'm bij de volgende aanroep opnieuw mee.
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
            Err(e) => { msg!("Computation verification failed: {}", e); return Err(ErrorCode::AbortedComputation.into()) },
        };
        emit!(ReputationUpdatedEvent {
            completed_trades: o.ciphertexts[0],
            disputes_lost:    o.ciphertexts[1],
            score:             o.ciphertexts[2],
            nonce:             o.nonce.to_le_bytes(),
        });
        Ok(())
    }

    /// Enige publiek gerevealede output: 1 = boven drempel, 0 = niet.
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
            Err(e) => { msg!("Computation verification failed: {}", e); return Err(ErrorCode::AbortedComputation.into()) },
        };
        emit!(ThresholdCheckedEvent { passes: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
        Ok(())
    }

    // ============================================================
    // ESCROW-MECHANISME
    // Koper stort SOL bij create_escrow. Na bevestigde levering geeft
    // release_escrow vrij aan verkoper. Bij geschil (dispute_escrow)
    // beslecht de Squads-multisig (vault-PDA) via resolve_dispute.
    // Zonder dispute binnen de termijn mag de verkoper zelf claimen.
    // ============================================================

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

    // Pool-initialisatie: maakt het PoolState-account aan, initialisator wordt guardian
    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.guardian = ctx.accounts.user.key();
        pool.status = PoolStatus::Active;
        pool.last_heartbeat_slot = Clock::get()?.slot;
        msg!("Pool geinitialiseerd, guardian: {}", pool.guardian);
        Ok(())
    }
    // Exportcontrole: koper goedkeuren (alleen door de multisig-vault)
    pub fn approve_buyer(ctx: Context<ApproveBuyer>, region_code: u16, expires_at: i64) -> Result<()> {
        let att = &mut ctx.accounts.attestation;
        att.buyer = ctx.accounts.buyer.key();
        att.approved = true;
        att.region_code = region_code;
        att.approved_by = ctx.accounts.authority.key();
        att.expires_at = expires_at;
        msg!("Koper goedgekeurd voor export: {}", att.buyer);
        Ok(())
    }
    // Exportcontrole: goedkeuring intrekken (alleen door de multisig-vault)
    pub fn revoke_buyer(ctx: Context<RevokeBuyer>) -> Result<()> {
        let att = &mut ctx.accounts.attestation;
        att.approved = false;
        msg!("Goedkeuring ingetrokken voor: {}", att.buyer);
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
        msg!("Moeras-modus geactiveerd!");
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
}


// ComplianceAttestation account (exportcontrole)
#[account]
pub struct ComplianceAttestation {
    pub buyer: Pubkey,
    pub approved: bool,
    pub region_code: u16,
    pub approved_by: Pubkey,
    pub expires_at: i64,
}
// ApproveBuyer accounts context
#[derive(Accounts)]
pub struct ApproveBuyer<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + 32 + 1 + 2 + 32 + 8,
        seeds = [b"compliance", buyer.key().as_ref()],
        bump
    )]
    pub attestation: Account<'info, ComplianceAttestation>,
    /// CHECK: dit is enkel een adresverwijzing naar de koper die goedgekeurd wordt.
    pub buyer: UncheckedAccount<'info>,
    #[account(mut, address = VAULT_PDA)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
// RevokeBuyer accounts context
#[derive(Accounts)]
pub struct RevokeBuyer<'info> {
    #[account(mut, seeds = [b"compliance", attestation.buyer.as_ref()], bump)]
    pub attestation: Account<'info, ComplianceAttestation>,
    #[account(address = VAULT_PDA)]
    pub authority: Signer<'info>,
}

// PoolStatus enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PoolStatus {
    Active,
    Moeras,
}
// PoolState account
pub const CHIP_BOOK_CT_LEN: usize = 3001; // 500 offers x 6 ciphertext-elementen + 1 voor count
pub const CHIP_BOOK_MAX_OFFERS: usize = 500;

#[account]
pub struct ChipBookState {
    pub ciphertexts: [[u8; 32]; CHIP_BOOK_CT_LEN],
    pub owners: [Pubkey; CHIP_BOOK_MAX_OFFERS],
}
impl ChipBookState {
    pub const SPACE: usize = 8 + CHIP_BOOK_CT_LEN * 32 + CHIP_BOOK_MAX_OFFERS * 32;
}

#[queue_computation_accounts("init_chip_book", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct InitializeChipBook<'info> {
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
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_BOOK))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    #[account(init, payer = payer, space = ChipBookState::SPACE, seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("init_chip_book")]
#[derive(Accounts)]
pub struct InitChipBookCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_BOOK))]
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
    #[account(mut, seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
}

#[queue_computation_accounts("settle_chip", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct SettleChip<'info> {
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
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_SETTLE))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    pub moeras_pool: Account<'info, PoolState>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    #[account(mut, seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("settle_chip")]
#[derive(Accounts)]
pub struct SettleChipCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_SETTLE))]
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
    #[account(mut, seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
}

#[queue_computation_accounts("cancel_chip", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CancelChip<'info> {
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
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CANCEL))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    pub moeras_pool: Account<'info, PoolState>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    #[account(mut, seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("cancel_chip")]
#[derive(Accounts)]
pub struct CancelChipCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CANCEL))]
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
    #[account(mut, seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
}

#[init_computation_definition_accounts("init_chip_book", payer)]
#[derive(Accounts)]
pub struct InitInitChipBookCompDef<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
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

#[init_computation_definition_accounts("settle_chip", payer)]
#[derive(Accounts)]
pub struct InitSettleChipCompDef<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
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

#[init_computation_definition_accounts("cancel_chip", payer)]
#[derive(Accounts)]
pub struct InitCancelChipCompDef<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
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

#[queue_computation_accounts("register_chip", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct RegisterChip<'info> {
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
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_REGISTER))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    pub moeras_pool: Account<'info, PoolState>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    #[account(mut, seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("register_chip")]
#[derive(Accounts)]
pub struct RegisterChipCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_REGISTER))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
    /// CHECK: alleen public key nodig, om eigenaarschap te registreren.
    pub owner: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("register_chip", payer)]
#[derive(Accounts)]
pub struct InitRegisterChipCompDef<'info> {
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

#[queue_computation_accounts("match_chip", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct MatchChip<'info> {
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
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    pub moeras_pool: Account<'info, PoolState>,
    #[account(seeds = [b"compliance", payer.key().as_ref()], bump,
        constraint = compliance_attestation.approved @ ErrorCode::BuyerNotApproved)]
    pub compliance_attestation: Account<'info, ComplianceAttestation>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    #[account(seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("match_chip")]
#[derive(Accounts)]
pub struct MatchChipCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("match_chip", payer)]
#[derive(Accounts)]
pub struct InitMatchChipCompDef<'info> {
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

#[queue_computation_accounts("aggregate_volume", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct AggregateVolume<'info> {
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
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_AGGREGATE))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    #[account(seeds = [b"chip_book"], bump)]
    pub chip_book_state: Box<Account<'info, ChipBookState>>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("aggregate_volume")]
#[derive(Accounts)]
pub struct AggregateVolumeCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_AGGREGATE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)]
    /// CHECK: sysvar.
    pub instructions_sysvar: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("aggregate_volume", payer)]
#[derive(Accounts)]
pub struct InitAggregateVolumeCompDef<'info> {
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

#[event] pub struct ChipRegisteredEvent { pub placed_index: u64, pub nonce: [u8; 16] }
#[event] pub struct ChipMatchEvent { pub result: [u8; 32], pub nonce: [u8; 16], pub supply_idx: u64, pub demand_idx: u64 }
#[event] pub struct VolumeAggregatedEvent { pub total_offers: [u8; 32], pub supply_volume: [u8; 32], pub demand_volume: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct ReputationUpdatedEvent { pub completed_trades: [u8; 32], pub disputes_lost: [u8; 32], pub score: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct ThresholdCheckedEvent  { pub passes: [u8; 32], pub nonce: [u8; 16] }

#[error_code]
pub enum ErrorCode {
    #[msg("Aanbod-index buiten bereik.")]
    InvalidOfferIndex,
    #[msg("Dit aanbod is niet van jou.")]
    NotOfferOwner,
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Escrow-bedrag moet groter dan nul zijn")]
    InvalidEscrowAmount,
    #[msg("Escrow staat niet in de verwachte status voor deze actie")]
    InvalidEscrowStatus,
    #[msg("Alleen koper of verkoper mag dit escrow-account aanroepen")]
    UnauthorizedEscrowAction,
    #[msg("Alleen de multisig-vault mag een geschil beslechten")]
    UnauthorizedArbiter,
    #[msg("Dispute-termijn is al verstreken")]
    DisputeWindowClosed,
    #[msg("Dispute-termijn is nog niet verstreken")]
    DisputeWindowStillOpen,
    #[msg("Onbevoegde aanroep. Alleen de guardian mag dit doen.")]
    UnauthorizedGuardian,
    #[msg("Moeras-modus is actief: dit is tijdelijk bevroren voor beveiligingsonderzoek")]
    MoerasModeActive,
    #[msg("Deze koper is niet goedgekeurd voor export-gecontroleerde handel")]
    BuyerNotApproved,
    #[msg("De exportgoedkeuring van deze koper is verlopen")]
    AttestationExpired,
}

// ============================================================
// ESCROW: constanten, state, Accounts-structs, errors, events
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
