use std::collections::HashMap;

struct Generator {
    weights: [[u64; 30]; 30],
    encoding: [char; 30],
    cutoff: u64,
    total: f64,
    stack: Vec<(u8, u64)>,
    next: u8,
}

impl Generator {
    fn keys(&self) -> impl Iterator<Item = (usize, u8)> + '_ {
        self.stack.iter()
            .map(|(c, _)| *c)
            .enumerate()
    }

    fn next_col(&self) -> impl Iterator<Item = usize> + '_ {
        self.keys()
            .filter_map(|(key, col)| (col == self.next).then_some(key))
    }

    fn next_full(&self) -> bool {
        (if self.next < 2 { 6 } else { 3 }) <= self.next_col().count()
    }

    fn next_empty(&self) -> bool {
        self.next_col().next().is_none()
    }

    fn score(&self) -> u64 {
        self.stack.last().map(|(_, score)| *score).unwrap_or(0)
    }

    fn step_score(&self) -> u64 {
        self.next_col()
            .map(|c| self.weights[self.stack.len()][c])
            .sum()
    }

    fn cutoff(&self) -> u64 {
        self.cutoff
    }

    fn try_complete(&self) -> Option<[char; 30]> {
        if self.stack.len() < 30 {
            return None;
        }
        let mut layout: [_; 30] = std::array::from_fn(|i| (self.stack[i].0, self.encoding[i]));
        layout.sort_by_key(|(col, _)| *col);
        #[rustfmt::skip]
        let translation: [_; 30] = [
            25, 22, 13,  7, 10,   4,  1, 16, 19, 28,
            24, 21, 12,  6,  9,   3,  0, 15, 18, 27,
            26, 23, 14,  8, 11,   5,  2, 17, 20, 29,
        ];
        Some(std::array::from_fn(|i| layout[translation[i]].1))
    }
}

impl Iterator for Generator {
    type Item = ([char; 30], f64);
    fn next(&mut self) -> Option<Self::Item> {
        // Tried all positions for a letter. Going back to the previous letter.
        if 8 <= self.next {
            self.next = self.stack.pop()?.0;
            // If last column was empty, we skip the ones of the same size (they
            // are empty too). Otherwise, we just continue to the next one.
            self.next = match (self.next_empty(), self.next) {
                (false, _) => self.next + 1,
                (true, 0..=1) => 2,
                (true, 2..=7) => 8,
                (true, 8..) => unreachable!(),
            };
            return self.next();
        }
        if let Some(layout) = self.try_complete() {
            let score = self.score() as f64 / self.total;
            // Last key has only one valid position.
            self.stack.pop();
            self.next = 8;
            return Some((layout, score));
        }
        let score = self.score().saturating_add(self.step_score());
        if self.next_full() || self.cutoff() <= score {
            self.next += 1;
            return self.next();
        }
        self.stack.push((self.next, score));
        self.next = 0;
        return self.next();
    }
}


pub fn generator(mut alphabet: [char; 30], bigrams: HashMap<[char; 2], u64>, cutoff: f64) -> impl Iterator<Item = ([char; 30], f64)> {
    // placing more common letters early allows for earlier pruning.
    alphabet.sort_by_cached_key(|c| {
        std::cmp::Reverse(
            bigrams
                .iter()
                .filter_map(|(bigram, w)| bigram.contains(c).then_some(w))
                .sum::<u64>(),
        )
    });
    let encoding = alphabet;
    let mut weights = [[0; 30]; 30];
    let mut total = 0u64;
    for i in 0..30 {
        for j in 0..i {
            let ic = encoding[i];
            let jc = encoding[j];
            let bigram = |i, j| bigrams.get(&[i, j]).copied().unwrap_or(0);
            let w = bigram(ic, jc).saturating_add(bigram(jc, ic));
            weights[i][j] = w;
            weights[j][i] = w;
            let Some(tmp) = total.checked_add(w) else {
                panic!("sum of all weights is higher than {} (u64::MAX)", u64::MAX);
            };
            total = tmp;
        }
    }
    let total = total as f64;
    let cutoff = (total * cutoff) as u64;
    Generator {
        weights,
        encoding,
        cutoff,
        total,
        stack: Vec::new(),
        next: 0,
    }
}
