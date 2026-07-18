use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_macros::circuit_hash;
use arcium_client::idl::arcium::types::{OffChainCircuitSource, CircuitSource};

const COMP_DEF_OFFSET_REGISTER: u32 = comp_def_offset("register_dataset");
const COMP_DEF_OFFSET_MATCH:    u32 = comp_def_offset("match_dataset");
const COMP_DEF_OFFSET_AGGREGATE:u32 = comp_def_offset("aggregate_gradient");

declare_id!("11111111111111111111111111111111");

const REGISTER_URL:  &str = "https://github.com/anoadder-ship-it/medical-circuits/releases/download/v0.1.0/register_dataset.arcis";
const MATCH_URL:     &str = "https://github.com/anoadder-ship-it/medical-circuits/releases/download/v0.1.0/match_dataset.arcis";
const AGGREGATE_URL: &str = "https://github.com/anoadder-ship-it/medical-circuits/releases/download/v0.1.0/aggregate_gradient.arcis";

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

    /// Registreer dataset-profiel — 5 encrypted velden in 1 struct
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

    /// Match dataset met zoekopdracht — 10 encrypted velden in 1 struct
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

    /// Gradient aggregatie voor federated learning
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
}

#[queue_computation_accounts("register_dataset", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct RegisterDataset<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account))] /// CHECK: arcium. pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))] /// CHECK: arcium. pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))] /// CHECK: arcium. pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_REGISTER))] pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))] pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)] pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)] pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("register_dataset")]
#[derive(Accounts)]
pub struct RegisterDatasetCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_REGISTER))] pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium. pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))] pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)] /// CHECK: sysvar. pub instructions_sysvar: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("register_dataset", payer)]
#[derive(Accounts)]
pub struct InitRegisterCompDef<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)] /// CHECK: not yet initialized. pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))] /// CHECK: arcium. pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)] /// CHECK: LUT. pub lut_program: UncheckedAccount<'info>,
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
    #[account(mut, address = derive_mempool_pda!(mxe_account))] /// CHECK: arcium. pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))] /// CHECK: arcium. pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))] /// CHECK: arcium. pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH))] pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))] pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)] pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)] pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("match_dataset")]
#[derive(Accounts)]
pub struct MatchDatasetCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH))] pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium. pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))] pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)] /// CHECK: sysvar. pub instructions_sysvar: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("match_dataset", payer)]
#[derive(Accounts)]
pub struct InitMatchCompDef<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)] /// CHECK: not yet initialized. pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))] /// CHECK: arcium. pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)] /// CHECK: LUT. pub lut_program: UncheckedAccount<'info>,
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
    #[account(mut, address = derive_mempool_pda!(mxe_account))] /// CHECK: arcium. pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account))] /// CHECK: arcium. pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account))] /// CHECK: arcium. pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_AGGREGATE))] pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account))] pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)] pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)] pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("aggregate_gradient")]
#[derive(Accounts)]
pub struct AggregateGradientCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_AGGREGATE))] pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())] pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: arcium. pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account))] pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::arcium_anchor::solana_instructions_sysvar::ID)] /// CHECK: sysvar. pub instructions_sysvar: UncheckedAccount<'info>,
}

#[init_computation_definition_accounts("aggregate_gradient", payer)]
#[derive(Accounts)]
pub struct InitAggregateCompDef<'info> {
    #[account(mut)] pub payer: Signer<'

info>,
    #[account(mut, address = derive_mxe_pda!())] pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)] /// CHECK: not yet initialized. pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))] /// CHECK: arcium. pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)] /// CHECK: LUT. pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[event] pub struct DatasetRegisteredEvent  { pub result: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct DatasetMatchedEvent     { pub compatible: [u8; 32], pub score: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct GradientAggregatedEvent { pub result: [u8; 32], pub nonce: [u8; 16] }

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
}
