import { uploadCircuit } from "@arcium-hq/client";
import {
  Connection, Keypair, PublicKey, Transaction,
} from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";

const HELIUS = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const DELAY_MS = 300; // 300ms tussen transacties = max ~3 TPS

async function sleep(ms: number) {
  return new Promise(r => setTimeout(r, ms));
}

async function main() {
  const connection = new Connection(HELIUS, { commitment: "confirmed" });
  const keypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(
      fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString()
    ))
  );

  // Rate-limited provider: serialiseert alle sends met delay
  const queue: Array<() => Promise<void>> = [];
  let processing = false;

  const processQueue = async () => {
    processing = true;
    while (queue.length > 0) {
      const fn = queue.shift()!;
      try { await fn(); } catch (e) { /* individuele fout, doorgaan */ }
      await sleep(DELAY_MS);
    }
    processing = false;
  };

  const rateLimitedSendAndConfirm = async (
    tx: Transaction,
    signers: Keypair[] = [],
    _opts: any = {}
  ): Promise<string> => {
    return new Promise((resolve, reject) => {
      queue.push(async () => {
        let attempts = 0;
        while (attempts < 5) {
          try {
            const { blockhash, lastValidBlockHeight } =
              await connection.getLatestBlockhash("confirmed");
            tx.recentBlockhash = blockhash;
            tx.lastValidBlockHeight = lastValidBlockHeight;
            tx.feePayer = keypair.publicKey;
            tx.partialSign(keypair);
            for (const s of signers) tx.partialSign(s);
            const raw = tx.serialize();
            const sig = await connection.sendRawTransaction(raw, {
              skipPreflight: true,
            });
            await connection.confirmTransaction(
              { signature: sig, blockhash, lastValidBlockHeight },
              "confirmed"
            );
            resolve(sig);
            return;
          } catch (e: any) {
            if (e.message?.includes("429")) {
              attempts++;
              console.log(`  429 — wacht ${DELAY_MS * attempts}ms, poging ${attempts}/5`);
              await sleep(DELAY_MS * attempts * 2);
            } else {
              reject(e);
              return;
            }
          }
        }
        reject(new Error("Max pogingen bereikt"));
      });
      if (!processing) processQueue();
    });
  };

  const provider = {
    connection,
    publicKey: keypair.publicKey,
    wallet: { publicKey: keypair.publicKey },
    sendAndConfirm: rateLimitedSendAndConfirm,
  };

  const programId = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");

  for (const ixName of ["place_order", "match_orders"]) {
    console.log(`\n### ${ixName} uploaden (geserialiseerd, ${DELAY_MS}ms vertraging)...`);
    try {
      const rawCircuit = fs.readFileSync(`build/${ixName}.arcis`);
      await uploadCircuit(
        provider as any,
        ixName,
        programId,
        rawCircuit,
        true,
        300,
        { commitment: "confirmed", skipPreflight: true },
      );
      console.log(`${ixName}: OK`);
    } catch (e: any) {
      console.log(`${ixName} fout:`, e.message?.slice(0, 200));
    }
  }

  console.log("\n=== Klaar ===");
}

main().catch(console.error);
