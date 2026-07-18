import * as anchor from "@anchor-lang/core";
import { PublicKey, Connection } from "@solana/web3.js";
import { RescueCipher, x25519 } from "@arcium-hq/client";
import * as fs from "fs";
import * as os from "os";

const PROGRAM_ID = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");
const HELIUS     = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";

async function main() {
  const connection = new Connection(HELIUS, { commitment: "confirmed" });
  
  // Wallet opbouwen met Keypair
  const sk       = Buffer.from(JSON.parse(fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString()));
  const owner    = anchor.web3.Keypair.fromSecretKey(sk);
  const wallet   = new anchor.Wallet(owner);
  
  console.log("Wallet:", owner.publicKey.toBase58());
  console.log("Program:", PROGRAM_ID.toBase58());

  // MXE pubkey ophalen via Arcium client
  const { getMXEPublicKey } = require("@arcium-hq/client");
  const provider = new anchor.AnchorProvider(connection, wallet, {});
  const mxePubkey = await getMXEPublicKey(provider, PROGRAM_ID);
  
  if (!mxePubkey) throw new Error("MXE pubkey niet gevonden");
  console.log("\nMXE pubkey:", Buffer.from(mxePubkey).toString('hex'));

  // Genereer eigen sleutelpaar
  const privKey = x25519.utils.randomSecretKey();
  const pubKey  = x25519.getPublicKey(privKey);
  
  console.log("\nEigen pubkey:", Buffer.from(pubKey).toString('hex'));

  // Shared secret + encryptie
  const sharedSecret = x25519.getSharedSecret(privKey, mxePubkey);
  const cipher       = new RescueCipher(sharedSecret);
  
  console.log("\n=== Encryptie test ===");
  const nonce = Buffer.alloc(16, 0x42);
  
  // place_order: [bid, size, is_buy]
  const ct_bid    = cipher.encrypt([BigInt(100)],     nonce);
  const ct_size   = cipher.encrypt([BigInt(10)],      nonce);
  const ct_isbuy  = cipher.encrypt([BigInt(1)],       nonce);
  
  console.log("Ciphertext bid:   ", Buffer.from(ct_bid[0]).toString('hex'));
  console.log("Ciphertext size:  ", Buffer.from(ct_size[0]).toString('hex'));
  console.log("Ciphertext is_buy:", Buffer.from(ct_isbuy[0]).toString('hex'));

  // Test batch encryptie
  const ct_batch = cipher.encrypt([BigInt(100), BigInt(10), BigInt(1)], nonce);
  console.log("\nBatch ciphertexts:");
  for (let i = 0; i < ct_batch.length; i++) {
    console.log(`  [${i}]:`, Buffer.from(ct_batch[i]).toString('hex'));
  }

  // Decryptie verifiëren
  const pt = cipher.decrypt(ct_batch, nonce);
  console.log("Decrypted batch:", pt.join(', '));

  // Format check - wat Rust verwacht: [u8;32]
  console.log("\n=== Argument format check ===");
  
  const arr32_bid    = Array.from(ct_bid[0]);
  const arr32_size   = Array.from(ct_size[0]);
  const arr32_isbuy  = Array.from(ct_isbuy[0]);
  
  console.log("arr32 length:", arr32_bid.length);
  console.log("arr32 type:", typeof arr32_bid[0], "value first4:", arr32_bid.slice(0,4).map(n => n.toString(16)));
  
  const pkArr = Array.from(pubKey);
  console.log("\npkArr length:", pkArr.length);
  console.log("pkArr type:", typeof pkArr[0], "value first8:", pkArr.slice(0,8).map(n => n.toString(16)));

  // Account check
  const { getMXEAccAddress } = require("@arcium-hq/client/pda.js");
  const mxeAddr = getMXEAccAddress(PROGRAM_ID);
  const accInfo = await connection.getAccountInfo(mxeAddr);
  console.log("\nMXE account:", accInfo ? `${accInfo.data.length} bytes` : "NIET GEVONDEN");

  // CompDef check
  const { getCompDefAccOffset, getCompDefAccAddress } = require("@arcium-hq/client/pda.js");
  for (const ix of ['place_order', 'match_orders']) {
    try {
      const offset   = getCompDefAccOffset(ix);
      const pdaBytes = Buffer.from(offset).readUInt32LE();
      const addr     = getCompDefAccAddress(PROGRAM_ID, pdaBytes);
      const cInfo    = await connection.getAccountInfo(addr);
      console.log(`CompDef[${ix}]:`, cInfo ? `BESTAAT (${cInfo.data.length} bytes)` : "NIET GEVONDEN");
    } catch(e) {
      console.log(`CompDef[${ix}]: FOUT - ${e.message}`);
    }
  }

  console.log("\n=== Klaar ===");
}

main().catch(e => { console.error("Fout:", e?.message || e); process.exit(1); });
