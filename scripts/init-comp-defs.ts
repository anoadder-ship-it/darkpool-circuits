import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getArciumEnv, getComputationAccAddress, getClusterAccAddress, getMXEAccAddress } from "@arcium-hq/client";
import { PublicKey } from "@solana/web3.js";

async function main() {
    const provider = new anchor.AnchorProvider(
        process.env.HELIUS!,
        anchor.AnchorProvider.env().wallet,
        { commitment: "confirmed" }
    );
    anchor.setProvider(provider);

    const program = anchor.workspace.SolanaDarkpool as Program<any>;
    const arciumEnv = getArciumEnv();
    const clusterOffset = 456;

    console.log("Initializing computation definitions...");

    // Initialize get_stats comp def
    try {
        const tx = await program.methods
            .initGetStatsCompDef()
            .accountsPartial({
                computationAccount: getComputationAccAddress(clusterOffset, new anchor.BN(0)),
                clusterAccount: getClusterAccAddress(clusterOffset),
                mxeAccount: getMXEAccAddress(program.programId),
            })
            .rpc({ commitment: "confirmed" });
        console.log("✓ get_stats comp def initialized:", tx);
    } catch (e) {
        console.log("get_stats comp def error:", e.message);
    }

    console.log("Done!");
}

main().catch(console.error);
