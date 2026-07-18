const fs = require('fs');
const anchor = require('@coral-xyz/anchor');
const { Connection, Keypair, PublicKey } = require('@solana/web3.js');
const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
const PROGRAM_ID = new PublicKey('h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX');
const CLUSTER = 456;
const POOL_ADDRESS = new PublicKey('Hp9jftCAo9UWE6tGmkYYqT8ChybJMnwi8uTFHg2fu2fq');

function loadKeypair(path) {
  const raw = JSON.parse(fs.readFileSync(path, 'utf8'));
  return Keypair.fromSecretKey(new Uint8Array(raw));
}
function readUInt32LE(bytes) {
  return ((bytes[0]) | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24)) >>> 0;
}

async function main() {
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });
  const guardian = loadKeypair('/home/michel/solana_darkpool/test-wallets/buyer.json');
  const idl = JSON.parse(fs.readFileSync('/home/michel/solana_darkpool/target/idl/solana_darkpool.json', 'utf8'));
  const wallet = new anchor.Wallet(guardian);
  const provider = new anchor.AnchorProvider(conn, wallet, { commitment: 'confirmed' });
  anchor.setProvider(provider);
  const program = new anchor.Program(idl, provider);

  const arcium = require('@arcium-hq/client');

  async function tryPlaceOrder(label) {
    const off = new anchor.BN(arcium.randomBytes ? arcium.randomBytes(8) : require('crypto').randomBytes(8), 'hex');
    const compDefOffset = readUInt32LE(arcium.getCompDefAccOffset('place_order'));
    try {
      const sig = await program.methods
        .placeOrder(
          off,
          new Array(32).fill(0),
          new Array(32).fill(0),
          new Array(32).fill(0),
          new Array(32).fill(0),
          new anchor.BN(0)
        )
        .accountsPartial({
          payer: guardian.publicKey,
          computationAccount: arcium.getComputationAccAddress(CLUSTER, off),
          clusterAccount: arcium.getClusterAccAddress(CLUSTER),
          mxeAccount: arcium.getMXEAccAddress(PROGRAM_ID),
          mempoolAccount: arcium.getMempoolAccAddress(CLUSTER),
          executingPool: arcium.getExecutingPoolAccAddress(CLUSTER),
          compDefAccount: arcium.getCompDefAccAddress(PROGRAM_ID, compDefOffset),
          moerasPool: POOL_ADDRESS,
        })
        .rpc();
      console.log(`[${label}] place_order GESLAAGD, tx: ${sig}`);
      return true;
    } catch (e) {
      console.log(`[${label}] place_order GEFAALD: ${e.message.slice(0, 200)}`);
      return false;
    }
  }

  console.log('=== TEST 1: place_order met status=Active (moet slagen) ===');
  const okBefore = await tryPlaceOrder('VOOR trigger');

  console.log('\n=== TEST 2: trigger_moeras aanroepen ===');
  const triggerSig = await program.methods
    .triggerMoeras()
    .accounts({ pool: POOL_ADDRESS, signer: guardian.publicKey })
    .signers([guardian])
    .rpc();
  console.log('trigger_moeras tx:', triggerSig);
  let state = await program.account.poolState.fetch(POOL_ADDRESS);
  console.log('Status na trigger:', JSON.stringify(state.status));

  console.log('\n=== TEST 3: place_order met status=Moeras (moet FALEN) ===');
  const okDuring = await tryPlaceOrder('TIJDENS Moeras');

  console.log('\n=== TEST 4: reactivate_pool aanroepen ===');
  const reactivateSig = await program.methods
    .reactivatePool()
    .accounts({ pool: POOL_ADDRESS, signer: guardian.publicKey })
    .signers([guardian])
    .rpc();
  console.log('reactivate_pool tx:', reactivateSig);
  state = await program.account.poolState.fetch(POOL_ADDRESS);
  console.log('Status na reactivatie:', JSON.stringify(state.status));

  console.log('\n=== TEST 5: place_order na reactivatie (moet weer slagen) ===');
  const okAfter = await tryPlaceOrder('NA reactivatie');

  console.log('\n=== SAMENVATTING ===');
  console.log('Voor trigger (verwacht: true) :', okBefore);
  console.log('Tijdens Moeras (verwacht: false):', okDuring);
  console.log('Na reactivatie (verwacht: true) :', okAfter);
  const alles_goed = okBefore === true && okDuring === false && okAfter === true;
  console.log(alles_goed ? '\nKILLSWITCH WERKT VOLLEDIG CORRECT' : '\nKILLSWITCH GEDRAAGT ZICH ONVERWACHT, controleer hierboven');
}
main().catch((e) => {
  console.error('FOUT:', e);
  process.exit(1);
});
