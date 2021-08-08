# Keyed Deserialization

This crate is designed to help serialize and deserialize differing objects
by allowing users to encode keys into the serialized format that can then be
used when deserializing. It mainly provides `serialize_with_key()` and
`deserialize_by_key()` to facilitate that.

The primary motivator is the deserialization of trait objects. If you have
the option, the [`typetag`](https://crates.io/crates/typetag) crate is
easier to use, whereas this crate is much more manual.

---

An example usage:

```rust
use serde::{Deserialize, Serialize};

trait TestTrait: erased_serde::Serialize {
    fn key(&self) -> &'static str;
}

mod test_trait {
    use std::collections::HashMap;
    use std::ops::Deref;
    use keyedes::DesFnSync;
    use once_cell::sync::Lazy;
    use serde::{Deserializer, Serializer};
    use super::TestTrait;

    static MAP: Lazy<HashMap<String, DesFnSync<Box<dyn TestTrait>>>> =
        Lazy::new(|| {
            let mut map = HashMap::<String, DesFnSync<Box<dyn TestTrait>>>::new();
            // fill out the map
            map
        });

    pub(super) fn serialize<S>(value: &Box<dyn TestTrait>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        keyedes::serialize_with_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            value.key(),
            value.deref(),
            serializer,
        )
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Box<dyn TestTrait>, D::Error>
    where
        D: Deserializer<'de>,
    {
        keyedes::deserialize_by_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            |key: String, deserializer| {
                MAP.get(&key)
                    .ok_or_else(keyedes::unknown_key)
                    .and_then(|f| f(deserializer))
            },
            deserializer,
        )
    }
}

#[derive(Serialize, Deserialize)]
struct Wrapper {
    #[serde(with = "test_trait")]
    test: Box<dyn TestTrait>,
}
```
