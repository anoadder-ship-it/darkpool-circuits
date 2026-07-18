const { Connection, Keypair } = require('@solana/web3.js');
const anchor = require('@coral-xyz/anchor');
const reader = require('@arcium-hq/reader');

async function main() {
  const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });
  const wallet = new anchor.Wallet(Keypair.generate());
  const provider = new anchor.AnchorProvider(conn, wallet, {});
  anchor.setProvider(provider);

  const arciumProgram = reader.getArciumProgram(provider);
  const CLUSTER_OFFSET = 456;
  const clusterPubkey = reader.getClusterAccAddress(CLUSTER_OFFSET);
  console.log('Cluster PDA:', clusterPubkey.toString());

  const info = await reader.getClusterAccInfo(arciumProgram, clusterPubkey, 'confirmed');
  console.log('Cluster-info:', JSON.stringify(info, (k, v) => typeof v === 'bigint' ? v.toString() : v, 2));
}

main().catch(e => { console.error('FOUT:', e); process.exit(1); });
