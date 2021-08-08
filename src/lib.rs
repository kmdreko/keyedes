use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use serde::de::Error;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::private::{KeyValueVisitor, KeyValueVisitor2};

mod private;

pub type DeserializationFn<T> = Box<
    dyn Fn(&mut dyn erased_serde::Deserializer) -> Result<T, erased_serde::Error> + Send + Sync,
>;

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
        F: Fn(&mut dyn erased_serde::Deserializer) -> Result<V, erased_serde::Error>
            + 'static
            + Send
            + Sync,
    {
        self.insert(key, Box::new(f));
    }

    pub fn insert_unit_fn<F>(&mut self, key: K, f: F)
    where
        F: Fn() -> Result<V, erased_serde::Error> + 'static + Send + Sync,
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

#[macro_export]
macro_rules! deserialize_into_boxed_trait {
    ($ty:ty) => {
        Box::new(|deserializer| <$ty>::deserialize(deserializer).map(|v| Box::new(v) as Box<_>))
    };
}

#[macro_export]
macro_rules! impl_serialize {
    ($key_name:ident : $key_type:ty, $data_name:ident : $data_type:ty, $f:expr) => {
        pub fn serialize<S>(data: &$data_type, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            use ::serde::ser::SerializeStruct;

            struct ErasedSerdeWrapper<'a, V: ?Sized>(&'a V);
            impl<'a, V: ?Sized> ::serde::Serialize for ErasedSerdeWrapper<'a, V>
            where
                V: ::erased_serde::Serialize,
            {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    erased_serde::serialize(self.0, serializer)
                }
            }

            fn f<F: Fn(&$data_type) -> $key_type>(f: F) -> F {
                f
            }

            let f = f($f);

            let mut state = serializer.serialize_struct(std::stringify!($data_type), 2)?;
            state.serialize_field(std::stringify!($key_name), &f(data))?;
            state.serialize_field(std::stringify!($data_name), &ErasedSerdeWrapper(&**data))?;
            state.end()
        }
    };
}

#[macro_export]
macro_rules! impl_deserialize {
    ($key_name:ident : $key_type:ty, $data_name:ident : $data_type:ty, $f:expr) => {
        pub fn deserialize<'de, D>(deserializer: D) -> Result<$data_type, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            deserializer.deserialize_struct(
                std::stringify!($data_type),
                &[std::stringify!($key_name), std::stringify!($data_name)],
                crate::private::KeyValueVisitor2::<_, $key_type, $data_type> {
                    deserialization_fn: $f,
                    key_name: std::stringify!($key_name),
                    value_name: std::stringify!($data_name),
                    _dummy: std::marker::PhantomData,
                },
            )
        }
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

pub struct SerializationMap<K, V: ?Sized> {
    type_name: &'static str,
    field_names: &'static [&'static str; 2],
    key_fn: Box<dyn Fn(&V) -> K>,
}

impl<K, V: ?Sized> SerializationMap<K, V> {
    pub fn new<F>(
        type_name: &'static str,
        field_names: &'static [&'static str; 2],
        key_fn: F,
    ) -> SerializationMap<K, V>
    where
        F: Fn(&V) -> K + 'static,
    {
        SerializationMap {
            type_name,
            field_names,
            key_fn: Box::new(key_fn),
        }
    }
}

impl<K, V: ?Sized> SerializationMap<K, V>
where
    K: Serialize,
    V: erased_serde::Serialize,
{
    pub fn serialize_with_key<S, VI>(&self, v: &VI, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        VI: AsRef<V>,
    {
        struct ValueWrapper<'a, V: ?Sized>(&'a V);
        impl<'a, V: ?Sized> Serialize for ValueWrapper<'a, V>
        where
            V: erased_serde::Serialize,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                erased_serde::serialize(self.0, serializer)
            }
        }

        let v = v.as_ref();
        let mut state = serializer.serialize_struct(self.type_name, 2)?;
        state.serialize_field(self.field_names[0], &(self.key_fn)(v))?;
        state.serialize_field(self.field_names[1], &ValueWrapper(v))?;
        state.end()
    }
}

pub fn serialize_with_key<S, K, V>(
    type_name: &'static str,
    field_names: &'static [&'static str; 2],
    key: &K,
    value: &V,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    K: ?Sized + ::serde::Serialize,
    V: ?Sized + ::erased_serde::Serialize,
    S: Serializer,
{
    struct ErasedSerdeSerializeWrapper<'a, V: ?Sized>(&'a V);
    impl<'a, V: ?Sized> Serialize for ErasedSerdeSerializeWrapper<'a, V>
    where
        V: erased_serde::Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            erased_serde::serialize(self.0, serializer)
        }
    }

    let mut state = serializer.serialize_struct(type_name, 2)?;
    state.serialize_field(field_names[0], key)?;
    state.serialize_field(field_names[1], &ErasedSerdeSerializeWrapper(value))?;
    state.end()
}

pub fn deserialize_by_key<'de, D, K, V, F>(
    type_name: &'static str,
    field_names: &'static [&'static str; 2],
    f: F,
    deserializer: D,
) -> Result<V, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    F: Fn(K, &mut dyn ::erased_serde::Deserializer) -> Result<V, ::erased_serde::Error>,
{
    deserializer.deserialize_struct(
        type_name,
        field_names,
        KeyValueVisitor2 {
            deserialization_fn: f,
            key_name: field_names[0],
            value_name: field_names[1],
            _dummy: std::marker::PhantomData,
        },
    )
}

pub fn unknown_key() -> erased_serde::Error {
    erased_serde::Error::custom("unknown deserialization key")
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn seralize_with_key_creates_correct_output() {
        let value1 = Box::new(TestStructA {
            name: "chuck norris".to_string(),
        }) as Box<dyn TestTrait>;
        let value2 = Box::new(TestEnumB::Pizza) as Box<dyn TestTrait>;
        let value3 = Box::new(TestUnitC) as Box<dyn TestTrait>;

        let map = SerializationMap::<&'static str, dyn TestTrait>::new(
            "Box<dyn TestTrait>",
            &["id", "data"],
            |t| t.key(),
        );

        let mut serializer = serde_json::Serializer::new(Vec::new());
        map.serialize_with_key(&value1, &mut serializer).unwrap();
        assert_eq!(
            serializer.into_inner().as_slice(),
            br#"{"id":"A","data":{"name":"chuck norris"}}"#
        );

        let mut serializer = serde_json::Serializer::new(Vec::new());
        map.serialize_with_key(&value2, &mut serializer).unwrap();
        assert_eq!(
            serializer.into_inner().as_slice(),
            br#"{"id":"B","data":"Pizza"}"#
        );

        let mut serializer = serde_json::Serializer::new(Vec::new());
        map.serialize_with_key(&value3, &mut serializer).unwrap();
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

            use desfn::{deserialize_into_boxed_trait, DeserializationFn};
            use once_cell::sync::Lazy;
            use serde::{Deserialize, Deserializer, Serializer};

            use super::tests::{TestEnumB, TestStructA, TestTrait, TestUnitC};

            static MAP: Lazy<RwLock<HashMap<String, DeserializationFn<Box<dyn TestTrait>>>>> =
                Lazy::new(|| {
                    let mut map = HashMap::<String, DeserializationFn<Box<dyn TestTrait>>>::new();

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
