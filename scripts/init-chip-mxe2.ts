import * as anchor from "@anchor-lang/core";
import { BN } from "@anchor-lang/core";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";
import { getArciumProgram, getMXEAccAddress, initMxePart2 } from "@arcium-hq/client";
import { randomBytes } from "crypto";
import * as fs from "fs";
import * as os from "os";

const CHIP_PROG = new PublicKey("6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o");
const MED_PROG  = new PublicKey("CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4");
const HELIUS    = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const CLUSTER   = 456;

async function main() {
  const conn    = new Connection(HELIUS, { commitment: "confirmed" });
  const owner   = Keypair.fromSecretKey(new Uint8Array(
    JSON.parse(fs.readFileSync(os.homedir() + "/.config/solana/id.json").toString())
  ));
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), { commitment: "confirmed" });
  anchor.setProvider(provider);
  const arcProg = getArciumProgram(provider);

  const medMxeAddr = getMXEAccAddress(MED_PROG);
  const medMxe     = await arcProg.account.mxeAccount.fetch(medMxeAddr);
  console.log("Medical lutOffset:", medMxe.lutOffsetSlot?.toString());

  const chipMxeAddr = getMXEAccAddress(CHIP_PROG);
  const chipExists  = await conn.getAccountInfo(chipMxeAddr);
  if (chipExists) {
    console.log("Chip MXE already exists:", chipExists.data.length, "bytes");
    return;
  }

  console.log("Running initMxePart2 (Part1 already done)...");
  try {
    const keygenOff      = new BN(randomBytes(8), "hex");
    const keyRecoveryOff = new BN(randomBytes(8), "hex");
    const lutOff         = new BN(medMxe.lutOffsetSlot?.toString() || "0");
    const recoveryPeers  = Array(100).fill(0);

    console.log("keygenOff:", keygenOff.toString());
    console.log("lutOff:", lutOff.toString());

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
    console.log("Part2 error:", e.message?.slice(0, 300));
    if (e.logs) e.logs.slice(0,10).forEach((l: string) => console.log(" log:", l));
  }
}

main().catch(e => console.error("Fatal:", e?.message));
