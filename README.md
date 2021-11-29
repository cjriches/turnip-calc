# Turnip Pattern Calculator
This tool calculates the chance of having each turnip pattern given your prices so far (and optionally, last week's pattern).
It is based off of the reverse-engineered turnip code[^1] from New Horizons, which seems to apply to New Leaf as well.

Specifically, the tool calculates the probability that each pattern could have produced the prices you observe, and uses Bayes' theorem to turn this into the probability of each pattern being correct.
Bayes' theorem states:
```
P(A|B) = P(B|A)P(A) / P(B)
```
We can ignore the `P(B)` denominator since we are only making relative comparisons; it cancels out.
Therefore, we need `P(A)`, the prior probability of any given pattern occuring, which is known on average and can be defined precisely if you specify last week's pattern.
We also need `P(B|A)`, the conditional probability of pattern `A` generating prices `B`; this can be calculated by the tool since we know how each pattern works.
Therefore, we can obtain `P(A|B)`, the conditional probability of pattern `A` being correct given observed prices `B`.
This is the output produced by the tool.

Given this method of calculation, the tool is (in theory) perfectly accurate, i.e. mathematically there is no better estimate that could be made with the same input data.

This is a command-line tool written in Rust - you need [Cargo](https://www.rust-lang.org/learn/get-started) to build and run it.
I created it mostly for my own interest - if you want a tool that's significantly easier to use and based on the same information, look [here](https://turnipprophet.io/).

To get started, clone the repository and run `cargo run --release -- --help`.
Example usage:
```
$ cargo run --release -- -l largespike 95 102 127
Analysis:
Random: 92%
SmallSpike: 8%
```

[^1]: The results of the reverse-engineered code can be found [here](https://docs.google.com/document/d/1bSVNpOnH_dKxkAGr718-iqh8s8Z0qQ54L-0mD-lbrXo/edit).
