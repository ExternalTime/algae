use std::collections::HashMap;

struct Generator {
    weights: [[u64; 30]; 30],
    encoding: [char; 30],
    cutoff: u64,
    total: u64,
}

fn in_col(layout: &[(u8, u64)], col: u8) -> impl Iterator<Item = usize> + '_ {
    layout
        .iter()
        .map(|(c, _)| c)
        .enumerate()
        .filter_map(move |(i, c)| (*c == col).then_some(i))
}

impl Generator {
    fn step_score(&self, tail: &[(u8, u64)], col: u8) -> u64 {
        let tmp = &self.weights[tail.len()];
        in_col(tail, col).map(|i| tmp[i]).sum()
    }

    fn found(&mut self, layout: &[(u8, u64)]) {
        let score = layout.last().unwrap().1;
        //self.cutoff = score;
        let score = score as f64 / self.total as f64;
        println!("Found layout with score {score}");

        // Converting columns into "layout"
        let mut layout: Vec<(usize, u8)> = layout.iter().map(|(col, _)| *col).enumerate().collect();
        layout.sort_by_key(|(_, col)| *col);
        let layout: Vec<_> = layout.into_iter().map(|(char, _)| char).collect();
        #[rustfmt::skip]
        let translation: [_; 30] = [
            25, 22, 13,  7, 10,   4,  1, 16, 19, 28,
            24, 21, 12,  6,  9,   3,  0, 15, 18, 27,
            26, 23, 14,  8, 11,   5,  2, 17, 20, 29,
        ];
        for i in 0..30 {
            print!("{}", self.encoding[layout[translation[i]]]);
            if i % 10 == 9 {
                println!();
            } else {
                print!(" ");
            }
        }
    }

    fn cutoff(&self) -> u64 {
        self.cutoff
    }
}

fn is_full(layout: &[(u8, u64)], col: u8) -> bool {
    (if col < 2 { 6 } else { 3 }) <= in_col(layout, col).count()
}

fn is_empty(layout: &[(u8, u64)], col: u8) -> bool {
    in_col(layout, col).next().is_none()
}

fn actually_generate(gen: &mut Generator) {
    let mut stack = Vec::with_capacity(30);
    let mut next = 0;
    let mut max = 0;
    loop {
        // Tried all positions for a letter. Going back to the previous letter.
        if 8 <= next {
            let Some((c, _)) = stack.pop() else {
                break;
            };
            // If last column was empty, we skip the ones of the same size (they
            // are empty too). Otherwise, we just continue to the next one.
            next = match (is_empty(&stack, c), c) {
                (false, _) => c + 1,
                (true, 0..=1) => 2,
                (true, 2..=7) => 8,
                (true, 8..) => unreachable!(),
            };
            continue;
        }
        // Layout is complete.
        if 30 <= stack.len() {
            gen.found(&stack);
            // Last key has only one valid position.
            stack.pop();
            next = 8;
            continue;
        }
        let score = stack
            .last()
            .map(|(_, w)| *w)
            .unwrap_or(0)
            .saturating_add(gen.step_score(&stack, next));
        if is_full(&stack, next) || gen.cutoff() <= score {
            next += 1;
            continue;
        }
        stack.push((next, score));
        next = 0;
    }
}

pub fn generate(mut alphabet: [char; 30], bigrams: HashMap<[char; 2], u64>, cutoff: f64) {
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
                println!("sum of all weights is higher than {} (u64::MAX)", u64::MAX);
                return;
            };
            total = tmp;
        }
    }
    let cutoff = (total as f64 * cutoff) as u64;
    println!("Generator set up. Starting.");
    let mut generator = Generator {
        weights,
        encoding,
        cutoff,
        total,
    };
    actually_generate(&mut generator);
    println!("Finished.");
}
