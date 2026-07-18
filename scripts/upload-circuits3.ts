import * as anchorLang from "@anchor-lang/core";
import { uploadCircuit } from "@arcium-hq/client";
import {
  Connection, Keypair, PublicKey,
  Transaction, sendAndConfirmTransaction,
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

  // Bouw een provider die sendAndConfirm correct afhandelt
  const wallet = new anchorLang.Wallet(keypair);
  const provider = new anchorLang.AnchorProvider(
    connection,
    wallet,
    { commitment: "confirmed", skipPreflight: true }
  );

  // Monkey-patch: zorg dat sendAndConfirm altijd een geldig commitment heeft
  const originalSendAndConfirm = provider.sendAndConfirm.bind(provider);
  (provider as any).sendAndConfirm = async (
    tx: Transaction,
    signers: any[],
    opts: any
  ) => {
    const safeOpts = {
      commitment: "confirmed" as const,
      skipPreflight: true,
      ...(opts || {}),
    };
    // verwijder undefined/null velden
    Object.keys(safeOpts).forEach(k => {
      if ((safeOpts as any)[k] === undefined) delete (safeOpts as any)[k];
    });
    return originalSendAndConfirm(tx, signers || [], safeOpts);
  };

  const programId = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");

  for (const ixName of ["place_order", "match_orders"]) {
    console.log(`\n### ${ixName} uploaden...`);
    try {
      const rawCircuit = fs.readFileSync(`build/${ixName}.arcis`);
      await uploadCircuit(
        provider as any,
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
