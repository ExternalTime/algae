use crate::NGrams;
use std::collections::HashMap;

pub type Metric<Key, const D: usize> = Box<dyn Fn([Key; D]) -> u64>;

pub struct AppliedMetric {
    pub ngrams: String,
    pub metric: String,
    pub constraint: Option<u64>,
    // weight is applied after constraint, which makes it possible
    // to have metric that doesn't contribute towards score, but
    // acts as a constraint.
    pub weight: u64,
}

pub struct Analyzer<'a, Key, const A: usize, const D: usize> {
    pub ngram_sets: &'a HashMap<String, NGrams<D>>,
    pub metrics: &'a HashMap<String, Metric<Key, D>>,
    pub applied_metrics: [AppliedMetric; A],
}
