import { uploadCircuit } from "@arcium-hq/client";
import {
  Connection, Keypair, PublicKey, Transaction,
} from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";

async function main() {
  const connection = new Connection(
    "https://api.devnet.solana.com",
    { commitment: "confirmed" }
  );
  const keypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(
      fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString()
    ))
  );

  // Custom provider die web3.js v1 confirmTransaction correct aanroept
  const customProvider = {
    connection,
    wallet: { publicKey: keypair.publicKey },
    publicKey: keypair.publicKey,
    sendAndConfirm: async (tx: Transaction, signers: Keypair[] = [], _opts: any = {}) => {
      tx.feePayer = keypair.publicKey;
      tx.partialSign(keypair);
      for (const s of signers) tx.partialSign(s);
      const raw = tx.serialize();
      const sig = await connection.sendRawTransaction(raw, { skipPreflight: true });
      await connection.confirmTransaction({
        signature: sig,
        blockhash: tx.recentBlockhash!,
        lastValidBlockHeight: tx.lastValidBlockHeight!,
      }, "confirmed");
      return sig;
    },
  };

  const programId = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");

  for (const ixName of ["place_order", "match_orders"]) {
    console.log(`\n### ${ixName} uploaden...`);
    try {
      const rawCircuit = fs.readFileSync(`build/${ixName}.arcis`);
      await uploadCircuit(
        customProvider as any,
        ixName,
        programId,
        rawCircuit,
        true,
        500,
        { commitment: "confirmed", skipPreflight: true },
      );
      console.log(`${ixName}: OK`);
    } catch (e: any) {
      console.log(`${ixName} fout:`, e.message?.slice(0, 300));
    }
  }
  console.log("\n=== Klaar ===");
}

main().catch(console.error);
