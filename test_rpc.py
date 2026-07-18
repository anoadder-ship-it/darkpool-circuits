import asyncio
from solana.rpc.async_api import AsyncClient

async def main():
    async with AsyncClient("https://api.mainnet-beta.solana.com") as client:
        res = await client.is_connected()
        print(f"Verbonden: {res}")

asyncio.run(main())
