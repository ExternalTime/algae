use super::{NDArray, SparseTensor};
use crate::generation::{Encoding, PartialLayout, Translatable};
use std::collections::HashMap;

pub struct Analyzer<const N: usize, const A: usize, const D: usize> {
    ngram_sets: Vec<SparseTensor<N, D>>,
    metrics: Vec<NDArray<N, D>>,
    applied_metrics: [(usize, usize, u64, u64); A],
}

impl<const N: usize, const A: usize, const D: usize> Analyzer<N, A, D> {
    pub fn new<Key>(
        chars: &Encoding<char, N>,
        keys: &Encoding<Key, N>,
        analyzer: &crate::Analyzer<Key, A, D>,
    ) -> Self
    where
        Key: Translatable,
    {
        let mut ngram_translation = HashMap::new();
        let mut metric_translation = HashMap::new();
        let mut ngram_sets = Vec::new();
        let mut metrics = Vec::new();
        let mut applied_metrics = Vec::new();
        for (i, (key, ngrams)) in analyzer.ngram_sets.iter().enumerate() {
            ngram_translation.insert(key, i);
            ngram_sets.push(SparseTensor::new(chars, |ngram| {
                *ngrams.get(&ngram).unwrap_or(&0)
            }));
        }
        for (i, (key, metric)) in analyzer.metrics.iter().enumerate() {
            metric_translation.insert(key, i);
            metrics.push(NDArray::new(keys, metric));
        }
        for applied_metric in analyzer.applied_metrics.iter() {
            applied_metrics.push((
                *ngram_translation
                    .get(&applied_metric.ngrams)
                    .expect("corpus should be defined before being used"),
                *metric_translation
                    .get(&applied_metric.metric)
                    .expect("metri should be defined before being used"),
                applied_metric.constraint.unwrap_or(u64::MAX),
                applied_metric.weight,
            ));
        }
        let applied_metrics = applied_metrics.try_into().unwrap();
        Self {
            ngram_sets,
            metrics,
            applied_metrics,
        }
    }

    pub fn init_score(&self) -> [u64; A] {
        [0; A]
    }

    pub fn score(&self, layout: &PartialLayout<N>, score: &mut [u64; A]) -> bool {
        for (i, &(ngrams, metric, constraint, _)) in self.applied_metrics.iter().enumerate() {
            for (ngram, occurences) in self.ngram_sets[ngrams][layout.len() - 1].iter() {
                score[i] += self.metrics[metric][ngram.map(|char| layout[char])] * occurences;
                if constraint < score[i] {
                    return false;
                }
            }
        }
        true
    }

    pub fn weighted_sum(&self, score: &[u64; A]) -> u64 {
        let mut total = 0;
        for (score, (_, _, _, weight)) in score.iter().zip(self.applied_metrics.iter()) {
            total += score * weight;
        }
        total
    }
}
