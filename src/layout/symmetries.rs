use super::Key;

// Symmetry is defined by a function by saying whether key is required before another
// All symmetries here assume 3x10 keyboard

// Keys in columns and columns can be swapped
pub fn column_set_symmetries(key: Key) -> Option<Key> {
    interior_column_symmetries(key).or_else(|| match key.1 {
        0 | 3 => None,
        4 | 5 => panic!(),
        6 => Some(Key(1, 3)),
        7 => Some(Key(1, 2)),
        n => Some(Key(1, n - 1)),
    })
}

// Keys in columns can be swapped. Can't be mirrored
pub fn interior_column_symmetries(key: Key) -> Option<Key> {
    match key {
        Key(0, n) => Some(Key(1, n)),
        Key(2, n) => Some(Key(0, n)),
        Key(1, 4) => Some(Key(2, 3)),
        Key(1, 5) => Some(Key(2, 6)),
        Key(1, _) => None,
        _ => panic!(),
    }
}

// keys in bottom row can be swapped with ones on top
pub fn sfb_distance_symmetries(key: Key) -> Option<Key> {
    match key {
        Key(2, 4) => Some(Key(0, 3)),
        Key(0, 4) => Some(Key(0, 3)),
        Key(2, 5) => Some(Key(0, 6)),
        Key(0, 5) => Some(Key(0, 6)),
        Key(2, n) => Some(Key(0, n)),
        _ => None,
    }
}
