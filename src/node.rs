use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::pattern::Pattern;

const MAX_HALF_DAYS: i32 = 12;

/// This allows us to swap in different methods for constructing the next Node
/// in a polymorphic fashion.
trait NextNode {
    fn after(&self, prev: &Node, chance: f64) -> Node;
}

/// The simplest NextNode, which only passes on the probability and lengths.
struct SimpleNode {
    after: Node,
}

impl SimpleNode {
    fn new(after: Node) -> Option<Rc<dyn NextNode>> {
        Some(Rc::new(SimpleNode { after }))
    }
}

impl NextNode for SimpleNode {
    fn after(&self, prev: &Node, chance: f64) -> Node {
        let mut after = self.after.clone();

        after.prob *= prev.prob * chance;
        after.lengths = prev.lengths.clone();
        after.lengths.push(prev.length);

        return after;
    }
}

/// A NextNode which sets the lengths from previous lengths.
struct ConditionalLengthNode<F> {
    after: Node,
    length_func: F,
}

impl<F: 'static> ConditionalLengthNode<F>
    where F: Fn(&Vec<i32>) -> (i32, i32)
{
    fn new(after: Node, length_func: F) -> Option<Rc<dyn NextNode>> {
        Some(Rc::new(ConditionalLengthNode { after, length_func }))
    }
}

impl<F> NextNode for ConditionalLengthNode<F>
    where F: Fn(&Vec<i32>) -> (i32, i32)
{
    fn after(&self, prev: &Node, chance: f64) -> Node {
        let mut after = self.after.clone();

        after.prob *= prev.prob * chance;
        after.lengths = prev.lengths.clone();
        after.lengths.push(prev.length);

        let (min_len, max_len) = (self.length_func)(&prev.lengths);
        after.min_len = min_len;
        after.max_len = max_len;

        return after;
    }
}

#[derive(Clone)]
pub struct Node {
    /// The pattern represented by this node.
    pattern: Pattern,
    /// A name for debug identification purposes.
    name: String,
    /// The base price (turnip buying price on Sunday).
    base_price: u32,
    /// The current probability.
    prob: f64,
    /// The minimum length of this phase before the next one.
    min_len: i32,
    /// The maximum length of this phase before the next one.
    max_len: i32,
    /// The minimum factor of the base price to allow.
    min_fac: f64,
    /// The maximum factor of the base price to allow.
    max_fac: f64,
    /// The optional range to decrease `min_fac` and `max_fac` by each iteration.
    decrement: Option<(f64, f64)>,
    /// The current length of this phase.
    length: i32,
    /// The lengths of all previous phases.
    lengths: Vec<i32>,
    /// The node that appears after this one.
    next_node: Option<Rc<dyn NextNode>>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:.2}\n{}\nLength: {}\nRemaining Length: ({}, {})\n\
                   Previous Lengths: {:?}\nFactors: ({}, {})\nDecrement: {:?}",
               self.pattern, self.prob, self.name, self.length, self.min_len,
               self.max_len, self.lengths, self.min_fac, self.max_fac, self.decrement)
    }
}

impl Node {
    /// Get a fresh collection of starting nodes, representing all patterns.
    pub fn new_set(base_price: u32, prev_pattern: Option<Pattern>) -> Vec<Self> {
        let mut nodes = Vec::new();
        nodes.push(Node::decreasing(base_price, prev_pattern));
        nodes.extend(Node::random(base_price, prev_pattern));
        nodes.extend(Node::small_spike(base_price, prev_pattern));
        nodes.push(Node::large_spike(base_price, prev_pattern));
        return nodes;
    }

    /// Get the pattern and probability of this Node.
    pub fn value(&self) -> (Pattern, f64) {
        (self.pattern, self.prob)
    }

    /// Given the next price, what possible children are there?
    pub fn children(self, price: Option<u32>) -> Vec<Self> {
        // Ensure that the price is within the given range.
        if let Some(p) = price {
            if p < self.mult(self.min_fac) || p > self.mult(self.max_fac) {
                return vec![];
            }
        }
        // Adjust for situations where Pattern A could be in 50-100 while Pattern B
        // could be in 60-70; if our observed price is in 60-70, then Pattern B
        // is more likely than Pattern A given no other information.
        let chance = 1.0 / (self.max_fac - self.min_fac);

        // If we're below the minimum length, return the next node.
        if self.min_len > 1 {
            return vec![self.next(price, chance)];
        }

        // If we're between min and max length, branch.
        if self.max_len > 1 {
            return vec![self.next(price, chance), self.after(chance)];
        }

        // If we're at max length, return `after`.
        return vec![self.after(chance)];
    }

    /// Construct a new Decreasing pattern.
    fn decreasing(base_price: u32, prev_pattern: Option<Pattern>) -> Self {
        Node {
            pattern: Pattern::Decreasing,
            name: "Decreasing".into(),
            base_price,
            prob: Pattern::Decreasing.prior(prev_pattern),
            min_len: MAX_HALF_DAYS,
            max_len: MAX_HALF_DAYS,
            min_fac: 0.85,
            max_fac: 0.90,
            decrement: Some((0.03, 0.05)),
            length: 1,
            lengths: vec![],
            next_node: None,
        }
    }

    /// Construct a new Random pattern.
    fn random(base_price: u32, prev_pattern: Option<Pattern>) -> Vec<Self> {
        let final_increasing = ConditionalLengthNode::new(Node {
            pattern: Pattern::Random,
            name: "Final Increasing".into(),
            base_price,
            prob: 1.0,
            min_len: -1,
            max_len: -1,
            min_fac: 0.90,
            max_fac: 1.40,
            decrement: None,
            length: 1,
            lengths: vec![],
            next_node: None,
        }, remaining_length);

        let dec_2_length = |lengths: &Vec<i32>| {
            let dec_1_length = *lengths.get(1).unwrap();
            assert!(dec_1_length == 2 || dec_1_length == 3);
            let length = 5 - dec_1_length;
            (length, length)
        };

        let second_decreasing = ConditionalLengthNode::new(Node {
            pattern: Pattern::Random,
            name: "Second Decreasing".into(),
            base_price,
            prob: 1.0,
            min_len: -1,
            max_len: -1,
            min_fac: 0.60,
            max_fac: 0.80,
            decrement: Some((0.04, 0.10)),
            length: 1,
            lengths: vec![],
            next_node: final_increasing,
        }, dec_2_length);

        let inc_2_length = |lengths: &Vec<i32>| {
            let inc_1_length = *lengths.get(0).unwrap();
            assert!(inc_1_length >= 0 && inc_1_length <= 6);
            let max_length = 7 - inc_1_length;
            (1, max_length)
        };

        let second_increasing = ConditionalLengthNode::new(Node {
            pattern: Pattern::Random,
            name: "Second Increasing".into(),
            base_price,
            prob: 1.0,
            min_len: -1,
            max_len: -1,
            min_fac: 0.90,
            max_fac: 1.40,
            decrement: None,
            length: 1,
            lengths: vec![],
            next_node: second_decreasing,
        }, inc_2_length);

        let mut initial_decreasing = Node {
            pattern: Pattern::Random,
            name: "Initial Decreasing".into(),
            base_price,
            prob: 1.0,
            min_len: 2,
            max_len: 3,
            min_fac: 0.60,
            max_fac: 0.80,
            decrement: Some((0.04, 0.10)),
            length: 1,
            lengths: vec![],
            next_node: second_increasing,
        };

        let prior = Pattern::Random.prior(prev_pattern);

        let initial_increasing = Node {
            pattern: Pattern::Random,
            name: "Initial Increasing".into(),
            base_price,
            prob: prior * 6.0 / 7.0,
            min_len: 1,
            max_len: 6,
            min_fac: 0.90,
            max_fac: 1.40,
            decrement: None,
            length: 1,
            lengths: vec![],
            next_node: SimpleNode::new(initial_decreasing.clone()),
        };

        initial_decreasing.prob = prior / 7.0;
        initial_decreasing.lengths.push(0);

        return vec![initial_increasing, initial_decreasing];
    }

    /// Construct a new Small Spike pattern.
    fn small_spike(base_price: u32, prev_pattern: Option<Pattern>) -> Vec<Self> {
        let final_decreasing = ConditionalLengthNode::new(Node {
            pattern: Pattern::SmallSpike,
            name: "Final Decreasing".into(),
            base_price,
            prob: 1.0,
            min_len: -1,
            max_len: -1,
            min_fac: 0.40,
            max_fac: 0.90,
            decrement: Some((0.03, 0.05)),
            length: 1,
            lengths: vec![],
            next_node: None,
        }, remaining_length);

        let mut spike =
            Node::chain(Pattern::SmallSpike, "Spike", base_price,
                        final_decreasing, &vec![
                    (0.90, 1.40), (0.90, 1.40),
                    (1.40, 2.00), (1.40, 2.00), (1.40, 2.00)
                ]);

        let prior = Pattern::SmallSpike.prior(prev_pattern);

        let initial_decreasing = Node {
            pattern: Pattern::SmallSpike,
            name: "Initial Decreasing".into(),
            base_price,
            prob: prior * 7.0 / 8.0,
            min_len: 1,
            max_len: 7,
            min_fac: 0.40,
            max_fac: 0.90,
            decrement: Some((0.03, 0.05)),
            length: 1,
            lengths: vec![],
            next_node: SimpleNode::new(spike.clone()),
        };

        spike.prob = prior / 8.0;

        return vec![initial_decreasing, spike];
    }

    /// Construct a new Large Spike pattern.
    fn large_spike(base_price: u32, prev_pattern: Option<Pattern>) -> Self {
        let final_decreasing = ConditionalLengthNode::new(Node {
            pattern: Pattern::LargeSpike,
            name: "Final Decreasing".into(),
            base_price,
            prob: 1.0,
            min_len: -1,
            max_len: -1,
            min_fac: 0.40,
            max_fac: 0.90,
            decrement: None,
            length: 1,
            lengths: vec![],
            next_node: None,
        }, remaining_length);

        let spike = SimpleNode::new(
            Node::chain(Pattern::LargeSpike, "Spike", base_price,
                        final_decreasing, &vec![
                    (0.90, 1.40), (1.40, 2.00), (2.00, 6.00),
                    (1.40, 2.00), (0.90, 1.40)
                ]));

        let initial_decreasing = Node {
            pattern: Pattern::LargeSpike,
            name: "Initial Decreasing".into(),
            base_price,
            prob: Pattern::LargeSpike.prior(prev_pattern),
            min_len: 1,
            max_len: 7,
            min_fac: 0.85,
            max_fac: 0.90,
            decrement: Some((0.03, 0.05)),
            length: 1,
            lengths: vec![],
            next_node: spike,
        };

        return initial_decreasing;
    }

    /// Construct a chain of nodes all with the given pattern and base price.
    /// The final node in the chain will have the given `after`, and their factors will
    /// be set according to the supplied vector.
    fn chain(pattern: Pattern, name: &str, base_price: u32,
             after: Option<Rc<dyn NextNode>>, factors: &Vec<(f64, f64)>) -> Self {
        assert!(factors.len() > 0);

        // Do the last factor.
        let (min_fac, max_fac) = factors.last().unwrap();  // Unwrap guaranteed safe by initial assert.
        let mut node = Node {
            pattern,
            name: name.into(),
            base_price,
            prob: 1.0,
            min_len: 1,
            max_len: 1,
            min_fac: *min_fac,
            max_fac: *max_fac,
            decrement: None,
            length: 1,
            lengths: vec![],
            next_node: after,
        };

        // Now iterate over the rest.
        for (min_fac, max_fac) in factors.iter().rev().skip(1) {
            node = Node {
                pattern,
                name: name.into(),
                base_price,
                prob: 1.0,
                min_len: 1,
                max_len: 1,
                min_fac: *min_fac,
                max_fac: *max_fac,
                decrement: None,
                length: 1,
                lengths: vec![],
                next_node: SimpleNode::new(node),
            };
        }
        return node;
    }

    /// Multiply the base price by the given factor and round up.
    fn mult(&self, factor: f64) -> u32 {
        (self.base_price as f64 * factor).ceil() as u32
    }

    /// Get the next node in this current phase.
    fn next(&self, price: Option<u32>, mut chance: f64) -> Self {
        let (min_fac, max_fac) = match self.decrement {
            Some((dec_min, dec_max)) => {
                match price {
                    Some(p) => {
                        // We have a decrement operation and a known price.

                        // We must be careful of rounding when working back from
                        // the price to the factor: the true factor could be
                        // anywhere from price/base to (price-1)/base.
                        let factor_max = p as f64 / self.base_price as f64;
                        let factor_min = (p-1) as f64 / self.base_price as f64;
                        (factor_min - dec_max, factor_max - dec_min)
                    }
                    None => {
                        // We have a decrement operation but unknown price.
                        (self.min_fac - dec_max, self.max_fac - dec_min)
                    }
                }
            }
            None => {
                // No decrement operation.
                (self.min_fac, self.max_fac)
            }
        };

        // If this is a branch, we must account for the chance of staying with
        // this phase rather than moving to the next one.
        if self.min_len <= 1 && self.max_len > 1 {
            let branch_chance = (self.max_len - 1) as f64 / self.max_len as f64;
            chance *= branch_chance;
        }

        Node {
            pattern: self.pattern,
            name: self.name.clone(),
            base_price: self.base_price,
            prob: self.prob * chance,
            min_len: self.min_len - 1,
            max_len: self.max_len - 1,
            min_fac,
            max_fac,
            decrement: self.decrement,
            length: self.length + 1,
            lengths: self.lengths.clone(),
            next_node: self.next_node.clone(),
        }
    }

    /// Get the node after the current phase.
    fn after(mut self, mut chance: f64) -> Self {
        // If this is a branch, we must account for the chance of moving to the
        // next phase rather than staying with this one.
        if self.min_len <= 1 && self.max_len > 1 {
            let branch_chance = 1.0 / self.max_len as f64;
            chance *= branch_chance;
        }

        return self.next_node
            .take()
            .expect("BUG: Tree terminated early!")
            .after(&self, chance);
    }
}

/// Calculate the remaining number of half-days.
fn remaining_length(lengths: &Vec<i32>) -> (i32, i32) {
    let total: i32 = lengths.iter().sum();
    let remaining = MAX_HALF_DAYS - total;
    (remaining, remaining)
}
