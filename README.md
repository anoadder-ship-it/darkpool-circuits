# Solana Confidential Dark Pool

Privacy-preserving order matching on Solana using Arcium MPC.

**Program:** h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX
**Explorer:** https://explorer.solana.com/address/h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX?cluster=devnet

## The problem

Traditional DEXs expose every order on-chain. Anyone can see bid price, position size, and trading intent before a match, enabling front-running and sandwich attacks.

## The solution

1. Buyer encrypts bid locally using MXE public key
2. Seller encrypts ask independently
3. Arcium MPC nodes compare encrypted values without decrypting
4. Only the outcome is returned, also encrypted

No price, size, or identity ever appears in plaintext.

## Circuits

| Circuit | Input | Output |
|---|---|---|
| place_order | {bid, size, is_buy} encrypted | 1 confirmation |
| match_orders | {buy_bid, sell_bid} encrypted | 1 match or 0 no match |
| cancel_order | order_id encrypted | 1 confirmation |
| get_stats | {buy_vol, sell_vol} encrypted | buy_vol + sell_vol |

Circuit binaries: https://github.com/anoadder-ship-it/darkpool-circuits/releases/tag/v0.11.4

## Test results



## Benchmarks

- Average CU: 124,268
- Fee per TX: 0.000005 SOL (~ash.00085)
- MPC latency: 1-3s (avg 2s)

## Stack

- Solana / Anchor 1.1.2
- Arcium 0.11.2
- Arcis circuit definition language
- ARM64 / DGX Spark via FEX-EMU

## Commands

```bash
make build    # build with --skip-keys-sync
make deploy   # deploy via buffer
make test     # run full test with decryption
make status   # comp def states + balance
```

## Related

- [Medical Darkpool](README-medical.md)
- [Supply Chain Darkpool](README-supply.md)

## License

MIT
