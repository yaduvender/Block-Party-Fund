# 🏘️ Community Pool — Soroban Smart Contract

> A trustless fund-pooling contract on the Stellar network that lets neighbors collectively raise money for shared community events — no middleman, no spreadsheets, just code.

---

## Project Description

Community Pool is a [Soroban](https://soroban.stellar.org) smart contract built on the Stellar blockchain. It solves a classic neighborhood problem: when a group of people want to split the cost of something — a block party, a shared garden, a movie night — there's always friction around who holds the money, who paid what, and what happens if plans fall through.

This contract handles all of that on-chain. An organizer (admin) initializes a pool with a fundraising goal and deadline, neighbors contribute tokens directly from their wallets, and the admin either finalizes the pool (sweeping funds to the event recipient) or cancels it (enabling full refunds to every contributor). Every action is recorded as a ledger event and fully auditable.

---

## What It Does

```
Organizer deploys contract
        │
        ▼
  initialize(goal, deadline, token)
        │
        ▼
Neighbors call contribute(amount)  ──► tokens locked in contract
        │
        ├──► Admin calls finalize(recipient)
        │         └──► All funds sent to recipient  ✅
        │
        └──► Admin calls cancel()
                  └──► Each neighbor calls refund()
                            └──► Their tokens returned  ↩️
```

The contract operates in three exclusive states:

| State | Meaning |
|---|---|
| `active` | Pool is open; contributions accepted until deadline |
| `finalized` | Admin swept funds to the event recipient |
| `cancelled` | Admin cancelled; contributors may claim refunds |

---

## Features

**Permissionless Contributions**
Any wallet can contribute any positive amount at any time before the deadline. No allowlist, no KYC.

**Token-Agnostic**
Works with any [SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) compatible Stellar asset — USDC, EURC, native XLM wrapped token, or any custom asset contract.

**Contribution Tracking**
The contract records exactly how much each address has contributed, queryable on-chain at any time via `contribution_of(address)`.

**Goal Visibility**
`goal_reached()` and `total_raised()` are public read functions, making it easy for a frontend or explorer to show live progress toward the fundraising target.

**Hard Deadline**
Contributions are rejected after the configured Unix timestamp. The organizer sets this at initialization and it cannot be changed, preventing last-minute manipulation.

**Safe Cancellation & Refunds**
If the event falls through, the admin calls `cancel()`. Every contributor can then independently call `refund()` to receive their exact contribution back. Refunds are zeroed before transfer to prevent re-entrancy.

**On-Chain Event Log**
All major state changes (`init`, `contrib`, `finalize`, `cancel`, `refund`) emit Soroban contract events, making the full history indexable by any Horizon-compatible client.

**One-Time Initialization**
The `initialize` function can only be called once. Attempting to re-initialize panics, preventing contract hijacking.

---

## Project Structure

```
community_pool/
├── Cargo.toml          # Package manifest & Soroban SDK dependency
└── src/
    ├── lib.rs          # Contract logic
    └── test.rs         # Unit tests (happy path, cancel/refund, deadline enforcement)
```

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) with `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --features opt
```

### Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled `.wasm` file lands at:
```
target/wasm32-unknown-unknown/release/community_pool.wasm
```

### Test

```bash
cargo test
```

Three tests are included:
- `test_full_happy_path` — full contribution → finalize flow
- `test_cancel_and_refund` — cancel then refund cycle
- `test_contribution_after_deadline_panics` — deadline enforcement

### Deploy to Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/community_pool.wasm \
  --network testnet \
```

### Initialize

```bash
stellar contract invoke \
  --network testnet \
  -- initialize \
  --token TOKEN_CONTRACT_ID \
  --event_name "Summer Block Party" \
  --goal_amount 800000000 \
  --deadline 1800000000
```

---

## Contract Interface

| Function | Who Calls | Description |
|---|---|---|
| `initialize(admin, token, event_name, goal_amount, deadline)` | Deployer | One-time setup |
| `contribute(contributor, amount)` | Any neighbor | Lock tokens in the pool |
| `finalize(recipient)` | Admin | Sweep funds to event wallet |
| `cancel()` | Admin | Open the pool for refunds |
| `refund(contributor)` | Any contributor | Reclaim tokens after cancel |
| `total_raised()` | Anyone | Current total (read-only) |
| `goal()` | Anyone | Fundraising target (read-only) |
| `goal_reached()` | Anyone | `true` if total ≥ goal (read-only) |
| `contribution_of(address)` | Anyone | Individual contribution (read-only) |
| `status()` | Anyone | `"active"` / `"finalized"` / `"cancelled"` |

---

## License

MIT

Wallet Address = GB55VDM3MJRMPN3OHQ72UGGS55XJAIIHIY2YLH7OEEAXOAFDCNRR3XWW 

Contract Address = CDDPILZU7OBQE4UTMU2SPHXRIGZHEB3ZVR6ZAX5A7MSTXJRN36DQXSZD

https://stellar.expert/explorer/testnet/contract/CDDPILZU7OBQE4UTMU2SPHXRIGZHEB3ZVR6ZAX5A7MSTXJRN36DQXSZD


<img width="1918" height="812" alt="image" src="https://github.com/user-attachments/assets/bfd73d3c-7f83-4245-b27e-a5f17e5e3b0f" />
