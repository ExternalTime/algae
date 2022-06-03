#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Finger {
    pub kind: FingerKind,
    pub hand: Hand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FingerKind {
    Pinky,
    Ring,
    Middle,
    Index,
    Thumb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}
