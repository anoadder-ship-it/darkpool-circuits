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
  const buyer = loadKeypair('/home/michel/solana_darkpool/test-wallets/buyer.json');
  const seller = loadKeypair('/home/michel/solana_darkpool/test-wallets/seller.json');
  console.log('Buyer:', buyer.publicKey.toString());
  console.log('Seller:', seller.publicKey.toString());
  const idl = JSON.parse(fs.readFileSync('/home/michel/solana_darkpool/target/idl/solana_darkpool.json', 'utf8'));
  const wallet = new anchor.Wallet(buyer);
  const provider = new anchor.AnchorProvider(conn, wallet, { commitment: 'confirmed' });
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const amount = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);

  // --- TEST A: create_escrow -> release_escrow ---
  const seedA = new anchor.BN(Date.now());
  const [escrowA] = PublicKey.findProgramAddressSync(
    [Buffer.from('escrow'), buyer.publicKey.toBuffer(), seller.publicKey.toBuffer(), seedA.toArrayLike(Buffer, 'le', 8)],
    PROGRAM_ID
  );
  console.log('\n=== Escrow A (create/release):', escrowA.toString(), '===');
  console.log('=== TEST: create_escrow ===');
  let sig = await program.methods
    .createEscrow(amount, seller.publicKey, seedA)
    .accounts({ buyer: buyer.publicKey, escrowAccount: escrowA, systemProgram: SystemProgram.programId })
    .signers([buyer]).rpc();
  console.log('create_escrow tx:', sig);
  let state = await program.account.escrowAccount.fetch(escrowA);
  console.log('Status na create:', JSON.stringify(state.status));

  console.log('=== TEST: release_escrow ===');
  sig = await program.methods
    .releaseEscrow()
    .accounts({ buyer: buyer.publicKey, escrowAccount: escrowA, seller: seller.publicKey })
    .signers([buyer]).rpc();
  console.log('release_escrow tx:', sig);
  state = await program.account.escrowAccount.fetch(escrowA);
  console.log('Status na release:', JSON.stringify(state.status));
  const releaseOk = 'released' in state.status;
  console.log(releaseOk ? 'TEST A GESLAAGD' : 'TEST A GEFAALD');

  // --- TEST B: create_escrow -> dispute_escrow ---
  await new Promise(r => setTimeout(r, 1500));
  const seedB = new anchor.BN(Date.now());
  const [escrowB] = PublicKey.findProgramAddressSync(
    [Buffer.from('escrow'), buyer.publicKey.toBuffer(), seller.publicKey.toBuffer(), seedB.toArrayLike(Buffer, 'le', 8)],
    PROGRAM_ID
  );
  console.log('\n=== Escrow B (dispute, voor resolve_dispute-test):', escrowB.toString(), '===');
  console.log('=== TEST: create_escrow ===');
  sig = await program.methods
    .createEscrow(amount, seller.publicKey, seedB)
    .accounts({ buyer: buyer.publicKey, escrowAccount: escrowB, systemProgram: SystemProgram.programId })
    .signers([buyer]).rpc();
  console.log('create_escrow tx:', sig);

  console.log('=== TEST: dispute_escrow ===');
  sig = await program.methods
    .disputeEscrow()
    .accounts({ disputer: buyer.publicKey, escrowAccount: escrowB })
    .signers([buyer]).rpc();
  console.log('dispute_escrow tx:', sig);
  state = await program.account.escrowAccount.fetch(escrowB);
  console.log('Status na dispute:', JSON.stringify(state.status));
  const disputeOk = 'disputed' in state.status;
  console.log(disputeOk ? 'TEST B GESLAAGD' : 'TEST B GEFAALD');

  console.log('\n=== SAMENVATTING ===');
  console.log('Escrow A (release):', escrowA.toString(), '-> Released:', releaseOk);
  console.log('Escrow B (dispute, klaar voor resolve_dispute):', escrowB.toString());
}
main().catch((e) => {
  console.error('FOUT:', e);
  process.exit(1);
});
