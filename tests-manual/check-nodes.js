const { Connection, Keypair } = require('@solana/web3.js');
const anchor = require('@coral-xyz/anchor');
const reader = require('@arcium-hq/reader');

function ipFromBytes(b) {
  return b.join('.');
}

async function main() {
  const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });
  const wallet = new anchor.Wallet(Keypair.generate());
  const provider = new anchor.AnchorProvider(conn, wallet, {});
  anchor.setProvider(provider);
  const arciumProgram = reader.getArciumProgram(provider);

  const nodeOffsets = [45548129, 743829];

  for (const offset of nodeOffsets) {
    const addr = reader.getArxNodeAccAddress(offset);
    console.log('\n=== Node offset', offset, '->', addr.toString(), '===');
    try {
      const info = await reader.getArxNodeAccInfo(arciumProgram, addr, 'confirmed');
      console.log(JSON.stringify(info, (k, v) => typeof v === 'bigint' ? v.toString() : v, 2));
      if (info.ip) {
        console.log('IP (leesbaar):', ipFromBytes(info.ip));
      }
    } catch (e) {
      console.log('FOUT bij ophalen node-info:', e.message);
    }
  }
}

main().catch(e => { console.error('FOUT:', e); process.exit(1); });
