const fs = require('fs');
const anchor = require('@coral-xyz/anchor');
const { Connection, Keypair, PublicKey, SystemProgram } = require('@solana/web3.js');

const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
const PROGRAM_ID = new PublicKey('6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o');

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

  const idl = JSON.parse(fs.readFileSync('/home/michel/solana_darkpool/target/idl/chip_darkpool.json', 'utf8'));

  const wallet = new anchor.Wallet(buyer);
  const provider = new anchor.AnchorProvider(conn, wallet, { commitment: 'confirmed' });
  anchor.setProvider(provider);

  const program = new anchor.Program(idl, provider);

  const amount = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);
  const seedId = new anchor.BN(Date.now());

  const [escrowPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('escrow'),
      buyer.publicKey.toBuffer(),
      seller.publicKey.toBuffer(),
      seedId.toArrayLike(Buffer, 'le', 8),
    ],
    PROGRAM_ID
  );
  console.log('Escrow PDA:', escrowPda.toString());

  console.log('\n=== TEST: create_escrow ===');
  const createSig = await program.methods
    .createEscrow(amount, seller.publicKey, seedId)
    .accounts({
      buyer: buyer.publicKey,
      escrowAccount: escrowPda,
      systemProgram: SystemProgram.programId,
    })
    .signers([buyer])
    .rpc();
  console.log('create_escrow tx:', createSig);

  let escrowState = await program.account.escrowAccount.fetch(escrowPda);
  console.log('Status na create:', JSON.stringify(escrowState.status));

  console.log('\n=== TEST: dispute_escrow (door koper) ===');
  const disputeSig = await program.methods
    .disputeEscrow()
    .accounts({
      disputer: buyer.publicKey,
      escrowAccount: escrowPda,
    })
    .signers([buyer])
    .rpc();
  console.log('dispute_escrow tx:', disputeSig);

  escrowState = await program.account.escrowAccount.fetch(escrowPda);
  console.log('Status na dispute:', JSON.stringify(escrowState.status));

  console.log('\n=== RESULTAAT ===');
  console.log('Escrow PDA (voor resolve-dispute-tool.html):', escrowPda.toString());
  console.log('Koper (buyer):', buyer.publicKey.toString());
  console.log('Verkoper (seller):', seller.publicKey.toString());
  console.log('\nGebruik deze drie adressen in de resolve-dispute-tool om verder te gaan.');
}

main().catch((e) => {
  console.error('FOUT:', e);
  process.exit(1);
});
