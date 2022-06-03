use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

pub type NGrams<const N: usize> = HashMap<[char; N], u64>;

/// Stats of a corpus. Can be used to calculate occurences of ngrams up to N long.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(serialize = "[char; N]: Serialize"))]
#[serde(bound(deserialize = "[char; N]: Deserialize<'de>"))]
pub struct CorpusData<const N: usize> {
    ngrams: NGrams<N>,
    // Remembers which ngram is last (it's contained in both maps).
    // Helps calculate ngrams accurately for ones shorter than N.
    // Especially important when dealing with many short texts.
    // Would have stored N-1 but for now compiler doesn't let me.
    tails: NGrams<N>,
}

impl<const N: usize> CorpusData<N> {
    /// Creates empty `CorpusData`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Adds occurences from supplied text onto self. Panics if text is shorter than N.
    pub fn add(&mut self, mut iter: impl Iterator<Item = char>) {
        let mut deq = VecDeque::with_capacity(2 * N + 2);
        (0..N)
            .flat_map(|_| iter.next())
            .for_each(|c| deq.push_back(c));
        for c in iter {
            let ngram: [char; N] = deq
                .make_contiguous()
                .try_into()
                .expect("deque should hold constant nuber of chars");
            *self.ngrams.entry(ngram).or_insert(0) += 1;
            deq.pop_front();
            deq.push_back(c);
        }
        let tail: [char; N] = deq
            .make_contiguous()
            .try_into()
            .expect("attempted to calculate ngrams for text shorter than N");
        *self.tails.entry(tail).or_insert(0) += 1;
        *self.ngrams.entry(tail).or_insert(0) += 1;
    }
    /// Merges other `CorpusData` onto self.
    pub fn append(&mut self, other: &Self) {
        for (&ngram, &occurences) in other.ngrams.iter() {
            *self.ngrams.entry(ngram).or_insert(0) += occurences;
        }
        for (&tail, &occurences) in other.tails.iter() {
            *self.tails.entry(tail).or_insert(0) += occurences;
        }
    }

    /// Returns it's own ngrams.
    pub fn into_ngrams(self) -> NGrams<N> {
        self.ngrams
    }

    // Calculates ngrams of length K
    pub fn ngrams<const K: usize>(&self) -> NGrams<K> {
        assert!(K <= N);
        let mut ngrams = NGrams::new();
        for (ngram, occurences) in self.ngrams.iter() {
            *ngrams.entry(ngram[..K].try_into().unwrap()).or_insert(0) += occurences;
        }
        for (ngram, occurences) in self.tails.iter() {
            for i in 1..(N - K) {
                *ngrams
                    .entry(ngram[i..][..K].try_into().unwrap())
                    .or_insert(0) += occurences;
            }
        }
        ngrams
    }
}

impl<const N: usize> FromIterator<char> for CorpusData<N> {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut res = Self::new();
        res.add(iter.into_iter());
        res
    }
}
