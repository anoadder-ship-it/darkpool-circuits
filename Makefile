HELIUS := https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0
PROG   := h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX

.PHONY: status test

status:
	solana balance --url $(HELIUS)

test:
	ANCHOR_PROVIDER_URL=$(HELIUS) ANCHOR_WALLET=~/.config/solana/id.json yarn ts-node -P tsconfig.json scripts/decrypt-test.ts
