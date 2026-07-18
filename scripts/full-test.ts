
import * as anchor from "@anchor-lang/core";
import { BN } from "@anchor-lang/core";
import { PublicKey, ComputeBudgetProgram, Connection, Keypair, Transaction } from "@solana/web3.js";
import { getCompDefAccOffset, RescueCipher, deserializeLE, getMXEPublicKey, getMXEAccAddress, getMempoolAccAddress, getCompDefAccAddress, getExecutingPoolAccAddress, getComputationAccAddress, getClusterAccAddress, x25519 } from "@arcium-hq/client";
import { randomBytes } from "crypto";
import * as fs from "fs";
import * as os from "os";

const PROGRAM_ID = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");
const HELIUS     = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const CLUSTER    = 456;
const sleep      = (ms: number) => new Promise(r => setTimeout(r, ms));

let cipher: RescueCipher, pubKeyArr: any, conn: Connection, owner: Keypair, prog: any;

async function tx(t: Transaction): Promise<string> {
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
      throw new Error(`FOUT: ${JSON.stringify(s.value.err)} | ${e}`);
    }
    if (s.value?.confirmationStatus === "confirmed" || s.value?.confirmationStatus === "finalized") return sig;
  }
  throw new Error("timeout");
}

function accs(name: string, offset: BN) {
  const arciumProgramId = new anchor.web3.PublicKey("arcXp3PaEP2Tw9pueJVG33xSicD4yansduCucDtH");
  const [signPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ArciumSignerAccount")],
    arciumProgramId
  );

  return {
    payer: (prog.provider as any).wallet.publicKey,
    authority: (prog.provider as any).wallet.publicKey,
    signPdaAccount: signPda,
    arciumProgram: arciumProgramId,
    systemProgram: anchor.web3.SystemProgram.programId,
    computationAccount: getComputationAccAddress(CLUSTER, offset),
    clusterAccount: getClusterAccAddress(CLUSTER),
    mxeAccount: getMXEAccAddress(PROGRAM_ID),
    mempoolAccount: getMempoolAccAddress(CLUSTER),
    executingPool: getExecutingPoolAccAddress(CLUSTER),
    compDefAccount: getCompDefAccAddress(PROGRAM_ID, Buffer.from(getCompDefAccOffset(name)).readUInt32LE())
  };
}
  return "timeout";
}

async function run(name: string, fn: () => Promise<void>) {
  try { await fn(); console.log("  PASS:", name); }
  catch(e: any) { console.log("  FAIL:", name, "-", e.message.slice(0,100)); }
}

async function main() {
  conn  = new Connection(HELIUS, { commitment: "confirmed" });
  owner = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString())));
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: "confirmed" });
  anchor.setProvider(provider);
  const IDL = JSON.parse(fs.readFileSync("target/idl/solana_darkpool.json").toString());
  IDL.address = PROGRAM_ID.toBase58();
  prog = new anchor.Program(IDL as any, provider);
  const mxeKey = await getMXEPublicKey(provider, PROGRAM_ID);
  if (!mxeKey) throw new Error("MXE key niet gevonden");
  const priv = x25519.utils.randomSecretKey();
  const pub  = x25519.getPublicKey(priv);
  cipher    = new RescueCipher(x25519.getSharedSecret(priv, mxeKey));
  pubKeyArr = Array.from(pub) as any;

  console.log("=== Dark Pool Testsuit ===");
  console.log("Wallet:", owner.publicKey.toBase58());

  // TEST 1: place_order werkt
  await run("place_order (bid=100)", async () => {
    const b = enc(BigInt(100)), s = enc(BigInt(10)), ib = enc(BigInt(1));
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).placeOrder()
      .accountsPartial({ ...accs("place_order", off), payer: owner.publicKey, authority: owner.publicKey })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
      .transaction();
    const sig = await tx(t);
    console.log("    TX:", sig.slice(0,20));
    console.log("    Wachten op callback...");
    const cb = await waitCB(90000);
    if (cb === "timeout") throw new Error("geen callback binnen 180s");
    console.log("    Callback:", cb);
  });

  await sleep(3000);

  // TEST 2: match_orders positief (100 >= 95)
  await run("match_orders positief (buy=100, sell=95 → match)", async () => {
    const b = enc(BigInt(100)), s = enc(BigInt(95));
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).matchOrders()
      .accountsPartial({ ...accs("match_orders", off), payer: owner.publicKey, authority: owner.publicKey })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
      .transaction();
    const sig = await tx(t);
    console.log("    TX:", sig.slice(0,20));
    console.log("    Wachten op callback...");
    const cb = await waitCB(90000);
    if (cb === "timeout") throw new Error("geen callback binnen 180s");
    console.log("    Callback:", cb);
  });

  await sleep(3000);

  // TEST 3: match_orders negatief (80 < 95 → geen match)
  await run("match_orders negatief (buy=80, sell=95 → geen match)", async () => {
    const b = enc(BigInt(80)), s = enc(BigInt(95));
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).matchOrders()
      .accountsPartial({ ...accs("match_orders", off), payer: owner.publicKey, authority: owner.publicKey })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
      .transaction();
    const sig = await tx(t);
    console.log("    TX:", sig.slice(0,20));
    console.log("    Wachten op callback...");
    const cb = await waitCB(90000);
    if (cb === "timeout") throw new Error("geen callback binnen 180s");
    console.log("    Callback:", cb, "(resultaat moet 0 zijn)");
  });

  await sleep(3000);

  // TEST 4: cancel_order
  await run("cancel_order (order_id=42)", async () => {
    const o = enc(BigInt(42));
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).cancelOrder(off, o.ct, pubKeyArr, o.n)
      .accountsPartial(accs("cancel_order", off))
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
      .transaction();
    const sig = await tx(t);
    console.log("    TX:", sig.slice(0,20));
    console.log("    Wachten op callback...");
    const cb = await waitCB(90000);
    if (cb === "timeout") throw new Error("geen callback binnen 180s");
    console.log("    Callback:", cb);
  });

  await sleep(3000);

  // TEST 5: get_stats
  await run("get_stats (buy_vol=150, sell_vol=100)", async () => {
    const b = enc(BigInt(150)), s = enc(BigInt(100));
    const off = new BN(randomBytes(8), "hex");
    const t = await (prog.methods as any).getStats(off, b.ct, s.ct, pubKeyArr, b.n)
      .accountsPartial(accs("get_stats", off))
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
      .transaction();
    const sig = await tx(t);
    console.log("    TX:", sig.slice(0,20));
    console.log("    Wachten op callback...");
    const cb = await waitCB(90000);
    if (cb === "timeout") throw new Error("geen callback binnen 180s");
    console.log("    Callback:", cb);
  });

  console.log("\n=== Testsuit klaar ===");
  console.log(`Explorer: https://explorer.solana.com/address/${PROGRAM_ID.toBase58()}?cluster=devnet`);
}
main().catch(e => { console.error("Fout:", e?.message || e); process.exit(1); });
