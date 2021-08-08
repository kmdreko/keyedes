//! This crate is designed to help serialize and deserialize trait objects by
//! allowing users to encode keys into the serialized format that can then be
//! used when deserializing. It primarily provides `serialize_with_key` and
//! `deserialize_by_key` to facilitate that.
//!
//! An example usage:
//!
//! ```ignore
//! use serde::{Deserialize, Serialize};
//!
//! trait TestTrait: erased_serde::Serialize {
//!     fn key(&self) -> &'static str;
//! }
//!
//! mod test_trait {
//!     use std::collections::HashMap;
//!     use desfn::DesFnSync;
//!     use once_cell::sync::Lazy;
//!     use serde::{Deserializer, Serializer};
//!     use super::TestTrait;
//!
//!     static MAP: Lazy<HashMap<String, DesFnSync<Box<dyn TestTrait>>>> =
//!         Lazy::new(|| {
//!             let mut map = HashMap::<String, DesFnSync<Box<dyn TestTrait>>>::new();
//!             // fill out the map
//!             map
//!         });
//!
//!     pub(super) fn serialize<S>(value: &Box<dyn TestTrait>, serializer: S) -> Result<S::Ok, S::Error>
//!     where
//!         S: Serializer,
//!     {
//!         desfn::serialize_with_key(...)
//!     }
//!
//!     pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Box<dyn TestTrait>, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         desfn::deserialize_by_key(...)
//!     }
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct Wrapper {
//!     #[serde(with = "test_trait")]
//!     test: Box<dyn TestTrait>,
//! }
//! ```

use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::private::{ErasedSerdeSerializeWrapper, KeyValueVisitor};

mod private;

pub type DesFnError = erased_serde::Error;
pub type DesFn<T> = Box<dyn Fn(&mut dyn erased_serde::Deserializer) -> Result<T, DesFnError>>;
pub type DesFnSync<T> =
    Box<dyn Fn(&mut dyn erased_serde::Deserializer) -> Result<T, DesFnError> + Send + Sync>;

/// Will serialize a struct with the two given names and values that can then be
/// deserialized with a similar [`deserialize_by_key()`] call.
pub fn serialize_with_key<S, K, V>(
    type_name: &'static str,
    field_names: &'static [&'static str; 2],
    key: &K,
    value: &V,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    K: ?Sized + Serialize,
    V: ?Sized + erased_serde::Serialize,
    S: Serializer,
{
    use serde::ser::SerializeStruct;

    let mut state = serializer.serialize_struct(type_name, 2)?;
    state.serialize_field(field_names[0], key)?;
    state.serialize_field(field_names[1], &ErasedSerdeSerializeWrapper(value))?;
    state.end()
}

/// Will deserialize a struct with the two given names and values that was
/// serialized with a similar [`serialize_with_key()`] call.
pub fn deserialize_by_key<'de, D, K, V, F>(
    type_name: &'static str,
    field_names: &'static [&'static str; 2],
    f: F,
    deserializer: D,
) -> Result<V, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    F: Fn(K, &mut dyn erased_serde::Deserializer) -> Result<V, DesFnError>,
{
    deserializer.deserialize_struct(
        type_name,
        field_names,
        KeyValueVisitor {
            deserialization_fn: f,
            key_name: field_names[0],
            value_name: field_names[1],
            _dummy: PhantomData,
        },
    )
}

/// Helper function for returning a serialization error on unknown key.
#[must_use]
pub fn unknown_key() -> DesFnError {
    use serde::de::Error;

    DesFnError::custom("unknown deserialization key")
}

/// Helper macro to convert a `T: Deserialize + Trait` into a
/// `Box<dyn Fn(Deserializer<'_>) -> Result<Box<dyn Trait>, Error>>`. The
/// `dyn Trait` is inferred.
#[macro_export]
macro_rules! deserialize_into_boxed_trait {
    ($ty:ty) => {
        Box::new(|deserializer| {
            <$ty as ::serde::Deserialize>::deserialize(deserializer)
                .map(|v| ::std::boxed::Box::new(v) as ::std::boxed::Box<_>)
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::ops::Deref;

    trait TestTrait: erased_serde::Serialize {
        fn key(&self) -> &'static str;
        fn name(&self) -> &str;
    }

    #[derive(Serialize, Deserialize)]
    struct TestStructA {
        name: String,
    }

    impl TestTrait for TestStructA {
        fn key(&self) -> &'static str {
            "A"
        }
        fn name(&self) -> &str {
            self.name.as_str()
        }
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum TestEnumB {
        Pizza,
        Broccoli,
    }

    impl TestTrait for TestEnumB {
        fn key(&self) -> &'static str {
            "B"
        }
        fn name(&self) -> &str {
            match self {
                TestEnumB::Pizza => "pizza",
                TestEnumB::Broccoli => "yuck",
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    struct TestUnitC;

    impl TestTrait for TestUnitC {
        fn key(&self) -> &'static str {
            "C"
        }
        fn name(&self) -> &str {
            "just a c"
        }
    }

    #[test]
    fn deserialize_by_key_returns_correct_value() {
        let json1 = r#"{"id":"A","data":{"name":"chuck norris"}}"#;
        let json2 = r#"{"id":"B","data":"Pizza"}"#;
        let json3 = r#"{"id":"C","data":null}"#;
        let json4 = r#"{"data":{"name":"chuck norris"},"id":"A"}"#;

        let mut map = HashMap::<String, DesFn<Box<dyn TestTrait>>>::new();

        map.insert("A".to_string(), deserialize_into_boxed_trait!(TestStructA));
        map.insert("B".to_string(), deserialize_into_boxed_trait!(TestEnumB));
        map.insert("C".to_string(), deserialize_into_boxed_trait!(TestUnitC));

        let mut deserializer = serde_json::Deserializer::from_str(json1);
        let result = deserialize_by_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            |key: String, deserializer| map.get(&key).unwrap()(deserializer),
            &mut deserializer,
        )
        .unwrap();
        assert_eq!(result.name(), "chuck norris");

        let mut deserializer = serde_json::Deserializer::from_str(json2);
        let result = deserialize_by_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            |key: String, deserializer| map.get(&key).unwrap()(deserializer),
            &mut deserializer,
        )
        .unwrap();
        assert_eq!(result.name(), "pizza");

        let mut deserializer = serde_json::Deserializer::from_str(json3);
        let result = deserialize_by_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            |key: String, deserializer| map.get(&key).unwrap()(deserializer),
            &mut deserializer,
        )
        .unwrap();
        assert_eq!(result.name(), "just a c");

        let mut deserializer = serde_json::Deserializer::from_str(json4);
        let result = deserialize_by_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            |key: String, deserializer| map.get(&key).unwrap()(deserializer),
            &mut deserializer,
        )
        .unwrap();
        assert_eq!(result.name(), "chuck norris");
    }

    #[test]
    fn deserialize_by_key_returns_error_if_required_data_is_missing() {
        let json1 = r#"{"id":"A"}"#;
        let json2 = r#"{"id":"B"}"#;
        let json3 = r#"{"id":"C"}"#;

        let mut map = HashMap::<String, DesFn<Box<dyn TestTrait>>>::new();

        map.insert("A".to_string(), deserialize_into_boxed_trait!(TestStructA));
        map.insert("B".to_string(), deserialize_into_boxed_trait!(TestEnumB));
        map.insert("C".to_string(), deserialize_into_boxed_trait!(TestUnitC));

        let mut deserializer = serde_json::Deserializer::from_str(json1);
        let result = deserialize_by_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            |key: String, deserializer| map.get(&key).unwrap()(deserializer),
            &mut deserializer,
        );
        assert!(result.is_err());

        let mut deserializer = serde_json::Deserializer::from_str(json2);
        let result = deserialize_by_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            |key: String, deserializer| map.get(&key).unwrap()(deserializer),
            &mut deserializer,
        );
        assert!(result.is_err());

        let mut deserializer = serde_json::Deserializer::from_str(json3);
        let result = deserialize_by_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            |key: String, deserializer| map.get(&key).unwrap()(deserializer),
            &mut deserializer,
        )
        .unwrap();
        assert_eq!(result.name(), "just a c");
    }

    #[test]
    fn seralize_with_key_creates_correct_output() {
        let value1 = Box::new(TestStructA {
            name: "chuck norris".to_string(),
        }) as Box<dyn TestTrait>;
        let value2 = Box::new(TestEnumB::Pizza) as Box<dyn TestTrait>;
        let value3 = Box::new(TestUnitC) as Box<dyn TestTrait>;

        let mut serializer = serde_json::Serializer::new(Vec::new());
        serialize_with_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            value1.key(),
            value1.deref(),
            &mut serializer,
        )
        .unwrap();
        assert_eq!(
            serializer.into_inner().as_slice(),
            br#"{"id":"A","data":{"name":"chuck norris"}}"#
        );

        let mut serializer = serde_json::Serializer::new(Vec::new());
        serialize_with_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            value2.key(),
            value2.deref(),
            &mut serializer,
        )
        .unwrap();
        assert_eq!(
            serializer.into_inner().as_slice(),
            br#"{"id":"B","data":"Pizza"}"#
        );

        let mut serializer = serde_json::Serializer::new(Vec::new());
        serialize_with_key(
            "Box<dyn TestTrait>",
            &["id", "data"],
            value3.key(),
            value3.deref(),
            &mut serializer,
        )
        .unwrap();
        assert_eq!(
            serializer.into_inner().as_slice(),
            br#"{"id":"C","data":null}"#
        );
    }

    #[test]
    fn ergonomics() {
        use serde::{Deserialize, Serialize};

        mod test_trait {
            use crate as desfn;

            use std::collections::HashMap;
            use std::ops::Deref;
            use std::sync::RwLock;

            use desfn::{deserialize_into_boxed_trait, DesFnSync};
            use once_cell::sync::Lazy;
            use serde::{Deserializer, Serializer};

            use super::tests::{TestEnumB, TestStructA, TestTrait, TestUnitC};

            static MAP: Lazy<RwLock<HashMap<String, DesFnSync<Box<dyn TestTrait>>>>> =
                Lazy::new(|| {
                    let mut map = HashMap::<String, DesFnSync<Box<dyn TestTrait>>>::new();

                    map.insert("A".to_string(), deserialize_into_boxed_trait!(TestStructA));
                    map.insert("B".to_string(), deserialize_into_boxed_trait!(TestEnumB));
                    map.insert("C".to_string(), deserialize_into_boxed_trait!(TestUnitC));

                    RwLock::new(map)
                });

            pub(super) fn serialize<S>(
                value: &Box<dyn TestTrait>,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                desfn::serialize_with_key(
                    "Box<dyn TestTrait>",
                    &["id", "data"],
                    value.key(),
                    value.deref(),
                    serializer,
                )
            }

            pub(super) fn deserialize<'de, D>(
                deserializer: D,
            ) -> Result<Box<dyn TestTrait>, D::Error>
            where
                D: Deserializer<'de>,
            {
                desfn::deserialize_by_key(
                    "Box<dyn TestTrait>",
                    &["id", "data"],
                    |key: String, deserializer| {
                        MAP.read()
                            .unwrap()
                            .get(&key)
                            .ok_or_else(desfn::unknown_key)
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
    }
}
