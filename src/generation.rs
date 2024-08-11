use std::collections::HashMap;


// Sorted by frequency. Allows for earlier pruning.
struct Encoding([char; 30]);

impl Encoding {
    fn new(mut alphabet: [char; 30], frequency: &impl Fn(&char) -> u64) -> Self {
        alphabet.sort_by_cached_key(|c| std::cmp::Reverse(frequency(c)));
        Self(alphabet)
    }

    fn decode_bigram(&self, bigram: [usize; 2]) -> [char; 2] {
        bigram.map(|char| self.0[char])
    }

    fn decode_layout(&self, layout: &[u8; 30]) -> [char; 30] {
        let mut l: [_; 30] = std::array::from_fn(|i| i);
        // Because the sort is stable, keys in a column are sorted by frequency
        l.sort_by_key(|&i| layout[i]);
        // Putting columns into keyboard shape. While it's not amazing, it should help with importing into other tools
        #[rustfmt::skip]
        const TRANSLATION: [usize; 30] = [
            25, 22, 13,  7, 10,   4,  1, 16, 19, 28,
            24, 21, 12,  6,  9,   3,  0, 15, 18, 27,
            26, 23, 14,  8, 11,   5,  2, 17, 20, 29,
        ];
        TRANSLATION.map(|i| self.0[l[i]])
    }
}


struct Weights([[u64; 30]; 30]);

impl Weights {
    pub fn new(bigrams: &impl Fn([usize; 2]) -> u64) -> Self {
        let mut w = [[0; 30]; 30];
        for i in 0..30 {
            for j in 0..i {
                w[i][j] = bigrams([i, j]).saturating_add(bigrams([j, i]));
            }
        }
        Self(w)
    }

    pub fn total(&self) -> u64 {
        self.0.iter()
            .flatten()
            .fold(0, |a, &b| a.checked_add(b).expect("total weights exceeded u64 limit"))
    }

    pub fn step_score(&self, k1: usize, col: impl Iterator<Item = usize>) -> u64 {
        col.map(|k2| self.0[k1][k2]).sum()
    }
}


struct IncompleteLayout {
    char_positions: [u8; 30],
    scores: [u64; 30],
    len: u8,
}

impl IncompleteLayout {
    pub fn new() -> Self {
        Self {
            char_positions: [0; 30],
            scores: [0; 30],
            len: 0,
        }
    }

    pub fn try_complete(&self) -> Option<&[u8; 30]> {
        (self.len == 30).then_some(&self.char_positions)
    }

    pub fn push(&mut self, position: u8, score: u64) {
        let len: usize = self.len.into();
        self.char_positions[len] = position;
        self.scores[len] = score;
        self.len += 1;
    }

    pub fn layout(&self) -> &[u8] {
        &self.char_positions[..usize::from(self.len)]
    }

    pub fn len(&self) -> usize {
        self.layout().len()
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.layout()
            .last()
            .copied()
            .inspect(|_| self.len -= 1)
    }

    pub fn col(&self, col: u8) -> impl Iterator<Item = usize> + '_ {
        self.layout()
            .iter()
            .enumerate()
            .filter_map(move |(i, &c)| (c == col).then_some(i))
    }

    pub fn is_col_full(&self, col: u8) -> bool {
        (if col < 2 { 6 } else { 3 }) <= self.col(col).count()
    }

    pub fn is_col_empty(&self, col: u8) -> bool {
        self.col(col).next().is_none()
    }

    pub fn score(&self) -> u64 {
        self.scores[..usize::from(self.len)]
            .last()
            .copied()
            .unwrap_or(0)
    }
}

struct Generator {
    encoding: Encoding,
    weights: Weights,
    cutoff: u64,
    total: f64,
    layout: IncompleteLayout,
    next: u8,
}

impl Iterator for Generator {
    type Item = ([char; 30], f64);
    fn next(&mut self) -> Option<Self::Item> {
        // Tried all positions for a letter. Going back to the previous letter.
        if 8 <= self.next {
            self.next = self.layout.pop()?;
            // If last column was empty, we skip the ones of the same size (they
            // are empty too). Otherwise, we just continue to the next one.
            self.next = match (self.layout.is_col_empty(self.next), self.next) {
                (false, _) => self.next + 1,
                (true, 0..=1) => 2,
                (true, 2..=7) => 8,
                (true, 8..) => unreachable!(),
            };
            return self.next();
        }
        if let Some(layout) = self.layout.try_complete() {
            let layout = self.encoding.decode_layout(layout);
            let score = self.layout.score() as f64 / self.total;
            // Last key has only one valid position.
            let _ = self.layout.pop();
            self.next = 8;
            return Some((layout, score));
        }
        let score = self.layout.score() + self.weights.step_score(self.layout.len(), self.layout.col(self.next));
        if self.layout.is_col_full(self.next) || self.cutoff <= score {
            self.next += 1;
            return self.next();
        }
        self.layout.push(self.next, score);
        self.next = 0;
        self.next()
    }
}


pub fn generator(alphabet: [char; 30], bigrams: HashMap<[char; 2], u64>, cutoff: f64) -> impl Iterator<Item = ([char; 30], f64)> {
    let encoding = Encoding::new(alphabet, &|char| bigrams.iter()
        .filter_map(|(bigram, w)| bigram.contains(char).then_some(w))
        .sum::<u64>());
    let weights = Weights::new(&|bigram| bigrams.get(&encoding.decode_bigram(bigram)).copied().unwrap_or(0));
    let total = weights.total() as f64;
    let cutoff = (total * cutoff) as u64;
    Generator {
        weights,
        encoding,
        cutoff,
        total,
        layout: IncompleteLayout::new(),
        next: 0,
    }
}
