use super::Translatable;
use std::collections::HashMap;
use std::ops::Index;

pub struct Encoding<T, const N: usize> {
    en: HashMap<T, usize>,
    de: [T; N],
}

impl<T: Translatable, const N: usize> Encoding<T, N> {
    pub fn new(values: [T; N]) -> Self {
        let en = values
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();
        Self { en, de: values }
    }

    pub fn len(&self) -> usize {
        self.de.len()
    }
}

impl<T: Translatable, const N: usize> Index<usize> for Encoding<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.de[index]
    }
}

impl<T: Translatable, const N: usize> Index<&T> for Encoding<T, N> {
    type Output = usize;
    fn index(&self, index: &T) -> &Self::Output {
        &self.en[index]
    }
}
