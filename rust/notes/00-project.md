# Step 0 — packages, crates, and the source map

## What exists now

`rust/` is one Cargo package. Its `Cargo.toml` declares its name, edition, and
dependencies. Cargo finds two crate roots by convention:

- `src/lib.rs` is the library crate root. Reusable pricing and protocol code
  will go here and in modules it declares.
- `src/main.rs` is the binary crate root. It supplies the executable's
  `main` function and will eventually connect configuration and I/O.

These are two crates in one package. Neither file needs a `mod` declaration
to become a crate root. When calling the library from the binary, its Rust
path will be `near_intents_solver::...`: hyphens in the package name become
underscores in the crate name.

The empty library is deliberate. We have not invented a public API before
examining the pricing requirements. The executable prints its checkpoint and
exits successfully. It makes no network requests.

## Trace the TypeScript

Read these files on the pinned baseline:

| TypeScript source | Responsibility | Planned Rust location |
| --- | --- | --- |
| `src/main.ts`, `src/app.ts` | Startup and assembly | `src/main.rs` |
| `src/services/quoter.service.ts`: `getAmountOut`, `getAmountIn` | Pure pricing formulas | `src/amm.rs`, step 1 |
| `src/interfaces/websocket.interface.ts` | Relay request/response shapes | `src/protocol.rs`, step 2 |
| `src/services/quoter.service.ts`: `getQuoteResponse` | Validation and signed quote assembly | Split across pricing, protocol, and signing |
| `src/utils/hashing.ts`, `src/services/near.service.ts` | Serialization and signing | `src/signing.rs`, step 4 |
| `src/services/intents.service.ts` | Contract balance calls and nonce | `src/near.rs`, steps 4–5 |
| `src/services/websocket-connection.service.ts` | Relay lifecycle | `src/relay.rs`, step 6 |
| `src/configs/`, `src/services/cron.service.ts` | Configuration and refresh | Configuration and runtime wiring, step 7 |

Rust locations are a plan and may evolve. We will preserve protocol behavior
without copying the TypeScript service/class structure mechanically.

## First observation for step 1

The pricing functions accept an exact input or an exact output. Fees are
converted to basis points with `Math.floor(marginPercent * 100)`. Exact-input
quotes round the output down; exact-output quotes round the required input up.
The TypeScript uses `big.js`; choosing a Rust numeric representation requires
checking intermediate products, its division precision, and edge cases rather
than assuming every amount fits in `u64` or that floating point is suitable.

## Read alongside the code

- **Rust 1**, chapter 1, “Hello, Cargo!”, pp. 7–11: manifest, build, and run.
- **Rust 1**, chapter 7, “Packages and Crates”, p. 124: distinguish this
  package from its two crates. “Separating Modules into Different Files”,
  p. 143, will become useful when we add `amm.rs`.
- **Rust 2**, chapter 5, “Project Structure”, pp. 67–84. For this step,
  focus on “Project Configuration”, p. 73. Its workspace discussion starts
  on p. 70; we do not need a multi-package workspace yet.

Rust 1 is the primary reading. Rust 2 is optional depth.

## Run and inspect

Run the commands in the parent README. `cargo test` currently discovers zero
tests: a successful skeleton build is not evidence that solver behavior works.
Step 1 will introduce tests for actual pricing behavior.

Small exercise: explain why `main.rs` and `lib.rs` are separate crates even
though both belong to the same Cargo package. Then identify which one Cargo
runs when you type `cargo run`.
