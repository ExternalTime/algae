use crate::Translatable;
use std::fmt;
use std::hash::Hash;

#[derive(Clone, Copy, fmt::Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(pub usize, pub usize);

impl Translatable for Key {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout(Vec<Vec<char>>);

impl Layout {
    pub fn qwerty() -> Layout {
        Layout(vec![
            "qwertyuiop".chars().collect(),
            "asdfghjkl'".chars().collect(),
            "zxcvbnm,./".chars().collect(),
        ])
    }
    pub fn into_mapping(self) -> Vec<(Key, char)> {
        let mut res = Vec::new();
        for (row, r) in self.0.into_iter().enumerate() {
            for (col, char) in r.into_iter().enumerate() {
                res.push((Key(row, col), char));
            }
        }
        res.sort_by_key(|(key, _)| *key);
        res
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.0.iter() {
            for (col, char) in row.iter().enumerate() {
                write!(fmt, "{} ", char)?;
                if col == 4 {
                    write!(fmt, "  ")?;
                }
            }
            writeln!(fmt)?;
        }
        Ok(())
    }
}

impl FromIterator<(Key, char)> for Layout {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Key, char)>,
    {
        let mut vec: Vec<_> = iter.into_iter().collect();
        vec.sort_by_key(|&(key, _)| key);
        let max_y = *vec
            .iter()
            .map(|(Key(y, _), _)| y)
            .max()
            .expect("layout should be not empty")
            + 1;
        let mut res = Vec::new();
        for _ in 0..max_y {
            res.push(Vec::new());
        }
        for (Key(y, _), char) in vec {
            res[y].push(char);
        }
        Layout(res)
    }
}
