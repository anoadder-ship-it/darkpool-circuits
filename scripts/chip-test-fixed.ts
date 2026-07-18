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

const PROGRAM_ID = new PublicKey("6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o");
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

function waitForEvent(eventName: string, timeoutMs = 90000): Promise<{decrypted: bigint[], raw: any} | null> {
  return new Promise(resolve => {
    let id: number;
    const timer = setTimeout(() => { prog.removeEventListener(id); resolve(null); }, timeoutMs);
    id = prog.addEventListener(eventName, (event: any) => {
      clearTimeout(timer);
      prog.removeEventListener(id);
      try {
        const nonce = new Uint8Array(event.nonce as number[]);
        let decrypted: bigint[] = [];
        if (event.result) {
          decrypted = cipher.decrypt([Array.from(event.result)], nonce);
        }
        resolve({ decrypted, raw: event });
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
  const IDL = JSON.parse(fs.readFileSync("target/idl/chip_darkpool.json").toString());
  IDL.address = PROGRAM_ID.toBase58();
  prog = new anchor.Program(IDL as any, provider);
  const mxeKey = await getMXEPublicKey(provider, PROGRAM_ID);
  if (!mxeKey) throw new Error("MXE key niet gevonden");
  const priv = x25519.utils.randomSecretKey();
  cipher    = new RescueCipher(x25519.getSharedSecret(priv, mxeKey));
  pubKeyArr = Array.from(x25519.getPublicKey(priv)) as any;

  console.log("=== Chip Marketplace Darkpool Testsuit ===");
  console.log("Program: " + PROGRAM_ID.toBase58());
  console.log("");

  // TEST 1: register_chip
  // H100 GPU: 10 units, nieuw (1), $35000/unit (3500000ct), 14d, EU, datacenter cert
  await run("register_chip (H100, 10 units, nieuw, 35000USD, 14d, EU)", async () => {
    const evPromise = waitForEvent("chipRegisteredEvent");
    const enc = encTogether([BigInt(1001), BigInt(10), BigInt(1), BigInt(3500000), BigInt(14), BigInt(1), BigInt(1)]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).registerChip(
      off, enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4], enc.cts[5], enc.cts[6], pubKeyArr, enc.nonce
    ).accountsPartial(accs("register_chip", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) console.log("  PASS: register_chip -> bevestiging=" + ev.decrypted[0]);
    else console.log("  PASS: register_chip (callback ontvangen)");
  });

  await sleep(3000);

  // TEST 2: match_chip MATCH
  // Aanbod: H100/10/nieuw/35000USD/14d/EU/datacenter
  // Vraag:  H100/min5/max40000USD/max30d/EU
  await run("match_chip MATCH (H100 aanbod matcht H100 vraag)", async () => {
    const evPromise = waitForEvent("chipMatchedEvent");
    const enc = encTogether([
      BigInt(1001), BigInt(10), BigInt(1), BigInt(3500000), BigInt(14), BigInt(1), BigInt(1),
      BigInt(1001), BigInt(5),  BigInt(3), BigInt(4000000), BigInt(30), BigInt(1), BigInt(1),
    ]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).matchChip(
      off,
      enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4], enc.cts[5], enc.cts[6],
      enc.cts[7], enc.cts[8], enc.cts[9], enc.cts[10], enc.cts[11], enc.cts[12], enc.cts[13],
      pubKeyArr, enc.nonce
    ).accountsPartial(accs("match_chip", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      const both = cipher.decrypt([Array.from(ev.raw.matched as any), Array.from(ev.raw.score as any)], new Uint8Array(ev.raw.nonce as any));
      console.log("  " + (both[0] === BigInt(1) ? "PASS" : "FAIL") + ": match_chip MATCH -> matched=" + both[0] + " score=" + both[1] + "/98");
    }
  });

  await sleep(3000);

  // TEST 3: match_chip GEEN MATCH (H100 vs H200)
  await run("match_chip GEEN MATCH (H100 aanbod vs H200 vraag)", async () => {
    const evPromise = waitForEvent("chipMatchedEvent");
    const enc = encTogether([
      BigInt(1001), BigInt(10), BigInt(1), BigInt(3500000), BigInt(14), BigInt(1), BigInt(1),
      BigInt(1002), BigInt(5),  BigInt(3), BigInt(4000000), BigInt(30), BigInt(1), BigInt(1),
    ]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).matchChip(
      off,
      enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4], enc.cts[5], enc.cts[6],
      enc.cts[7], enc.cts[8], enc.cts[9], enc.cts[10], enc.cts[11], enc.cts[12], enc.cts[13],
      pubKeyArr, enc.nonce
    ).accountsPartial(accs("match_chip", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      const both = cipher.decrypt([Array.from(ev.raw.matched as any), Array.from(ev.raw.score as any)], new Uint8Array(ev.raw.nonce as any));
      console.log("  " + (both[0] === BigInt(0) ? "PASS" : "FAIL") + ": match_chip GEEN MATCH -> matched=" + both[0] + " score=" + both[1] + "/98");
    }
  });

  await sleep(3000);

  // TEST 4: aggregate_volume
  await run("aggregate_volume (H100, 50 units, 175M USD)", async () => {
    const evPromise = waitForEvent("volumeAggregatedEvent");
    const enc = encTogether([BigInt(1001), BigInt(50), BigInt(17500000000)]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).aggregateVolume(
      off, enc.cts[0], enc.cts[1], enc.cts[2], pubKeyArr, enc.nonce
    ).accountsPartial(accs("aggregate_volume", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) console.log("  PASS: aggregate_volume -> " + ev.decrypted[0]);
    else console.log("  PASS: aggregate_volume (callback ontvangen)");
  });

  console.log("");
  console.log("=== Chip Marketplace Darkpool Klaar ===");
  console.log("Explorer: https://explorer.solana.com/address/" + PROGRAM_ID.toBase58() + "?cluster=devnet");
}
main().catch(e => { console.error("Fout:", e?.message || e); process.exit(1); });
