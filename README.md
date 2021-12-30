# Turnip Pattern Calculator
This tool calculates the chance of having each turnip pattern given your prices so far (and optionally, last week's pattern).
It is based off of the reverse-engineered turnip code[^1] from New Horizons, which seems to apply to New Leaf as well.

The tool calculates the probability that each pattern could have produced the prices you observe, and uses Bayes' theorem to turn this into the probability of each pattern being correct.
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

[^1]: The results of the reverse-engineered code can be found [here](https://docs.google.com/document/d/1bSVNpOnH_dKxkAGr718-iqh8s8Z0qQ54L-0mD-lbrXo/edit).

## Usage
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

## Inner Workings
There are four price patterns: Decreasing, Random, Small Spike, and Large Spike.
Each pattern has a number of phases, each of which has different price behaviour, and lasts for a potentially variable amount of time.

For example, the initial decreasing phase of a Large Spike pattern first sets a price within 85-90% of the base (Sunday) price, and then decreases it between 3-5% each half day, lasting 1-7 half days in total.
The following increasing phase sets a price between 90-140%, then a price between 140-200%, then a price between 200-600%.

We can think of this as a tree of possibilities:
```
D--|--D--|--D--|--D--|--D--|--D--|--D-----I
   |     |     |     |     |     |
   |--I  |--I  |--I  |--I  |--I  |--I
```

We start off with a guaranteed Decreasing node, then have six branches where we may continue decreasing, or move to an Increasing node.
If after all six branches we are still on the decreasing path, we must move to an Increasing node.

We can define the possible prices at each node, and then compare our observed prices against these, traversing the tree.
If a child node is compatible with the next price, we will add it to our processing queue, exploring the tree in a breadth-first manner.

At any point we can stop, total up the probabilities of all current nodes, and calculate the probability of each pattern.

If we match multiple children we will explore them all, potentially leading to an exponential number of nodes under consideration.
We have four trees, each of which never branches with degree higher than two and has maximum depth 12.
This gives us around 16,000 possible nodes, most of which will be pruned by failing to match the prices.
Thus, despite the exponential worst-case, the algorithm still runs in very reasonable time. 
