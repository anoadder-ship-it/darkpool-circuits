import * as anchor from "@anchor-lang/core";
import { PublicKey, Connection, Keypair, Transaction } from "@solana/web3.js";
import { RescueCipher, x25519, getCompDefAccAddress, getMXEAccAddress, getMempoolAccAddress,
        getExecutingPoolAccAddress, getComputationAccAddress,
        getClusterAccAddress, getCompDefAccOffset } from "@arcium-hq/client";
import * as fs from "fs";
import * as os from "os";

async function main() {
    const connection = new Connection("https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0", "confirmed");
    
    const sk       = Buffer.from(JSON.parse(fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString()));
    const owner    = Keypair.fromSecretKey(sk);
    const wallet   = new anchor.Wallet(owner);
    const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });

    const idlData  = JSON.parse(fs.readFileSync("target/idl/solana_darkpool.json").toString());
    
    console.log("=== IDL instructions (place_order) ===");
    const placeOrderIx = idlData.instructions?.find((ix: any) => ix.name === "place_order");
    if (placeOrderIx) {
        console.log("\nArgs:");
        placeOrderIx.args.forEach((arg: any, i: number) => {
            const typeStr = typeof arg.type === 'string' ? arg.type : JSON.stringify(arg.type);
            console.log(`  [${i}] ${arg.name}:`, typeStr.slice(0, 60));
        });
    }

    // Get MXE pubkey
    const mxePubkey = await require("@arcium-hq/client").getMXEPublicKey(provider, new PublicKey('h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX'));

    // Encryptie
    const privKey = x25519.utils.randomSecretKey();
    const pubKey  = x25519.getPublicKey(privKey);
    const cipher  = new RescueCipher(x25519.getSharedSecret(privKey, mxePubkey));
    
    // Method A: batch encrypt (3 ciphertexts)
    console.log("\n=== Method A: Batch encrypt [100, 10, 1] ===");
    const ctBatch = cipher.encrypt([BigInt(100), BigInt(10), BigInt(1)], Buffer.alloc(16));
    
    console.log("ctBatch[0]:", Buffer.from(ctBatch[0]).toString('hex'));
    console.log("ctBatch[1]:", Buffer.from(ctBatch[1]).toString('hex'));
    console.log("ctBatch[2]:", Buffer.from(ctBatch[2]).toString('hex'));
    
    // Method B: separate encrypt (1 ciphertext each)
    console.log("\n=== Method B: Separate encrypt ===");
    const ctBid  = cipher.encrypt([BigInt(100)],     Buffer.alloc(16, 0x01))[0];
    const ctSize = cipher.encrypt([BigInt(10)],      Buffer.alloc(16, 0x02))[0];
    const ctBuy  = cipher.encrypt([BigInt(1)],       Buffer.alloc(16, 0x03))[0];

    console.log("ctBid:",   Buffer.from(ctBid).toString('hex'));
    console.log("ctSize: ", Buffer.from(ctSize).toString('hex'));
    console.log("ctBuy:  ", Buffer.from(ctBuy).toString('hex'));

    // Verify decrypt
    const ptBatch = cipher.decrypt([ctBatch[0], ctBatch[1], ctBatch[2]], Buffer.alloc(16));
    console.log("\nDecrypted batch:", ptBatch.join(', '));

    const ptBid = cipher.decrypt([ctBid],   Buffer.alloc(16, 0x01));
    const ptSize = cipher.decrypt([ctSize], Buffer.alloc(16, 0x02));
    const ptBuy = cipher.decrypt([ctBuy],   Buffer.alloc(16, 0x03));
    
    console.log("Decrypted separate:");
    console.log("  ctBid → ", ptBid[0]);
    console.log("  ctSize→ ", ptSize[0]);
    console.log("  ctBuy → ", ptBuy[0]);

    // Test met Buffer vs Uint8Array als argumenten
    console.log("\n=== Type checks ===");
    const bufBatch = [Buffer.from(ctBatch[0]), Buffer.from(ctBatch[1]), Buffer.from(ctBatch[2])];
    const uintArr  = ctBatch.map((c: number[]) => new Uint8Array(c));
    
    console.log("Buffer length:", bufBatch[0].length);
    console.log("Uint8Array length:", uintArr[0].length);

    // Test: wat verwacht Rust?
    console.log("\n=== Rust signature (place_order) ===");
    console.log("  computation_offset: u64       → BN offset");
    console.log("  encrypted_bid: [u8; 32]      → Buffer[0..31]");
    console.log("  encrypted_size: [u8; 32]     → Buffer[0..31]");
    console.log("  encrypted_is_buy: [u8; 32]   → Buffer[0..31]");
    console.log("  pubkey: [u8; 32]             → Uint8Array of length 32");
    console.log("  nonce: u128                  → BN(0x...)");

    // Test met Anchor Program method
    const PROGRAM_ID   = new PublicKey('h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX');
    const CLUSTER_NUM  = 456;
    
    // Create program instance
    const accountResolver = () => undefined;
    const program = new anchor.Program(idlData as any, provider, undefined, accountResolver);

    console.log("\n=== Test TX met batch ciphertexts ===");
    try {
        const offset1 = 0xdeadbeef;
        
        // Probeer eerst alleen de method call (geen accountsPartial)
        const builder = program.methods.placeOrder(
            new anchor.BN(offset1),
            Buffer.from(ctBatch[0]),
            Buffer.from(ctBatch[1]),
            Buffer.from(ctBatch[2]),
            pubKey,
            new anchor.BN(0xfeedface)
        );

        // Check builder object
        console.log("Builder methods:", Object.keys(builder).filter(k => typeof (builder as any)[k] === 'function'));

    } catch(e: any) {
        console.error("Fout bij method call:", e.message);
    }

    // Test met separate ciphertexts  
    console.log("\n=== Test TX met separate ciphertexts ===");
    try {
        const offset2 = 0xbeefdead;
        
        const builder2 = program.methods.placeOrder(
            new anchor.BN(offset2),
            Buffer.from(ctBid),
            Buffer.from(ctSize),
            Buffer.from(ctBuy),
            pubKey,
            new anchor.BN(0xfeedface + 1)
        );

        console.log("Builder methods:", Object.keys(builder2).filter(k => typeof (builder2 as any)[k] === 'function'));

    } catch(e: any) {
        console.error("Fout bij separate method call:", e.message);
    }

    // Test: wat als ik de ciphertexts combineer tot één [u8; 96]?
    console.log("\n=== Test: combineer tot één buffer? ===");
    const combined = Buffer.concat([ctBatch[0], ctBatch[1], ctBatch[2]]);
    console.log("Combined length:", combined.length, "(moet niet zijn — Rust wil apart)");
}

main();
