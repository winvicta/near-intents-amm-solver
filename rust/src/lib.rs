//! Rust port of the NEAR Intents AMM solver.
//!
//! This file is the root of the library crate. It declares the module tree,
//! and `src/main.rs` uses it through `near_intents_amm_solver::...` paths,
//! exactly as an outside crate would (Rust 1 pp.133–134).
//!
//! Modules are added bottom up: pure logic with unit tests first, network
//! and async wiring last.

pub mod config;
