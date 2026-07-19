const fs = require('fs');
const anchor = require('@coral-xyz/anchor');
const { Connection, Keypair, PublicKey } = require('@solana/web3.js');
const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
const arcium = require('@arcium-hq/client');

function loadKeypair(path) {
  const raw = JSON.parse(fs.readFileSync(path, 'utf8'));
  return Keypair.fromSecretKey(new Uint8Array(raw));
}
function readUInt32LE(bytes) {
  return ((bytes[0]) | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24)) >>> 0;
}

const CLUSTER = 456;

const TESTS = [
  {
    label: 'Medical',
    programId: 'CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4',
    idlPath: '/home/michel/solana_darkpool/target/idl/medical_darkpool.json',
    poolAddress: '529GWLXSt4WkWghjaeWYKxCwVxXczdDjTvYNv1JrXFht',
    ixName: 'registerDataset',
    ixCircuitName: 'register_dataset',
    numEncArgs: 5,
  },
  {
    label: 'Supply Chain',
    programId: '3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4',
    idlPath: '/home/michel/solana_darkpool/target/idl/supply_chain_darkpool.json',
    poolAddress: 'GDTxnJMw5FJnvvoz2kQgp2YUbxb5no3XZspKwVPTJn56',
    ixName: 'registerSupply',
    ixCircuitName: 'register_supply',
    numEncArgs: 6,
  },
  {
    label: 'Chip',
    programId: '6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o',
    idlPath: '/home/michel/solana_darkpool/target/idl/chip_darkpool.json',
    poolAddress: 'C5Qty81mfacsL1YEmyLjYpdHK3Hrukg6KDbvgpe3PVN8',
    ixName: 'registerChip',
    ixCircuitName: 'register_chip',
    numEncArgs: 7,
  },
];

async function main() {
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });
  const guardian = loadKeypair('/home/michel/solana_darkpool/test-wallets/buyer.json');

  for (const t of TESTS) {
    console.log(`\n########## ${t.label} ##########`);
    const idl = JSON.parse(fs.readFileSync(t.idlPath, 'utf8'));
    const wallet = new anchor.Wallet(guardian);
    const provider = new anchor.AnchorProvider(conn, wallet, { commitment: 'confirmed' });
    const program = new anchor.Program(idl, provider);
    const PROGRAM_ID = new PublicKey(t.programId);
    const POOL_ADDRESS = new PublicKey(t.poolAddress);

    async function tryRegister(label) {
      const off = new anchor.BN(require('crypto').randomBytes(8), 'hex');
      const compDefOffset = readUInt32LE(arcium.getCompDefAccOffset(t.ixCircuitName));
      const args = [off, ...Array(t.numEncArgs).fill(new Array(32).fill(0)), new Array(32).fill(0), new anchor.BN(0)];
      try {
        const sig = await program.methods[t.ixName](...args)
          .accountsPartial({
            payer: guardian.publicKey,
            computationAccount: arcium.getComputationAccAddress(CLUSTER, off),
            clusterAccount: arcium.getClusterAccAddress(CLUSTER),
            mxeAccount: arcium.getMXEAccAddress(PROGRAM_ID),
            mempoolAccount: arcium.getMempoolAccAddress(CLUSTER),
            executingPool: arcium.getExecutingPoolAccAddress(CLUSTER),
            compDefAccount: arcium.getCompDefAccAddress(PROGRAM_ID, compDefOffset),
            moerasPool: POOL_ADDRESS,
          })
          .rpc();
        console.log(`[${label}] GESLAAGD, tx: ${sig}`);
        return true;
      } catch (e) {
        console.log(`[${label}] GEFAALD: ${e.message.slice(0, 150)}`);
        return false;
      }
    }

    const okBefore = await tryRegister('VOOR trigger');

    await program.methods.triggerMoeras()
      .accounts({ pool: POOL_ADDRESS, signer: guardian.publicKey })
      .signers([guardian])
      .rpc();
    console.log('trigger_moeras uitgevoerd');

    const okDuring = await tryRegister('TIJDENS Moeras');

    await program.methods.reactivatePool()
      .accounts({ pool: POOL_ADDRESS, signer: guardian.publicKey })
      .signers([guardian])
      .rpc();
    console.log('reactivate_pool uitgevoerd');

    const okAfter = await tryRegister('NA reactivatie');

    const alles_goed = okBefore === true && okDuring === false && okAfter === true;
    console.log(`${t.label}: ${alles_goed ? 'KILLSWITCH WERKT CORRECT' : 'ONVERWACHT GEDRAG, controleer hierboven'}`);
  }
}

main().catch((e) => {
  console.error('FOUT:', e);
  process.exit(1);
});
