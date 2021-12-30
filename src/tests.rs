use insta::assert_debug_snapshot;

use super::{Pattern, run};

// Map a Vec<T> into a Vec<Option<T>> by wrapping in Some.
macro_rules! map_some {
    ($vec:expr) => {{
        $vec.into_iter().map(Some).collect()
    }}
}

// Assert that the results contain only the given pattern with 100% chance.
macro_rules! assert_only {
    ($results:expr, $pattern:ident) => {{
        assert_eq!($results.len(), 1);
        assert_eq!($results[0].0, Pattern::$pattern);
        assert_eq!($results[0].1, 1.0)
    }}
}

// Assert that the probability of a pattern is higher in one result than another.
macro_rules! assert_gt {
    ($results1:expr, $results2:expr, $pattern:ident) => {{
        let prob1 = $results1.iter()
            .find(|(p, _)| *p == Pattern::$pattern)
            .unwrap().1;
        let prob2 = $results2.iter()
            .find(|(p, _)| *p == Pattern::$pattern)
            .unwrap().1;
        assert!(prob1 > prob2);
    }}
}

#[test]
fn test_decreasing_full() {
    let base_price = 100;
    let prices = vec![
        90, 87, 82, 78,
        74, 69, 66, 61,
        58, 54, 50, 47];
    let results = run(None, base_price, map_some!(prices), true);
    assert_only!(results, Decreasing);
}

#[test]
fn test_decreasing_minimal() {
    let base_price = 100;
    let prices = vec![
        90, 87, 82, 78,
        74, 69, 66, 61];
    let results = run(None, base_price, map_some!(prices), true);
    assert_only!(results, Decreasing);
}

#[test]
fn test_decreasing_partial() {
    let base_price = 100;
    let prices = vec![
        90, 87, 82, 78,
        74, 69, 66];
    let results = run(None, base_price, map_some!(prices), true);
    assert_debug_snapshot!(results);
}

#[test]
fn test_random_full() {
    let base_price = 95;
    let prices = vec![
        102, 127, 112, 112, 97,
        65, 59,
        96, 121,
        57, 53, 43];
    let results = run(None, base_price, map_some!(prices), true);
    assert_only!(results, Random);
}

#[test]
fn test_random_minimal() {
    let base_price = 95;
    let prices = vec![102, 127, 112];
    let results = run(None, base_price, map_some!(prices), true);
    assert_only!(results, Random);
}

#[test]
fn test_random_partial() {
    let base_price = 95;
    let prices = vec![102, 127];
    let results = run(None, base_price, map_some!(prices), true);
    assert_debug_snapshot!(results);
}

#[test]
fn test_small_spike_full() {
    let base_price = 90;
    let prices = vec![
        55, 52, 48, 43, 38,
        90, 89, 135, 170, 165,
        81, 77];
    let results = run(None, base_price, map_some!(prices), true);
    assert_only!(results, SmallSpike);
}

#[test]
fn test_small_spike_minimal() {
    let base_price = 90;
    let prices = vec![55, 52, 48, 43];
    let results = run(None, base_price, map_some!(prices), true);
    assert_only!(results, SmallSpike);
}

#[test]
fn test_small_spike_partial() {
    let base_price = 90;
    let prices = vec![55, 52, 48];
    let results = run(None, base_price, map_some!(prices), true);
    assert_debug_snapshot!(results);
}

#[test]
fn test_large_spike_full() {
    let base_price = 104;
    let prices = vec![
        90, 86,
        128, 165, 455,
        147, 143,
        57, 53, 43, 94, 42];
    let results = run(None, base_price, map_some!(prices), true);
    assert_only!(results, LargeSpike);
}

#[test]
fn test_large_spike_minimal() {
    let base_price = 104;
    let prices = vec![
        90, 86,
        128, 165];
    let results = run(None, base_price, map_some!(prices), true);
    assert_only!(results, LargeSpike);
}

#[test]
fn test_large_spike_partial() {
    let base_price = 104;
    let prices = vec![90, 86];
    let results = run(None, base_price, map_some!(prices), true);
    assert_debug_snapshot!(results);
}

#[test]
fn test_prev_patterns() {
    let base_price = 104;
    let prices: Vec<Option<u32>> = map_some!(vec![90, 86]);
    let results_plain = run(None, base_price, prices.clone(), true);
    let results_ls = run(Some(Pattern::LargeSpike), base_price, prices.clone(), true);
    let results_d = run(Some(Pattern::Decreasing), base_price, prices, true);
    assert_gt!(results_d, results_plain, LargeSpike);
    assert_gt!(results_plain, results_ls, LargeSpike);
}

#[test]
fn test_invalid() {
    // Run the test and ensure there were no matching patterns.
    macro_rules! test {
        ($base_price:expr, $($prices:expr),*) => {{
            let prices = vec![$($prices),*];
            let results = run(None, $base_price, map_some!(prices), true);
            assert!(results.is_empty());
        }}
    }

    // Weird zeroes.
    test!(100, 0);
    test!(0, 100);
    test!(0, 0);

    // Bad patterns.
    test!(100, 200);
    test!(100, 130, 126, 122, 118, 114, 110, 106, 102);
}

#[test]
fn test_missing_prices() {
    let base_price = 90;
    let prices = vec![None, None, Some(48), Some(43)];
    let results = run(None, base_price, prices, true);
    assert_debug_snapshot!(results);
}
