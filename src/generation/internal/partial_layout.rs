use crate::generation::Symmetries;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartialLayout<const N: usize> {
    // keys[0..self.len] - keys with letters already assigne
    // keys[self.len..N] - missing ones
    keys: [usize; N],
    len: usize,
}

impl<const N: usize> PartialLayout<N> {
    pub fn new() -> Self {
        let mut keys = [0; N];
        for i in 0..N {
            keys[i] = i;
        }
        Self { keys, len: 0 }
    }

    pub fn try_complete(self) -> Result<[usize; N], Self> {
        if self.len < N {
            Err(self)
        } else {
            Ok(self.keys)
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn children<'a>(&'a self, symmetries: &'a Symmetries<N>) -> Children<'a, N> {
        Children::new(self, symmetries)
    }
}

impl<const N: usize> std::ops::Index<usize> for PartialLayout<N> {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.keys[..self.len][index]
    }
}

impl<const N: usize> AsRef<[usize]> for PartialLayout<N> {
    fn as_ref(&self) -> &[usize] {
        &self.keys[..self.len]
    }
}

pub struct Children<'a, const N: usize> {
    parent: &'a PartialLayout<N>,
    symmetries: &'a Symmetries<N>,
    i: usize,
}

impl<'a, const N: usize> Children<'a, N> {
    pub fn new(parent: &'a PartialLayout<N>, symmetries: &'a Symmetries<N>) -> Self {
        Self {
            parent,
            symmetries,
            i: parent.len,
        }
    }

    pub fn is_unique(&self) -> bool {
        if let Some(n) = self.symmetries[self.parent.keys[self.i]] {
            if self.parent.keys[self.parent.len..self.i].contains(&n) {
                return false;
            }
        }
        true
    }
}

impl<'a, const N: usize> Iterator for Children<'a, N> {
    type Item = PartialLayout<N>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.i < N {
            if self.is_unique() {
                let mut child = self.parent.clone();
                let range = child.len..=self.i;
                if !range.is_empty() {
                    child.keys[range].rotate_right(1);
                }
                child.len += 1;
                self.i += 1;
                return Some(child);
            } else {
                self.i += 1;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn children() {
        let parent = PartialLayout::new();
        let symmetries = [None, Some(0), None];
        let mut iter = parent.children(&symmetries);
        assert_eq!(iter.next().unwrap().as_ref(), [0]);
        assert_eq!(iter.next().unwrap().as_ref(), [2]);
        assert_eq!(iter.next(), None);
    }
}
