# Arcium Confidential Darkpool Suite — Handleiding

**Dark Matter Labs** | v0.2.0 | Solana Devnet

---

## Wat is dit?

Vier versleutelde darkpools op Solana. Invoer wordt versleuteld voordat het de keten raakt. Arcium MPC-nodes berekenen het resultaat zonder de echte waarden te zien.

---

## Deel 1: Live demo

**URL:** https://anoadder-ship-it.github.io/darkpool-demo-

1. Installeer Phantom wallet: https://phantom.app
2. Stel Phantom in op Devnet: tandwiel > Developer Settings > Network > Devnet
3. Haal gratis SOL op: https://faucet.solana.com
4. Open de demo, connect Phantom, klik Encrypt & match

---

## Deel 2: SDK installeren

```bash
npm install arcium-darkpool-sdk
```

### Trading

```typescript
import { DarkpoolClient } from "arcium-darkpool-sdk";
const client = await DarkpoolClient.create({ programId: "h6zsnHt28...", heliusApiKey: "...", cluster: "devnet" }, keypair, idl);
const result = await client.matchOrders({ buyBid: 100n, sellBid: 95n });
console.log(result.matched); // true
```

### Medical

```typescript
const result = await client.matchDataset({
  diseaseCode: 340n, sampleCount: 5000n, ageMean: 580n, genderFemale: 40n, dataModality: 2n,
  queryDisease: 340n, minSamples: 1000n, ageMin: 500n, ageMax: 700n, queryModality: 2n,
});
console.log(result.compatible, result.score.toString()); // true, "100"
```

### Supply chain

```typescript
const result = await client.matchSupply({
  materialCode: 1001n, quantity: 50000n, qualityGrade: 85n, pricePerUnit: 150n, deliveryDays: 7n, supplyRegion: 1n,
  reqMaterial: 1001n, minQuantity: 10000n, minQuality: 80n, maxPrice: 200n, maxDelivery: 10n, reqRegion: 1n,
});
console.log(result.matched, result.score.toString()); // true, "96"
```

### Chip marketplace

```typescript
import { ChipType, ChipCondition, Region, CertLevel } from "arcium-darkpool-sdk";
const result = await client.matchChip({
  chipType: BigInt(ChipType.H100), quantity: 10n, condition: BigInt(ChipCondition.New),
  pricePerUnit: 3500000n, deliveryDays: 14n, listRegion: BigInt(Region.EU), certLevel: BigInt(CertLevel.Datacenter),
  reqChipType: BigInt(ChipType.H100), minQuantity: 5n, maxCondition: BigInt(ChipCondition.Used),
  maxPrice: 4000000n, maxDelivery: 30n, reqRegion: BigInt(Region.EU), minCert: BigInt(CertLevel.Datacenter),
});
console.log(result.matched, result.score.toString()); // true, "98"
```

---

## Deel 3: Eigen darkpool bouwen

```bash
git clone https://github.com/anoadder-ship-it/darkpool-template
./darkpool-template/new-darkpool.sh mijnpool
```

1. Bewerk encrypted-ixs/src/lib.rs (circuit logica)
2. FEXBash -c "arcium build --skip-keys-sync"
3. solana program deploy target/deploy/mijnpool.so
4. ts-node scripts/init-mxe.ts
5. node scripts/test-mijnpool.js

---

## Deel 4: Codes en enums

### Chiptypen
- 1001=H100, 1002=H200, 1003=GB200, 1004=A100, 1005=L40S
- 2001=MI300X, 2002=MI250
- 3001=Gaudi3, 3002=Gaudi2
- 9001=GPU, 9002=CPU, 9003=ASIC, 9004=FPGA

### Conditie: 1=Nieuw, 2=Refurbished, 3=Gebruikt

### Regio: 1=EU, 2=VS, 3=Azie, 4=Globaal

### Certificaat: 1=Datacenter, 2=Workstation, 3=Consumer

### Materiaal: 1001=Staal, 1002=Aluminium, 1003=Koper, 2001=Chips

### Datamodaliteit: 1=Genomisch, 2=Beeldvorming, 3=Lab, 4=Klinisch

---

## Deel 5: Bekende bugs en oplossingen

### declare_id! overschreven
Altijd bouwen met: arcium build --skip-keys-sync

### anchor.workspace verkeerd program ID
```typescript
IDL.address = PROGRAM_ID.toBase58(); // voor new anchor.Program(IDL, provider)
```

### MXE init SizeMismatch
CLI bug — gebruik TypeScript SDK initMxePart1 + initMxePart2. Zie scripts/init-mxe.ts

### ts-node OOM
Gebruik: NODE_OPTIONS="--max-old-space-size=8192" yarn ts-node
Of compileer eerst met tsc en draai als plain node

---

## Deel 6: Program IDs (devnet)

- Trading:      h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX
- Medical:      CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4
- Supply chain: 3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4
- Chip:         6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o
- Cluster offset: 456

---

## Deel 7: Links

- Demo: https://anoadder-ship-it.github.io/darkpool-demo-
- npm: https://www.npmjs.com/package/arcium-darkpool-sdk
- Template: https://github.com/anoadder-ship-it/darkpool-template
- Blogpost: https://paragraph.com/@darkpool
- Arcium docs: https://docs.arcium.com
- Faucet: https://faucet.solana.com

---

*Dark Matter Labs — anoadder@gmail.com*
