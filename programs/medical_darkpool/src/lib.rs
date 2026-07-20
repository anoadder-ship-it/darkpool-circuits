use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_macros::circuit_hash;
use arcium_client::idl::arcium::types::{OffChainCircuitSource, CircuitSource};

const COMP_DEF_OFFSET_REGISTER: u32 = comp_def_offset("register_dataset");
const COMP_DEF_OFFSET_MATCH:    u32 = comp_def_offset("match_dataset");
const COMP_DEF_OFFSET_AGGREGATE:u32 = comp_def_offset("aggregate_gradient");
const COMP_DEF_OFFSET_UPDATE_REPUTATION: u32 = comp_def_offset("update_reputation");
const COMP_DEF_OFFSET_CHECK_THRESHOLD:   u32 = comp_def_offset("check_threshold");

declare_id!("CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4");

const REGISTER_URL:  &str = "https://github.com/anoadder-ship-it/medical-circuits/releases/download/v0.1.0/register_dataset.arcis";
const MATCH_URL:     &str = "https://github.com/anoadder-ship-it/medical-circuits/releases/download/v0.1.0/match_dataset.arcis";
const AGGREGATE_URL: &str = "https://github.com/anoadder-ship-it/medical-circuits/releases/download/v0.1.0/aggregate_gradient.arcis";
const UPDATE_REPUTATION_URL: &str = "https://github.com/anoadder-ship-it/medical-circuits/releases/download/v0.2.0/update_reputation.arcis";
const CHECK_THRESHOLD_URL:   &str = "https://github.com/anoadder-ship-it/medical-circuits/releases/download/v0.2.0/check_threshold.arcis";

#[arcium_program]
pub mod medical_darkpool {
    use super::*;

    pub fn init_register_comp_def(ctx: Context<InitRegisterCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: REGISTER_URL.to_string(),
            hash: circuit_hash!("register_dataset"),
        })))?;
        Ok(())
    }

    pub fn init_match_comp_def(ctx: Context<InitMatchCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: MATCH_URL.to_string(),
            hash: circuit_hash!("match_dataset"),
        })))?;
        Ok(())
    }

    pub fn init_aggregate_comp_def(ctx: Context<InitAggregateCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: AGGREGATE_URL.to_string(),
            hash: circuit_hash!("aggregate_gradient"),
        })))?;
        Ok(())
    }

    pub fn register_dataset(
        ctx: Context<RegisterDataset>,
        computation_offset: u64,
        enc_disease:  [u8; 32],
        enc_samples:  [u8; 32],
        enc_age:      [u8; 32],
        enc_gender:   [u8; 32],
        enc_modality: [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        require!(ctx.accounts.moeras_pool.status == PoolStatus::Active, ErrorCode::MoerasModeActive);
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_disease)
            .encrypted_u64(enc_samples)
            .encrypted_u64(enc_age)
            .encrypted_u64(enc_gender)
            .encrypted_u64(enc_modality)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![RegisterDatasetCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "register_dataset")]
    pub fn register_dataset_callback(
        ctx: Context<RegisterDatasetCallback>,
        output: SignedComputationOutputs<RegisterDatasetOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(RegisterDatasetOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(DatasetRegisteredEvent { result: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
        Ok(())
    }

    pub fn match_dataset(
        ctx: Context<MatchDataset>,
        computation_offset: u64,
        enc_disease:       [u8; 32],
        enc_samples:       [u8; 32],
        enc_age:           [u8; 32],
        enc_gender:        [u8; 32],
        enc_modality:      [u8; 32],
        enc_q_disease:     [u8; 32],
        enc_q_min_samples: [u8; 32],
        enc_q_age_min:     [u8; 32],
        enc_q_age_max:     [u8; 32],
        enc_q_modality:    [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        require!(ctx.accounts.moeras_pool.status == PoolStatus::Active, ErrorCode::MoerasModeActive);
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_disease)
            .encrypted_u64(enc_samples)
            .encrypted_u64(enc_age)
            .encrypted_u64(enc_gender)
            .encrypted_u64(enc_modality)
            .encrypted_u64(enc_q_disease)
            .encrypted_u64(enc_q_min_samples)
            .encrypted_u64(enc_q_age_min)
            .encrypted_u64(enc_q_age_max)
            .encrypted_u64(enc_q_modality)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![MatchDatasetCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "match_dataset")]
    pub fn match_dataset_callback(
        ctx: Context<MatchDatasetCallback>,
        output: SignedComputationOutputs<MatchDatasetOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(MatchDatasetOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(DatasetMatchedEvent {
            compatible: o.ciphertexts[0],
            score:      o.ciphertexts[1],
            nonce:      o.nonce.to_le_bytes(),
        });
        Ok(())
    }

    pub fn aggregate_gradient(
        ctx: Context<AggregateGradient>,
        computation_offset: u64,
        enc_gradient: [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_gradient)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![AggregateGradientCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "aggregate_gradient")]
    pub fn aggregate_gradient_callback(
        ctx: Context<AggregateGradientCallback>,
        output: SignedComputationOutputs<AggregateGradientOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(AggregateGradientOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(GradientAggregatedEvent { result: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
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

    // ============================================================
    // Escrow-mechanisme voor Medical Darkpool
    // Koper stort SOL bij dataset/model-transactie; verkoper claimt na
    // levering, of koper disput binnen de termijn. Disputes worden
    // beslecht via de Squads-multisig (vault-PDA) via resolve_dispute.
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

#[queue_computation_accounts("register_dataset", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct RegisterDataset<'info> {
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
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("register_dataset")]
#[derive(Accounts)]
pub struct RegisterDatasetCallback<'info> {
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
}

#[init_computation_definition_accounts("register_dataset", payer)]
#[derive(Accounts)]
pub struct InitRegisterCompDef<'info> {
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

#[queue_computation_accounts("match_dataset", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct MatchDataset<'info> {
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
    #[account(mut, address = derive_cluster_pda!(mxe_account))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("match_dataset")]
#[derive(Accounts)]
pub struct MatchDatasetCallback<'info> {
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

#[init_computation_definition_accounts("match_dataset", payer)]
#[derive(Accounts)]
pub struct InitMatchCompDef<'info> {
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

#[queue_computation_accounts("aggregate_gradient", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct AggregateGradient<'info> {
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
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("aggregate_gradient")]
#[derive(Accounts)]
pub struct AggregateGradientCallback<'info> {
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

#[init_computation_definition_accounts("aggregate_gradient", payer)]
#[derive(Accounts)]
pub struct InitAggregateCompDef<'info> {
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

#[event] pub struct DatasetRegisteredEvent  { pub result: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct DatasetMatchedEvent     { pub compatible: [u8; 32], pub score: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct GradientAggregatedEvent { pub result: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct ReputationUpdatedEvent { pub completed_trades: [u8; 32], pub disputes_lost: [u8; 32], pub score: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct ThresholdCheckedEvent  { pub passes: [u8; 32], pub nonce: [u8; 16] }

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
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
    #[msg("Onbevoegde aanroep. Alleen de guardian mag dit doen.")]
    UnauthorizedGuardian,
    #[msg("Moeras-modus is actief: dit is tijdelijk bevroren voor beveiligingsonderzoek")]
    MoerasModeActive,
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
