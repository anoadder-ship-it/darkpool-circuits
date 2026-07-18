import * as anchor from "@anchor-lang/core";
import { BN } from "@anchor-lang/core";
import { PublicKey, ComputeBudgetProgram, Connection, Keypair, Transaction } from "@solana/web3.js";
import { getCompDefAccOffset, RescueCipher, deserializeLE, getMXEPublicKey,
         getMXEAccAddress, getMempoolAccAddress, getCompDefAccAddress,
         getExecutingPoolAccAddress, getComputationAccAddress,
         getClusterAccAddress, x25519 } from "@arcium-hq/client";
import { randomBytes } from "crypto";
import * as fs from "fs";
import * as os from "os";

const PROGRAM_ID  = new PublicKey("3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4");
const HELIUS      = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const CLUSTER     = 456;
const CONCURRENCY = 4;
const N_TESTS     = 10;
const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

let cipher: RescueCipher, pubKeyArr: any, conn: Connection, owner: Keypair, prog: any;

async function withRetry<T>(fn: () => Promise<T>, retries = 3, delayMs = 500): Promise<T> {
  let lastErr: any;
  for (let a = 1; a <= retries; a++) {
    try { return await fn(); }
    catch (e) { lastErr = e; if (a < retries) await sleep(delayMs * a); }
  }
  throw lastErr;
}

async function sendTx(t: Transaction): Promise<string> {
  const { blockhash, lastValidBlockHeight } = await conn.getLatestBlockhash("confirmed");
  t.recentBlockhash = blockhash; t.lastValidBlockHeight = lastValidBlockHeight;
  t.feePayer = owner.publicKey; t.partialSign(owner);
  const sig = await withRetry(() => conn.sendRawTransaction(t.serialize(), { skipPreflight: true }));
  await withRetry(() => conn.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, "confirmed"));
  return sig;
}

function accs(name: string, offset: BN) {
  return {
    computationAccount: getComputationAccAddress(CLUSTER, offset),
    clusterAccount: getClusterAccAddress(CLUSTER),
    mxeAccount: getMXEAccAddress(PROGRAM_ID),
    mempoolAccount: getMempoolAccAddress(CLUSTER),
    executingPool: getExecutingPoolAccAddress(CLUSTER),
    compDefAccount: getCompDefAccAddress(PROGRAM_ID, Buffer.from(getCompDefAccOffset(name)).readUInt32LE()),
  };
}

function encTogether(values: bigint[]): { cts: any[], nonce: BN } {
  const nb = randomBytes(16);
  const encrypted = cipher.encrypt(values, nb);
  return { cts: encrypted.map(ct => Array.from(ct) as any), nonce: new BN(deserializeLE(nb).toString()) };
}

const txCache = new Map<string, Promise<any>>();
function getTxCached(signature: string) {
  if (!txCache.has(signature)) {
    txCache.set(signature, withRetry(() =>
      conn.getTransaction(signature, { commitment: "confirmed", maxSupportedTransactionVersion: 0 })
    ));
  }
  return txCache.get(signature)!;
}

function txBevatAccount(tx: any, pubkeyBase58: string): boolean {
  const msg = tx.transaction.message;
  const keys = msg.staticAccountKeys ? msg.staticAccountKeys : msg.accountKeys;
  return keys.some((k: any) => k.toBase58() === pubkeyBase58);
}

const globalBaseline = new Set<string>();

async function waitForEvent(eventName: string, computationAccountBase58: string, timeoutMs = 90000): Promise<bigint[] | null> {
  const localSeen = new Set<string>();
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    let sigs: any[];
    try {
      sigs = await withRetry(() => conn.getSignaturesForAddress(prog.programId, { limit: 20 }));
    } catch (e) { await sleep(1500); continue; }
    for (const s of sigs) {
      if (globalBaseline.has(s.signature) || localSeen.has(s.signature)) continue;
      if (s.err) { localSeen.add(s.signature); continue; }
      let tx: any;
      try { tx = await getTxCached(s.signature); } catch (e) { continue; }
      localSeen.add(s.signature);
      if (!tx) continue;
      if (!txBevatAccount(tx, computationAccountBase58)) continue;
      const logs: string[] = tx?.meta?.logMessages || [];
      for (const log of logs) {
        if (!log.startsWith("Program data: ")) continue;
        let decoded: any;
        try { decoded = prog.coder.events.decode(log.slice("Program data: ".length)); }
        catch (e) { continue; }
        if (!decoded || decoded.name !== eventName) continue;
        try {
          const nonce = new Uint8Array(decoded.data.nonce as number[]);
          if (decoded.data.result) {
            return cipher.decrypt([Array.from(decoded.data.result)], nonce);
          } else if (decoded.data.matched !== undefined) {
            return cipher.decrypt([Array.from(decoded.data.matched), Array.from(decoded.data.score)], nonce);
          }
        } catch (e) { return null; }
      }
    }
    await sleep(1500);
  }
  return null;
}

async function oneTest(i: number): Promise<boolean> {
  const matchCase = i % 2 === 0; // even = MATCH, oneven = GEEN MATCH
  const queryMaterial = matchCase ? BigInt(1001) : BigInt(1002); // staal vs aluminium
  const enc = encTogether([
    BigInt(1001), BigInt(50000), BigInt(85), BigInt(150), BigInt(7), BigInt(1),  // aanbod: staal
    queryMaterial, BigInt(10000), BigInt(80), BigInt(200), BigInt(10), BigInt(1), // vraag
  ]);
  const off = new BN(randomBytes(8), "hex");
  const compAccBase58 = getComputationAccAddress(CLUSTER, off).toBase58();
  try {
    const t = await (prog.methods as any).matchSupply(
      off, enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4], enc.cts[5],
      enc.cts[6], enc.cts[7], enc.cts[8], enc.cts[9], enc.cts[10], enc.cts[11],
      pubKeyArr, enc.nonce
    ).accountsPartial(accs("match_supply", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    await sendTx(t);
  } catch (e: any) {
    console.log("TIMEOUT #" + i + " (indienen mislukt: " + e.message.slice(0, 100) + ")");
    return false;
  }
  const result = await waitForEvent("supplyMatchedEvent", compAccBase58);
  if (!result) { console.log("TIMEOUT #" + i + " (geen callback binnen 90s)"); return false; }
  const matched = result[0] === BigInt(1);
  const expected = matchCase;
  const ok = matched === expected;
  console.log((ok ? "PASS" : "FAIL") + " #" + i + "  case=" + (matchCase ? "MATCH" : "GEEN MATCH") +
              "  matched=" + matched + " score=" + result[1] + "/100");
  return ok;
}

async function runInWaves(total: number, concurrency: number): Promise<boolean[]> {
  const results: boolean[] = new Array(total);
  for (let start = 0; start < total; start += concurrency) {
    const batch: number[] = [];
    for (let i = start; i < Math.min(start + concurrency, total); i++) batch.push(i);
    const batchResults = await Promise.all(batch.map(i => oneTest(i)));
    batch.forEach((i, idx) => { results[i] = batchResults[idx]; });
  }
  return results;
}

async function main() {
  conn  = new Connection(HELIUS, { commitment: "confirmed" });
  owner = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync(os.homedir() + "/.config/solana/id.json").toString())));
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: "confirmed" });
  anchor.setProvider(provider);
  const IDL = JSON.parse(fs.readFileSync("target/idl/supply_chain_darkpool.json").toString());
  IDL.address = PROGRAM_ID.toBase58();
  prog = new anchor.Program(IDL as any, provider);
  const mxeKey = await getMXEPublicKey(provider, PROGRAM_ID);
  if (!mxeKey) throw new Error("MXE key niet gevonden");
  const priv = x25519.utils.randomSecretKey();
  cipher    = new RescueCipher(x25519.getSharedSecret(priv, mxeKey));
  pubKeyArr = Array.from(x25519.getPublicKey(priv)) as any;

  const baseline = await conn.getSignaturesForAddress(PROGRAM_ID, { limit: 20 });
  baseline.forEach(s => globalBaseline.add(s.signature));

  console.log("=== Supply Chain Darkpool Stress Test: " + N_TESTS + " transacties, " + CONCURRENCY + " tegelijk ===");
  const start = Date.now();
  const results = await runInWaves(N_TESTS, CONCURRENCY);
  const passed = results.filter(Boolean).length;
  const elapsed = ((Date.now() - start) / 1000).toFixed(1);
  console.log("");
  console.log("=== " + passed + "/" + N_TESTS + " PASS in " + elapsed + "s ===");
}
main().catch(e => { console.error("Fout:", e?.message || e); process.exit(1); });
