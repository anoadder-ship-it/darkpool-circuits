import { uploadCircuit, getCircuitState, getArciumProgram,
         getArciumAccountBaseSeed, getArciumProgramId,
         getCompDefAccOffset } from "@arcium-hq/client";
import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";
import * as https from "https";

const HELIUS = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const DELAY = 800;
const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

async function fetchBase64(url: string): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    https.get(url, res => {
      const chunks: Buffer[] = [];
      res.on("data", c => chunks.push(c));
      res.on("end", () => resolve(Buffer.from(Buffer.concat(chunks).toString(), "base64")));
      res.on("error", reject);
    });
  });
}

async function main() {
  const connection = new Connection(HELIUS, { commitment: "confirmed" });
  const keypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString()))
  );
  const programId = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");

  const sendAndConfirm = async (tx: Transaction, signers: Keypair[] = []): Promise<string> => {
    await sleep(DELAY);
    for (let i = 0; i < 10; i++) {
      try {
        const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
        tx.recentBlockhash = blockhash;
        tx.lastValidBlockHeight = lastValidBlockHeight;
        tx.feePayer = keypair.publicKey;
        tx.signatures = [];
        tx.partialSign(keypair);
        for (const s of signers) tx.partialSign(s);
        const sig = await connection.sendRawTransaction(tx.serialize(), { skipPreflight: true });
        await connection.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, "confirmed");
        return sig;
      } catch (e: any) {
        if (e?.message?.includes("429") || e?.message?.includes("Too Many")) {
          const wait = DELAY * Math.pow(2, Math.min(i, 4));
          console.log(`  429 — wacht ${wait}ms`);
          await sleep(wait);
        } else if (e?.message?.includes("already been processed")) {
          return "already-processed";
        } else {
          throw e;
        }
      }
    }
    throw new Error("Max pogingen");
  };

  const provider = { connection, publicKey: keypair.publicKey,
    wallet: { publicKey: keypair.publicKey }, sendAndConfirm };

  const circuits: Record<string, string> = {
    "place_order": "https://gist.githubusercontent.com/anoadder-ship-it/d0054c8b3de6ef1b88dbc498eb9200bd/raw/ed06aecc6053b8a4df887c46b0ab17e80207aad2/place_order.arcis.b64",
    "match_orders": "https://gist.githubusercontent.com/anoadder-ship-it/d0054c8b3de6ef1b88dbc498eb9200bd/raw/32d0ebed46931b56098d936db16498d1680785b1/match_orders.arcis.b64",
  };

  for (const [ixName, url] of Object.entries(circuits)) {
    console.log(`\n### ${ixName} downloaden van Gist...`);
    const rawCircuit = await fetchBase64(url);
    console.log(`  Downloaded: ${rawCircuit.length} bytes`);

    console.log(`### ${ixName} uploaden naar devnet...`);
    try {
      await uploadCircuit(provider as any, ixName, programId, rawCircuit,
        true, 200, { commitment: "confirmed", skipPreflight: true });
      console.log(`${ixName}: OK`);
    } catch (e: any) {
      console.log(`${ixName} fout:`, JSON.stringify(e)?.slice(0, 200) || e?.message);
    }
  }
  console.log("\n=== Klaar ===");
}

main().catch(console.error);
