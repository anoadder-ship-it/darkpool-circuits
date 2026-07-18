import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey, ComputeBudgetProgram } from "@solana/web3.js";
import { SolanaDarkpool } from "../target/types/solana_darkpool";
import {
  getArciumAccountBaseSeed,
  getArciumProgramId,
  getArciumProgram,
  getCompDefAccOffset,
  getMXEAccAddress,
  getLookupTableAddress,
  uploadCircuit,
} from "@arcium-hq/client";
import * as fs from "fs";
import * as os from "os";

async function main() {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.SolanaDarkpool as Program<SolanaDarkpool>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const arciumProgram = getArciumProgram(provider);

  const owner = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(
      fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString()
    ))
  );

  const mxeAccount = getMXEAccAddress(program.programId);
  const mxeAcc = await arciumProgram.account.mxeAccount.fetch(mxeAccount);
  const lutAddress = getLookupTableAddress(program.programId, mxeAcc.lutOffsetSlot);

  // ── Stap 1: match_orders comp def initialiseren ──────────────────────────
  console.log("### match_orders comp def initialiseren...");
  const baseSeed = getArciumAccountBaseSeed("ComputationDefinitionAccount");
  const offset = getCompDefAccOffset("match_orders");
  const compDefPDA = PublicKey.findProgramAddressSync(
    [baseSeed, program.programId.toBuffer(), offset],
    getArciumProgramId(),
  )[0];

  try {
    const sig = await (program.methods as any)
      .initMatchOrdersCompDef()
      .accounts({
        compDefAccount: compDefPDA,
        payer: owner.publicKey,
        mxeAccount,
        addressLookupTable: lutAddress,
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 }),
      ])
      .signers([owner])
      .rpc({ skipPreflight: true, commitment: "confirmed" });
    console.log("match_orders comp def OK:", sig);
  } catch (e: any) {
    console.log("match_orders comp def:", e.message.slice(0, 100));
  }

  // ── Stap 2: circuits uploaden (overwrite: false) ─────────────────────────
  for (const ixName of ["place_order", "match_orders"]) {
    console.log(`\n### ${ixName} circuit uploaden (overwrite: false)...`);
    try {
      const rawCircuit = fs.readFileSync(`build/${ixName}.arcis`);
      await uploadCircuit(
        provider, ixName, program.programId, rawCircuit,
        false, 500,
        { skipPreflight: true, commitment: "confirmed" },
      );
      console.log(`${ixName}: OK`);
    } catch (e: any) {
      console.log(`${ixName} fout: ${e.message.slice(0, 150)}`);
    }
  }

  console.log("\n=== Klaar ===");
}

main().catch(console.error);
