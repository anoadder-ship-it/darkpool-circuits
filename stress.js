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
const CONCURRENCY = 4;
const sleep = ms => new Promise(r => setTimeout(r, ms));

async function withRetry(fn, { retries = 3, delayMs = 700, label = 'actie' } = {}) {
  let lastErr;
  for (let attempt = 1; attempt <= retries; attempt++) {
    try { return await fn(); }
    catch (e) {
      lastErr = e;
      if (attempt < retries) await sleep(delayMs * attempt);
    }
  }
  throw lastErr;
}

// Gedeelde tx-cache: als meerdere gelijktijdige tests dezelfde signature
// tegenkomen, wordt die maar EEN keer via RPC opgehaald (bespaart calls).
const txCache = new Map();
async function getTxCached(conn, signature) {
  if (txCache.has(signature)) return txCache.get(signature);
  const p = withRetry(
    () => conn.getTransaction(signature, { commitment: 'confirmed', maxSupportedTransactionVersion: 0 }),
    { retries: 3, delayMs: 700, label: 'getTransaction' }
  );
  txCache.set(signature, p);
  return p;
}

// Checkt of een tx een specifiek account-adres bevat (werkt voor legacy EN v0 messages).
function txBevatAccount(tx, pubkeyBase58) {
  const msg = tx.transaction.message;
  const keys = msg.staticAccountKeys ? msg.staticAccountKeys : msg.accountKeys;
  return keys.some(k => k.toBase58() === pubkeyBase58);
}

const globalBaseline = new Set(); // oude sigs van voor de run, altijd negeren

// Poll op het programma-adres. BELANGRIJK voor parallel draaien: elke test
// controleert of de callback-tx ZIJN EIGEN computationAccount bevat, niet
// zomaar de eerste 'matchEvent' die langskomt (die kan van een andere,
// gelijktijdige test zijn).
async function waitForCallback(conn, prog, computationAccountPubkey, timeoutMs, pollIntervalMs) {
  const compAccBase58 = computationAccountPubkey.toBase58();
  const localSeen = new Set();
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    let sigs;
    try {
      sigs = await withRetry(
        () => conn.getSignaturesForAddress(prog.programId, { limit: 20 }),
        { retries: 3, delayMs: 700, label: 'getSignaturesForAddress' }
      );
    } catch (e) {
      await sleep(pollIntervalMs);
      continue;
    }
    for (const s of sigs) {
      if (globalBaseline.has(s.signature) || localSeen.has(s.signature)) continue;
      if (s.err) { localSeen.add(s.signature); continue; }
      let tx;
      try {
        tx = await getTxCached(conn, s.signature);
      } catch (e) { continue; } // blijvend mislukt, volgende poll opnieuw proberen
      localSeen.add(s.signature);
      const logs = tx?.meta?.logMessages || [];
      if (!logs.some(l => l.includes('MatchOrdersCallback'))) continue;
      if (!txBevatAccount(tx, compAccBase58)) continue; // hoort bij een ANDERE test

      for (const log of logs) {
        if (log.startsWith('Program data: ')) {
          const data = log.slice('Program data: '.length);
          try {
            const decoded = prog.coder.events.decode(data);
            if (decoded && decoded.name === 'matchEvent') {
              return decoded.data;
            }
          } catch (e) { /* geen geldig event, overslaan */ }
        }
      }
    }
    await sleep(pollIntervalMs);
  }
  return null;
}

async function buildSignedTx(prog, owner, off, cts, pubKeyArr, nonce, conn) {
  const pid = new PublicKey(PROG_ID);
  const accs = {
    computationAccount: getComputationAccAddress(CLUSTER, off),
    clusterAccount:     getClusterAccAddress(CLUSTER),
    mxeAccount:         getMXEAccAddress(pid),
    mempoolAccount:     getMempoolAccAddress(CLUSTER),
    executingPool:      getExecutingPoolAccAddress(CLUSTER),
    compDefAccount:     getCompDefAccAddress(pid, Buffer.from(getCompDefAccOffset('match_orders')).readUInt32LE()),
  };
  const tx = await prog.methods.matchOrders(off, cts[0], cts[1], pubKeyArr, nonce)
    .accountsPartial({...accs, payer: owner.publicKey})
    .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 })])
    .transaction();
  const { blockhash, lastValidBlockHeight } = await conn.getLatestBlockhash('confirmed');
  tx.recentBlockhash      = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;
  tx.feePayer             = owner.publicKey;
  tx.partialSign(owner);
  return { tx, blockhash, lastValidBlockHeight, accs };
}

async function submitAndConfirm(conn, prog, owner, off, cts, pubKeyArr, nonce, maxAttempts, i) {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    const { tx, blockhash, lastValidBlockHeight, accs } = await buildSignedTx(prog, owner, off, cts, pubKeyArr, nonce, conn);
    let sig;
    try {
      sig = await withRetry(
        () => conn.sendRawTransaction(tx.serialize(), { skipPreflight: true }),
        { retries: 2, delayMs: 500, label: 'sendRawTransaction' }
      );
    } catch (e) {
      if (attempt === maxAttempts) throw e;
      console.log('Test #' + i + ': indienen mislukt (poging ' + attempt + '), opnieuw...');
      continue;
    }
    try {
      await conn.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, 'confirmed');
      return { sig, accs };
    } catch (e) {
      try {
        const status = await conn.getSignatureStatus(sig, { searchTransactionHistory: true });
        if (status?.value && !status.value.err) return { sig, accs };
      } catch (e2) { /* status-check zelf mislukte */ }
      if (attempt === maxAttempts) throw e;
      console.log('Test #' + i + ': bevestiging mislukt (poging ' + attempt + '), tx landde niet, nieuwe poging...');
    }
  }
  throw new Error('Alle ' + maxAttempts + ' pogingen mislukt');
}

async function oneTest(conn, owner, cipher, pubKeyArr, i) {
  const buy  = Math.floor(Math.random() * 200) + 1;
  const sell = Math.floor(Math.random() * 200) + 1;
  const expected = buy >= sell;

  const nb    = randomBytes(16);
  const cts   = cipher.encrypt([BigInt(buy), BigInt(sell)], nb).map(ct => Array.from(ct));
  const nonce = new BN(deserializeLE(nb).toString());
  const off   = new BN(randomBytes(8), 'hex');

  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: 'confirmed' });
  const IDL = JSON.parse(fs.readFileSync('target/idl/solana_darkpool.json').toString());
  IDL.address = PROG_ID;
  const prog = new anchor.Program(IDL, provider);

  let sig, accs;
  try {
    ({ sig, accs } = await submitAndConfirm(conn, prog, owner, off, cts, pubKeyArr, nonce, 3, i));
  } catch (e) {
    console.log('TIMEOUT #' + i + '  (indienen/bevestigen definitief mislukt: ' + e.message + ')');
    return false;
  }

  const matchedEv = await waitForCallback(conn, prog, accs.computationAccount, 30000, 1500);

  if (!matchedEv) {
    console.log('TIMEOUT #' + i + '  (geen callback binnen 30s, sig=' + sig + ')');
    return false;
  }

  const d = cipher.decrypt([Array.from(matchedEv.result)], new Uint8Array(matchedEv.nonce));
  const result = { matched: d[0] === 1n, buy, sell, expected };
  const ok = result.matched === result.expected;
  console.log((ok ? 'PASS' : 'FAIL') + ' #' + i + '  buy=' + buy + ' sell=' + sell + ' matched=' + result.matched + ' expected=' + result.expected);
  return ok;
}

async function runInWaves(total, concurrency, taskFn) {
  const results = new Array(total);
  for (let start = 0; start < total; start += concurrency) {
    const batch = [];
    for (let i = start; i < Math.min(start + concurrency, total); i++) batch.push(i);
    const batchResults = await Promise.all(batch.map(i => taskFn(i)));
    batch.forEach((i, idx) => { results[i] = batchResults[idx]; });
  }
  return results;
}

async function main() {
  const conn  = new Connection(HELIUS, { commitment: 'confirmed' });
  const owner = Keypair.fromSecretKey(new Uint8Array(JSON.parse(
    fs.readFileSync(os.homedir() + '/.config/solana/id.json').toString()
  )));
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: 'confirmed' });
  anchor.setProvider(provider);
  const mxeKey = await getMXEPublicKey(provider, new PublicKey(PROG_ID));
  const priv   = x25519.utils.randomSecretKey();
  const cipher = new RescueCipher(x25519.getSharedSecret(priv, mxeKey));
  const pubArr = Array.from(x25519.getPublicKey(priv));

  const baseline = await conn.getSignaturesForAddress(new PublicKey(PROG_ID), { limit: 20 });
  baseline.forEach(s => globalBaseline.add(s.signature));

  console.log('=== Stress test: 10 transacties, ' + CONCURRENCY + ' tegelijk (met automatisch herstel) ===');
  const start = Date.now();
  const results = await runInWaves(10, CONCURRENCY, i => oneTest(conn, owner, cipher, pubArr, i));
  const passed = results.filter(Boolean).length;
  const elapsed = ((Date.now() - start) / 1000).toFixed(1);
  console.log('');
  console.log('=== ' + passed + '/10 PASS in ' + elapsed + 's ===');
  console.log('Gem per tx: ' + (elapsed / 10).toFixed(1) + 's');
}
main().catch(e => console.error('Fatal:', e.message));
