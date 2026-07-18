use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_macros::circuit_hash;
use arcium_client::idl::arcium::types::{OffChainCircuitSource, CircuitSource};

const COMP_DEF_OFFSET_REGISTER_SUPPLY: u32 = comp_def_offset("register_supply");
const COMP_DEF_OFFSET_MATCH_SUPPLY:    u32 = comp_def_offset("match_supply");
const COMP_DEF_OFFSET_MATCH_CARBON:    u32 = comp_def_offset("match_carbon");

declare_id!("3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4");

const REGISTER_SUPPLY_URL: &str = "https://github.com/anoadder-ship-it/supply-chain-circuits/releases/download/v0.1.0/register_supply.arcis";
const MATCH_SUPPLY_URL:    &str = "https://github.com/anoadder-ship-it/supply-chain-circuits/releases/download/v0.1.0/match_supply.arcis";
const MATCH_CARBON_URL:    &str = "https://github.com/anoadder-ship-it/supply-chain-circuits/releases/download/v0.1.0/match_carbon.arcis";

#[arcium_program]
pub mod supply_chain_darkpool {
    use super::*;

    pub fn init_register_supply_comp_def(ctx: Context<InitRegisterSupplyCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: REGISTER_SUPPLY_URL.to_string(),
            hash: circuit_hash!("register_supply"),
        })))?;
        Ok(())
    }

    pub fn init_match_supply_comp_def(ctx: Context<InitMatchSupplyCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: MATCH_SUPPLY_URL.to_string(),
            hash: circuit_hash!("match_supply"),
        })))?;
        Ok(())
    }

    pub fn init_match_carbon_comp_def(ctx: Context<InitMatchCarbonCompDef>) -> Result<()> {
        init_computation_def(ctx.accounts, Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: MATCH_CARBON_URL.to_string(),
            hash: circuit_hash!("match_carbon"),
        })))?;
        Ok(())
    }

    pub fn register_supply(
        ctx: Context<RegisterSupply>,
        computation_offset: u64,
        enc_material:  [u8; 32],
        enc_quantity:  [u8; 32],
        enc_quality:   [u8; 32],
        enc_price:     [u8; 32],
        enc_delivery:  [u8; 32],
        enc_region:    [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_material)
            .encrypted_u64(enc_quantity)
            .encrypted_u64(enc_quality)
            .encrypted_u64(enc_price)
            .encrypted_u64(enc_delivery)
            .encrypted_u64(enc_region)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![RegisterSupplyCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "register_supply")]
    pub fn register_supply_callback(
        ctx: Context<RegisterSupplyCallback>,
        output: SignedComputationOutputs<RegisterSupplyOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(RegisterSupplyOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(SupplyRegisteredEvent { result: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
        Ok(())
    }

    pub fn match_supply(
        ctx: Context<MatchSupply>,
        computation_offset: u64,
        enc_material:  [u8; 32],
        enc_quantity:  [u8; 32],
        enc_quality:   [u8; 32],
        enc_price:     [u8; 32],
        enc_delivery:  [u8; 32],
        enc_s_region:  [u8; 32],
        enc_req_mat:   [u8; 32],
        enc_min_qty:   [u8; 32],
        enc_min_qual:  [u8; 32],
        enc_max_price: [u8; 32],
        enc_max_del:   [u8; 32],
        enc_req_reg:   [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_material)
            .encrypted_u64(enc_quantity)
            .encrypted_u64(enc_quality)
            .encrypted_u64(enc_price)
            .encrypted_u64(enc_delivery)
            .encrypted_u64(enc_s_region)
            .encrypted_u64(enc_req_mat)
            .encrypted_u64(enc_min_qty)
            .encrypted_u64(enc_min_qual)
            .encrypted_u64(enc_max_price)
            .encrypted_u64(enc_max_del)
            .encrypted_u64(enc_req_reg)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![MatchSupplyCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "match_supply")]
    pub fn match_supply_callback(
        ctx: Context<MatchSupplyCallback>,
        output: SignedComputationOutputs<MatchSupplyOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(MatchSupplyOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(SupplyMatchedEvent {
            matched: o.ciphertexts[0],
            score:   o.ciphertexts[1],
            nonce:   o.nonce.to_le_bytes(),
        });
        Ok(())
    }

    pub fn match_carbon(
        ctx: Context<MatchCarbon>,
        computation_offset: u64,
        enc_credits:   [u8; 32],
        enc_price:     [u8; 32],
        enc_vintage:   [u8; 32],
        enc_cert:      [u8; 32],
        enc_req_cred:  [u8; 32],
        enc_max_price: [u8; 32],
        enc_req_vint:  [u8; 32],
        enc_req_cert:  [u8; 32],
        pubkey: [u8; 32],
        nonce:  u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(enc_credits)
            .encrypted_u64(enc_price)
            .encrypted_u64(enc_vintage)
            .encrypted_u64(enc_cert)
            .encrypted_u64(enc_req_cred)
            .encrypted_u64(enc_max_price)
            .encrypted_u64(enc_req_vint)
            .encrypted_u64(enc_req_cert)
            .build();
        queue_computation(ctx.accounts, computation_offset, args,
            vec![MatchCarbonCallback::callback_ix(computation_offset, &ctx.accounts.mxe_account, &[])?],
            1, 0, 0)?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "match_carbon")]
    pub fn match_carbon_callback(
        ctx: Context<MatchCarbonCallback>,
        output: SignedComputationOutputs<MatchCarbonOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(MatchCarbonOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        emit!(CarbonMatchedEvent { result: o.ciphertexts[0], nonce: o.nonce.to_le_bytes() });
        Ok(())
    }

    // ============================================================
    // Escrow-mechanisme voor Supply Chain + Carbon Darkpool
    // Koper stort SOL bij capaciteit/carbon-credit-transactie; verkoper
    // claimt na levering, of koper disput binnen de termijn. Disputes
    // worden beslecht via de Squads-multisig (vault-PDA) via
    // resolve_dispute. Zonder dispute binnen de termijn mag de
    // verkoper zelf claimen.
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
}

#[queue_computation_accounts("register_supply", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct RegisterSupply<'info> {
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
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_REGISTER_SUPPLY))]
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

#[callback_accounts("register_supply")]
#[derive(Accounts)]
pub struct RegisterSupplyCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_REGISTER_SUPPLY))]
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

#[init_computation_definition_accounts("register_supply", payer)]
#[derive(Accounts)]
pub struct InitRegisterSupplyCompDef<'info> {
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

#[queue_computation_accounts("match_supply", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct MatchSupply<'info> {
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
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_SUPPLY))]
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

#[callback_accounts("match_supply")]
#[derive(Accounts)]
pub struct MatchSupplyCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_SUPPLY))]
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

#[init_computation_definition_accounts("match_supply", payer)]
#[derive(Accounts)]
pub struct InitMatchSupplyCompDef<'info> {
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

#[queue_computation_accounts("match_carbon", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct MatchCarbon<'info> {
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
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_CARBON))]
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

#[callback_accounts("match_carbon")]
#[derive(Accounts)]
pub struct MatchCarbonCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_CARBON))]
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

#[init_computation_definition_accounts("match_carbon", payer)]
#[derive(Accounts)]
pub struct InitMatchCarbonCompDef<'info> {
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

#[event] pub struct SupplyRegisteredEvent { pub result:  [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct SupplyMatchedEvent    { pub matched: [u8; 32], pub score: [u8; 32], pub nonce: [u8; 16] }
#[event] pub struct CarbonMatchedEvent    { pub result:  [u8; 32], pub nonce: [u8; 16] }

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
