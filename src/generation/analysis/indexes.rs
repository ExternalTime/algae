pub struct Indexes<const D: usize> {
    max: usize,
    current: [usize; D],
}

impl<const D: usize> Indexes<D> {
    pub fn new(max: usize) -> Self {
        Self {
            max: max - 1,
            current: [0; D],
        }
    }
}

impl<const D: usize> Iterator for Indexes<D> {
    type Item = [usize; D];
    fn next(&mut self) -> Option<Self::Item> {
        if self.current[D - 1] <= self.max {
            let res = self.current;
            for i in self.current.iter_mut().rev() {
                if *i < self.max {
                    *i += 1;
                    return Some(res);
                } else {
                    *i = 0;
                }
            }
            self.current[D - 1] = self.max + 1;
            Some(res)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_order() {
        let mut iter = Indexes::<3>::new(2);
        assert_eq!(iter.next(), Some([0, 0, 0]));
        assert_eq!(iter.next(), Some([0, 0, 1]));
        assert_eq!(iter.next(), Some([0, 1, 0]));
        assert_eq!(iter.next(), Some([0, 1, 1]));
        assert_eq!(iter.next(), Some([1, 0, 0]));
        assert_eq!(iter.next(), Some([1, 0, 1]));
        assert_eq!(iter.next(), Some([1, 1, 0]));
        assert_eq!(iter.next(), Some([1, 1, 1]));
        assert_eq!(iter.next(), None);
    }
}
