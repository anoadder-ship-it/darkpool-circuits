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
const N_TESTS = parseInt(process.env.N_TESTS || '2', 10);
const TIMEOUT_MS = parseInt(process.env.TIMEOUT_MS || '20000', 10);
const sleep = ms => new Promise(r => setTimeout(r, ms));

// GEDEELDE seen-set over de hele run (voorkomt verwarring tussen tests)
const globalSeen = new Set();

async function waitForCallback(conn, prog, ourSig, i, timeoutMs, pollIntervalMs) {
  globalSeen.add(ourSig);
  const start = Date.now();
  let pollNum = 0;
  while (Date.now() - start < timeoutMs) {
    pollNum++;
    let sigs;
    try {
      sigs = await conn.getSignaturesForAddress(prog.programId, { limit: 15 });
    } catch (e) {
      console.log('  [poll ' + pollNum + '] getSignaturesForAddress faalde: ' + e.message);
      await sleep(pollIntervalMs);
      continue;
    }
    const nieuw = sigs.filter(s => !globalSeen.has(s.signature));
    console.log('  [poll ' + pollNum + ', t+' + ((Date.now()-start)/1000).toFixed(1) + 's] ' + sigs.length + ' sigs opgehaald, ' + nieuw.length + ' nieuw');

    for (const s of nieuw) {
      globalSeen.add(s.signature);
      if (s.err) { console.log('    - ' + s.signature.slice(0,12) + '... ERR, overslaan'); continue; }
      let tx;
      try {
        tx = await conn.getTransaction(s.signature, { commitment: 'confirmed', maxSupportedTransactionVersion: 0 });
      } catch (e) {
        console.log('    - ' + s.signature.slice(0,12) + '... getTransaction faalde: ' + e.message);
        continue;
      }
      const logs = tx?.meta?.logMessages || [];
      const heeftCallback = logs.some(l => l.includes('MatchOrdersCallback'));
      console.log('    - ' + s.signature.slice(0,12) + '... OK, MatchOrdersCallback=' + heeftCallback);
      if (heeftCallback) {
        for (const log of logs) {
          if (log.startsWith('Program data: ')) {
            const data = log.slice('Program data: '.length);
            try {
              const decoded = prog.coder.events.decode(data);
              if (decoded && decoded.name === 'matchedEvent') {
                console.log('    -> matchedEvent gevonden!');
                return decoded.data;
              }
            } catch (e) { /* geen geldig event */ }
          }
        }
        console.log('    -> had callback-log maar geen matchedEvent gedecodeerd (coder-probleem?)');
      }
    }
    await sleep(pollIntervalMs);
  }
  return null;
}

async function oneTest(conn, owner, cipher, pubKeyArr, i) {
  const buy  = Math.floor(Math.random() * 200) + 1;
  const sell = Math.floor(Math.random() * 200) + 1;
  const expected = buy >= sell;

  const nb    = randomBytes(16);
  const cts   = cipher.encrypt([BigInt(buy), BigInt(sell)], nb).map(ct => Array.from(ct));
  const nonce = new BN(deserializeLE(nb).toString());
  const off   = new BN(randomBytes(8), 'hex');
  const pid   = new PublicKey(PROG_ID);

  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: 'confirmed' });
  anchor.setProvider(provider);
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

  console.log('Test #' + i + ': buy=' + buy + ' sell=' + sell + ' -- transactie samenstellen...');

  const tx = await prog.methods.matchOrders(off, cts[0], cts[1], pubKeyArr, nonce)
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

  try {
    await conn.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, 'confirmed');
    console.log('Bevestigd op t+0s, nu pollen op callback...');
  } catch (e) {
    console.log('TIMEOUT #' + i + '  (indienen mislukt: ' + e.message + ')');
    return false;
  }

  const matchedEv = await waitForCallback(conn, prog, sig, i, TIMEOUT_MS, 1500);

  if (!matchedEv) {
    console.log('TIMEOUT #' + i + '  (geen callback binnen ' + (TIMEOUT_MS/1000) + 's)');
    return false;
  }

  const d = cipher.decrypt([Array.from(matchedEv.matched)], new Uint8Array(matchedEv.nonce));
  const result = { matched: d[0] === 1n, buy, sell, expected };
  const ok = result.matched === result.expected;
  console.log((ok ? 'PASS' : 'FAIL') + ' #' + i + '  buy=' + buy + ' sell=' + sell + ' matched=' + result.matched + ' expected=' + result.expected);
  return ok;
}

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

  // Baseline: alle huidige signatures alvast als 'seen' markeren, zodat
  // oude/onafhankelijke transacties niet per ongeluk als callback tellen.
  const baseline = await conn.getSignaturesForAddress(new PublicKey(PROG_ID), { limit: 15 });
  baseline.forEach(s => globalSeen.add(s.signature));
  console.log('Baseline: ' + baseline.length + ' bestaande signatures gemarkeerd als bekend.');

  console.log('=== DIAGNOSE-test: ' + N_TESTS + ' transacties, timeout=' + (TIMEOUT_MS/1000) + 's ===');
  const start = Date.now();
  const results = [];
  for (let i = 0; i < N_TESTS; i++) {
    results.push(await oneTest(conn, owner, cipher, pubArr, i));
    if (i < N_TESTS - 1) await sleep(1000);
  }
  const passed = results.filter(Boolean).length;
  const elapsed = ((Date.now() - start) / 1000).toFixed(1);
  console.log('');
  console.log('=== ' + passed + '/' + N_TESTS + ' PASS in ' + elapsed + 's ===');
}
main().catch(e => console.error('Fatal:', e.message, e.stack));
