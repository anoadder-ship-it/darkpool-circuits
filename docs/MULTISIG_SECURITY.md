# Multisig-beveiliging darkpool-programma's (opgezet 9 juli 2026)

## Wat er gebeurd is
De oorspronkelijke upgrade-authority-sleutel (`G1qgHzMxNHqewWEKzEoV46GUXjDrsuD4P8LQ97T6gNXp`,
bestand `~/.config/solana/id.json`) raakte gecompromitteerd. Alle 4 darkpool-programma's
gebruikten deze ene sleutel. Opgelost door de upgrade-authority over te zetten naar een
Squads 2-van-3-multisig.

## Belangrijke adressen

**Multisig-account:** `J6dar1yhhx8NPVYbRRF2EJXRnS7eD7J4NT6X2ohGfs1b`
**Vault (= huidige upgrade-authority van alle 4 programma's):** `EmYvQBX7WPmLDnYEhSGRPv9wWf9whAEgLnZviSc4xWqY`
**Drempel:** 2 van de 3 leden moeten goedkeuren

**Leden (Phantom-accounts, kunnen ondertekenen):**
1. `HnQPCEiCTT8fzvPpRpz5J3fxsL3vELRVuaqfVFM154Ly`
2. `DDzVGAfzrFCu5QEFstv2KNHsxRTgQVAC6nSqp1PWh46d`
3. `4sRKi6mErV1fJCXNyY1RevU7Gv29gFJw82frweK86bmx`

**Gecompromitteerde sleutel (heeft nu GEEN macht meer, kan blijven staan):**
`G1qgHzMxNHqewWEKzEoV46GUXjDrsuD4P8LQ97T6gNXp`

## De 4 programma's (allemaal onder de multisig)
- Trading: `h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX`
- Medical: `CZQBaJFJnGA2pyEnrfxCmsUewcHJLDGHgzrcVjomzDD4`
- Supply Chain: `3HQHpSBSgYkx81E25bSJZVz4mGoW6nQFJWDtZL9fmMR4`
- Chip Marketplace: `6xLjbo4yfc5j2CMu69DkycTJrGZttHzeqieXf2NPvu8o`

## Belangrijk voor de toekomst
- Elke toekomstige upgrade van een van de 4 programma's vereist nu 2-van-3 goedkeuring
  via de multisig (bijv. via Squads' TypeScript SDK, of via app.squads.so als hun devnet-UI
  het toont -- werkte niet altijd betrouwbaar volgens bekende issues).
- Setup-script staat op: `~/multisig-setup/create_multisig_v2.js`
- Aanmaak-transactie-signature: `7dDUmnNQhopmmMRjdbWedNkGW1UqbNzNpW5jtnbfLJUGG2cRA3GT9MU9eXNGAMKMJxvT8H5EF9TPRz7eme3vsJZ`
