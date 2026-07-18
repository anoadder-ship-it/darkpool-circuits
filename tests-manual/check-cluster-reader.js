const { Connection } = require('@solana/web3.js');
const anchor = require('@coral-xyz/anchor');
const { Keypair } = require('@solana/web3.js');

async function main() {
  const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });
  const wallet = new anchor.Wallet(Keypair.generate());
  const provider = new anchor.AnchorProvider(conn, wallet, {});
  anchor.setProvider(provider);

  const reader = require('@arcium-hq/reader');
  console.log('Beschikbare exports in @arcium-hq/reader:', Object.keys(reader));

  const CLUSTER_OFFSET = 456;

  if (reader.getClusterAccAddress && reader.getClusterAccInfo) {
    const clusterPubkey = reader.getClusterAccAddress(CLUSTER_OFFSET);
    console.log('Cluster PDA:', clusterPubkey.toString());
    const info = await reader.getClusterAccInfo(conn, clusterPubkey);
    console.log('Cluster-info:', JSON.stringify(info, (k, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  } else {
    console.log('getClusterAccAddress/getClusterAccInfo niet gevonden, check exports hierboven.');
  }
}

main().catch(e => { console.error('FOUT:', e); process.exit(1); });
