const fs = require('fs');
const anchor = require('@coral-xyz/anchor');
const { Connection, Keypair, PublicKey, SystemProgram } = require('@solana/web3.js');
const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
const PROGRAM_ID = new PublicKey('h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX');
function loadKeypair(path) {
  const raw = JSON.parse(fs.readFileSync(path, 'utf8'));
  return Keypair.fromSecretKey(new Uint8Array(raw));
}
async function main() {
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });
  const guardian = loadKeypair('/home/michel/solana_darkpool/test-wallets/buyer.json');
  const poolState = loadKeypair('/home/michel/solana_darkpool/test-wallets/pool-state.json');
  console.log('Guardian (wordt eigenaar van de killswitch):', guardian.publicKey.toString());
  console.log('Pool-account:', poolState.publicKey.toString());
  const idl = JSON.parse(fs.readFileSync('/home/michel/solana_darkpool/target/idl/solana_darkpool.json', 'utf8'));
  const wallet = new anchor.Wallet(guardian);
  const provider = new anchor.AnchorProvider(conn, wallet, { commitment: 'confirmed' });
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  console.log('\n=== TEST: initialize_pool ===');
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
  console.log('Last heartbeat slot:', state.lastHeartbeatSlot.toString());
}
main().catch((e) => {
  console.error('FOUT:', e);
  process.exit(1);
});
