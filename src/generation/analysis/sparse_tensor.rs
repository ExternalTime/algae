use super::Indexes;
use crate::generation::{Encoding, Translatable};
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct SparseTensor<const SIDE: usize, const DIM: usize>([Vec<([usize; DIM], u64)>; SIDE]);

impl<const SIDE: usize, const DIM: usize> SparseTensor<SIDE, DIM> {
    pub fn new<T>(encoding: &Encoding<T, SIDE>, weights: impl Fn([T; DIM]) -> u64) -> Self
    where
        T: Translatable,
    {
        let mut data = [(); SIDE].map(|_| Vec::new());
        for (index, w) in Indexes::new(encoding.len())
            .map(|index| (index, weights(index.map(|n| encoding[n]))))
            .filter(|(_, w)| *w != 0)
        {
            let max = *index.iter().max().unwrap();
            data[max].push((index, w));
        }
        Self(data)
    }
}

impl<const N: usize, const D: usize> Deref for SparseTensor<N, D> {
    type Target = [Vec<([usize; D], u64)>; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
