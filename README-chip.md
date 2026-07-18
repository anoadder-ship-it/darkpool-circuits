# Chip Marketplace Darkpool

A privacy-preserving marketplace for semiconductors and AI hardware on Solana and [Arcium](https://arcium.com) MPC. Match GPU and AI accelerator orders without revealing inventory, pricing, export control information, or supply chain strategy.

**Program:** [6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o](https://explorer.solana.com/address/6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o?cluster=devnet)  
**SDK:** [arcium-darkpool-sdk](https://www.npmjs.com/package/arcium-darkpool-sdk)

---

## Problem

Companies holding AI chip inventory cannot advertise it without revealing competitive positioning, attracting export control scrutiny (H100, H200, GB200 fall under US BIS regulations), and disclosing infrastructure strategy to rivals. Buyers face the same disclosure problem in reverse.

## Solution

Sellers register encrypted chip listings. Buyers submit encrypted purchase orders. Arcium MPC nodes match them across seven criteria simultaneously without seeing any plaintext value. Region information (critical for export controls) is never revealed.

---

## Circuits

### match_chip: 7 criteria, 14 points each, max score 98

| Field | Listing (seller) | Order (buyer) |
|---|---|---|
| Chip type | H100=1001, H200=1002, GB200=1003, A100=1004, MI300X=2001, Gaudi3=3001 | Required chip |
| Quantity | Units available | Minimum required |
| Condition | 1=new, 2=refurbished, 3=used | Maximum acceptable |
| Price (cents/unit) | Asking price | Maximum budget |
| Delivery (days) | Lead time | Maximum acceptable |
| Region | EU=1, US=2, Asia=3, Global=4 | Required region |
| Cert level | 1=datacenter, 2=workstation, 3=consumer | Minimum required |

### aggregate_volume: Encrypted market intelligence

Compute aggregate market statistics without exposing individual transactions.

Binaries: [v0.1.0](https://github.com/anoadder-ship-it/chip-circuits/releases/tag/v0.1.0)

---

## Supported chips

| Code | Chip | Code | Chip |
|---|---|---|---|
| 1001 | NVIDIA H100 | 2001 | AMD MI300X |
| 1002 | NVIDIA H200 | 2002 | AMD MI250 |
| 1003 | NVIDIA GB200 | 3001 | Intel Gaudi3 |
| 1004 | NVIDIA A100 | 3002 | Intel Gaudi2 |
| 1005 | NVIDIA L40S | 9001-9004 | Generic GPU/CPU/ASIC/FPGA |

---

## Test results

```
PASS  register_chip
      H100 (1001), 10 units, new, $35,000/unit, 14 days, EU, datacenter cert
      confirmed=1

PASS  match_chip MATCH
      H100 listing vs H100 order, all 7 criteria satisfied
      matched=1, score=98/98

PASS  match_chip NO MATCH
      H100 (1001) listing vs H200 (1002) order
      matched=0, score=84/98  (6/7 match; chip type differs)

PASS  aggregate_volume  H100, 50 units -> 50

4/4 passing
```

---

## Quick start

```bash
npm install arcium-darkpool-sdk
```

```typescript
import { DarkpoolClient, ChipType, ChipCondition, Region, CertLevel } from "arcium-darkpool-sdk";

const client = await DarkpoolClient.create(config, walletKeypair, idl);
const result = await client.matchChip({
  chipType: BigInt(ChipType.H100), quantity: 10n,
  condition: BigInt(ChipCondition.New), pricePerUnit: 3500000n,
  deliveryDays: 14n, listRegion: BigInt(Region.EU),
  certLevel: BigInt(CertLevel.Datacenter),
  reqChipType: BigInt(ChipType.H100), minQuantity: 5n,
  maxCondition: BigInt(ChipCondition.Used), maxPrice: 4000000n,
  maxDelivery: 30n, reqRegion: BigInt(Region.EU),
  minCert: BigInt(CertLevel.Datacenter),
});
console.log(result.matched, result.score.toString()); // true, "98"
```

---

## Stress test (parallel execution)

10 concurrent `match_chip` transactions, 4 at a time, alternating MATCH/NO-MATCH cases,
on Solana devnet.

```
=== 10/10 PASS in 21.1s ===
```

---
## Related projects

| Project | Description |
|---|---|
| [Trading Dark Pool](https://github.com/anoadder-ship-it/darkpool-circuits) | Encrypted order matching |
| [Medical Darkpool](https://github.com/anoadder-ship-it/medical-circuits) | Encrypted dataset matching for healthcare |
| [Supply Chain Darkpool](https://github.com/anoadder-ship-it/supply-chain-circuits) | Encrypted inventory + carbon credits |
| [arcium-darkpool-sdk](https://www.npmjs.com/package/arcium-darkpool-sdk) | TypeScript SDK for all four darkpools |

## License

MIT
