use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PoolStatus {
    Active,
    Moeras,
}

#[account]
pub struct PoolState {
    pub guardian: Pubkey,
    pub status: PoolStatus,
    pub last_heartbeat_slot: u64,
}

#[derive(Accounts)]
pub struct SendHeartbeat<'info> {
    #[account(mut)]
    pub pool: Account<'info, PoolState>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct TriggerMoeras<'info> {
    #[account(mut)]
    pub pool: Account<'info, PoolState>,
    pub signer: Signer<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Onbevoegde aanroep. Alleen de DGX Spark Guardian mag dit doen.")]
    UnauthorizedGuardian,
}
