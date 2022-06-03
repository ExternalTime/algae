use super::{Analyzer, Encoding, PartialLayout, Symmetries, Translatable};

const DBG: usize = 0;

pub struct Generator<'a, const A2: usize, const A3: usize, const N: usize> {
    stack: Vec<(PartialLayout<N>, [u64; A2], [u64; A3])>,
    bigram_analyzer: &'a Analyzer<N, A2, 2>,
    trigram_analyzer: &'a Analyzer<N, A3, 3>,
    symmetries: &'a Symmetries<N>,
    max: u64,
    dbg: [usize; DBG],
}

impl<'a, const A2: usize, const A3: usize, const N: usize> Generator<'a, A2, A3, N> {
    pub fn new(
        init: PartialLayout<N>,
        bigram_analyzer: &'a Analyzer<N, A2, 2>,
        trigram_analyzer: &'a Analyzer<N, A3, 3>,
        symmetries: &'a Symmetries<N>,
        max: u64,
    ) -> Self {
        Self {
            stack: vec![(
                init,
                bigram_analyzer.init_score(),
                trigram_analyzer.init_score(),
            )],
            bigram_analyzer,
            trigram_analyzer,
            symmetries,
            max,
            dbg: [0; DBG],
        }
    }

    pub fn set_max(&mut self, max: u64) {
        self.max = max;
    }
}

impl<const A2: usize, const A3: usize, const N: usize> Iterator for Generator<'_, A2, A3, N> {
    type Item = ([usize; N], u64);
    fn next(&mut self) -> Option<([usize; N], u64)> {
        while let Some((layout, bigram_score, trigram_score)) = self.stack.pop() {
            let len = layout.len();
            if len < DBG {
                self.dbg[len] = self.stack.len();
                print!("\r");
                for i in 0..DBG {
                    print!("{:03} ", self.dbg[i]);
                }
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
            let total_score = self.bigram_analyzer.weighted_sum(&bigram_score)
                + self.trigram_analyzer.weighted_sum(&trigram_score);
            if total_score < self.max {
                match layout.try_complete() {
                    Ok(layout) => return Some((layout, total_score)),
                    Err(incomplete) => {
                        for child in incomplete.children(self.symmetries) {
                            let (mut bigram_score, mut trigram_score) =
                                (bigram_score, trigram_score);
                            if self.bigram_analyzer.score(&child, &mut bigram_score)
                                && self.trigram_analyzer.score(&child, &mut trigram_score)
                            {
                                self.stack.push((child, bigram_score, trigram_score));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

use crate::Analyzer as UAnalyzer;

/// Before using, make sure that characters are sorted in decreasing order of frequency. No idea how to handle this when using multiple corpora.
pub fn generate<Key, const A2: usize, const A3: usize, const N: usize>(
    chars: [char; N],
    keys: [Key; N],
    bigram_analyzer: &UAnalyzer<Key, A2, 2>,
    trigram_analyzer: &UAnalyzer<Key, A3, 3>,
    symmetries: &dyn Fn(Key) -> Option<Key>,
    max_score: u64,
) -> Option<[(Key, char); N]>
where
    Key: Translatable,
{
    println!("Translating data");
    let (key_encoding, symmetries) = Symmetries::new(keys, symmetries);
    let char_encoding = Encoding::new(chars);
    let bigram_analyzer = Analyzer::new(&char_encoding, &key_encoding, bigram_analyzer);
    let trigram_analyzer = Analyzer::new(&char_encoding, &key_encoding, trigram_analyzer);
    let mut generator = Generator::new(
        PartialLayout::new(),
        &bigram_analyzer,
        &trigram_analyzer,
        &symmetries,
        max_score,
    );
    println!("Generating");
    let mut best = [0; N];
    while let Some((layout, score)) = generator.next() {
        best = layout;
        generator.set_max(score);
        print!("\rBest score so far: {score} ");
    }
    println!("Finished generation");
    if best == [0; N] {
        None
    } else {
        let res = (0..N)
            .map(|i| (key_encoding[best[i]], char_encoding[i]))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Some(res)
    }
}
