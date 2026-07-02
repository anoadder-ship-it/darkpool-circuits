# darkpool-circuits

# Arcium Darkpool Circuits (v0.11.1)

A high-performance, confidential Darkpool implementation on Solana powered by Arcium's Multi-Party Computation (MPC) network and the Arcis framework.

## Overview
This repository contains the cryptographic circuits and client-side integration scripts required to run an end-to-end confidential darkpool. It allows users to place and match orders completely blindly, preventing MEV frontrunning and price slippage on-chain.

## Features
- **Confidential Order Entry (`place_order`):** Encrypts order parameters (bids and sizes) before routing them to the MPC execution environment (MXE).
- **On-Chain Matching Engine (`match_orders`):** Matches orders trustlessly without exposing raw data to the public ledger.
- **TypeScript SDK Integration:** Fixed parameter-mismatches and big number (`BN`) conversions for direct integration with Solana Devnet RPC nodes.

## Technical Details
- **Arcium Framework:** Arcis v0.11.x
- **Target Network:** Solana Devnet / Mainnet Alpha
- **Primary Program ID:** `h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX`
