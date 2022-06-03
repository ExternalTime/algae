use super::Encoding;
use super::Translatable;
use std::collections::HashMap;
use std::ops::Deref;

pub struct Symmetries<const N: usize>([Option<usize>; N]);

impl<const N: usize> Symmetries<N> {
    pub fn new<Key: Translatable>(
        keys: [Key; N],
        symmetries: &dyn Fn(Key) -> Option<Key>,
    ) -> (Encoding<Key, N>, Self) {
        // Dealing with keys we do have pointing to ones we don't.
        // Relation "must be placed before" is transitive.
        let symmetries: HashMap<_, _> = keys
            .iter()
            .copied()
            .flat_map(|key| {
                let mut target = key;
                while let Some(t) = symmetries(target) {
                    if keys.contains(&t) {
                        return Some((key, t));
                    } else {
                        target = t;
                    }
                }
                None
            })
            .collect();
        // Constructing a tree
        let mut roots = Vec::new();
        let mut children = HashMap::new();
        for key in keys {
            if let Some(parent) = symmetries.get(&key) {
                children.entry(parent).or_insert(Vec::new()).push(key);
            } else {
                roots.push(key);
            }
        }
        // Sorting keys so that no subtree is interrupted
        let mut keys = Vec::new();
        while let Some(key) = roots.pop() {
            keys.push(key);
            if let Some(children) = children.get(&key) {
                roots.extend_from_slice(&children);
            }
        }
        // And done
        let keys = keys
            .try_into()
            .expect("key placement order should be consistent (no cycles)");
        let encoding = Encoding::new(keys);
        let inner = keys.map(|key| symmetries.get(&key).map(|key| encoding[key]));
        (encoding, Self(inner))
    }
}

impl<const N: usize> Deref for Symmetries<N> {
    type Target = [Option<usize>; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
