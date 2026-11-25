# Rust solver

Independent Rust implementation based on the original TypeScript solver.
Current checkpoint: **0 — project skeleton**. This program currently prints a
status message; pricing, signing, and networking are not implemented.

## Run in Arch WSL

From a fresh clone:

```sh
git clone --branch codex/rust https://github.com/winvicta/near-intents-amm-solver.git
cd near-intents-amm-solver/rust
cargo run --locked
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

With an existing clone, run `git fetch origin` and
`git switch --track origin/codex/rust`, then enter `rust/`.

No Node.js installation, keys, account balances, or network connection is needed
to build this checkpoint with an installed Rust 1.85+ toolchain. There are no
external Rust dependencies yet. The lockfile is committed for reproducibility.

## Source and scope

Baseline: [TypeScript commit 0f29eb2](https://github.com/defuse-protocol/near-intents-amm-solver/tree/0f29eb26df7bb36bcb6fad97919680edfc8bdebb).
This branch starts at that commit, independently of the existing Rust branches.
The TypeScript source remains at the repository root. See the original
[MIT license](../LICENSE).

The initial target is one token pair, ordinary signing, and a local end-to-end
demonstration. TEE support is outside the first nine checkpoints.
Protocol behavior will be checked against current official documentation before
network integration.

## Nine checkpoints

| Step | Deliverable | Reading |
| --- | --- | --- |
| 0 | Cargo skeleton and source map | Rust 1 ch. 1, 7; Rust 2 ch. 5 |
| 1 | AMM exact-input/output pricing and parity cases | Rust 1 ch. 3, 6, 9, 11; Rust 2 ch. 4, 6 |
| 2 | Typed JSON requests and responses | Rust 1 ch. 5, 6, 10; Rust 2 ch. 2, 3 |
| 3 | Offline unsigned quote command | Rust 1 ch. 4, 9, 12; Rust 2 ch. 1, 4 |
| 4 | Payload construction and signing parity | Rust 1 ch. 8, 10; Rust 2 ch. 2, 6 |
| 5 | Read-only NEAR RPC balance retrieval | Rust 1 ch. 17; Rust 2 ch. 8 |
| 6 | Local WebSocket relay and reconnect handling | Rust 1 ch. 17; Rust 2 ch. 8 |
| 7 | Configuration, shared state, and shutdown | Rust 1 ch. 16, 17; Rust 2 ch. 10 |
| 8 | Local end-to-end verification and guide | Rust 1 ch. 11; Rust 2 ch. 6 |

These are planned deliverables, not claims of completed functionality. Each
checkpoint receives a commit and an accompanying explanation. We will advance
one checkpoint at a time.

**Rust 1:** The Rust Programming Language, third edition, Klabnik, Nichols,
and Krycho. **Rust 2:** Rust for Rustaceans, Gjengset. References use the
user's local editions; page numbers are printed pages, not PDF viewer numbers.
The books themselves are not included in this repository.

Start with [step 0](notes/00-project.md).
