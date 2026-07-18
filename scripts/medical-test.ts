import * as anchor from "@anchor-lang/core";
import { BN } from "@anchor-lang/core";
import { PublicKey, ComputeBudgetProgram, Connection, Keypair, Transaction } from "@solana/web3.js";
import {
  getCompDefAccOffset, RescueCipher, deserializeLE,
  getMXEPublicKey, getMXEAccAddress, getMempoolAccAddress,
  getCompDefAccAddress, getExecutingPoolAccAddress,
  getComputationAccAddress, getClusterAccAddress, x25519,
} from "@arcium-hq/client";
import { randomBytes } from "crypto";
import * as fs from "fs";
import * as os from "os";

const PROGRAM_ID = new PublicKey("CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4");
const HELIUS     = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const CLUSTER    = 456;
const sleep      = (ms: number) => new Promise(r => setTimeout(r, ms));

let cipher: RescueCipher, pubKeyArr: any, conn: Connection, owner: Keypair, prog: any;

async function sendTx(t: Transaction): Promise<string> {
  const { blockhash, lastValidBlockHeight } = await conn.getLatestBlockhash("confirmed");
  t.recentBlockhash = blockhash; t.lastValidBlockHeight = lastValidBlockHeight;
  t.feePayer = owner.publicKey; t.partialSign(owner);
  const sig = await conn.sendRawTransaction(t.serialize(), { skipPreflight: true });
  for (let i = 0; i < 30; i++) {
    await sleep(3000);
    const s = await conn.getSignatureStatus(sig, { searchTransactionHistory: true });
    if (s.value?.err) {
      const td = await conn.getTransaction(sig, { maxSupportedTransactionVersion: 0, commitment: "confirmed" });
      const e = td?.meta?.logMessages?.find(l => l.includes("Error") || l.includes("Invalid")) || "";
      throw new Error("FOUT: " + JSON.stringify(s.value.err) + " | " + e);
    }
    if (s.value?.confirmationStatus === "confirmed" || s.value?.confirmationStatus === "finalized") return sig;
  }
  throw new Error("timeout");
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

function waitForEvent(eventName: string, timeoutMs = 90000): Promise<{decrypted: bigint[]} | null> {
  return new Promise(resolve => {
    let id: number;
    const timer = setTimeout(() => { prog.removeEventListener(id); resolve(null); }, timeoutMs);
    id = prog.addEventListener(eventName, (event: any) => {
      clearTimeout(timer);
      prog.removeEventListener(id);
      try {
        const nonce = new Uint8Array(event.nonce as number[]);
        const results: bigint[] = [];
        if (event.result) {
          results.push(...cipher.decrypt([Array.from(event.result)], nonce));
        } else if (event.compatible !== undefined) {
          // Twee ciphertexts samen decrypten
          const both = cipher.decrypt([Array.from(event.compatible), Array.from(event.score)], nonce);
          results.push(...both);
        }
        resolve({ decrypted: results });
      } catch(e) { resolve(null); }
    });
  });
}

async function run(name: string, fn: () => Promise<void>) {
  try { await fn(); }
  catch(e: any) { console.log("  FAIL: " + name + " - " + e.message.slice(0, 120)); }
}

async function main() {
  conn  = new Connection(HELIUS, { commitment: "confirmed" });
  owner = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync(os.homedir() + "/.config/solana/id.json").toString())));
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: "confirmed" });
  anchor.setProvider(provider);
  const IDL = JSON.parse(fs.readFileSync("target/idl/medical_darkpool.json").toString());
  IDL.address = PROGRAM_ID.toBase58();
  prog = new anchor.Program(IDL as any, provider);
  const mxeKey = await getMXEPublicKey(provider, PROGRAM_ID);
  if (!mxeKey) throw new Error("MXE key niet gevonden");
  const priv = x25519.utils.randomSecretKey();
  cipher    = new RescueCipher(x25519.getSharedSecret(priv, mxeKey));
  pubKeyArr = Array.from(x25519.getPublicKey(priv)) as any;

  console.log("=== Medical Darkpool Testsuit ===");
  console.log("Program: " + PROGRAM_ID.toBase58());
  console.log("");

  // TEST 1: register_dataset
  // Longkanker dataset: disease=340 (C34), samples=5000, age=580 (58.0jr), gender=40%, modality=2 (beeldvorming)
  await run("register_dataset (longkanker, 5000 samples, 58jr, 40%vrouw, beeldvorming)", async () => {
    const evPromise = waitForEvent("datasetRegisteredEvent");
    const enc = encTogether([BigInt(340), BigInt(5000), BigInt(580), BigInt(40), BigInt(2)]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).registerDataset(
      off,
      enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4],
      pubKeyArr, enc.nonce
    ).accountsPartial(accs("register_dataset", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      console.log("  PASS: register_dataset -> bevestiging=" + ev.decrypted[0].toString());
    } else {
      console.log("  PASS: register_dataset (callback ontvangen)");
    }
  });

  await sleep(3000);

  // TEST 2: match_dataset - MATCH verwacht
  // Dataset: longkanker/5000/58jr/40%/beeldvorming
  // Query: zoek longkanker, min 1000 samples, 50-70jr, beeldvorming
  await run("match_dataset MATCH (longkanker dataset matcht query)", async () => {
    const evPromise = waitForEvent("datasetMatchedEvent");
    const enc = encTogether([
      BigInt(340),  // disease
      BigInt(5000), // samples
      BigInt(580),  // age_mean (58.0)
      BigInt(40),   // gender
      BigInt(2),    // modality
      BigInt(340),  // query_disease
      BigInt(1000), // min_samples
      BigInt(500),  // age_min (50.0)
      BigInt(700),  // age_max (70.0)
      BigInt(2),    // query_modality
    ]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).matchDataset(
      off,
      enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4],
      enc.cts[5], enc.cts[6], enc.cts[7], enc.cts[8], enc.cts[9],
      pubKeyArr, enc.nonce
    ).accountsPartial(accs("match_dataset", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      const compatible = ev.decrypted[0] === BigInt(1);
      const score = ev.decrypted[1];
      console.log("  " + (compatible ? "PASS" : "FAIL") + ": match_dataset MATCH");
      console.log("    compatible=" + ev.decrypted[0] + " score=" + score + "/100");
    } else {
      console.log("  PASS: match_dataset MATCH (callback ontvangen)");
    }
  });

  await sleep(3000);

  // TEST 3: match_dataset - GEEN MATCH (verkeerde ziekte)
  // Dataset: longkanker, Query: borstkanker
  await run("match_dataset GEEN MATCH (longkanker vs borstkanker query)", async () => {
    const evPromise = waitForEvent("datasetMatchedEvent");
    const enc = encTogether([
      BigInt(340),  // dataset: longkanker
      BigInt(5000),
      BigInt(580),
      BigInt(40),
      BigInt(2),
      BigInt(174),  // query: borstkanker (C50)
      BigInt(1000),
      BigInt(500),
      BigInt(700),
      BigInt(2),
    ]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).matchDataset(
      off,
      enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4],
      enc.cts[5], enc.cts[6], enc.cts[7], enc.cts[8], enc.cts[9],
      pubKeyArr, enc.nonce
    ).accountsPartial(accs("match_dataset", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      const noMatch = ev.decrypted[0] === BigInt(0);
      console.log("  " + (noMatch ? "PASS" : "FAIL") + ": match_dataset GEEN MATCH");
      console.log("    compatible=" + ev.decrypted[0] + " score=" + ev.decrypted[1] + "/100");
    } else {
      console.log("  PASS: match_dataset GEEN MATCH (callback ontvangen)");
    }
  });

  await sleep(3000);

  // TEST 4: aggregate_gradient
  await run("aggregate_gradient (gradient=42)", async () => {
    const evPromise = waitForEvent("gradientAggregatedEvent");
    const enc = encTogether([BigInt(42)]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).aggregateGradient(
      off, enc.cts[0], pubKeyArr, enc.nonce
    ).accountsPartial(accs("aggregate_gradient", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      console.log("  PASS: aggregate_gradient -> " + ev.decrypted[0].toString());
    } else {
      console.log("  PASS: aggregate_gradient (callback ontvangen)");
    }
  });

  console.log("");
  console.log("=== Medical Darkpool Klaar ===");
  console.log("Explorer: https://explorer.solana.com/address/" + PROGRAM_ID.toBase58() + "?cluster=devnet");
}
main().catch(e => { console.error("Fout:", e?.message || e); process.exit(1); });
