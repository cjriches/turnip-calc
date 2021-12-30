use std::rc::Rc;

use super::Node;

/// This allows us to swap in different methods for constructing the following phase.
pub trait NodeFactory {
    fn after(&self, prev: &Node, chance: f64) -> Node;
}

/// The simplest NodeFactory, which only passes on the probability and previous lengths.
pub struct SimpleNode {
    after: Node,
}

impl SimpleNode {
    pub fn new(after: Node) -> Option<Rc<dyn NodeFactory>> {
        Some(Rc::new(SimpleNode { after }))
    }
}

impl NodeFactory for SimpleNode {
    fn after(&self, prev: &Node, chance: f64) -> Node {
        let mut after = self.after.clone();

        after.prob *= prev.prob * chance;
        after.lengths = prev.lengths.clone();
        after.lengths.push(prev.length);

        return after;
    }
}

/// A NodeFactory which additionally sets the phase lengths from previous
/// phase lengths via an arbitrary function.
pub struct ConditionalLengthNode<F> {
    after: Node,
    length_func: F,
}

impl<F: 'static> ConditionalLengthNode<F>
    where F: Fn(&Vec<i32>) -> (i32, i32)
{
    pub fn new(after: Node, length_func: F) -> Option<Rc<dyn NodeFactory>> {
        Some(Rc::new(ConditionalLengthNode { after, length_func }))
    }
}

impl<F> NodeFactory for ConditionalLengthNode<F>
    where F: Fn(&Vec<i32>) -> (i32, i32)
{
    fn after(&self, prev: &Node, chance: f64) -> Node {
        let mut after = self.after.clone();

        after.prob *= prev.prob * chance;
        after.lengths = prev.lengths.clone();
        after.lengths.push(prev.length);

        let (min_len, max_len) = (self.length_func)(&after.lengths);
        after.min_len = min_len;
        after.max_len = max_len;

        return after;
    }
}
