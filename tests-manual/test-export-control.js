const fs = require('fs');
const anchor = require('@coral-xyz/anchor');
const { Connection, Keypair, PublicKey } = require('@solana/web3.js');
const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
const arcium = require('@arcium-hq/client');

function loadKeypair(path) {
  const raw = JSON.parse(fs.readFileSync(path, 'utf8'));
  return Keypair.fromSecretKey(new Uint8Array(raw));
}
function readUInt32LE(bytes) {
  return ((bytes[0]) | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24)) >>> 0;
}

const CLUSTER = 456;
const PROGRAM_ID = new PublicKey('6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o');
const POOL_ADDRESS = new PublicKey('C5Qty81mfacsL1YEmyLjYpdHK3Hrukg6KDbvgpe3PVN8');

async function main() {
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });
  const idl = JSON.parse(fs.readFileSync('/home/michel/solana_darkpool/target/idl/chip_darkpool.json', 'utf8'));

  async function tryMatch(label, payerKeypair) {
    const wallet = new anchor.Wallet(payerKeypair);
    const provider = new anchor.AnchorProvider(conn, wallet, { commitment: 'confirmed' });
    const program = new anchor.Program(idl, provider);

    const off = new anchor.BN(require('crypto').randomBytes(8), 'hex');
    const compDefOffset = readUInt32LE(arcium.getCompDefAccOffset('match_chip'));
    const args = [off, ...Array(14).fill(new Array(32).fill(0)), new Array(32).fill(0), new anchor.BN(0)];

    const [attestationPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('compliance'), payerKeypair.publicKey.toBytes()],
      PROGRAM_ID
    );

    try {
      const sig = await program.methods.matchChip(...args)
        .accountsPartial({
          payer: payerKeypair.publicKey,
          computationAccount: arcium.getComputationAccAddress(CLUSTER, off),
          clusterAccount: arcium.getClusterAccAddress(CLUSTER),
          mxeAccount: arcium.getMXEAccAddress(PROGRAM_ID),
          mempoolAccount: arcium.getMempoolAccAddress(CLUSTER),
          executingPool: arcium.getExecutingPoolAccAddress(CLUSTER),
          compDefAccount: arcium.getCompDefAccAddress(PROGRAM_ID, compDefOffset),
          moerasPool: POOL_ADDRESS,
          complianceAttestation: attestationPda,
        })
        .rpc();
      console.log(`[${label}] GESLAAGD, tx: ${sig}`);
      return true;
    } catch (e) {
      console.log(`[${label}] GEFAALD: ${e.message.slice(0, 200)}`);
      return false;
    }
  }

  console.log('=== TEST 1: goedgekeurde koper (buyer.json) ===');
  const approvedBuyer = loadKeypair('/home/michel/solana_darkpool/test-wallets/buyer.json');
  const okApproved = await tryMatch('GOEDGEKEURDE koper', approvedBuyer);

  console.log('\n=== TEST 2: niet-goedgekeurde koper (seller.json) ===');
  const unapprovedBuyer = loadKeypair('/home/michel/solana_darkpool/test-wallets/seller.json');
  const okUnapproved = await tryMatch('NIET-goedgekeurde koper', unapprovedBuyer);

  console.log('\n=== SAMENVATTING ===');
  console.log('Goedgekeurde koper (verwacht: true) :', okApproved);
  console.log('Niet-goedgekeurd (verwacht: false)   :', okUnapproved);
  const alles_goed = okApproved === true && okUnapproved === false;
  console.log(alles_goed ? '\nEXPORTCONTROLE WERKT VOLLEDIG CORRECT' : '\nONVERWACHT GEDRAG, controleer hierboven');
}

main().catch((e) => {
  console.error('FOUT:', e);
  process.exit(1);
});
