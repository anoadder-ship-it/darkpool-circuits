import { uploadCircuit, getArciumProgram, getCompDefAccOffset,
         getArciumAccountBaseSeed, getArciumProgramId } from "@arcium-hq/client";
import {
  Connection, Keypair, PublicKey, Transaction, VersionedTransaction,
} from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";

const HELIUS = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const DELAY_MS = 500;

const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

async function main() {
  const connection = new Connection(HELIUS, { commitment: "confirmed" });
  const keypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(
      fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString()
    ))
  );

  let txCount = 0;

  const sendAndConfirm = async (
    tx: Transaction | VersionedTransaction,
    signers: Keypair[] = [],
    _opts: any = {}
  ): Promise<string> => {
    await sleep(DELAY_MS);
    txCount++;
    if (txCount % 20 === 0) console.log(`  ${txCount} txs verzonden...`);

    for (let attempt = 0; attempt < 8; attempt++) {
      try {
        const { blockhash, lastValidBlockHeight } =
          await connection.getLatestBlockhash("confirmed");

        let raw: Buffer;
        const isVersioned = (tx as any).message?.version !== undefined
          || typeof (tx as VersionedTransaction).serialize === 'function'
          && (tx as any).signatures?.length > 0
          && !('recentBlockhash' in tx);

        if ('message' in tx && 'version' in (tx as any).message) {
          // VersionedTransaction
          const vTx = tx as VersionedTransaction;
          (vTx.message as any).recentBlockhash = blockhash;
          vTx.sign([keypair, ...signers]);
          raw = Buffer.from(vTx.serialize());
        } else {
          // Legacy Transaction
          const legacyTx = tx as Transaction;
          legacyTx.recentBlockhash = blockhash;
          legacyTx.lastValidBlockHeight = lastValidBlockHeight;
          legacyTx.feePayer = keypair.publicKey;
          legacyTx.signatures = [];
          legacyTx.partialSign(keypair);
          for (const s of signers) legacyTx.partialSign(s);
          raw = legacyTx.serialize();
        }

        const sig = await connection.sendRawTransaction(raw, { skipPreflight: true });
        await connection.confirmTransaction(
          { signature: sig, blockhash, lastValidBlockHeight },
          "confirmed"
        );
        return sig;

      } catch (e: any) {
        const msg = e?.message || String(e) || "geen bericht";
        if (msg.includes("429") || msg.includes("Too Many")) {
          const wait = DELAY_MS * Math.pow(2, attempt);
          console.log(`  429 — wacht ${wait}ms (poging ${attempt + 1}/8)`);
          await sleep(wait);
        } else if (msg.includes("already been processed") || msg.includes("Blockhash not found")) {
          console.log(`  Transactie conflict — opnieuw proberen`);
          await sleep(1000);
        } else {
          throw new Error(`TX fout: ${JSON.stringify(e, null, 2).slice(0, 500)}`);
        }
      }
    }
    throw new Error("Max pogingen bereikt");
  };

  const provider = {
    connection,
    publicKey: keypair.publicKey,
    wallet: { publicKey: keypair.publicKey },
    sendAndConfirm,
  };

  const programId = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");

  for (const ixName of ["place_order", "match_orders"]) {
    console.log(`\n### ${ixName} uploaden...`);
    txCount = 0;
    try {
      const rawCircuit = fs.readFileSync(`build/${ixName}.arcis`);
      await uploadCircuit(
        provider as any, ixName, programId, rawCircuit,
        true, 300,
        { commitment: "confirmed", skipPreflight: true },
      );
      console.log(`${ixName}: OK (${txCount} txs)`);
    } catch (e: any) {
      console.log(`${ixName} fout:`, e?.message || e);
    }
  }
  console.log("\n=== Klaar ===");
}

main().catch(e => {
  console.error("Fatale fout:", e?.message || e);
  process.exit(1);
});
