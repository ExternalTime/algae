use std::fmt::Debug;
use std::hash::Hash;

pub trait Translatable: Copy + Debug + Eq + Hash {}

macro_rules! def_translatable {
    ( $($type:ty),+ $(,)? ) => {$(
        impl Translatable for $type {}
    )+};
}

def_translatable!(char, u8);
