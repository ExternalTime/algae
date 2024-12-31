use std::collections::HashMap;
use serde::Deserialize;
use serde::de::{Deserializer, Visitor, MapAccess, Error};

fn char_array<const N: usize>(str: &str) -> Option<[char; N]> {
    let mut arr = ['\0'; N];
    let mut chars = str.chars();
    for c in arr.iter_mut() {
        *c = chars.next()?;
    }
    chars.next()
        .is_none()
        .then_some(arr)
}

fn alphabet<'de, D: Deserializer<'de>>(de: D) -> Result<Option<[char; 30]>, D::Error> {
    Option::<String>::deserialize(de).and_then(|opt| match opt {
        Some(str) => char_array(&str).map(Some).ok_or(D::Error::custom("alphabet must have exactly 30 characters")),
        None => Ok(None),
    })
}

fn bigrams<'de, D: Deserializer<'de>>(deserializer: D) -> Result<HashMap<[char; 2], u64>, D::Error> {
    struct BigramsVisitor;
    impl<'de> Visitor<'de> for BigramsVisitor {
        type Value = HashMap<[char; 2], u64>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("bigrams")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
            let mut values = HashMap::new();
            // TODO: check how to deserialize into Cow without it cloning anyway
            while let Some((bigram, count)) = access.next_entry::<Box<str>, _>()? {
                values.insert(char_array(&bigram).ok_or(A::Error::custom("each bigram must be exactly 2 characters long"))?, count);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(BigramsVisitor)
}

#[derive(Deserialize)]
struct Options {
    #[serde(deserialize_with = "alphabet")]
    alphabet: Option<[char; 30]>,
    #[serde(deserialize_with = "bigrams")]
    weights: HashMap<[char; 2], u64>,
}

pub fn parse(str: &str) -> Result<(Option<[char; 30]>, HashMap<[char; 2], u64>), impl std::error::Error> {
    let Options { alphabet, weights } = serde_json::from_str::<Options>(str)?;
    Ok::<_, serde_json::Error>((alphabet, weights))
}
