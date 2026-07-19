# Circuit-source per darkpool

Elke darkpool heeft zijn eigen zelfstandige `<naam>_darkpool/encrypted-ixs/`
submap met Arcis MPC-circuitcode, naar het patroon van `medical_darkpool/`.

- `trading_darkpool/encrypted-ixs/` -> place_order, match_orders, cancel_order, get_stats
- `medical_darkpool/encrypted-ixs/` -> register_dataset, match_dataset, aggregate_gradient
- `supply_chain_darkpool/encrypted-ixs/` -> register_supply, match_supply, match_carbon
- `chip_darkpool/encrypted-ixs/` -> register_chip, match_chip, aggregate_volume

Bouwen per darkpool: `cd <naam>_darkpool && arcium build --skip-keys-sync`

De gedeelde root-map `encrypted-ixs/` bestond voorheen en werd om beurten
overschreven tussen darkpools -- dit was de oorzaak van verloren circuit-source
voor Trading en Supply Chain (hersteld op 2026-07-20 uit chatgeschiedenis).
Deze root-map is verwijderd; gebruik voortaan uitsluitend de submappen hierboven.
