import { RescueCipher, x25519 } from "@arcium-hq/client";
const { Connection, PublicKey, Keypair } = require("@solana/web3.js");

async function main() {
    const conn = new Connection("https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0", "confirmed");

    const wallet   = new (require("@anchor-lang/core")).Wallet(
        Keypair.fromSecretKey(Buffer.from(JSON.parse(require('fs').readFileSync(`${require('os').homedir()}/.config/solana/id.json`).toString())))
    );
    
    // Get MXE pubkey via RPC
    const { getMXEPublicKey } = require("@arcium-hq/client");
    const mxePubkey = await getMXEPublicKey(
        new (require("@anchor-lang/core")).AnchorProvider(conn, wallet, {}),
        new PublicKey('h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX')
    );

    if (!mxePubkey) { console.log("MXE pubkey niet gevonden"); return; }

    // Genereer eigen keypair
    const privKey = x25519.utils.randomSecretKey();
    const pubKey  = x25519.getPublicKey(privKey);

    // Bereken shared secret
    const sharedSecret = x25519.getSharedSecret(privKey, mxePubkey);
    console.log("Shared secret:", Buffer.from(sharedSecret).toString('hex'));

    // Encryptie met ECHTE MXE key
    const cipher  = new RescueCipher(sharedSecret);
    const nonce   = Buffer.alloc(16, 0x42);

    // Encodere [bid, size, is_buy] als één batch → krijgt 3 ciphertexts terug
    const ctBatch = cipher.encrypt([BigInt(100), BigInt(10), BigInt(1)], nonce);

    console.log("\nEncrypted values (with REAL MXE key):");
    console.log("ctBid:",     Buffer.from(ctBatch[0]).toString('hex'));
    console.log("ctSize:   ", Buffer.from(ctBatch[1]).toString('hex'));
    console.log("ctBuy:      ", Buffer.from(ctBatch[2]).toString('hex'));

    // Decryptie verifiëren
    const pt = cipher.decrypt(ctBatch, nonce);
    console.log("\nDecrypted:", pt.join(', '));  // Moet 100, 10, 1 zijn
    
    // Test met pubKey als argument (Arcium verwacht een pubkey)
    console.log("\npubkey length:", Buffer.from(pubKey).length);
    console.log("pubkey hex:",   Buffer.from(pubKey).toString('hex'));
}

main();
