const anchor = require('@anchor-lang/core');
const { BN } = require('@anchor-lang/core');
const { PublicKey, ComputeBudgetProgram, Connection, Keypair } = require('@solana/web3.js');
const { getCompDefAccOffset, RescueCipher, deserializeLE, getMXEPublicKey,
        getMXEAccAddress, getMempoolAccAddress, getCompDefAccAddress,
        getExecutingPoolAccAddress, getComputationAccAddress,
        getClusterAccAddress, x25519 } = require('@arcium-hq/client');
const { randomBytes } = require('crypto');
const fs = require('fs'), os = require('os');

const HELIUS  = process.env.HELIUS;
const CLUSTER = 456;
const PROG_ID = 'h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX';
const sleep = ms => new Promise(r => setTimeout(r, ms));

async function main() {
  const conn  = new Connection(HELIUS, { commitment: 'confirmed' });
  const owner = Keypair.fromSecretKey(new Uint8Array(JSON.parse(
    fs.readFileSync(os.homedir() + '/.config/solana/id.json').toString()
  )));
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: 'confirmed' });
  const mxeKey = await getMXEPublicKey(provider, new PublicKey(PROG_ID));
  const priv   = x25519.utils.randomSecretKey();
  const cipher = new RescueCipher(x25519.getSharedSecret(priv, mxeKey));
  const pubArr = Array.from(x25519.getPublicKey(priv));

  const buy  = 100, sell = 50, expected = buy >= sell;
  const nb    = randomBytes(16);
  const cts   = cipher.encrypt([BigInt(buy), BigInt(sell)], nb).map(ct => Array.from(ct));
  const nonce = new BN(deserializeLE(nb).toString());
  const off   = new BN(randomBytes(8), 'hex');
  const pid   = new PublicKey(PROG_ID);

  const IDL = JSON.parse(fs.readFileSync('target/idl/solana_darkpool.json').toString());
  IDL.address = PROG_ID;
  const prog = new anchor.Program(IDL, provider);

  const accs = {
    computationAccount: getComputationAccAddress(CLUSTER, off),
    clusterAccount:     getClusterAccAddress(CLUSTER),
    mxeAccount:         getMXEAccAddress(pid),
    mempoolAccount:     getMempoolAccAddress(CLUSTER),
    executingPool:      getExecutingPoolAccAddress(CLUSTER),
    compDefAccount:     getCompDefAccAddress(pid, Buffer.from(getCompDefAccOffset('match_orders')).readUInt32LE()),
  };

  console.log('Test: buy=' + buy + ' sell=' + sell);

  const tx = await prog.methods.matchOrders(off, cts[0], cts[1], pubArr, nonce)
    .accountsPartial({...accs, payer: owner.publicKey})
    .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 })])
    .transaction();
  const { blockhash, lastValidBlockHeight } = await conn.getLatestBlockhash('confirmed');
  tx.recentBlockhash      = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;
  tx.feePayer             = owner.publicKey;
  tx.partialSign(owner);

  const sig = await conn.sendRawTransaction(tx.serialize(), { skipPreflight: true });
  console.log('Ingediend: ' + sig);
  await conn.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, 'confirmed');
  console.log('Bevestigd. Nu pollen (max 20s)...');

  const seen = new Set([sig]);
  const start = Date.now();
  let gevonden = false;

  while (Date.now() - start < 20000 && !gevonden) {
    const sigs = await conn.getSignaturesForAddress(prog.programId, { limit: 15 });
    const nieuw = sigs.filter(s => !seen.has(s.signature));
    if (nieuw.length > 0) {
      console.log('t+' + ((Date.now()-start)/1000).toFixed(1) + 's: ' + nieuw.length + ' nieuwe sigs');
    }
    for (const s of nieuw) {
      seen.add(s.signature);
      if (s.err) { console.log('  ' + s.signature + ' -> ERR, skip'); continue; }
      const txd = await conn.getTransaction(s.signature, { commitment: 'confirmed', maxSupportedTransactionVersion: 0 });
      const logs = txd?.meta?.logMessages || [];
      const heeftCallback = logs.some(l => l.includes('MatchOrdersCallback'));
      console.log('  ' + s.signature + ' -> OK, MatchOrdersCallback=' + heeftCallback);
      if (!heeftCallback) continue;

      console.log('  --- ALLE Program data regels in deze tx ---');
      for (const log of logs) {
        if (log.startsWith('Program data: ')) {
          const data = log.slice('Program data: '.length);
          console.log('  RAW:', data);
          try {
            const decoded = prog.coder.events.decode(data);
            console.log('  DECODE OK -> naam:', decoded ? decoded.name : '(null)');
            console.log('  DECODE data:', decoded ? JSON.stringify(decoded.data, (k,v)=> typeof v==='bigint'?v.toString():v) : '(n.v.t.)');
          } catch (e) {
            console.log('  DECODE FOUT:', e.message);
          }
        }
      }
      gevonden = true;
    }
    if (!gevonden) await sleep(1500);
  }

  if (!gevonden) console.log('Geen callback gevonden binnen 20s.');
}
main().catch(e => console.error('Fatal:', e.message, e.stack));
