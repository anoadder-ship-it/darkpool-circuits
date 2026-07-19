const fs = require('fs');
const anchor = require('@coral-xyz/anchor');
const { Connection, Keypair, PublicKey, SystemProgram } = require('@solana/web3.js');
const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';

function loadKeypair(path) {
  const raw = JSON.parse(fs.readFileSync(path, 'utf8'));
  return Keypair.fromSecretKey(new Uint8Array(raw));
}

const POOLS = [
  {
    label: 'Medical',
    programId: 'CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4',
    idlPath: '/home/michel/solana_darkpool/target/idl/medical_darkpool.json',
    poolKeypairPath: '/home/michel/solana_darkpool/test-wallets/pool-state-medical.json',
  },
  {
    label: 'Supply Chain',
    programId: '3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4',
    idlPath: '/home/michel/solana_darkpool/target/idl/supply_chain_darkpool.json',
    poolKeypairPath: '/home/michel/solana_darkpool/test-wallets/pool-state-supply.json',
  },
  {
    label: 'Chip',
    programId: '6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o',
    idlPath: '/home/michel/solana_darkpool/target/idl/chip_darkpool.json',
    poolKeypairPath: '/home/michel/solana_darkpool/test-wallets/pool-state-chip.json',
  },
];

async function main() {
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });
  const guardian = loadKeypair('/home/michel/solana_darkpool/test-wallets/buyer.json');

  for (const pool of POOLS) {
    console.log(`\n=== ${pool.label} ===`);
    const poolState = loadKeypair(pool.poolKeypairPath);
    const idl = JSON.parse(fs.readFileSync(pool.idlPath, 'utf8'));
    const wallet = new anchor.Wallet(guardian);
    const provider = new anchor.AnchorProvider(conn, wallet, { commitment: 'confirmed' });
    const program = new anchor.Program(idl, provider);

    try {
      const sig = await program.methods
        .initializePool()
        .accounts({
          pool: poolState.publicKey,
          user: guardian.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([guardian, poolState])
        .rpc();
      console.log('initialize_pool tx:', sig);

      const state = await program.account.poolState.fetch(poolState.publicKey);
      console.log('Guardian on-chain:', state.guardian.toString());
      console.log('Status:', JSON.stringify(state.status));
      console.log('Pool-adres:', poolState.publicKey.toString());
    } catch (e) {
      console.error(`FOUT bij ${pool.label}:`, e.message);
    }
  }
}

main().catch((e) => {
  console.error('FOUT:', e);
  process.exit(1);
});
