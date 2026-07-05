# Solana Confidential Dark Pool

Privacy-preserving order matching on Solana using [Arcium](https://arcium.com) MPC. Bid prices, order sizes, and trader identities stay encrypted end-to-end.

**Program:** [h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX](https://explorer.solana.com/address/h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX?cluster=devnet)  
**SDK:** [arcium-darkpool-sdk](https://www.npmjs.com/package/arcium-darkpool-sdk)

---

## Problem

Traditional DEXs expose every order on-chain. Bid price, position size, and trading intent are visible before execution, enabling front-running, sandwich attacks, and information leakage.

## Solution

1. Buyer encrypts bid locally using the MXE public key
2. Seller encrypts ask independently
3. Arcium MPC nodes compare encrypted values without decrypting them
4. Only the match outcome is returned, encrypted, decrypted client-side

No price, size, or identity ever appears in plaintext.

---

## Circuits

| Circuit | Encrypted input | Output |
|---|---|---|
| `place_order` | { bid, size, is_buy } | 1 (confirmed) |
| `match_orders` | { buy_bid, sell_bid } | 1 match / 0 no match |
| `cancel_order` | { order_id } | 1 (confirmed) |
| `get_stats` | { buy_volume, sell_volume } | aggregate total |

Binaries: [v0.11.4](https://github.com/anoadder-ship-it/darkpool-circuits/releases/tag/v0.11.4)

---

## Test results

```
PASS  place_order    confirmed=1
PASS  match_orders+  matched=1   (buy=100 >= sell=95)
PASS  match_orders-  matched=0   (buy=80  <  sell=95)
PASS  cancel_order   confirmed=1
PASS  get_stats      total=250   (150 + 100)

5/5 passing
```

---

## Performance (Solana devnet)

| Metric | Value |
|---|---|
| Average compute units | 124,268 CU |
| Fee per transaction | 0.000005 SOL (~$0.00085) |
| 1,000 tx/day | ~$0.85/day |
| MPC latency | 1-3 seconds (avg 2s) |

---

## Quick start

```bash
npm install arcium-darkpool-sdk
```

```typescript
import { DarkpoolClient } from "arcium-darkpool-sdk";

const client = await DarkpoolClient.create(config, walletKeypair, idl);
const result = await client.matchOrders({ buyBid: 100n, sellBid: 95n });
console.log(result.matched); // true
```

---

## Related projects

| Project | Description |
|---|---|
| [Medical Darkpool](https://github.com/anoadder-ship-it/medical-circuits) | Encrypted dataset matching for healthcare |
| [Supply Chain Darkpool](https://github.com/anoadder-ship-it/supply-chain-circuits) | Encrypted inventory + carbon credits |
| [Chip Marketplace](https://github.com/anoadder-ship-it/chip-circuits) | Encrypted semiconductor marketplace |
| [arcium-darkpool-sdk](https://www.npmjs.com/package/arcium-darkpool-sdk) | TypeScript SDK for all four darkpools |

## Stack

Solana / Anchor 1.1.2 + Arcium 0.12.0 + Arcis + NVIDIA DGX Spark (aarch64)

## License

MIT
