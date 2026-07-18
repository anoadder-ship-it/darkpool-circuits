import * as anchor from "@anchor-lang/core";
import { BN } from "@anchor-lang/core";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";
import { getArciumProgram, getMXEAccAddress, initMxePart2 } from "@arcium-hq/client";
import { randomBytes } from "crypto";
import * as fs from "fs";
import * as os from "os";

const CHIP_PROG = new PublicKey("6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o");
const HELIUS = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const CLUSTER = 456;

async function main() {
  const conn = new Connection(HELIUS, { commitment: "confirmed" });
  const owner = Keypair.fromSecretKey(new Uint8Array(
    JSON.parse(fs.readFileSync(os.homedir() + "/.config/solana/id.json").toString())
  ));
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: "confirmed" });
  anchor.setProvider(provider);

  const chipMxeAddr = getMXEAccAddress(CHIP_PROG);
  const chipExists = await conn.getAccountInfo(chipMxeAddr);
  if (chipExists) {
    console.log("Chip MXE already exists:", chipExists.data.length, "bytes");
    return;
  }

  // Get current slot for lutOffset
  const currentSlot = await conn.getSlot("confirmed");
  const lutOff = new BN(currentSlot);
  const keygenOff = new BN(randomBytes(8), "hex");
  const keyRecoveryOff = new BN(randomBytes(8), "hex");
  // Must match Part1 - empty array
  const recoveryPeers: number[] = [];

  console.log("currentSlot:", currentSlot);
  console.log("keygenOff:", keygenOff.toString());
  console.log("lutOff:", lutOff.toString());
  console.log("recoveryPeers:", recoveryPeers.length);

  console.log("Running initMxePart2...");
  try {
    const sig2 = await initMxePart2(
      provider,
      CLUSTER,
      CHIP_PROG,
      keygenOff,
      keyRecoveryOff,
      recoveryPeers,
      lutOff
    );
    console.log("Part2 OK:", sig2);
    console.log("Chip MXE initialized!");
  } catch(e: any) {
    console.log("Part2 error:", e.message?.slice(0, 400));
    if (e.logs) e.logs.slice(0,15).forEach((l: string) => console.log(" log:", l));
  }
}

main().catch(e => console.error("Fatal:", e?.message));
