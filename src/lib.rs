use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Deserializer};

use crate::private::KeyValueVisitor;

mod private;

pub type DeserializationFn<T> =
    Box<dyn Fn(&mut dyn erased_serde::Deserializer) -> Result<T, erased_serde::Error>>;

pub struct DeserializationMap<K, V> {
    type_name: &'static str,
    field_names: &'static [&'static str; 2],
    map: HashMap<K, DeserializationFn<V>>,
}

impl<K, V> DeserializationMap<K, V> {
    pub fn new(
        type_name: &'static str,
        field_names: &'static [&'static str; 2],
    ) -> DeserializationMap<K, V> {
        DeserializationMap {
            type_name,
            field_names,
            map: HashMap::new(),
        }
    }
}

impl<K, V> DeserializationMap<K, V>
where
    K: Eq + Hash,
{
    pub fn deserialize_by_key<'de, D>(&self, deserializer: D) -> Result<V, D::Error>
    where
        D: Deserializer<'de>,
        K: Deserialize<'de>,
    {
        deserializer.deserialize_struct(
            self.type_name,
            self.field_names,
            KeyValueVisitor::<K, V> {
                deserialization_map: self,
                key_name: self.field_names[0],
                value_name: self.field_names[1],
            },
        )
    }

    pub fn insert_fn<F>(&mut self, key: K, f: F)
    where
        F: Fn(&mut dyn erased_serde::Deserializer) -> Result<V, erased_serde::Error> + 'static,
    {
        self.insert(key, Box::new(f));
    }

    pub fn insert_unit_fn<F>(&mut self, key: K, f: F)
    where
        F: Fn() -> Result<V, erased_serde::Error> + 'static,
    {
        self.insert(
            key,
            Box::new(move |deserializer| {
                let _ = <()>::deserialize(deserializer)?;
                f()
            }),
        );
    }
}

// TODO: Consider replacing with proper `insert_fn_coerced` method if/when
// `#[feature(coerce_unsized)]` is stabilized.
#[macro_export]
macro_rules! insert_fn_boxed {
    ($map:expr, $key:expr, $f:expr) => {
        $crate::DeserializationMap::insert_fn($map, $key, |deserializer| {
            $f(deserializer).map(|v| Box::new(v) as _)
        })
    };
}

impl<K, T> Deref for DeserializationMap<K, T> {
    type Target = HashMap<K, DeserializationFn<T>>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<K, T> DerefMut for DeserializationMap<K, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait TestTrait {
        fn name(&self) -> &str;
    }

    #[derive(Deserialize)]
    struct TestStructA {
        name: String,
    }

    impl TestTrait for TestStructA {
        fn name(&self) -> &str {
            self.name.as_str()
        }
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum TestEnumB {
        Pizza,
        Broccoli,
    }

    impl TestTrait for TestEnumB {
        fn name(&self) -> &str {
            match self {
                TestEnumB::Pizza => "pizza",
                TestEnumB::Broccoli => "yuck",
            }
        }
    }

    #[derive(Deserialize)]
    struct TestUnitC;

    impl TestTrait for TestUnitC {
        fn name(&self) -> &str {
            "just a c"
        }
    }

    #[test]
    fn deserialize_by_key_returns_correct_value() {
        let json1 = r#"{"id":"A","data":{"name":"chuck norris"}}"#;
        let json2 = r#"{"id":"B","data":"Pizza"}"#;
        let json3 = r#"{"id":"C","data":null}"#;

        let mut map = DeserializationMap::<String, Box<dyn TestTrait>>::new(
            "Box<dyn TestTrait>",
            &["id", "data"],
        );

        insert_fn_boxed!(&mut map, "A".to_string(), TestStructA::deserialize);
        insert_fn_boxed!(&mut map, "B".to_string(), TestEnumB::deserialize);
        insert_fn_boxed!(&mut map, "C".to_string(), TestUnitC::deserialize);

        let mut deserializer = serde_json::Deserializer::from_str(json1);
        let result = map.deserialize_by_key(&mut deserializer).unwrap();
        assert_eq!(result.name(), "chuck norris");

        let mut deserializer = serde_json::Deserializer::from_str(json2);
        let result = map.deserialize_by_key(&mut deserializer).unwrap();
        assert_eq!(result.name(), "pizza");

        let mut deserializer = serde_json::Deserializer::from_str(json3);
        let result = map.deserialize_by_key(&mut deserializer).unwrap();
        assert_eq!(result.name(), "just a c");
    }

    #[test]
    fn deserialize_by_key_returns_error_if_required_data_is_missing() {
        let json1 = r#"{"id":"A"}"#;
        let json2 = r#"{"id":"B"}"#;
        let json3 = r#"{"id":"C"}"#;

        let mut map = DeserializationMap::<String, Box<dyn TestTrait>>::new(
            "Box<dyn TestTrait>",
            &["id", "data"],
        );

        insert_fn_boxed!(&mut map, "A".to_string(), TestStructA::deserialize);
        insert_fn_boxed!(&mut map, "B".to_string(), TestEnumB::deserialize);
        insert_fn_boxed!(&mut map, "C".to_string(), TestUnitC::deserialize);

        let mut deserializer = serde_json::Deserializer::from_str(json1);
        let result = map.deserialize_by_key(&mut deserializer);
        assert!(result.is_err());

        let mut deserializer = serde_json::Deserializer::from_str(json2);
        let result = map.deserialize_by_key(&mut deserializer);
        assert!(result.is_err());

        let mut deserializer = serde_json::Deserializer::from_str(json3);
        let result = map.deserialize_by_key(&mut deserializer).unwrap();
        assert_eq!(result.name(), "just a c");
    }

    #[test]
    fn deserialize_by_key_returns_error_on_unknown_key_type() {
        let json = r#"{"id":"D","data":5.02}"#;

        let mut map = DeserializationMap::<String, Box<dyn TestTrait>>::new(
            "Box<dyn TestTrait>",
            &["id", "data"],
        );

        insert_fn_boxed!(&mut map, "A".to_string(), TestStructA::deserialize);
        insert_fn_boxed!(&mut map, "B".to_string(), TestEnumB::deserialize);
        insert_fn_boxed!(&mut map, "C".to_string(), TestUnitC::deserialize);

        let mut deserializer = serde_json::Deserializer::from_str(json);
        let result = map.deserialize_by_key(&mut deserializer);
        assert!(result.is_err());
    }
}
