use std::collections::HashMap;

/// Possible patterns for the week.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    Unknown,
    Decreasing,
    Random,
    SmallSpike,
    LargeSpike,
}

impl Pattern {
    /// Prior probability of this pattern occurring, given last week's pattern.
    pub fn prior(&self, prev: &Pattern) -> f64 {
        match prev {
            Pattern::Unknown => {
                // We don't know the previous pattern, so respond with the average.
                // These were empirically calculated by running many iterations.
                match self {
                    Pattern::Unknown => panic!("Invalid pattern."),
                    Pattern::Decreasing => 0.1477,
                    Pattern::Random => 0.3504,
                    Pattern::SmallSpike => 0.2542,
                    Pattern::LargeSpike => 0.2477,
                }
            }
            Pattern::Decreasing => {
                match self {
                    Pattern::Unknown => panic!("Invalid pattern."),
                    Pattern::Decreasing => 0.05,
                    Pattern::Random => 0.25,
                    Pattern::SmallSpike => 0.25,
                    Pattern::LargeSpike => 0.45,
                }
            }
            Pattern::Random => {
                match self {
                    Pattern::Unknown => panic!("Invalid pattern."),
                    Pattern::Decreasing => 0.15,
                    Pattern::Random => 0.20,
                    Pattern::SmallSpike => 0.35,
                    Pattern::LargeSpike => 0.30,
                }
            }
            Pattern::SmallSpike => {
                match self {
                    Pattern::Unknown => panic!("Invalid pattern."),
                    Pattern::Decreasing => 0.15,
                    Pattern::Random => 0.45,
                    Pattern::SmallSpike => 0.15,
                    Pattern::LargeSpike => 0.25,
                }
            }
            Pattern::LargeSpike => {
                match self {
                    Pattern::Unknown => panic!("Invalid pattern."),
                    Pattern::Decreasing => 0.20,
                    Pattern::Random => 0.50,
                    Pattern::SmallSpike => 0.25,
                    Pattern::LargeSpike => 0.05,
                }
            }
        }
    }

    /// Guess the pattern from a sequence of prices.
    pub fn guess(last_week: &Pattern, base_price: u32,
                 prices: impl IntoIterator<Item = u32>) -> HashMap<Pattern, f64> {
        let mut prices = prices.into_iter();
        let mut chances = HashMap::with_capacity(4);
        chances.insert(Pattern::Decreasing, Pattern::Decreasing.prior(last_week));
        chances.insert(Pattern::Random, Pattern::Random.prior(last_week));
        chances.insert(Pattern::SmallSpike, Pattern::SmallSpike.prior(last_week));
        chances.insert(Pattern::LargeSpike, Pattern::LargeSpike.prior(last_week));

        // Helper macro to normalise the `chances`.
        macro_rules! normalise {
            () => {{
                let total: f64 = chances.values().sum();
                for chance in chances.values_mut() {
                    *chance /= total;
                }
            }}
        }

        // Helper macro to get the next price, returning if there are no more prices.
        macro_rules! next {
            () => {{
                if let Some(price) = prices.next() {
                    price
                } else {
                    normalise!();
                    return chances;
                }
            }}
        }

        // Helper macro to multiply the base price and round up.
        macro_rules! mult {
            ($factor:expr) => {{
                (base_price as f64 * $factor).ceil() as u32
            }}
        }

        // Helper macro to remove a pattern from consideration.
        macro_rules! eliminate {
            ($pattern:ident) => {{
                chances.remove(&Pattern::$pattern)
                    .expect("Pattern removed twice!");
            }}
        }

        // Check the first price.
        // Decreasing starts 85-90.
        // Random starts 90-140 (6/7) or 60-80 (1/7).
        // Small starts 40-90 (7/8) or 90-140 (1/8).
        // Large starts 85-90.
        let first = next!();
        assert!(first >= mult!(0.40));
        if first < mult!(0.60) {
            // If we are in the range 40-60, this must be a small spike.
            eliminate!(Decreasing);
            eliminate!(Random);
            eliminate!(LargeSpike);
            normalise!();
            return chances;
        } else if first < mult!(0.85) {
            // If we are in the range 60-85, this must be random.
            eliminate!(Decreasing);
            eliminate!(SmallSpike);
            eliminate!(LargeSpike);
            normalise!();
            return chances;
        } else if first < mult!(0.90) {
            // If we are in the range 85-90, this could be anything except random.
            eliminate!(Random);
            // Decreasing always satisfies this.
            // Small spike satisfies this 7/8 of the time.
            *chances.get_mut(&Pattern::SmallSpike).unwrap() *= 7.0 / 8.0;
            // Large spike always satisfies this.

            // Now inspect the following prices.
            // We expect to decrease by 3-5% each price.
            // This will happen 0-6 times for a spike, or indefinitely for decreasing.
            let mut min = first as f64 / base_price as f64;
            let mut max = min;
            let mut spike = false;
            let mut price = 0;
            for i in 1..=6 {
                min -= 0.05;
                max -= 0.03;
                price = next!();
                assert!(price >= mult!(min));
                if price > mult!(max) {
                    spike = true;
                    break;
                }
                // Reduce spike chances.
                let factor = (6 - i) as f64 / (7 - i) as f64;
                *chances.get_mut(&Pattern::SmallSpike).unwrap() *= factor;
                *chances.get_mut(&Pattern::LargeSpike).unwrap() *= factor;
            }

            if !spike {
                // No spike; this is decreasing.
                eliminate!(SmallSpike);
                eliminate!(LargeSpike);
                normalise!();
                return chances;
            }

            // Spike! But which one?
            eliminate!(Decreasing);
            assert!(price >= mult!(0.90));
            assert!(price < mult!(1.40));
            price = next!();
            assert!(price >= mult!(0.90));
            if price < mult!(1.40) {
                // Small spike.
                eliminate!(LargeSpike);
            } else {
                // Large spike.
                eliminate!(SmallSpike);
            }
            normalise!();
            return chances;
        } else if first < mult!(1.40) {
            // If we are in the range 90-140, it could be random or small spike.
            eliminate!(Decreasing);
            eliminate!(LargeSpike);
            // Random satisfies this 6/7 of the time.
            *chances.get_mut(&Pattern::Random).unwrap() *= 6.0 / 7.0;
            // Small spike satisfies this 1/8 of the time.
            *chances.get_mut(&Pattern::SmallSpike).unwrap() /= 8.0;

            // If this is random, the next price is 60-80 with probability 1/6,
            // or 90-140 again with probability 5/6. If this is small spike,
            // the next price is definitely 90-140.
            let mut price = next!();
            if price < mult!(0.90) {
                // Random.
                eliminate!(SmallSpike);
                normalise!();
                return chances;
            } else {
                *chances.get_mut(&Pattern::Random).unwrap() *= 5.0 / 6.0;
            }

            // The next price will tell us for sure.
            // Small spike will be at least 140, random will be at most 140.
            price = next!();
            if price < mult!(1.40) {
                // Random.
                eliminate!(SmallSpike);
            } else {
                // Small spike.
                eliminate!(Random);
            }
            normalise!();
            return chances;
        } else {
            unreachable!();
        }
    }
}
