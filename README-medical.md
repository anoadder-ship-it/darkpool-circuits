# Medical Data Darkpool

Privacy-preserving dataset matching for healthcare research. Hospitals discover compatible datasets and collaborate on federated learning without sharing raw patient data or violating GDPR.

**Program:** [CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4](https://explorer.solana.com/address/CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4?cluster=devnet)  
**SDK:** [arcium-darkpool-sdk](https://www.npmjs.com/package/arcium-darkpool-sdk)

---

## Problem

Hospitals hold valuable patient datasets that could train better AI for cancer detection and rare disease identification. But sharing raw data violates GDPR Article 9 and HIPAA. Datasets stay siloed. Research that could save lives never happens.

## Solution

Hospitals register encrypted dataset profiles. Researchers submit encrypted queries. Matching runs entirely over encrypted data. Only the outcome and a score (0-100) are returned, without revealing which criteria failed.

---

## Circuits

| Circuit | Encrypted input | Output |
|---|---|---|
| `register_dataset` | 5 fields: disease, samples, age, gender, modality | 1 (confirmed) |
| `match_dataset` | 10 fields: dataset profile + query | compatible (0/1), score (0-100) |
| `aggregate_gradient` | gradient value | aggregated gradient |

Binaries: [v0.1.0](https://github.com/anoadder-ship-it/medical-circuits/releases/tag/v0.1.0)

---

## Dataset profile

| Field | Encoding | Example |
|---|---|---|
| disease_code | ICD-10 as u64 | 340 = C34 lung cancer |
| sample_count | integer | 5000 patients |
| age_mean | integer x10 | 580 = 58.0 years |
| gender_female | percentage 0-100 | 40 |
| data_modality | 1=genomic 2=imaging 3=lab 4=clinical | 2 |

Score = 25 points per matching criterion, maximum 100.

---

## Test results

```
PASS  register_dataset
      lung cancer (C34), 5000 samples, 58yr avg, 40% female, imaging
      confirmed=1

PASS  match_dataset MATCH
      lung cancer query vs lung cancer dataset
      compatible=1, score=100/100

PASS  match_dataset NO MATCH
      breast cancer (C50) query vs lung cancer dataset
      compatible=0, score=75/100  (3/4 criteria match; disease differs)

PASS  aggregate_gradient  gradient=42 -> 42

4/4 passing
```

---

## Regulatory position

Designed for GDPR Article 9 and HIPAA compatibility. Raw patient data never leaves the hospital. Only encrypted aggregate metadata enters the system. MPC produces only non-reversible results. *Not legal advice.*

---

## Roadmap

| Version | Status | Description |
|---|---|---|
| v0.1 | Current | 3 circuits on Solana devnet |
| v0.2 | Planned | On-chain hospital registry |
| v0.3 | Planned | Multi-party federated learning across n hospitals |
| v1.0 | Planned | Mainnet, token-gated access |

---

## Stress test (parallel execution)

10 concurrent `match_dataset` transactions, 4 at a time, alternating MATCH/NO-MATCH cases,
on Solana devnet.

```
=== 10/10 PASS in 16.9s ===
```

---
## Related projects

| Project | Description |
|---|---|
| [Trading Dark Pool](https://github.com/anoadder-ship-it/darkpool-circuits) | Encrypted order matching |
| [Supply Chain Darkpool](https://github.com/anoadder-ship-it/supply-chain-circuits) | Encrypted inventory + carbon credits |
| [Chip Marketplace](https://github.com/anoadder-ship-it/chip-circuits) | Encrypted semiconductor marketplace |
| [arcium-darkpool-sdk](https://www.npmjs.com/package/arcium-darkpool-sdk) | TypeScript SDK for all four darkpools |

## License

MIT
