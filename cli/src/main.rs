use clap::{Arg, App, app_from_crate, crate_authors, crate_description,
           crate_name, crate_version, Error, value_t, Values};

use turnip_calc_lib::Pattern;

// Pattern names.
const DECREASING: &str = "decreasing";
const RANDOM: &str = "random";
const SMALL_SPIKE: &str = "smallspike";
const LARGE_SPIKE: &str = "largespike";
const PATTERNS: [&str; 4] = [DECREASING, RANDOM, SMALL_SPIKE, LARGE_SPIKE];

// Argument names.
const LAST_WEEK: &str = "last_week";
const BASE_PRICE: &str = "BASE_PRICE";
const PRICES: &str = "PRICES";
const DEBUG: &str = "DEBUG";

// Argument values.
const MISSING_PRICE: &str = "?";

fn cli() -> App<'static, 'static> {
    // Hack to make the build dirty when the toml changes.
    include_str!("../Cargo.toml");

    app_from_crate!()
        .after_help("This tool calculates the chance of each pattern based on \
        observed turnip prices. Specifying last week's pattern will increase the \
        accuracy of results, since the previous pattern affects the chance of \
        the next one.\n\n\
        In theory, this tool is 100% accurate. It should always list precisely \
        the remaining possible patterns, with the most accurate estimate of \
        probability that one can theoretically calculate. This assumes that \
        the reverse-engineered turnip code from New Horizons is accurate, as \
        this tool mirrors its calculations.\n\n\
        Example usage: turnip-calc 90 --last-week smallspike 55 52 ? 43")
        .max_term_width(80)
        .arg(Arg::with_name(LAST_WEEK)
            .help("Last week's pattern.")
            .short("l")
            .long("last-week")
            .takes_value(true)
            .possible_values(&PATTERNS)
            .case_insensitive(true))
        .arg(Arg::with_name(BASE_PRICE)
            .help("The price you bought turnips for.")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name(PRICES)
            .help("The sell prices observed so far in order. \
                   Missed prices can be replaced with '?'.")
            .takes_value(true)
            .required(false)
            .multiple(true)
            .min_values(0)
            .max_values(12))
        .arg(Arg::with_name(DEBUG)
            .help("Enable debug dumps.")
            .short("d")
            .long("debug")
            .takes_value(false))
}

fn main() {
    let args = cli().get_matches();
    let last_week = args.value_of(LAST_WEEK).map(|val| {
        match val.to_ascii_lowercase().as_str() {
            DECREASING => Pattern::Decreasing,
            RANDOM => Pattern::Random,
            SMALL_SPIKE => Pattern::SmallSpike,
            LARGE_SPIKE => Pattern::LargeSpike,
            _ => unreachable!(),
        }
    });
    let base_price = value_t!(args, BASE_PRICE, u32).unwrap_or_else(|e| e.exit());
    let prices = match args.values_of(PRICES) {
        Some(args) => parse_prices(args),
        None => Vec::new(),
    };
    let debug = args.is_present(DEBUG);

    let results = turnip_calc_lib::run(last_week, base_price, prices, debug);
    if results.is_empty() {
        println!("These prices did not match any known pattern. Either your \
                  numbers are wrong, or there is a bug.");
        return;
    }

    println!("Analysis:");
    for (pattern, chance) in results.iter() {
        println!("{:?}: {:.0}%", pattern, chance * 100.0);
    }
}

fn parse_prices(args: Values) -> Vec<Option<u32>> {
    let mut prices = Vec::with_capacity(args.len());
    for arg in args {
        if arg == MISSING_PRICE {
            prices.push(None);
        } else {
            let price = match arg.parse::<u32>() {
                Ok(p) => p,
                _ => {
                    let msg = format!(
                        "The argument '{}' should be a non-negative integer or \
                         the character '?'", arg);
                    let err = Error::value_validation_auto(msg);
                    err.exit();
                }
            };
            prices.push(Some(price));
        }
    }
    return prices;
}
