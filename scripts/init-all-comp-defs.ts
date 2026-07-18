import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey, ComputeBudgetProgram, Connection, Keypair, Transaction } from "@solana/web3.js";
import { SolanaDarkpool } from "../target/types/solana_darkpool";
import {
  getArciumAccountBaseSeed, getArciumProgramId, getArciumProgram,
  getCompDefAccOffset, getMXEAccAddress, getLookupTableAddress,
} from "@arcium-hq/client";
import * as fs from "fs";
import * as os from "os";

const HELIUS = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

async function sendTx(connection: Connection, tx: Transaction, signers: Keypair[]): Promise<string> {
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;
  tx.feePayer = signers[0].publicKey;
  for (const s of signers) tx.partialSign(s);
  const sig = await connection.sendRawTransaction(tx.serialize(), { skipPreflight: true });
  for (let i = 0; i < 60; i++) {
    const s = await connection.getSignatureStatus(sig);
    const cs = s.value?.confirmationStatus;
    if (cs === "confirmed" || cs === "finalized") return sig;
    if (s.value?.err) throw new Error(JSON.stringify(s.value.err));
    await sleep(1000);
  }
  throw new Error("Timeout");
}

async function main() {
  const connection = new Connection(HELIUS, { commitment: "confirmed" });
  const owner = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString()))
  );
  const wallet = new anchor.Wallet(owner);
  const provider = new anchor.AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);
  const program = anchor.workspace.SolanaDarkpool as Program<SolanaDarkpool>;
  const arciumProgram = getArciumProgram(provider);
  const mxeAccount = getMXEAccAddress(program.programId);
  const mxeAcc = await arciumProgram.account.mxeAccount.fetch(mxeAccount);
  const lutAddress = getLookupTableAddress(program.programId, mxeAcc.lutOffsetSlot);

  const defs = [
    ["place_order",  "initPlaceOrderCompDef"],
    ["match_orders", "initMatchOrdersCompDef"],
    ["cancel_order", "initCancelOrderCompDef"],
    ["get_stats",    "initGetStatsCompDef"],
  ] as const;

  for (const [ixName, methodName] of defs) {
    console.log(`\n### ${ixName}...`);
    try {
      const baseSeed = getArciumAccountBaseSeed("ComputationDefinitionAccount");
      const offset = getCompDefAccOffset(ixName);
      const compDefPDA = PublicKey.findProgramAddressSync(
        [baseSeed, program.programId.toBuffer(), offset],
        getArciumProgramId(),
      )[0];

      const tx: Transaction = await (program.methods as any)[methodName]()
        .accounts({ compDefAccount: compDefPDA, payer: owner.publicKey, mxeAccount, addressLookupTable: lutAddress })
        .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
        .transaction();

      const sig = await sendTx(connection, tx, [owner]);
      console.log(`OK: ${sig}`);
      await sleep(2000);
    } catch (e: any) {
      console.log(`Fout: ${e.message?.slice(0, 200)}`);
    }
  }
  console.log("\n=== Klaar ===");
}
main().catch(console.error);
