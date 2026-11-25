use near_intents_solver::amm::{QuoteError, get_amount_in, get_amount_out};

#[test]
fn upstream_examples_and_reverse_trades() {
    let output = get_amount_out(10, 10_000, 100_000_000, 30).unwrap();
    assert_eq!(output, 99_600);
    assert_eq!(
        get_amount_out(output, 100_000_000 - output, 10_010, 30),
        Ok(9)
    );

    let input = get_amount_in(10, 100_000_000, 10_000, 30).unwrap();
    assert_eq!(input, 100_402);
    assert_eq!(get_amount_in(input, 9_990, 100_000_000 + input, 30), Ok(11));
}

#[test]
fn rounds_in_the_pool_favor_including_exact_division() {
    assert_eq!(get_amount_out(1, 3, 10, 0), Ok(2)); // 10 / 4
    assert_eq!(get_amount_in(2, 3, 10, 0), Ok(1)); // 6 / 8
    assert_eq!(get_amount_out(10, 10, 100, 0), Ok(50));
    assert_eq!(get_amount_in(50, 10, 100, 0), Ok(10));
    assert_eq!(get_amount_out(1, 1_000_000, 1, 30), Ok(0));
}

#[test]
fn rejects_zero_amounts_empty_pools_and_exhausted_reserves() {
    assert_eq!(get_amount_out(0, 100, 100, 30), Err(QuoteError::ZeroAmount));
    assert_eq!(get_amount_in(0, 100, 100, 30), Err(QuoteError::ZeroAmount));
    for (reserve_in, reserve_out) in [(0, 100), (100, 0), (0, 0)] {
        assert_eq!(
            get_amount_out(1, reserve_in, reserve_out, 30),
            Err(QuoteError::InsufficientLiquidity)
        );
        assert_eq!(
            get_amount_in(1, reserve_in, reserve_out, 30),
            Err(QuoteError::InsufficientLiquidity)
        );
    }
    for amount in [100, 101] {
        assert_eq!(
            get_amount_in(amount, 100, 100, 30),
            Err(QuoteError::InsufficientLiquidity)
        );
    }
}

#[test]
fn validates_fee_boundaries() {
    for fee in [10_000, u16::MAX] {
        assert_eq!(
            get_amount_out(1, 100, 100, fee),
            Err(QuoteError::InvalidFee)
        );
        assert_eq!(get_amount_in(1, 100, 100, fee), Err(QuoteError::InvalidFee));
    }
    assert_eq!(get_amount_out(100, 100, 100, 9_999), Ok(0));
    assert_eq!(get_amount_in(1, 100, 100, 9_999), Ok(10_102));
}

#[test]
fn intermediate_products_can_exceed_u128() {
    let max = u128::MAX;
    assert_eq!(get_amount_out(max, max, max, 0), Ok(max / 2));
    assert_eq!(get_amount_in(max / 2, max, max, 0), Ok(max - 1));
    assert_eq!(
        get_amount_in(max - 1, max, max, 30),
        Err(QuoteError::AmountOverflow)
    );
}

#[test]
fn exact_output_input_is_sufficient_and_minimal_on_small_grid() {
    for reserve_in in 1..=12 {
        for reserve_out in 2..=12 {
            for output in 1..reserve_out {
                for fee in [0, 30, 9_999] {
                    let input = get_amount_in(output, reserve_in, reserve_out, fee).unwrap();
                    assert!(get_amount_out(input, reserve_in, reserve_out, fee).unwrap() >= output);
                    if input > 1 {
                        assert!(
                            get_amount_out(input - 1, reserve_in, reserve_out, fee).unwrap()
                                < output
                        );
                    }
                }
            }
        }
    }
}

/// Columns: direction, amount, reserve_in, reserve_out, fee_bps,
/// big.js result, exact integer result.
#[test]
fn recorded_typescript_parity_and_documented_precision_boundaries() {
    let mut matching = 0;
    let mut differing = 0;
    for line in include_str!("fixtures/pricing.tsv").lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        let amount = fields[1].parse().unwrap();
        let reserve_in = fields[2].parse().unwrap();
        let reserve_out = fields[3].parse().unwrap();
        let fee = fields[4].parse().unwrap();
        let actual = match fields[0] {
            "out" => get_amount_out(amount, reserve_in, reserve_out, fee),
            "in" => get_amount_in(amount, reserve_in, reserve_out, fee),
            _ => panic!("unknown fixture direction"),
        };
        assert_eq!(actual.unwrap().to_string(), fields[6], "{line}");
        if fields[5] == fields[6] {
            matching += 1;
        } else {
            differing += 1;
        }
    }
    assert!(matching >= 10);
    assert_eq!(differing, 2, "keep both big.js precision regression cases");
}
