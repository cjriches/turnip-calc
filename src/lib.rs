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
    fn prior(&self, prev: &Pattern) -> f64 {
        match prev {
            Pattern::Unknown => {
                // We don't know the previous pattern, so respond with the average.
                match self {
                    Pattern::Unknown => panic!("Invalid pattern."),
                    Pattern::Decreasing => 0.15,
                    Pattern::Random => 0.35,
                    Pattern::SmallSpike => 0.25,
                    Pattern::LargeSpike => 0.25,
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
                 prices: impl IntoIterator<Item = u32>) -> Option<HashMap<Pattern, f64>> {
        let mut prices = prices.into_iter();
        let mut chances = HashMap::with_capacity(4);
        chances.insert(Pattern::Decreasing, Pattern::Decreasing.prior(last_week));
        chances.insert(Pattern::Random, Pattern::Random.prior(last_week));
        chances.insert(Pattern::SmallSpike, Pattern::SmallSpike.prior(last_week));
        chances.insert(Pattern::LargeSpike, Pattern::LargeSpike.prior(last_week));

        // Helper macro to normalise the `chances` before returning.
        macro_rules! done {
            () => {{
                let total: f64 = chances.values().sum();
                for chance in chances.values_mut() {
                    *chance /= total;
                }
                return Some(chances);
            }}
        }

        // Helper macro to get the next price, returning if there are no more prices.
        macro_rules! next {
            () => {{
                if let Some(price) = prices.next() {
                    price
                } else {
                    done!();
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

        // Helper macro to check an invariant.
        macro_rules! invariant {
            ($condition:expr) => {{
                if !$condition {
                    return None;
                }
            }}
        }

        // Sanity check the base price.
        invariant!(base_price >= 90);
        invariant!(base_price <= 110);

        // Check the first price.
        // Decreasing starts at 85-90%.
        // Random starts at 90-140% (6/7 of the time) or 60-80% (1/7 of the time).
        // Small starts 40-90% (7/8 of the time) or 90-140% (1/8 of the time).
        // Large starts 85-90%.
        let first = next!();
        invariant!(first >= mult!(0.40));
        if first < mult!(0.60) {
            // If we are in the range 40-60%, this must be a small spike.
            eliminate!(Decreasing);
            eliminate!(Random);
            eliminate!(LargeSpike);
            done!()
        } else if first < mult!(0.80) {
            // If we are in the range 60-80%, this could be random or small spike.
            eliminate!(Decreasing);
            eliminate!(LargeSpike);
            // Random satisfies this only 1/7 of the time.
            *chances.get_mut(&Pattern::Random).unwrap() /= 7.0;
            // Small spike satisfies this only 7/8 of the time.
            *chances.get_mut(&Pattern::SmallSpike).unwrap() *= 7.0 / 8.0;

            // Now inspect the following prices.
            // We expect to decrease by 3-5% each time.
            // This will happen 0-6 times for a spike, or 1-2 times for random.
            let mut min = first as f64 / base_price as f64;
            let mut max = min;
            let mut price = next!();
            min -= 0.05;
            max -= 0.03;
            invariant!(price >= mult!(min));
            if price > mult!(max) {
                // Zero decreases, must be a spike.
                eliminate!(Random);
                done!()
            }
            // At least one decrease.
            *chances.get_mut(&Pattern::SmallSpike).unwrap() *= 5.0 / 6.0;
            // Check if there are exactly one or two decreases.
            let mut decreases = 1;
            for i in 2..=3 {
                price = next!();
                min -= 0.05;
                max -= 0.03;
                invariant!(price >= mult!(min));
                if price > mult!(max) {
                    break;
                }
                let factor = (6 - i) as f64 / (7 - i) as f64;
                *chances.get_mut(&Pattern::SmallSpike).unwrap() *= factor;
                *chances.get_mut(&Pattern::Random).unwrap() /= 2.0;
                decreases = i;
            }

            if decreases > 2 {
                // There were more than two decreases, so must be a spike.
                eliminate!(Random);
                done!()
            }

            // There were one or two decreases, still unclear.
            // This price was the first increase. It will always be 90-140%.
            invariant!(price >= mult!(0.90));
            invariant!(price < mult!(1.40));

            // The next price may go back down if random (1/7 chance),
            // or can be 90-140% again in either case.
            price = next!();
            if price < mult!(0.90) {
                eliminate!(SmallSpike);
                done!()
            } else {
                *chances.get_mut(&Pattern::Random).unwrap() *= 6.0 / 7.0;
            }

            // The next price will tell us for sure. If this is a small spike,
            // it will be at least 140%.
            price = next!();
            if price >= mult!(1.40) {
                eliminate!(Random)
            } else {
                eliminate!(SmallSpike);
            }
            done!()
        } else if first < mult!(0.85) {
            // Only small spike can produce 80-85%.
            eliminate!(Decreasing);
            eliminate!(Random);
            eliminate!(LargeSpike);
            done!()
        } else if first < mult!(0.90) {
            // If we are in the range 85-90%, this could be anything except random.
            eliminate!(Random);
            // Small spike satisfies this only 7/8 of the time.
            *chances.get_mut(&Pattern::SmallSpike).unwrap() *= 7.0 / 8.0;

            // Now inspect the following prices.
            // We expect to decrease by 3-5% each time.
            // This will happen 0-6 times for a spike, or indefinitely for decreasing.
            let mut min = first as f64 / base_price as f64;
            let mut max = min;
            let mut spike = false;
            let mut price = 0;
            for i in 1..=6 {
                min -= 0.05;
                max -= 0.03;
                price = next!();
                invariant!(price >= mult!(min));
                if price > mult!(max) {
                    spike = true;
                    break;
                }
                // Reduce spike chances proportionally on each successive decrease.
                let factor = (6 - i) as f64 / (7 - i) as f64;
                *chances.get_mut(&Pattern::SmallSpike).unwrap() *= factor;
                *chances.get_mut(&Pattern::LargeSpike).unwrap() *= factor;
            }

            if !spike {
                // No spike; this is decreasing.
                eliminate!(SmallSpike);
                eliminate!(LargeSpike);
                done!()
            }

            // Spike! But which one?
            eliminate!(Decreasing);
            invariant!(price >= mult!(0.90));
            invariant!(price < mult!(1.40));
            price = next!();
            invariant!(price >= mult!(0.90));
            if price < mult!(1.40) {
                // Small spike.
                eliminate!(LargeSpike);
            } else {
                // Large spike.
                eliminate!(SmallSpike);
            }
            done!()
        } else if first < mult!(1.40) {
            // If we are in the range 90-140%, it could be random or small spike.
            eliminate!(Decreasing);
            eliminate!(LargeSpike);
            // Random satisfies this only 6/7 of the time.
            *chances.get_mut(&Pattern::Random).unwrap() *= 6.0 / 7.0;
            // Small spike satisfies this only 1/8 of the time.
            *chances.get_mut(&Pattern::SmallSpike).unwrap() /= 8.0;

            // If this is random, the next price is 60-80% with probability 1/6,
            // or 90-140% again with probability 5/6. If this is small spike,
            // the next price is definitely 90-140%.
            let mut price = next!();
            if price < mult!(0.90) {
                // Random.
                eliminate!(SmallSpike);
                done!()
            } else {
                *chances.get_mut(&Pattern::Random).unwrap() *= 5.0 / 6.0;
            }

            // The next price will tell us for sure.
            // Small spike will be at least 140%, random will be at most 140%.
            price = next!();
            if price < mult!(1.40) {
                // Random.
                eliminate!(SmallSpike);
            } else {
                // Small spike.
                eliminate!(Random);
            }
            done!()
        } else {
            // Invalid first price.
            return None;
        }
    }
}
