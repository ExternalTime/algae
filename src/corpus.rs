use std::collections::HashMap;

pub struct Unshift<I> {
    chars: I,
    shifted: Option<char>,
}

impl<I: Iterator<Item = char>> Iterator for Unshift<I> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        self.shifted.take().or_else(|| {
            self.chars.next().map(|c| match c.is_ascii_uppercase() {
                true => {
                    self.shifted = Some(c);
                    '⇧'
                }
                false => c,
            })
        })
    }
}

fn calc_weights(mut chars: impl Iterator<Item = char>) -> HashMap<[char; 2], u64> {
    let mut map = HashMap::new();
    let [Some(mut c1), Some(mut c2)] = [chars.next(), chars.next()] else {
        return map;
    };
    map.insert([c1, c2], 4);
    for c3 in chars {
        *map.entry([c2, c3]).or_insert(0) += 4;
        *map.entry([c1, c3]).or_insert(0) += 1;
        [c1, c2] = [c2, c3];
    }
    map
}

// Weights are pretty arbitrary, but if you want custom ones, calculate them yourself
pub fn weights(str: &str) -> HashMap<[char; 2], u64> {
    calc_weights(Unshift {
        chars: str.chars(),
        shifted: None,
    })
}
