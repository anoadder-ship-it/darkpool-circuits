import * as anchor from "@anchor-lang/core";
import { uploadCircuit } from "@arcium-hq/client";
import * as fs from "fs";

async function main() {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const programId = new anchor.web3.PublicKey(
    "h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX"
  );

  for (const ixName of ["place_order", "match_orders"]) {
    console.log(`\n### ${ixName} circuit uploaden...`);
    try {
      const rawCircuit = fs.readFileSync(`build/${ixName}.arcis`);
      await uploadCircuit(
        provider,
        ixName,
        programId,
        rawCircuit,
        true,
        500,
        { skipPreflight: true, commitment: "confirmed" },
      );
      console.log(`${ixName}: OK`);
    } catch (e: any) {
      console.log(`${ixName} fout:`, e.message);
      console.log("Opnieuw proberen...");
      try {
        const rawCircuit = fs.readFileSync(`build/${ixName}.arcis`);
        await uploadCircuit(
          provider,
          ixName,
          programId,
          rawCircuit,
          true,
          500,
          { skipPreflight: true, commitment: "confirmed" },
        );
        console.log(`${ixName}: OK (tweede poging)`);
      } catch (e2: any) {
        console.log(`${ixName} definitief mislukt:`, e2.message);
      }
    }
  }

  console.log("\n=== Circuits geüpload ===");
}

main().catch(console.error);
