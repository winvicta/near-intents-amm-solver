# Step 0: library + thin binary, and the config module

Books: **Rust 1** = *The Rust Programming Language*, 3rd ed. **R4R** = *Rust for Rustaceans*. Page numbers are the printed ones (Rust 1 printed = PDF − 32; R4R printed = PDF − 26).

## What this step adds

| File | What it is | TS original |
|---|---|---|
| `Cargo.toml` | The package. No dependencies yet. | `package.json` |
| `src/lib.rs` | Library crate root. Declares `pub mod config;`. | (none) |
| `src/main.rs` | Thin binary. Builds a `Config`, prints it, exits. | `src/main.ts`, `src/app.ts` |
| `src/config.rs` | Reads and checks every non-TEE setting, with 10 unit tests. | `src/configs/*.ts`, `env.validation.ts` |

There is no Tokio, no networking and no `.env` loading yet. Those come last, once the pure core is done.

## Try it

```bash
cd ~/near-intents-amm-solver-claude/rust
cargo test                       # 10 tests, all in src/config.rs
cargo run                        # fails: "config error: AMM_TOKEN1_ID is required"
AMM_TOKEN1_ID=usdt.tether-token.near AMM_TOKEN2_ID=wrap.near \
NEAR_ACCOUNT_ID=solver1.near NEAR_PRIVATE_KEY=ed25519:fake \
cargo run                        # prints the config; the key shows as <redacted>
```

## Concepts, in the order you meet them

1. **Package, library crate, binary crate.** One package can hold a `src/lib.rs` library and a `src/main.rs` binary; both get the package name. Rust 1 p.124. The recommended split is "just enough code in the binary to start the program, everything else in the library", so `main.rs` uses the library through its public API like any outside crate would. Rust 1 pp.133–134. Chapter 12 does the same refactor on a real program: Rust 1 p.266.

2. **Modules in their own files.** `pub mod config;` in `lib.rs` tells the compiler to load `src/config.rs`. Rust 1 p.143. `pub` on the module, the struct and each field is what lets `main.rs` reach them. Rust 1 p.131 and p.134 (struct fields are private unless marked `pub`).

3. **Paths and `use`.** `main.rs` writes `use near_intents_amm_solver::config::Config;`, starting with the crate name because it is outside the library. Inside the library, `use super::*;` in the tests module means "everything from the parent module". Rust 1 p.129, p.134 (`super`), p.136.

4. **A config struct, built once.** Rust 1 chapter 12 groups settings into a `Config` struct with a constructor that returns `Result` instead of panicking. Rust 1 p.258, p.260, p.262. This port does the same instead of the TS module-level `const`s.

5. **Enums for closed sets of values.** `NearNetwork` and `LogLevel` only allow listed values, so a typo becomes a parse error instead of a bad string flowing through the program. Rust 1 p.106, `match` on p.112.

6. **`Option<T>` instead of `undefined`.** `lookup` returns `Option<String>`. `filter`, `or_else`, `unwrap_or_else`, `ok_or` and `is_some_and` replace the TS `||` chains. Rust 1 p.110. (Those methods are in the `Option` docs rather than the book.)

7. **`Result<T, E>` and `?`.** `from_lookup` returns `Result<Config, ConfigError>`, and each `?` returns early with the error. Rust 1 p.171, p.175. `main` handles the error with `unwrap_or_else` plus `process::exit(1)`, as in Rust 1 p.263, and prints it with `eprintln!` so it goes to stderr. Rust 1 p.279.

8. **Traits from the standard library.**
   - `#[derive(Debug, Clone, PartialEq)]` writes trait impls for you. Rust 1 p.96 and Appendix C (p.567). R4R p.39 says to implement the common traits (`Debug`, `Clone`, `PartialEq`, ...) eagerly on public types, since users of the crate can't add them later.
   - `FromStr` is the trait behind `"testnet".parse::<NearNetwork>()`. Implementing a trait on your own type: Rust 1 p.199.
   - `Display` and `std::error::Error` on `ConfigError` make it a proper error type. The `Display` pattern is shown in Rust 1 p.497.

9. **A library error enum.** `ConfigError` is an enum with `Missing` and `Invalid` variants, so a caller can `match` on what went wrong. R4R p.58 ("Enumeration") recommends this for libraries; p.59 covers the opaque alternative (`Box<dyn Error>`, `anyhow`) that binaries often use. Rust 1 p.180 shows `Box<dyn Error>` in `main`.

10. **Newtype for secrets.** `PrivateKey(String)` is a one-field tuple struct with a hand-written `Debug` that prints `<redacted>`, so the key can't leak through `{:?}`. Tuple structs: Rust 1 p.91. Newtype pattern: Rust 1 p.497. R4R p.40 ("Wrapper Types").

11. **Closures and generic bounds.** `from_lookup<F>(lookup: F) where F: Fn(&str) -> Option<String>` takes any function that maps a name to a value. `from_env` passes one that reads real env vars, and the tests pass one that reads a `HashMap`. Closures: Rust 1 p.282; the `Fn` traits: Rust 1 p.288; `where` clauses: Rust 1 p.204.

    Why not just set env vars in the tests? Tests run in parallel threads (Rust 1 p.237), and changing the process environment from several threads at once races. Since edition 2024, `std::env::set_var` is an `unsafe` function for exactly that reason. Passing the lookup in avoids the problem entirely.

12. **Unit tests next to the code.** `#[cfg(test)] mod tests` is compiled only by `cargo test`. Rust 1 p.222 (writing tests), p.229 (`assert_eq!`), p.243 (unit tests and `#[cfg(test)]`). R4R p.86 explains the test harness, and p.88 explains `#[cfg(test)]`.

## Differences from the TypeScript on purpose

- **TEE mode is left out.** `TEE_ENABLED`, `SOLVER_REGISTRY_CONTRACT` and `SOLVER_POOL_ID` are not read, so `NEAR_ACCOUNT_ID` and `NEAR_PRIVATE_KEY` are always required.
- **All settings are read in one place,** instead of 7 config files that each read `process.env` on import.
- **`MARGIN_PERCENT=NaN` is rejected.** JavaScript's `Number("NaN")` and Rust's `"NaN".parse::<f64>()` both give NaN, and Joi's `.positive()` rejects it too, so this matches the TS validation.
