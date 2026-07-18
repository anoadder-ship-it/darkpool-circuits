const anchor = require('@coral-xyz/anchor');
const { Connection, PublicKey } = require('@solana/web3.js');

async function main() {
  const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });

  // Arcium cluster PDA-derivatie: seeds = ["Cluster", cluster_offset (u32 LE)]
  const ARCIUM_PROGRAM_ID = new PublicKey('Arcj82pX7HxYKLR92qvgZUAd7vGS1k4hQvAFcPATFdEQ');
  const CLUSTER_OFFSET = 456;

  const offsetBuf = Buffer.alloc(4);
  offsetBuf.writeUInt32LE(CLUSTER_OFFSET, 0);

  const [clusterPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('Cluster'), offsetBuf],
    ARCIUM_PROGRAM_ID
  );

  console.log('Cluster PDA (offset 456):', clusterPda.toString());

  const accInfo = await conn.getAccountInfo(clusterPda);
  if (!accInfo) {
    console.log('GEEN account gevonden op dit adres — cluster bestaat niet (meer) op deze RPC/cluster.');
    return;
  }

  console.log('Account gevonden.');
  console.log('Owner:', accInfo.owner.toString());
  console.log('Lamports:', accInfo.lamports);
  console.log('Data-lengte:', accInfo.data.length, 'bytes');
  console.log('Eerste 200 bytes (hex):', accInfo.data.subarray(0, 200).toString('hex'));
}

main().catch(e => { console.error('FOUT:', e); process.exit(1); });
