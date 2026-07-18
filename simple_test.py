#!/usr/bin/env python3
import json
from solders.keypair import Keypair
from solders.rpc.api import Client

RPC_URL = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0"
WALLET_PATH = "/home/michel/.config/solana/id.json"

# Laad je wallet
with open(WALLET_PATH, 'r') as f:
    secret_key = json.load(f)

keypair = Keypair.from_bytes(bytes(secret_key))
client = Client(RPC_URL)

print("✅ Connected successfully!")
print(f"Wallet address: {keypair.pubkey()}")
print(f"Balance: {client.get_balance(keypair.pubkey()).value / 1_000_000_000:.4f} SOL")
