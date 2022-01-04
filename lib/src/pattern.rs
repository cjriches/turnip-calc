/// Possible patterns for the week.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    Decreasing,
    Random,
    SmallSpike,
    LargeSpike,
}

impl Pattern {
    /// Prior probability of this pattern occurring, given last week's pattern.
    pub fn prior(&self, prev: Option<Pattern>) -> f64 {
        match prev {
            None => {
                // We don't know the previous pattern, so respond with the average.
                match self {
                    Pattern::Decreasing => 0.15,
                    Pattern::Random => 0.35,
                    Pattern::SmallSpike => 0.25,
                    Pattern::LargeSpike => 0.25,
                }
            }
            Some(Pattern::Decreasing) => {
                match self {
                    Pattern::Decreasing => 0.05,
                    Pattern::Random => 0.25,
                    Pattern::SmallSpike => 0.25,
                    Pattern::LargeSpike => 0.45,
                }
            }
            Some(Pattern::Random) => {
                match self {
                    Pattern::Decreasing => 0.15,
                    Pattern::Random => 0.20,
                    Pattern::SmallSpike => 0.35,
                    Pattern::LargeSpike => 0.30,
                }
            }
            Some(Pattern::SmallSpike) => {
                match self {
                    Pattern::Decreasing => 0.15,
                    Pattern::Random => 0.45,
                    Pattern::SmallSpike => 0.15,
                    Pattern::LargeSpike => 0.25,
                }
            }
            Some(Pattern::LargeSpike) => {
                match self {
                    Pattern::Decreasing => 0.20,
                    Pattern::Random => 0.50,
                    Pattern::SmallSpike => 0.25,
                    Pattern::LargeSpike => 0.05,
                }
            }
        }
    }
}
