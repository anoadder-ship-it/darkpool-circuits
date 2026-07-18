const { Connection, Keypair } = require('@solana/web3.js');
const anchor = require('@coral-xyz/anchor');
const reader = require('@arcium-hq/reader');
const client = require('@arcium-hq/client');

async function checkCompDef(conn, programIdStr, ixName, label) {
  const { PublicKey } = require('@solana/web3.js');
  const programId = new PublicKey(programIdStr);
  const offsetBytes = client.getCompDefAccOffset(ixName);
  const offset = Buffer.from(offsetBytes).readUInt32LE(0);
  const addr = reader.getCompDefAccAddress(programId, offset);
  const info = await conn.getAccountInfo(addr);
  console.log(`${label} (${ixName}) -> ${addr.toString()} : ${info ? 'BESTAAT (' + info.data.length + ' bytes)' : 'BESTAAT NIET'}`);
}

async function main() {
  const HELIUS = 'https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b';
  const conn = new Connection(HELIUS, { commitment: 'confirmed' });

  console.log('=== Trading (solana_darkpool) ===');
  await checkCompDef(conn, 'h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX', 'place_order', 'Trading');
  await checkCompDef(conn, 'h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX', 'match_orders', 'Trading');

  console.log('\n=== Medical ===');
  await checkCompDef(conn, 'CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4', 'register_dataset', 'Medical');
  await checkCompDef(conn, 'CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4', 'match_dataset', 'Medical');

  console.log('\n=== Supply Chain ===');
  await checkCompDef(conn, '3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4', 'register_supply', 'Supply');
  await checkCompDef(conn, '3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4', 'match_supply', 'Supply');

  console.log('\n=== Chip (ter vergelijking, hoort te bestaan) ===');
  await checkCompDef(conn, '6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o', 'match_chip', 'Chip');
}

main().catch(e => { console.error('FOUT:', e); process.exit(1); });
