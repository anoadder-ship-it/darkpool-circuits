# Supply Chain Darkpool

Encrypted matching of surplus industrial inventory and carbon credits between competitors. Find buyers and sellers without revealing inventory levels, pricing, or identities.

**Program:** [3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4](https://explorer.solana.com/address/3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4?cluster=devnet)  
**SDK:** [arcium-darkpool-sdk](https://www.npmjs.com/package/arcium-darkpool-sdk)

---

## Problem

Companies with surplus inventory cannot sell it without signaling overproduction to competitors. Carbon credit buyers must disclose emission levels on public exchanges. Traditional brokers require disclosing prices and volumes before any introduction.

## Solution

Offers and requests are encrypted before submission. Arcium MPC nodes match them without seeing any plaintext value. A match is confirmed only when all criteria align simultaneously.

---

## Circuits

### match_supply: 6 criteria, 16 points each, max score 96

| Field | Offer | Demand |
|---|---|---|
| Material code | 1001=steel, 1002=aluminum, 1003=copper | Required material |
| Quantity | Amount available | Minimum required |
| Quality grade (1-100) | Your grade | Minimum acceptable |
| Price (cents/unit) | Asking price | Maximum budget |
| Delivery (days) | Lead time | Maximum acceptable |
| Region | EU=1, Asia=2, US=3, Global=4 | Required region |

### match_carbon: 4 criteria

| Field | Offer | Demand |
|---|---|---|
| Credits | Amount for sale | Minimum needed |
| Price (cents/credit) | Asking price | Maximum budget |
| Vintage (year) | Year of issuance | Minimum year |
| Certificate | VCS=1, Gold=2, CDM=3 | Required type |

Binaries: [v0.1.0](https://github.com/anoadder-ship-it/supply-chain-circuits/releases/tag/v0.1.0)

---

## Test results

```
PASS  register_supply
      steel (1001), 50,000 kg, grade 85, 150 ct/kg, 7 days, EU
      confirmed=1

PASS  match_supply MATCH
      steel offer vs steel demand, all 6 criteria satisfied
      matched=1, score=96/96

PASS  match_supply NO MATCH
      steel offer (1001) vs aluminum demand (1002)
      matched=0, score=80/96  (5/6 match; material type differs)

PASS  match_carbon MATCH
      1000 VCS credits, vintage 2022, 12 ct vs min 500, max 15, min 2020, VCS
      matched=1

4/4 passing
```

---

## Use cases

- **Surplus inventory:** find buyers among competitors anonymously
- **Carbon credits:** trade without revealing emission levels
- **Circular economy:** waste from company A becomes feedstock for company B
- **Capacity exchange:** spare factory capacity matched to overflow demand
- **Joint procurement:** aggregated buying power, individual volumes stay private

---

## Stress test (parallel execution)

10 concurrent `match_supply` transactions, 4 at a time, alternating MATCH/NO-MATCH cases,
on Solana devnet.

```
=== 10/10 PASS in 17.3s ===
```

---
## Related projects

| Project | Description |
|---|---|
| [Trading Dark Pool](https://github.com/anoadder-ship-it/darkpool-circuits) | Encrypted order matching |
| [Medical Darkpool](https://github.com/anoadder-ship-it/medical-circuits) | Encrypted dataset matching for healthcare |
| [Chip Marketplace](https://github.com/anoadder-ship-it/chip-circuits) | Encrypted semiconductor marketplace |
| [arcium-darkpool-sdk](https://www.npmjs.com/package/arcium-darkpool-sdk) | TypeScript SDK for all four darkpools |

## License

MIT
