use near_intents_solver::amm::{get_amount_in, get_amount_out};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reserve_in = 10_000;
    let reserve_out = 100_000_000;
    let fee_bps = 30; // 0.30%
    let amount_in = 10;
    let amount_out = get_amount_out(amount_in, reserve_in, reserve_out, fee_bps)?;
    println!("Exact input: {amount_in} -> {amount_out} base units (fee: {fee_bps} bps)");

    let required_in = get_amount_in(amount_out, reserve_in, reserve_out, fee_bps)?;
    println!("Exact output: {amount_out} base units requires {required_in} input units");
    Ok(())
}
