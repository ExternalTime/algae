use super::Indexes;
use crate::generation::{Encoding, Translatable};
use std::ops::Index;

#[derive(Clone, Debug)]
pub struct NDArray<const SIDE: usize, const DIM: usize>(Box<[u64]>);

impl<const SIDE: usize, const DIM: usize> NDArray<SIDE, DIM> {
    const TOTAL_LEN: usize = SIDE.pow(DIM as u32);

    pub fn new<T>(encoding: &Encoding<T, SIDE>, weights: impl Fn([T; DIM]) -> u64) -> Self
    where
        T: Translatable,
    {
        let mut data = Vec::with_capacity(Self::TOTAL_LEN);
        for index in Indexes::new(SIDE) {
            data.push(weights(index.map(|c| encoding[c])));
        }
        Self(data.into_boxed_slice())
    }
}

impl<const SIDE: usize, const DIM: usize> Index<[usize; DIM]> for NDArray<SIDE, DIM> {
    type Output = u64;
    fn index(&self, index: [usize; DIM]) -> &u64 {
        /*
        &self.0[index
            .iter()
            .inspect(|i| assert!(**i < N))
            .enumerate()
            .map(|(i, index)| N.pow(i as u32) * index)
            .sum::<usize>()]
        */
        let mut sum = 0;
        for i in 0..DIM {
            sum *= SIDE;
            sum += index[i];
        }
        &self.0[sum]
    }
}
