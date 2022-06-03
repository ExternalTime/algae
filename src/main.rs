mod analyzer;
mod corpus_data;
mod generation;
/// Shitty implementation for layout stuff. Doesn't really matter for the generator. It's here to show show that the generator works.
pub mod layout;

pub use analyzer::{Analyzer, AppliedMetric};
pub use corpus_data::{CorpusData, NGrams};
pub use generation::{generate, Translatable};
use layout::{column_set_symmetries, Finger, FingerKind, Hand, Key, Layout};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
pub fn corpus_data<const N: usize>(
    src: &str,
    data_file: &str,
) -> Result<CorpusData<N>, Box<dyn std::error::Error>>
where
    [char; N]: Serialize,
    for<'a> [char; N]: Deserialize<'a>,
{
    use std::fs::File;
    if let Ok(file) = File::open(data_file) {
        Ok(bincode::deserialize_from(file)?)
    } else {
        let corpus = std::fs::read_to_string(src)?;
        let corpus_data: CorpusData<N> = corpus.chars().collect();
        let data_file = File::create(data_file)?;
        bincode::serialize_into(data_file, &corpus_data)?;
        Ok(corpus_data)
    }
}

pub fn finger(key: Key) -> Finger {
    let hand = if key.1 > 4 { Hand::Right } else { Hand::Left };
    let kind = match key.1 {
        1 | 8 => FingerKind::Ring,
        2 | 7 => FingerKind::Middle,
        3 | 4 | 5 | 6 => FingerKind::Index,
        _ => FingerKind::Pinky,
    };
    Finger { hand, kind }
}

pub fn sfb(bigram: [Key; 2]) -> u64 {
    if finger(bigram[0]) == finger(bigram[1]) {
        1
    } else {
        0
    }
}

pub fn chars(char_freq: &NGrams<1>) -> [char; 30] {
    use std::cmp::Reverse;
    let mut res = "abcdefghijklmnopqrstuvwxyz',./".chars().collect::<Vec<_>>();
    res.sort_by_key(|char| Reverse(char_freq[&[*char]]));
    res.try_into().unwrap()
}

pub fn keys() -> [Key; 30] {
    let mut res = Vec::with_capacity(30);
    for i in 0..3 {
        for j in 0..10 {
            res.push(Key(i, j));
        }
    }
    return res.try_into().unwrap();
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading corpus");
    let corpus_data = corpus_data::<3>("iweb-lower.txt", "iweb-lower.bin")?;
    println!("Calculating ngrams");
    let char_occs = corpus_data.ngrams::<1>();
    println!("Getting generation parameters");
    let chars = chars(&char_occs);
    let keys = keys();
    let mut bimetrics: HashMap<_, Box<dyn Fn(_) -> _>> = HashMap::with_capacity(1);
    bimetrics.insert(String::from("sfb"), Box::new(sfb));
    let mut bigram_sets = HashMap::with_capacity(1);
    bigram_sets.insert(String::from("iweb_bigrams"), corpus_data.ngrams::<2>());
    let applied_metrics = [AppliedMetric {
        ngrams: String::from("iweb_bigrams"),
        metric: String::from("sfb"),
        constraint: None,
        weight: 1,
    }];
    let bigram_analyzer = Analyzer {
        ngram_sets: &bigram_sets,
        metrics: &bimetrics,
        applied_metrics,
    };
    let trigram_sets = HashMap::new();
    let trimetrics = HashMap::new();
    let trigram_analyzer = Analyzer {
        ngram_sets: &trigram_sets,
        metrics: &trimetrics,
        applied_metrics: [],
    };
    if let Some(mapping) = generate(
        chars,
        keys,
        &bigram_analyzer,
        &trigram_analyzer,
        &column_set_symmetries,
        15_000_000,
    ) {
        println!("Best layout found:");
        let layout: Layout = mapping.into_iter().collect();
        println!("{}", layout);
    } else {
        println!("Didn't find any layouts under supplied constraints");
    }
    Ok(())
}
