#!/usr/bin/env python3
"""Heartbeat test met AsyncClient en correcte solders imports."""
import asyncio, json, time
from solana.rpc.async_api import AsyncClient
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from solders.transaction import Transaction
from solders.instruction import Instruction, AccountMeta

RPC_URL = "https://devnet.helius-rpc.com/?api-key=723d756a-9a08-40a3-a7ab-00431beb3c6b"
WALLET_PATH = "/home/michel/.config/solana/id.json"

def load_keypair():
    with open(WALLET_PATH, 'r') as f:
        return Keypair.from_bytes(bytes(json.load(f)))

async def send_heartbeat(keypair):
    async with AsyncClient(RPC_URL) as client:
        latest_blockhash = await client.get_latest_blockhash()
        
        # Transfer 1000 lamports naar zelf als heartbeat marker
        data = bytes([0]) + (1000).to_bytes(8, 'little')
        instruction = Instruction(
            program_id=Pubkey.from_string("SystemProgram"),
            accounts=[AccountMeta(pubkey=keypair.pubkey(), is_signer=True, is_writable=False)],
            data=data
        )
        
        transaction = Transaction.new_with_payer([instruction], keypair)
        result = await client.send_transaction(transaction)
        return str(result.value)

async def main():
    print("=== Heartbeat Test ===")
    
    try:
        keypair = load_keypair()
        tx_sig = await send_heartbeat(keypair)
        print(f"✅ Succesvol! TX Signature: {tx_sig}")
        
        # Poll voor confirmatie (max 30 seconden)
        for i in range(10):
            async with AsyncClient(RPC_URL) as client:
                status = await client.get_signature_status(tx_sig, search_transaction_history=True)
                if status.value and not status.value.err:
                    print(f"✅ Transactie bevestigd na {i+1} pogingen")
                    return True
            time.sleep(3)
        
        print("⚠️  Timeout bij confirmatie (transactie mogelijk nog in mempool)")
    except Exception as e:
        print(f"❌ Fout: {str(e)}")

if __name__ == "__main__":
    asyncio.run(main())
