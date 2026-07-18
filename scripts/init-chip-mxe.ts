import * as anchor from "@anchor-lang/core";
import { BN } from "@anchor-lang/core";
import { PublicKey, Connection } from "@solana/web3.js";
import {
  initMxePart2,
} from "@arcium-hq/client";
import * as fs from "fs";
import * as os from "os";

const CHIP_PROG = new PublicKey("6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o");
const RPC_URL = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const CLUSTER_OFFSET = 456;

function loadKeypair() {
  const home = os.homedir();
  const path = `${home}/.config/solana/id.json`;
  const secret = JSON.parse(fs.readFileSync(path, "utf8"));
  return anchor.web3.Keypair.fromSecretKey(Uint8Array.from(secret));
}

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");
  const wallet = loadKeypair();
  
  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  );

  console.log("🔑 Wallet:", wallet.publicKey.toBase58());
  console.log("📦 Running initMxePart2 for CHIP...");

  const keygenOffset       = new BN(Math.floor(Math.random() * 900000000) + 100000);
  const keyRecoveryOffset  = new BN(Math.floor(Math.random() * 900000000) + 1000000);
  const lutOffset          = new BN(473951446);
  const recoveryPeers      = new Array(100).fill(0);

  console.log("keygenOffset     :", keygenOffset.toString());
  console.log("keyRecoveryOffset:", keyRecoveryOffset.toString());
  console.log("lutOffset        :", lutOffset.toString());

  try {
    const signature = await initMxePart2(
      provider,
      CLUSTER_OFFSET,
      CHIP_PROG,
      recoveryPeers,
      keygenOffset,
      keyRecoveryOffset,
      lutOffset
    );

    console.log("\n✅ SUCCESS! Signature:", signature);
    console.log("🎉 CHIP MXE is initialized!");
  } catch (e: any) {
    console.error("\n❌ Error:", e.message);
    if (e.logs) console.log("Logs:", e.logs);
  }
}

main().catch(console.error);
