const anchor = require("@anchor-lang/core");
const { Connection, PublicKey } = require("@solana/web3.js");

async function main() {
    const conn = new Connection("https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0", "confirmed");

    // Check accounts voor place_order
    const idl = JSON.parse(require('fs').readFileSync("target/idl/solana_darkpool.json").toString());
    
    console.log("=== place_order accounts (IDL) ===");
    const po = idl.instructions?.find(ix => ix.name === "place_order");
    
    if (po && po.accounts) {
        // Genereer de PDA adressen en check of ze bestaan op chain
        for (const acc of po.accounts) {
            let addr;
            let info = "";

            const name = typeof acc === 'string' ? acc : acc.name || "unknown";
            
            if (typeof acc !== 'string') {
                if (acc.address) {
                    addr = new PublicKey(acc.address);
                    info += ` fixed=${addr.toBase58()}`;
                } else if (acc.pda) {
                    // PDA seeds
                    try {
                        const seeds = [];
                        for (const seed of acc.pda.seeds) {
                            if (seed.kind === "const") {
                                seeds.push(Buffer.from(seed.value || "", seed.value_encoding || "utf-8"));
                            } else if (seed.kind === "account" && seed.account === "signer") {
                                // Dit is een PDA van de wallet signer, geen vaste waarde
                                seeds.push("signer_pubkey");
                            } else if (seed.kind === "arg" || seed.kind === "discriminator") {
                                // Ignoreren voor nu
                            }
                        }
                        
                        const [pdaAddr] = PublicKey.findProgramAddressSync(seeds, new PublicKey('h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX'));
                        addr = pdaAddr;
                        info += ` PDA (${seeds.length} seeds)`;
                    } catch(e) {
                        info += ` PDA ERROR: ${e.message}`;
                    }
                } else if (acc.key === "signer") {
                    // Dit is de wallet signer zelf
                    addr = undefined;
                    info += " signer";
                }
            }

            console.log(`\n${name}:`);
            if (addr) {
                const accInfo = await conn.getAccountInfo(addr);
                console.log(`  ${addr.toBase58()} (${info})`);
                console.log(`  Account: ${accInfo ? "EXISTEERT" : "NIET"} | data=${accInfo?.data.length || 0} bytes | lamports=${(accInfo?.lamports / 1e9 || 0).toFixed(4)} SOL`);
            } else {
                console.log(`  [wallet signer - geen vaste adres] (${info})`);
            }

            // Check writable
            if (typeof acc !== 'string' && acc.writable) {
                console.log("  WRITABLE: true");
            }
        }
    }

    // Check ook match_orders accounts
    const mo = idl.instructions?.find(ix => ix.name === "match_orders");
    if (mo && mo.accounts) {
        console.log("\n\n=== match_orders accounts (IDL) ===");
        for (const acc of mo.accounts.slice(0, 15)) { // eerste 15 voor overzicht
            const name = typeof acc === 'string' ? acc : acc.name || "unknown";
            let info = "";
            
            if (typeof acc !== 'string') {
                if (acc.address) {
                    info += ` fixed=${acc.address}`;
                } else if (acc.pda) {
                    info += " PDA";
                }
            }

            console.log(`  ${name}${info ? ' (' + info.slice(0,60) + ')' : ''}`);
        }
    }

    // Check accounts voor ArciumSignerAccount (de sign_pda_account)
    console.log("\n\n=== Account types in IDL ===");
    if (idl.accounts) {
        for (const acc of idl.accounts.slice(0, 15)) {
            const name = typeof acc === 'string' ? acc : acc.name || "unknown";
            let info = "";
            
            if (typeof acc !== 'string') {
                if (acc.type) {
                    info += ` ${JSON.stringify(acc.type).slice(0, 80)}`;
                } else if (acc.discriminator) {
                    info += ` discriminator=${acc.discriminator?.toString('hex') || "unknown"}`;
                }
            }

            console.log(`  ${name}${info ? ' (' + info.slice(0,80) + ')' : ''}`);
        }
    }

    // Conclusie: welke accounts mis ik?
    console.log("\n\n=== CONCLUSIE ===");
    console.log("Mijn huidige accountsPartial (6 accounts):");
    console.log("  computationAccount ✓");
    console.log("  clusterAccount ✓");  
    console.log("  mxeAccount ✓");
    console.log("  mempoolAccount ✓");
    console.log("  executingPool ✓");
    console.log("  compDefAccount ✓");
    
    console.log("\nIDL vereist accounts voor place_order:");
    if (po && po.accounts) {
        for (const acc of po.accounts.slice(0, 15)) {
            const name = typeof acc === 'string' ? acc : acc.name || "unknown";
            const exists = ['computationAccount', 'clusterAccount', 'mxeAccount', 
                           'mempoolAccount', 'executingPool', 'compDefAccount'].includes(name);
            console.log(`  ${name}: ${exists ? '✓ (in mijn code)' : '✗ MISSING IN MY CODE'}`);
        }
    }

    // Fix voorbeeld: wat moet er bij?
    const missing = ['sign_pda_account', 'pool_account'].filter(name => 
        !['computationAccount', 'clusterAccount', 'mxeAccount', 'mempoolAccount', 'executingPool', 'compDefAccount'].includes(name)
    );

    console.log("\n\n=== FIX: toevoegen aan accountsPartial ===");
    if (po && po.accounts) {
        for (const acc of po.accounts) {
            const name = typeof acc === 'string' ? acc : acc.name || "unknown";
            if (!['computationAccount', 'clusterAccount', 'mxeAccount', 
                  'mempoolAccount', 'executingPool', 'compDefAccount'].includes(name)) {
                console.log(`\n  ${name}:`);
                
                // Probeer het adres te genereren
                try {
                    if (typeof acc !== 'string' && acc.pda) {
                        const seeds = [];
                        for (const seed of acc.pda.seeds) {
                            if (seed.kind === "const" && seed.value_encoding === "base58") {
                                seeds.push(Buffer.from(seed.value || "", "utf-8"));
                            } else if (seed.kind === "const" && !seed.value_encoding) {
                                // UTF-8 of geen encoding
                                const val = seed.value || "";
                                seeds.push(typeof val === 'string' ? Buffer.from(val, 'utf-8') : Buffer.from(val));
                            }
                        }
                        
                        if (seeds.length > 0 && !seeds.some(s => s.toString() === "signer_pubkey")) {
                            const [pdaAddr] = PublicKey.findProgramAddressSync(seeds, new PublicKey('h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX'));
                            console.log(`    PDA address: ${pdaAddr.toBase58()}`);
                        } else {
                            console.log("    [PDA van wallet signer - moet dynamisch berekend worden]");
                        }
                    }
                } catch(e) {}
            }
        }
    }

    // Check ArciumSignerAccount structuur
    console.log("\n\n=== ArciumSignerAccount check ===");
    const arcAcc = idl.accounts?.find(a => (typeof a === 'string' ? a : a.name)?.includes("Arcium") || (a.type?.Discriminator?.discriminant));
    if (arcAcc) {
        console.log(JSON.stringify(arcAcc, null, 2).slice(0, 500));
    }
}

main();
