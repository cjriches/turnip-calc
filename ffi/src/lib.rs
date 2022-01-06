#[cfg(feature = "java-bindings")]
pub mod java_bindings;

/// C-compatible pattern representation.
#[repr(u8)]
pub enum Pattern {
    Decreasing = 1,
    Random = 2,
    SmallSpike = 3,
    LargeSpike = 4,
}

impl TryFrom<u8> for Pattern {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == Pattern::Decreasing as u8 {
            Ok(Pattern::Decreasing)
        } else if value == Pattern::Random as u8 {
            Ok(Pattern::Random)
        } else if value == Pattern::SmallSpike as u8 {
            Ok(Pattern::SmallSpike)
        } else if value == Pattern::LargeSpike as u8 {
            Ok(Pattern::LargeSpike)
        } else {
            Err(())
        }
    }
}

impl From<turnip_calc_lib::Pattern> for Pattern {
    fn from(p: turnip_calc_lib::Pattern) -> Self {
        match p {
            turnip_calc_lib::Pattern::Decreasing => Pattern::Decreasing,
            turnip_calc_lib::Pattern::Random => Pattern::Random,
            turnip_calc_lib::Pattern::SmallSpike => Pattern::SmallSpike,
            turnip_calc_lib::Pattern::LargeSpike => Pattern::LargeSpike,
        }
    }
}

impl Into<turnip_calc_lib::Pattern> for Pattern {
    fn into(self) -> turnip_calc_lib::Pattern {
        match self {
            Pattern::Decreasing => turnip_calc_lib::Pattern::Decreasing,
            Pattern::Random => turnip_calc_lib::Pattern::Random,
            Pattern::SmallSpike => turnip_calc_lib::Pattern::SmallSpike,
            Pattern::LargeSpike => turnip_calc_lib::Pattern::LargeSpike,
        }
    }
}

/// C-compatible pattern result representation.
#[repr(C)]
pub struct PatternResult {
    pattern: Pattern,
    probability: f64,
}

impl From<(turnip_calc_lib::Pattern, f64)> for PatternResult {
    fn from((pattern, probability): (turnip_calc_lib::Pattern, f64)) -> Self {
        Self {
            pattern: pattern.into(),
            probability,
        }
    }
}

/// C-compatible representation of the result of running the calculator.
#[repr(C)]
pub struct CalcResult {
    success: bool,
    results: *mut PatternResult,
    num: usize,
}

/// Run the turnip calculator.
/// This produces a CalcResult, ownership of which is transferred to the caller.
/// To free it, you must call `free_result` later.
///
/// If `prev_pattern` is not set to a valid value, it will be treated as unknown.
/// If any price is zero, it will be treated as missing.
#[no_mangle]
pub unsafe extern fn turnip_calc(prev_pattern: u8, base_price: u32,
                                 prices: *const u32, num_prices: usize) -> CalcResult {
    // Convert prev_pattern.
    let prev_pattern: Option<Pattern> = prev_pattern.try_into().ok();

    // Convert prices.
    let mut prices_vec = Vec::with_capacity(num_prices);
    for i in 0..num_prices {
        let price = *prices.offset(i as isize);
        prices_vec.push(if price == 0 {
            None
        } else {
            Some(price)
        });
    }

    // Run calculator, catching any naughty panics.
    let results = std::panic::catch_unwind(move || {
        turnip_calc_lib::run(prev_pattern.map(Into::into), base_price,
                             prices_vec, false)
    });

    // Assemble the result.
    let fail_result = CalcResult {
        success: false,
        results: std::ptr::null_mut(),
        num: 0,
    };
    return match results {
        Ok(results) => {
            if results.is_empty() {
                fail_result
            } else {
                // Convert the data.
                let mut results_converted = Vec::with_capacity(results.len());
                for result in results {
                    results_converted.push(PatternResult::from(result));
                }
                // Release ownership.
                results_converted.shrink_to_fit();  // Should be a no-op, but a good sanity check.
                let vec_ptr = results_converted.as_mut_ptr();
                let num = results_converted.len();
                std::mem::forget(results_converted);
                // Construct the result.
                CalcResult {
                    success: true,
                    results: vec_ptr,
                    num,
                }
            }
        }
        Err(_) => {
            fail_result
        }
    }
}

/// Free a `CalcResult` previously returned from `turnip_calc`.
#[no_mangle]
pub unsafe extern fn free_result(result: CalcResult) {
    if result.num > 0 {
        let vec = Vec::from_raw_parts(result.results, result.num, result.num);
        std::mem::drop(vec);
    }
}
