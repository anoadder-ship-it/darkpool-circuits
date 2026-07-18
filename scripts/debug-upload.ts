import * as anchor from "@anchor-lang/core";
import { uploadCircuit, getCircuitState, getCompDefAccOffset,
         getArciumProgram, getArciumAccountBaseSeed, getArciumProgramId,
         getMXEAccAddress } from "@arcium-hq/client";
import { PublicKey } from "@solana/web3.js";
import * as fs from "fs";

async function main() {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const programId = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");
  const arciumProgram = getArciumProgram(provider);

  for (const ixName of ["place_order", "match_orders"]) {
    const baseSeed = getArciumAccountBaseSeed("ComputationDefinitionAccount");
    const offset = getCompDefAccOffset(ixName);
    const compDefPDA = PublicKey.findProgramAddressSync(
      [baseSeed, programId.toBuffer(), offset],
      getArciumProgramId(),
    )[0];

    console.log(`\n### ${ixName} — compDefPDA: ${compDefPDA.toBase58()}`);
    try {
      const compDefAcc = await arciumProgram.account.computationDefinitionAccount.fetch(compDefPDA);
      console.log(`compDefAcc ophalen: OK`);
      console.log(`Circuit state:`, getCircuitState(compDefAcc as any));
    } catch (e: any) {
      console.log(`compDefAcc fout: ${e.message}`);
    }

    console.log(`Circuit uploaden...`);
    try {
      const rawCircuit = fs.readFileSync(`build/${ixName}.arcis`);
      await uploadCircuit(provider, ixName, programId, rawCircuit, true, 500,
        { skipPreflight: true, commitment: "confirmed" });
      console.log(`${ixName}: OK`);
    } catch (e: any) {
      console.log(`Upload fout (volledig):`, e);
    }
  }
}

main().catch(console.error);
