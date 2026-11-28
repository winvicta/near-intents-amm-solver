//! Binary entry point. TS original: `src/main.ts` + `src/app.ts`.
//!
//! Kept deliberately thin: it only calls into the library crate and decides
//! how to report failure. All logic lives in `src/lib.rs` and its modules.

use std::process;

use near_intents_amm_solver::config::Config;

fn main() {
    // `unwrap_or_else` takes a closure that runs only on `Err` (Rust 1 p.263).
    let config = Config::from_env().unwrap_or_else(|err| {
        // Errors go to stderr, normal output to stdout (Rust 1 p.279).
        eprintln!("config error: {err}");
        process::exit(1);
    });

    // `{:#?}` is pretty-printed Debug output. The private key prints as
    // <redacted> because `PrivateKey` has a hand-written Debug impl.
    println!("{config:#?}");
}
