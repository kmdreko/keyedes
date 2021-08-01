#![feature(coerce_unsized)]

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{CoerceUnsized, Deref, DerefMut};

use serde::{Deserialize, Deserializer};

use crate::private::KeyValueVisitor;

mod private;

pub type DeserializationFn<T> =
    Box<dyn Fn(&mut dyn erased_serde::Deserializer) -> Result<T, erased_serde::Error>>;

pub struct DeserializationMap<K, T> {
    type_name: &'static str,
    field_names: &'static [&'static str; 2],
    map: HashMap<K, DeserializationFn<T>>,
}

impl<K, T> DeserializationMap<K, T> {
    pub fn new(
        type_name: &'static str,
        field_names: &'static [&'static str; 2],
    ) -> DeserializationMap<K, T> {
        DeserializationMap {
            type_name,
            field_names,
            map: HashMap::new(),
        }
    }

    pub fn deserialize_by_key<'de, D>(&self, deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        K: Deserialize<'de> + Eq + Hash,
    {
        deserializer.deserialize_struct(
            self.type_name,
            self.field_names,
            KeyValueVisitor::<K, T> {
                deserialization_map: self,
                key_name: self.field_names[0],
                value_name: self.field_names[1],
            },
        )
    }

    pub fn insert_coerced_fn<U>(&mut self, k: K)
    where
        U: for<'de> Deserialize<'de>,
        Box<U>: CoerceUnsized<T>,
        K: Eq + Hash,
    {
        self.map.insert(
            k,
            Box::new(move |deserializer| {
                U::deserialize(deserializer).map(|u| -> T { Box::new(u) })
            }),
        );
    }
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
    fn does_it_work() {
        let json1 = r#"{"id":"A","data":{"name":"chuck norris"}}"#;
        let json2 = r#"{"id":"B","data":"Pizza"}"#;
        let json3 = r#"{"id":"C","data":null}"#;
        let json4 = r#"{"id":"C"}"#;

        let mut map =
            DeserializationMap::<String, Box<dyn TestTrait>>::new("TestTrait", &["id", "data"]);
        map.insert_coerced_fn::<TestStructA>("A".to_string());
        map.insert_coerced_fn::<TestEnumB>("B".to_string());
        map.insert_coerced_fn::<TestUnitC>("C".to_string());

        let mut deserializer = serde_json::Deserializer::from_str(json1);
        let result = map.deserialize_by_key(&mut deserializer).unwrap();
        assert_eq!(result.name(), "chuck norris");

        let mut deserializer = serde_json::Deserializer::from_str(json2);
        let result = map.deserialize_by_key(&mut deserializer).unwrap();
        assert_eq!(result.name(), "pizza");

        let mut deserializer = serde_json::Deserializer::from_str(json3);
        let result = map.deserialize_by_key(&mut deserializer).unwrap();
        assert_eq!(result.name(), "just a c");

        let mut deserializer = serde_json::Deserializer::from_str(json4);
        let result = map.deserialize_by_key(&mut deserializer).unwrap();
        assert_eq!(result.name(), "just a c");
    }

    // #[test]
    // fn missing_data_is_handled() {
    //     let json1 = r#"{"id":"A"}"#;
    //     let json2 = r#"{"id":"B"}"#;
    //     let json3 = r#"{"id":"C"}"#;

    //     let mut map = DeserializationMap::<String, Test>::new("Test", &["id", "data"]);
    //     map.insert(
    //         "A".to_string(),
    //         Box::new(|deserializer| i32::deserialize(deserializer).map(Test::A)),
    //     );
    //     map.insert(
    //         "B".to_string(),
    //         Box::new(|deserializer| String::deserialize(deserializer).map(Test::B)),
    //     );
    //     map.insert(
    //         "C".to_string(),
    //         Box::new(|deserializer| {
    //             extern crate serde as _serde;

    //             // apparently this is how you're supposed to deserialize a unit
    //             // struct
    //             struct __Visitor;
    //             impl<'de> _serde::de::Visitor<'de> for __Visitor {
    //                 type Value = ();
    //                 fn expecting(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    //                     std::fmt::Formatter::write_str(__formatter, "unit struct ()")
    //                 }
    //                 #[inline]
    //                 fn visit_unit<__E>(self) -> Result<Self::Value, __E>
    //                 where
    //                     __E: serde::de::Error,
    //                 {
    //                     Ok(())
    //                 }
    //                 #[inline]
    //                 fn visit_none<__E>(self) -> Result<Self::Value, __E>
    //                 where
    //                     __E: serde::de::Error,
    //                 {
    //                     Ok(())
    //                 }
    //             }
    //             _serde::Deserializer::deserialize_unit_struct(deserializer, "()", __Visitor)?;

    //             Ok(Test::C)
    //         }),
    //     );

    //     let mut deserializer = serde_json::Deserializer::from_str(json1);
    //     let result = map.deserialize_by_key(&mut deserializer);
    //     assert!(result.is_err());

    //     let mut deserializer = serde_json::Deserializer::from_str(json2);
    //     let result = map.deserialize_by_key(&mut deserializer);
    //     assert!(result.is_err());

    //     let mut deserializer = serde_json::Deserializer::from_str(json3);
    //     let result = map.deserialize_by_key(&mut deserializer).unwrap();
    //     assert_eq!(result, Test::C);
    // }

    // #[test]
    // fn returns_error_on_unknown_key_type() {
    //     let json = r#"{"id":"D","data":5.02}"#;

    //     let mut map = DeserializationMap::<String, Test>::new("Test", &["id", "data"]);
    //     map.insert(
    //         "A".to_string(),
    //         Box::new(|deserializer| i32::deserialize(deserializer).map(Test::A)),
    //     );
    //     map.insert(
    //         "B".to_string(),
    //         Box::new(|deserializer| String::deserialize(deserializer).map(Test::B)),
    //     );

    //     let mut deserializer = serde_json::Deserializer::from_str(json);
    //     let result = map.deserialize_by_key(&mut deserializer);
    //     assert!(result.is_err());
    // }
}
