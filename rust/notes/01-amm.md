# Step 1 — AMM quote calculation

## Start here

From `~/near-intents-amm-solver-codex/rust`:

```sh
cargo run --locked
cargo test --locked
```

Expected demonstration:

```text
Exact input: 10 -> 99600 base units (fee: 30 bps)
Exact output: 99600 base units requires 10 input units
```

Cargo downloads dependencies on the first build. This checkpoint adds
`num-bigint` for intermediate arithmetic. After fetching dependencies, these
commands also work with `--offline`. No Node.js, keys, or funds are needed
for the demonstration or Rust tests.

This is a calculation using fixed sample reserves. It does not fetch balances,
connect to a relay, construct a signed quote, or execute a swap.

## Read in this order

1. `src/main.rs`: a short caller using the library.
2. `src/amm.rs`: the two pricing functions and their error type.
3. `tests/amm.rs`: examples, boundaries, and a small exhaustive grid.
4. Return here for the TypeScript comparison and optional exercise.

The step 0 guide stays in `notes/00-project.md`.

## What is the AMM calculating?

The pool contains reserves of two tokens. A constant-product AMM prices a
trade according to the relationship between those reserves. A larger trade
changes that relationship more, so output is not simply input times a fixed
exchange rate. We use the same fee placement as the TypeScript functions
`getAmountOut` and `getAmountIn` in `src/services/quoter.service.ts`.

All amounts are **integer base units**. For a token with 24 decimal places,
one whole token would be represented by 10^24 base units. Tokens can have
different decimal counts. The pricing function receives base-unit reserves
for each token and does not assume their units are interchangeable.

The caller supplies reserves in the trade's direction. To reverse a trade,
swap the reserve arguments. To simulate a second trade after a first trade,
also update the reserves: the full input enters the pool and output leaves it.

Let:
- `a` be the amount supplied,
- `x` be the input reserve,
- `y` be the output reserve,
- `f` be the fee in whole basis points,
- `m = 10000 - f`.

One basis point is 0.01 percentage points: 30 bps is 0.30%.

**Exact input:** “If I spend this much, how much do I receive?”

```text
output = floor(a * m * y / (x * 10000 + a * m))
```

**Exact output:** “If I need this much output, how much must I spend?”
Here `a` represents the desired output:

```text
input = ceil(x * a * 10000 / ((y - a) * m))
```

Rounding down prevents promising a fractional unit of output. Rounding up
prevents asking for too little input. For positive integers, ceiling division
can be written as `(numerator + denominator - 1) / denominator`. Adding one
unconditionally after division would be wrong when the division is exact.

## Follow the Rust types

Consider this signature:

```rust
pub fn get_amount_out(
    amount_in: u128,
    reserve_in: u128,
    reserve_out: u128,
    fee_bps: u16,
) -> Result<u128, QuoteError>
```

- `pub` exposes the function outside its module.
- The three `u128` values are unsigned 128-bit integers. Passing these values
  copies them because primitive integers implement `Copy`.
- `fee_bps` is a whole-number fee, so pricing needs no floating point.
- `Result<u128, QuoteError>` is the function's return type. Success is
  `Ok(amount)`; an expected failure is `Err(reason)`.

`QuoteError` is our enum. Each variant names a failure the caller may handle:
zero amount, insufficient liquidity, invalid fee, or a result beyond `u128`.
It is separate from the standard `Result` enum that carries success or failure.
`#[derive(Debug, Clone, Copy, PartialEq, Eq)]` asks the compiler to generate
implementations used for diagnostics and comparisons. We implement
`Display` for readable messages and `std::error::Error` so callers can
propagate this error alongside other error types.

The `fee_multiplier` helper is private. Both public functions check their
inputs before doing arithmetic. The fee must be less than 10,000 bps.
Zero fees are supported for calculation/tests even though the original
application configuration requires a positive margin. Negative amounts
cannot be represented by this API; text validation comes in a later step.

## Why a second integer type?

A `u128` can hold approximately 3.4 * 10^38. Two individually valid token
amounts multiplied together can exceed that limit. Our formulas multiply
several values before dividing, even when the final answer is small.

We convert inputs to `num_bigint::BigUint` for intermediate calculations.
It is an unsigned, arbitrary-precision integer stored using heap memory.
It avoids fixed-width intermediate overflow; it is still limited by available
memory. We keep `u128` at the public boundary rather than accepting unbounded
network amounts. Exact-output input can exceed `u128`; checked conversion
then returns `QuoteError::AmountOverflow`.

In `&amount_with_fee * ...`, `&` borrows the big integer so multiplication
does not consume our owned value; we use it again in the denominator.
Unlike `u128`, `BigUint` is not `Copy`. Its arithmetic operators are
provided by trait implementations. These are ordinary Rust operators applied
to a library type, not special syntax for large numbers.

`u128::try_from(...)` attempts a checked conversion. `.map_err(...)` turns
the library's conversion error into our public `QuoteError`. This is one
way to keep library implementation details out of a function's interface.

Official reference:
[num-bigint](https://docs.rs/num-bigint/0.4.6/num_bigint/).

## What does the caller do with Result?

In `main.rs`:

```rust
let amount_out = get_amount_out(amount_in, reserve_in, reserve_out, fee_bps)?;
```

The function call immediately returns `Result<u128, QuoteError>`. The `?`
operator extracts the value from `Ok`, or returns early with the error.
Therefore `amount_out` has type `u128`.

There is no future and no `.await` here: both pricing functions are
synchronous calculations.

The executable's return type is `Result<(), Box<dyn std::error::Error>>`.
`()` is the unit type, appropriate when success has no further value to
return. The boxed error lets this small executable propagate errors through
the standard error trait. You can study that return type more deeply later;
the library keeps its concrete `QuoteError`.

## Tests and comparison with TypeScript

Seven integration tests exercise the library through its public API.
`tests/amm.rs` is outside `src/`, so Cargo builds it as a separate test
crate. That is why it imports `near_intents_solver::amm`.

The tests cover the original examples and reverse trades, exact and
non-exact division, zero output from a tiny trade, empty reserves, requests
that exhaust reserves, fee boundaries, and large products. A small exhaustive
grid checks that the input calculated for an exact output is sufficient and
that one less input unit would be insufficient.

`tests/fixtures/pricing.tsv` records **79 cases** generated by running the
actual original TypeScript functions with `big.js` 6.2.2. Rust matches
**77 cases**. Two cases deliberately record a difference:

| Direction | Amount | Input reserve | Output reserve | TypeScript | Rust |
| --- | --- | --- | --- | --- | --- |
| Exact input | 10^30 | 1 | 2 | 2 | 1 |
| Exact output | 1 | 1 | 10^30 | 0 | 1 |

The original `big.js` defaults round division to 20 decimal places using
half-up rounding, *before* the explicit integer rounding. A value just below
2 can first become 2; a tiny positive value can first become 0. Rust applies
floor/ceiling directly to the exact integer ratio.

This is an intentional arithmetic correction, so this checkpoint does
**not** claim complete bit-for-bit equivalence with the TypeScript.
The regression fixture preserves both results. Later quote assembly must
still validate and decline unsuitable quotes; the table describes the pure
pricing functions, not a claim that the original application submits these
particular quotes.

The remaining interface choices are also explicit: whole integer amounts up
to `u128`, whole basis-point fees, and rejection of fees of 100% or more.
The TypeScript helper itself accepts broader decimal inputs and its
configuration computes `Math.floor(marginPercent * 100)`. Our later
configuration layer must convert percentage text to basis points deliberately.

Optional reference regeneration (Node 24+):

```sh
# From the repository root, install only the reference dependency separately.
npm install --prefix /tmp/solver-pricing-reference --ignore-scripts --no-audit --no-fund big.js@6.2.2
node rust/tools/pricing-reference.cjs /tmp/solver-pricing-reference/node_modules/big.js
```

The script checks the upstream file's Git blob hash, extracts its two exported
pricing functions, and removes TypeScript annotations using Node's built-in
type stripping. It runs those functions instead of maintaining a hand-copied
TypeScript formula. A separate JavaScript BigInt calculation supplies exact
integer expectations. The checked-in fixture means ordinary Rust tests need
neither Node nor npm.

For the division/rounding mechanism, see the
[big.js API](https://mikemcl.github.io/big.js/#dp). The fixture generator
checks the installed version and its actual defaults.

## Book references

All page numbers below are printed pages in your local editions.

**Rust 1 — primary reading**

- Chapter 6, “Defining an Enum”, p. 106: read `QuoteError` as a type with
  named alternatives.
- Chapter 9, “Recoverable Errors with Result”, p. 171, and “Propagating
  Errors”, p. 175: follow the pricing function's return type and `?`.
- Chapter 11, “How to Write Tests”, p. 222, and “Test Organization”, p. 243:
  compare our integration tests with the book's test modules.
- Chapter 4 for borrowing, and chapter 10 for traits, when inspecting
  borrowed `BigUint` arithmetic.

**Rust 2 — optional depth**

- Chapter 4, “Representing Errors” and “Enumeration”, p. 58: why the pricing
  library returns a concrete enum instead of an opaque message.
- Chapter 4, “Propagating Errors”, p. 62.
- Chapter 6, “The Test Harness”, p. 86: how Cargo executes the tests.

## One exercise

Before running the program with different values, predict the output for
`get_amount_out(10, 10, 100, 0)`. Then calculate the required input for
exactly that output with `get_amount_in`.

Why should the exact-output function return 10 rather than 11 in this case?
The exact-division test is your checkpoint.

Next: step 2 introduces typed requests and responses. Pricing remains a
small synchronous library that can be tested without a server.
