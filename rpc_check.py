import asyncio
from solana.rpc.async_api import AsyncClient

RPC = "https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0"

async def main():
    async with AsyncClient(RPC) as client:
        print("Verbonden:", await client.is_connected())
        print("Slot:", (await client.get_slot()).value)
        print("Versie:", (await client.get_version()).value)

asyncio.run(main())
