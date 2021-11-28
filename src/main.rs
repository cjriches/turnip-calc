use clap::{Arg, App, app_from_crate, crate_authors, crate_description,
           crate_name, crate_version, ErrorKind, value_t, values_t};

use turnip_calc::Pattern;

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
        this tool mirrors its calculations.")
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
            .help("The observed sell prices in order.")
            .takes_value(true)
            .required(false)
            .multiple(true)
            .min_values(0)
            .max_values(12))
}

fn main() {
    let args = cli().get_matches();
    let last_week = match args.value_of(LAST_WEEK).map(str::to_ascii_lowercase) {
        Some(val) => {
            match val.as_str() {
                DECREASING => Pattern::Decreasing,
                RANDOM => Pattern::Random,
                SMALL_SPIKE => Pattern::SmallSpike,
                LARGE_SPIKE => Pattern::LargeSpike,
                _ => unreachable!(),
            }
        }
        None => Pattern::Unknown,
    };
    let base_price = value_t!(args, BASE_PRICE, u32).unwrap_or_else(|e| e.exit());
    let prices = values_t!(args, PRICES, u32).unwrap_or_else(|e| {
        match e.kind {
            ErrorKind::ArgumentNotFound => Vec::new(),
            _ => e.exit(),
        }
    });

    let mut results: Vec<(Pattern, f64)> = match Pattern::guess(&last_week, base_price, prices) {
        Some(r) => r.into_iter().collect(),
        None => {
            println!("Invalid pattern. Either your numbers are wrong or there is a bug.");
            return;
        }
    };
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("Analysis:");
    for (pattern, chance) in results.iter() {
        println!("{:?}: {:.0}%", pattern, chance * 100.0);
    }
}
