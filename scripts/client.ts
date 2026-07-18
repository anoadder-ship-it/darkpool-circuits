import * as anchor from "@anchor-lang/core";
import { BN } from "@anchor-lang/core";
import { PublicKey, ComputeBudgetProgram, Connection, Keypair, Transaction } from "@solana/web3.js";
import { getCompDefAccOffset, RescueCipher, deserializeLE,
  getMXEPublicKey, getMXEAccAddress, getMempoolAccAddress,
  getCompDefAccAddress, getExecutingPoolAccAddress,
  getComputationAccAddress, getClusterAccAddress, x25519 } from "@arcium-hq/client";
import { randomBytes } from "crypto";
import * as fs from "fs";
import * as os from "os";

const PROGRAM_ID = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");
const HELIUS     = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0";
const CLUSTER_NUM = 456;

async function sendTx(connection: Connection, tx: Transaction, signers: Keypair[]): Promise<string> {
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
    tx.recentBlockhash = blockhash;
    tx.lastValidBlockHeight = lastValidBlockHeight;
    tx.feePayer = signers[0].publicKey;
    for (const s of signers) tx.partialSign(s);
    const sig = await connection.sendRawTransaction(tx.serialize(), { skipPreflight: true });
    console.log("  TX:", sig.slice(0, 24) + "...");
    for (let i = 0; i < 30; i++) {
        await new Promise(r => setTimeout(r, 3000));
        const s = await connection.getSignatureStatus(sig);
        if (s.value?.err) throw new Error(`TX mislukt: ${JSON.stringify(s.value.err)}`);
        if (s.value?.confirmationStatus === "confirmed" || s.value?.confirmationStatus === "finalized") return sig;
    }
    throw new Error("Timeout");
}

function encOne(value: bigint, cipher: RescueCipher): { ct: any; nonce: BN } {
    const nb = randomBytes(16);
    const cts = cipher.encrypt([value], nb);
    return { ct: Array.from(cts[0]), nonce: new BN(deserializeLE(nb).toString()) };
}

async function main() {
    try {
        const connection = new Connection(HELIUS, { commitment: "confirmed" });
        const owner = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync(`${os.homedir()}/.config/solana/id.json`).toString())));
        const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(owner), {});
        anchor.setProvider(provider);

        const IDL = JSON.parse(fs.readFileSync("target/idl/solana_darkpool.json").toString());
        IDL.address = PROGRAM_ID.toBase58();
        const program = new anchor.Program(IDL as any, provider);

        console.log("Wallet:", owner.publicKey.toBase58());
        console.log("Program:", PROGRAM_ID.toBase58());

        const mxePubkey = await getMXEPublicKey(provider, PROGRAM_ID);
        if (!mxePubkey) throw new Error("MXE key niet gevonden");

        const privKey = x25519.utils.randomSecretKey();
        const pubKeyArr = Array.from(x25519.getPublicKey(privKey));
        const cipher = new RescueCipher(x25519.getSharedSecret(privKey, mxePubkey));

        console.log("\n=== Test 1: Kooporder bid=100 ===");
        {
            const b = encOne(BigInt(100), cipher);
            const s = encOne(BigInt(10), cipher);
            const ib = encOne(BigInt(1), cipher);
            const offset = new BN(randomBytes(8), "hex");

            await sendTx(connection, await (program.methods as any).placeOrder(offset, b.ct, s.ct, ib.ct, pubKeyArr, b.nonce, s.nonce, ib.nonce)
                .accountsPartial({ computationAccount: getComputationAccAddress(CLUSTER_NUM, offset), clusterAccount: getClusterAccAddress(CLUSTER_NUM), mxeAccount: getMXEAccAddress(PROGRAM_ID), mempoolAccount: getMempoolAccAddress(CLUSTER_NUM), executingPool: getExecutingPoolAccAddress(CLUSTER_NUM), compDefAccount: getCompDefAccAddress(PROGRAM_ID, Buffer.from(getCompDefAccOffset("place_order")).readUInt32LE()) })
                .transaction(), [owner]);
            console.log("  ✓ Kooporder verstuurd");
        }

        console.log("\n=== Test 2: Verkooporder bid=95 ===");
        {
            const b = encOne(BigInt(95), cipher);
            const s = encOne(BigInt(10), cipher);
            const ib = encOne(BigInt(0), cipher);
            const offset = new BN(randomBytes(8), "hex");

            await sendTx(connection, await (program.methods as any).placeOrder(offset, b.ct, s.ct, ib.ct, pubKeyArr, b.nonce, s.nonce, ib.nonce)
                .accountsPartial({ computationAccount: getComputationAccAddress(CLUSTER_NUM, offset), clusterAccount: getClusterAccAddress(CLUSTER_NUM), mxeAccount: getMXEAccAddress(PROGRAM_ID), mempoolAccount: getMempoolAccAddress(CLUSTER_NUM), executingPool: getExecutingPoolAccAddress(CLUSTER_NUM), compDefAccount: getCompDefAccAddress(PROGRAM_ID, Buffer.from(getCompDefAccOffset("place_order")).readUInt32LE()) })
                .transaction(), [owner]);
            console.log("  ✓ Verkooporder verstuurd");
        }

        console.log("\n=== Test 3: Match ===");
        {
            const b = encOne(BigInt(100), cipher);
            const s = encOne(BigInt(95), cipher);
            const offset = new BN(randomBytes(8), "hex");

            await sendTx(connection, await (program.methods as any).matchOrders(offset, b.ct, s.ct, pubKeyArr, b.nonce, s.nonce)
                .accountsPartial({ computationAccount: getComputationAccAddress(CLUSTER_NUM, offset), clusterAccount: getClusterAccAddress(CLUSTER_NUM), mxeAccount: getMXEAccAddress(PROGRAM_ID), mempoolAccount: getMempoolAccAddress(CLUSTER_NUM), executingPool: getExecutingPoolAccAddress(CLUSTER_NUM), compDefAccount: getCompDefAccAddress(PROGRAM_ID, Buffer.from(getCompDefAccOffset("match_orders")).readUInt32LE()) })
                .transaction(), [owner]);
            console.log("  ✓ Match verstuurd");

            await new Promise(r => setTimeout(r, 60000));
            const sigs = await connection.getSignaturesForAddress(PROGRAM_ID, { limit: 10 });
            for (const s of sigs.filter(s => !s.err).slice(0, 5)) {
                const t = await connection.getTransaction(s.signature, { maxSupportedTransactionVersion: 0 });
                if (t?.meta?.logMessages?.join("\n").includes("Callback")) {
                    console.log("  ✓ Callback:", s.signature.slice(0, 24));
                }
            }
        }

        console.log(`\nExplorer: https://explorer.solana.com/address/${PROGRAM_ID.toBase58()}?cluster=devnet`);
    } catch(e: any) { console.error("Fout:", e?.message || e); process.exit(1); }
}

main();
