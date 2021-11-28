# Turnip Pattern Calculator
This tool calculates the chance of having each turnip pattern given your prices so far (and optionally, last week's pattern).
It is based off of the reverse-engineered turnip code[^1] from New Horizons, which seems to apply to New Leaf as well from my own observations.

This is a command-line tool written in Rust - you need [Cargo](https://www.rust-lang.org/learn/get-started) to build and run it.
I created it mostly for my own interest - if you want a tool that's significantly easier to use and equally accurate, look [here](https://turnipprophet.io/).

To get started, clone the repository and run `cargo run --release -- --help`.
Example usage:
```
$ cargo run --release -- -l largespike 95 102 127
Analysis:
Random: 92%
SmallSpike: 8%
```

[^1]: The results of the reverse-engineered code can be found [here](https://docs.google.com/document/d/1bSVNpOnH_dKxkAGr718-iqh8s8Z0qQ54L-0mD-lbrXo/edit).
