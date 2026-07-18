import * as anchor from "@anchor-lang/core";
import { BN } from "@anchor-lang/core";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";
import { getArciumProgram, getMXEAccAddress, initMxePart1, initMxePart2 } from "@arcium-hq/client";
import { randomBytes } from "crypto";
import * as fs from "fs";
import * as os from "os";

const CHIP = new PublicKey("JOUW_PROGRAM_ID_HIER");
const MED  = new PublicKey("CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4");
const RPC  = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";

async function main() {
  const conn = new Connection(RPC, "confirmed");
  const kp   = Keypair.fromSecretKey(new Uint8Array(
    JSON.parse(fs.readFileSync(os.homedir()+"/.config/solana/id.json","utf8"))
  ));
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(kp), { commitment: "confirmed" });
  anchor.setProvider(provider);
  const arc = getArciumProgram(provider);

  // Get current slot for lutOffset
  const slot = await conn.getSlot("confirmed");
  console.log("Current slot:", slot);

  // Read medical MXE for cluster info
  const medMxe = await arc.account.mxeAccount.fetch(getMXEAccAddress(MED));
  console.log("Med lutOffsetSlot:", medMxe.lutOffsetSlot?.toString());

  // Check chip MXE
  const chipMxeAddr = getMXEAccAddress(CHIP);
  const chipAcc = await conn.getAccountInfo(chipMxeAddr);
  if (chipAcc) {
    console.log("Chip MXE already exists:", chipAcc.data.length, "bytes");
    return;
  }

  // Empty recovery peers (no staking on devnet)
  const peers: number[] = [];

  // Part 1
  console.log("Part1...");
  try {
    const s1 = await initMxePart1(provider, CHIP, peers);
    console.log("Part1 OK:", s1);
  } catch(e: any) {
    if (e.message?.includes("already in use") || e.message?.includes("already exists")) {
      console.log("Part1 already done, continuing to Part2");
    } else {
      console.log("Part1 error:", e.message?.slice(0,200));
      if (e.logs) e.logs.slice(0,10).forEach((l:string) => console.log(" >", l));
      return;
    }
  }

  await new Promise(r => setTimeout(r, 5000));

  // Part 2 — v0.10.x order: keygenOffset, keyRecoveryOffset, recoveryPeers, lutOffset
  console.log("Part2...");
  const keygenOff = new BN(randomBytes(8), "hex");
  const keyRecOff = new BN(randomBytes(8), "hex");
  const lutOff    = new BN(slot);

  console.log("keygenOff:", keygenOff.toString());
  console.log("keyRecOff:", keyRecOff.toString());
  console.log("lutOff:", lutOff.toString());
  console.log("peers:", peers.length);

  try {
    const s2 = await initMxePart2(provider, 456, CHIP, keygenOff, keyRecOff, peers, lutOff);
    console.log("Part2 OK:", s2);
    console.log("SUCCESS: Chip MXE initialized");
  } catch(e: any) {
    console.log("Part2 error:", e.message?.slice(0,400));
    if (e.logs) e.logs.slice(0,15).forEach((l:string) => console.log(" >", l));
  }
}

main().catch(e => console.error("Fatal:", e?.message));
