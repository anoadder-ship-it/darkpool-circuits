import * as coralAnchor from "@coral-xyz/anchor";
import { uploadCircuit } from "@arcium-hq/client";
import * as fs from "fs";
import * as os from "os";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

async function main() {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const keypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(
      fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString()
    ))
  );

  const wallet = new coralAnchor.Wallet(keypair);
  const provider = new coralAnchor.AnchorProvider(connection, wallet, {
    skipPreflight: true,
    commitment: "confirmed",
  });

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
        { skipPreflight: true, commitment: "confirmed" },
      );
      console.log(`${ixName}: OK`);
    } catch (e: any) {
      console.log(`${ixName} fout:`, e.message?.slice(0, 200));
    }
  }
  console.log("\n=== Klaar ===");
}

main().catch(console.error);
