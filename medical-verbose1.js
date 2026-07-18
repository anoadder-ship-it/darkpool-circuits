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
const PROG_ID = 'CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4';
const sleep = ms => new Promise(r => setTimeout(r, ms));

(async () => {
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });
  const owner = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync(os.homedir() + '/.config/solana/id.json').toString())));
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: 'confirmed' });
  anchor.setProvider(provider);
  const pid = new PublicKey(PROG_ID);
  const IDL = JSON.parse(fs.readFileSync('target/idl/medical_darkpool.json').toString());
  IDL.address = PROG_ID;
  const prog = new anchor.Program(IDL, provider);
  console.log('prog.programId:', prog.programId.toBase58());

  const mxeKey = await getMXEPublicKey(provider, pid);
  const priv = x25519.utils.randomSecretKey();
  const cipher = new RescueCipher(x25519.getSharedSecret(priv, mxeKey));
  const pubArr = Array.from(x25519.getPublicKey(priv));

  const nb = randomBytes(16);
  console.log('Onze eigen nonce (hex):', nb.toString('hex'));
  const vals = [340,5000,580,40,2,340,1000,500,700,2].map(BigInt);
  const cts = cipher.encrypt(vals, nb).map(ct => Array.from(ct));
  const nonce = new BN(deserializeLE(nb).toString());
  const off = new BN(randomBytes(8), 'hex');

  const accs = {
    computationAccount: getComputationAccAddress(CLUSTER, off),
    clusterAccount: getClusterAccAddress(CLUSTER),
    mxeAccount: getMXEAccAddress(pid),
    mempoolAccount: getMempoolAccAddress(CLUSTER),
    executingPool: getExecutingPoolAccAddress(CLUSTER),
    compDefAccount: getCompDefAccAddress(pid, Buffer.from(getCompDefAccOffset('match_dataset')).readUInt32LE()),
  };

  const tx = await prog.methods.matchDataset(off, cts[0],cts[1],cts[2],cts[3],cts[4],cts[5],cts[6],cts[7],cts[8],cts[9], pubArr, nonce)
    .accountsPartial(accs)
    .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 })])
    .transaction();
  const { blockhash, lastValidBlockHeight } = await conn.getLatestBlockhash('confirmed');
  tx.recentBlockhash = blockhash; tx.lastValidBlockHeight = lastValidBlockHeight;
  tx.feePayer = owner.publicKey; tx.partialSign(owner);
  const sig = await conn.sendRawTransaction(tx.serialize(), { skipPreflight: true });
  console.log('Ingediend:', sig);
  await conn.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, 'confirmed');
  console.log('Bevestigd, nu pollen (max 30s)...');

  const seen = new Set([sig]);
  const start = Date.now();
  let gevonden = false;
  let pollNum = 0;
  while (Date.now() - start < 30000 && !gevonden) {
    pollNum++;
    const sigs = await conn.getSignaturesForAddress(prog.programId, { limit: 20 });
    const nieuw = sigs.filter(s => !seen.has(s.signature));
    console.log('[poll ' + pollNum + ', t+' + ((Date.now()-start)/1000).toFixed(1) + 's] ' + sigs.length + ' sigs, ' + nieuw.length + ' nieuw');
    for (const s of nieuw) {
      seen.add(s.signature);
      if (s.err) { console.log('  ' + s.signature.slice(0,12) + ' ERR, skip'); continue; }
      const txd = await conn.getTransaction(s.signature, { commitment: 'confirmed', maxSupportedTransactionVersion: 0 });
      const logs = txd?.meta?.logMessages || [];
      let matched = false;
      for (const log of logs) {
        if (!log.startsWith('Program data: ')) continue;
        let decoded;
        try { decoded = prog.coder.events.decode(log.slice('Program data: '.length)); }
        catch (e) { console.log('    decode fout:', e.message); continue; }
        if (!decoded) { console.log('    decode gaf null'); continue; }
        console.log('    event:', decoded.name);
        if (decoded.name !== 'datasetMatchedEvent') continue;
        const eventNonceHex = Buffer.from(decoded.data.nonce).toString('hex');
        console.log('    event nonce (hex):', eventNonceHex, ' onze nonce (hex):', nb.toString('hex'), ' gelijk?', eventNonceHex === nb.toString('hex'));
        if (eventNonceHex === nb.toString('hex')) {
          matched = true;
          const d = cipher.decrypt([Array.from(decoded.data.compatible), Array.from(decoded.data.score)], new Uint8Array(decoded.data.nonce));
          console.log('MATCH GEVONDEN! compatible=' + d[0] + ' score=' + d[1]);
          gevonden = true;
        }
      }
      if (!matched) console.log('  ' + s.signature.slice(0,12) + '... geen match in deze tx');
    }
    if (!gevonden) await sleep(1500);
  }
  if (!gevonden) console.log('NIET GEVONDEN binnen 30s');
})().catch(e => console.error('Fatal:', e.message, e.stack));
