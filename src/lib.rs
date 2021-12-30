use std::collections::HashMap;

mod node;
mod pattern;

pub use pattern::Pattern;

use node::Node;

/// Run the calculator on the given data, returning a (possibly empty) list
/// of potential patterns and associated probabilities, sorted in descending
/// order of likelihood.
pub fn run(prev_pattern: Option<Pattern>, base_price: u32,
           prices: Vec<Option<u32>>, debug: bool) -> Vec<(Pattern, f64)> {
    // Start off with the base set of pattern nodes.
    let mut nodes = Node::new_set(base_price, prev_pattern);

    if debug {
        println!("\n\nINITIAL:\n{:#?}", nodes);
    }

    // Iterate through all the prices, constructing and traversing the pattern trees.
    for (i, price) in prices.into_iter().enumerate() {
        if debug {
            println!("\n\nITERATION {} price {:?}:", i+1, price);
        }
        let mut new_nodes = Vec::new();
        for node in nodes {
            new_nodes.extend(node.children(price));
        }
        nodes = new_nodes;
        if debug {
            println!("{:#?}", nodes);
        }
    }

    // Aggregate the resulting probabilities.
    let mut probabilities: HashMap<Pattern, f64> = HashMap::with_capacity(4);
    for node in nodes {
        let (pattern, prob) = node.value();
        *probabilities.entry(pattern).or_insert(0.0) += prob;
    }

    // Normalise the distribution.
    let total: f64 = probabilities.values().sum();
    for prob in probabilities.values_mut() {
        *prob /= total;
    }

    // Sort descending.
    let mut results: Vec<(Pattern, f64)> = probabilities.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    return results;
}
