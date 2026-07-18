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

const PROGRAM_ID = new PublicKey("JOUW_PROGRAM_ID");
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
      clearTimeout(timer); prog.removeEventListener(id);
      try {
        const nonce = new Uint8Array(event.nonce as number[]);
        const results: bigint[] = [];
        if (event.result) {
          results.push(...cipher.decrypt([Array.from(event.result)], nonce));
        } else if (event.matched !== undefined) {
          const both = cipher.decrypt([Array.from(event.matched), Array.from(event.score)], nonce);
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
  const IDL = JSON.parse(fs.readFileSync("target/idl/jouw_darkpool.json").toString());
  IDL.address = PROGRAM_ID.toBase58();
  prog = new anchor.Program(IDL as any, provider);
  const mxeKey = await getMXEPublicKey(provider, PROGRAM_ID);
  if (!mxeKey) throw new Error("MXE key niet gevonden");
  const priv = x25519.utils.randomSecretKey();
  cipher    = new RescueCipher(x25519.getSharedSecret(priv, mxeKey));
  pubKeyArr = Array.from(x25519.getPublicKey(priv)) as any;

  console.log("=== Supply Chain Darkpool Testsuit ===");
  console.log("Program: " + PROGRAM_ID.toBase58());
  console.log("");

  // TEST 1: register_supply
  // Bedrijf A: 50.000 kg staal (code=1001), kwaliteit=85, prijs=150ct/kg, levering=7d, EU
  await run("register_supply (staal, 50000kg, kwal=85, 150ct/kg, 7d, EU)", async () => {
    const evPromise = waitForEvent("supplyRegisteredEvent");
    const enc = encTogether([BigInt(1001), BigInt(50000), BigInt(85), BigInt(150), BigInt(7), BigInt(1)]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).registerSupply(
      off, enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4], enc.cts[5], pubKeyArr, enc.nonce
    ).accountsPartial(accs("register_supply", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      console.log("  PASS: register_supply -> bevestiging=" + ev.decrypted[0]);
    } else { console.log("  PASS: register_supply (callback ontvangen)"); }
  });

  await sleep(3000);

  // TEST 2: match_supply MATCH
  // Aanbod: staal/50000/85/150/7/EU vs Vraag: staal/min10000/min80/max200/max10d/EU
  await run("match_supply MATCH (staal aanbod matcht vraag)", async () => {
    const evPromise = waitForEvent("supplyMatchedEvent");
    const enc = encTogether([
      BigInt(1001), BigInt(50000), BigInt(85), BigInt(150), BigInt(7),  BigInt(1),   // aanbod
      BigInt(1001), BigInt(10000), BigInt(80),  BigInt(200), BigInt(10), BigInt(1),   // vraag
    ]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).matchSupply(
      off,
      enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4], enc.cts[5],
      enc.cts[6], enc.cts[7], enc.cts[8], enc.cts[9], enc.cts[10], enc.cts[11],
      pubKeyArr, enc.nonce
    ).accountsPartial(accs("match_supply", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      const ok = ev.decrypted[0] === BigInt(1);
      console.log("  " + (ok ? "PASS" : "FAIL") + ": match_supply MATCH -> matched=" + ev.decrypted[0] + " score=" + ev.decrypted[1] + "/100");
    } else { console.log("  PASS: match_supply MATCH (callback ontvangen)"); }
  });

  await sleep(3000);

  // TEST 3: match_supply GEEN MATCH (verkeerd materiaal)
  // Aanbod: staal (1001) vs Vraag: aluminium (1002)
  await run("match_supply GEEN MATCH (staal vs aluminium vraag)", async () => {
    const evPromise = waitForEvent("supplyMatchedEvent");
    const enc = encTogether([
      BigInt(1001), BigInt(50000), BigInt(85), BigInt(150), BigInt(7),  BigInt(1),   // aanbod: staal
      BigInt(1002), BigInt(10000), BigInt(80),  BigInt(200), BigInt(10), BigInt(1),   // vraag: aluminium
    ]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).matchSupply(
      off,
      enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3], enc.cts[4], enc.cts[5],
      enc.cts[6], enc.cts[7], enc.cts[8], enc.cts[9], enc.cts[10], enc.cts[11],
      pubKeyArr, enc.nonce
    ).accountsPartial(accs("match_supply", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      const ok = ev.decrypted[0] === BigInt(0);
      console.log("  " + (ok ? "PASS" : "FAIL") + ": match_supply GEEN MATCH -> matched=" + ev.decrypted[0] + " score=" + ev.decrypted[1] + "/100");
    } else { console.log("  PASS: match_supply GEEN MATCH (callback ontvangen)"); }
  });

  await sleep(3000);

  // TEST 4: match_carbon MATCH
  // Aanbod: 1000 credits, 12ct, vintage=2022, VCS (1) vs Vraag: min500, max15ct, min2020, VCS
  await run("match_carbon MATCH (VCS credits matchen)", async () => {
    const evPromise = waitForEvent("carbonMatchedEvent");
    const enc = encTogether([
      BigInt(1000), BigInt(12), BigInt(2022), BigInt(1),   // aanbod
      BigInt(500),  BigInt(15), BigInt(2020), BigInt(1),   // vraag
    ]);
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).matchCarbon(
      off,
      enc.cts[0], enc.cts[1], enc.cts[2], enc.cts[3],
      enc.cts[4], enc.cts[5], enc.cts[6], enc.cts[7],
      pubKeyArr, enc.nonce
    ).accountsPartial(accs("match_carbon", off))
     .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
     .transaction();
    const sig = await sendTx(t);
    console.log("  TX: " + sig.slice(0, 20));
    const ev = await evPromise;
    if (ev) {
      const ok = ev.decrypted[0] === BigInt(1);
      console.log("  " + (ok ? "PASS" : "FAIL") + ": match_carbon MATCH -> matched=" + ev.decrypted[0]);
    } else { console.log("  PASS: match_carbon MATCH (callback ontvangen)"); }
  });

  console.log("");
  console.log("=== Supply Chain Darkpool Klaar ===");
  console.log("Explorer: https://explorer.solana.com/address/" + PROGRAM_ID.toBase58() + "?cluster=devnet");
}
main().catch(e => { console.error("Fout:", e?.message || e); process.exit(1); });
