use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::Hash;
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer};

// #[derive(Deserialize)]
// #[serde(tag = "id", content = "data")]
// pub enum TempEnum {
//     A(X),
//     B(Y),
//     C(Z),
// }

// { "id": "", "data": null}

type DeserializationMap<K, T> =
    HashMap<K, Box<dyn Fn(&mut dyn erased_serde::Deserializer) -> Result<T, erased_serde::Error>>>;

pub fn deserialize_by_map<'de, D, K, T>(
    deserializer: D,
    deserialization_map: &DeserializationMap<K, T>,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + Eq + Hash,
{
    extern crate serde as _serde;

    use _serde::de::Error;

    struct MissingFieldDeserializer<E>(&'static str, PhantomData<E>);
    impl<'de, E> Deserializer<'de> for MissingFieldDeserializer<E>
    where
        E: Error,
    {
        type Error = E;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, E>
        where
            V: _serde::de::Visitor<'de>,
        {
            Err(Error::missing_field(self.0))
        }

        fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, E>
        where
            V: _serde::de::Visitor<'de>,
        {
            visitor.visit_unit()
        }

        fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, E>
        where
            V: _serde::de::Visitor<'de>,
        {
            visitor.visit_unit()
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E>
        where
            V: _serde::de::Visitor<'de>,
        {
            visitor.visit_none()
        }

        _serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }
    struct __Seed<'de, 'a, K, T> {
        field: K,
        marker: PhantomData<T>,
        lifetime: PhantomData<&'de ()>,
        map: &'a DeserializationMap<K, T>,
    }
    impl<'de, 'a, K, T> _serde::de::DeserializeSeed<'de> for __Seed<'de, 'a, K, T>
    where
        K: Eq + Hash,
    {
        type Value = T;
        fn deserialize<__D>(self, __deserializer: __D) -> Result<Self::Value, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            let deserialization_fn = self
                .map
                .get(&self.field)
                .ok_or_else(|| __D::Error::custom("unknown deserialization key"))?;

            deserialization_fn(&mut <dyn erased_serde::Deserializer>::erase(__deserializer))
                .map_err(__D::Error::custom)
        }
    }
    struct __Visitor<'de, 'a, K, T> {
        marker: PhantomData<T>,
        lifetime: PhantomData<&'de ()>,
        map: &'a DeserializationMap<K, T>,
    }
    impl<'de, 'a, K, T> _serde::de::Visitor<'de> for __Visitor<'de, 'a, K, T>
    where
        K: Deserialize<'de> + Eq + Hash,
    {
        type Value = T;
        fn expecting(&self, __formatter: &mut Formatter) -> _serde::__private::fmt::Result {
            Formatter::write_str(__formatter, "adjacently tagged enum TempEnum")
        }
        fn visit_map<__A>(self, mut __map: __A) -> Result<Self::Value, __A::Error>
        where
            __A: _serde::de::MapAccess<'de>,
        {
            match {
                let mut __rk: Option<_serde::__private::de::TagOrContentField> = None;
                while let Some(__k) = match _serde::de::MapAccess::next_key_seed(
                    &mut __map,
                    _serde::__private::de::TagContentOtherFieldVisitor {
                        tag: "id",
                        content: "data",
                    },
                ) {
                    Ok(__val) => __val,
                    Err(__err) => {
                        return Err(__err);
                    }
                } {
                    match __k {
                        _serde::__private::de::TagContentOtherField::Other => {
                            match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                &mut __map,
                            ) {
                                Ok(__val) => __val,
                                Err(__err) => {
                                    return Err(__err);
                                }
                            };
                            continue;
                        }
                        _serde::__private::de::TagContentOtherField::Tag => {
                            __rk = Some(_serde::__private::de::TagOrContentField::Tag);
                            break;
                        }
                        _serde::__private::de::TagContentOtherField::Content => {
                            __rk = Some(_serde::__private::de::TagOrContentField::Content);
                            break;
                        }
                    }
                }
                __rk
            } {
                Some(_serde::__private::de::TagOrContentField::Tag) => {
                    let __field = match _serde::de::MapAccess::next_value(&mut __map) {
                        Ok(__val) => __val,
                        Err(__err) => {
                            return Err(__err);
                        }
                    };
                    match {
                        let mut __rk: Option<_serde::__private::de::TagOrContentField> = None;
                        while let Some(__k) = match _serde::de::MapAccess::next_key_seed(
                            &mut __map,
                            _serde::__private::de::TagContentOtherFieldVisitor {
                                tag: "id",
                                content: "data",
                            },
                        ) {
                            Ok(__val) => __val,
                            Err(__err) => {
                                return Err(__err);
                            }
                        } {
                            match __k {
                                _serde::__private::de::TagContentOtherField::Other => {
                                    match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                        &mut __map,
                                    ) {
                                        Ok(__val) => __val,
                                        Err(__err) => {
                                            return Err(__err);
                                        }
                                    };
                                    continue;
                                }
                                _serde::__private::de::TagContentOtherField::Tag => {
                                    __rk = Some(_serde::__private::de::TagOrContentField::Tag);
                                    break;
                                }
                                _serde::__private::de::TagContentOtherField::Content => {
                                    __rk = Some(_serde::__private::de::TagOrContentField::Content);
                                    break;
                                }
                            }
                        }
                        __rk
                    } {
                        Some(_serde::__private::de::TagOrContentField::Tag) => {
                            Err(<__A::Error as _serde::de::Error>::duplicate_field("id"))
                        }
                        Some(_serde::__private::de::TagOrContentField::Content) => {
                            let __ret = match _serde::de::MapAccess::next_value_seed(
                                &mut __map,
                                __Seed {
                                    field: __field,
                                    marker: PhantomData,
                                    lifetime: PhantomData,
                                    map: self.map,
                                },
                            ) {
                                Ok(__val) => __val,
                                Err(__err) => {
                                    return Err(__err);
                                }
                            };
                            match {
                                let mut __rk: Option<_serde::__private::de::TagOrContentField> =
                                    None;
                                while let Some(__k) = match _serde::de::MapAccess::next_key_seed(
                                    &mut __map,
                                    _serde::__private::de::TagContentOtherFieldVisitor {
                                        tag: "id",
                                        content: "data",
                                    },
                                ) {
                                    Ok(__val) => __val,
                                    Err(__err) => {
                                        return Err(__err);
                                    }
                                } {
                                    match __k {
                                        _serde::__private::de::TagContentOtherField::Other => {
                                            match _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(
                                                &mut __map
                                            ) {
                                                Ok(__val) => __val,
                                                Err(__err) => {
                                                    return Err(__err);
                                                }
                                            };
                                            continue;
                                        }
                                        _serde::__private::de::TagContentOtherField::Tag => {
                                            __rk =
                                                Some(_serde::__private::de::TagOrContentField::Tag);
                                            break;
                                        }
                                        _serde::__private::de::TagContentOtherField::Content => {
                                            __rk = Some(
                                                _serde::__private::de::TagOrContentField::Content,
                                            );
                                            break;
                                        }
                                    }
                                }
                                __rk
                            } {
                                Some(_serde::__private::de::TagOrContentField::Tag) => {
                                    Err(<__A::Error as _serde::de::Error>::duplicate_field("id"))
                                }
                                Some(_serde::__private::de::TagOrContentField::Content) => {
                                    Err(<__A::Error as _serde::de::Error>::duplicate_field("data"))
                                }
                                None => Ok(__ret),
                            }
                        }
                        None => {
                            let __deserializer =
                                MissingFieldDeserializer::<__A::Error>("data", PhantomData);

                            let deserialization_fn = self
                                .map
                                .get(&__field)
                                .ok_or_else(|| __A::Error::custom("unknown deserialization key"))?;

                            deserialization_fn(&mut <dyn erased_serde::Deserializer>::erase(
                                __deserializer,
                            ))
                            .map_err(__A::Error::custom)
                        }
                    }
                }
                Some(_serde::__private::de::TagOrContentField::Content) => {
                    let __content = match _serde::de::MapAccess::next_value::<
                        _serde::__private::de::Content,
                    >(&mut __map)
                    {
                        Ok(__val) => __val,
                        Err(__err) => {
                            return Err(__err);
                        }
                    };
                    match {
                        let mut __rk: Option<_serde::__private::de::TagOrContentField> = None;
                        while let Some(__k) = match _serde::de::MapAccess::next_key_seed(
                            &mut __map,
                            _serde::__private::de::TagContentOtherFieldVisitor {
                                tag: "id",
                                content: "data",
                            },
                        ) {
                            Ok(__val) => __val,
                            Err(__err) => {
                                return Err(__err);
                            }
                        } {
                            match __k {
                                _serde::__private::de::TagContentOtherField::Other => {
                                    match _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                        &mut __map,
                                    ) {
                                        Ok(__val) => __val,
                                        Err(__err) => {
                                            return Err(__err);
                                        }
                                    };
                                    continue;
                                }
                                _serde::__private::de::TagContentOtherField::Tag => {
                                    __rk = Some(_serde::__private::de::TagOrContentField::Tag);
                                    break;
                                }
                                _serde::__private::de::TagContentOtherField::Content => {
                                    __rk = Some(_serde::__private::de::TagOrContentField::Content);
                                    break;
                                }
                            }
                        }
                        __rk
                    } {
                        Some(_serde::__private::de::TagOrContentField::Tag) => {
                            let __deserializer = _serde::__private::de::ContentDeserializer::<
                                __A::Error,
                            >::new(__content);
                            let __val = match _serde::de::MapAccess::next_value(&mut __map) {
                                Ok(__val) => __val,
                                Err(__err) => {
                                    return Err(__err);
                                }
                            };

                            let deserialization_fn = self
                                .map
                                .get(&__val)
                                .ok_or_else(|| __A::Error::custom("unknown deserialization key"))?;

                            let __val2 = deserialization_fn(
                                &mut <dyn erased_serde::Deserializer>::erase(__deserializer),
                            )
                            .map_err(__A::Error::custom);

                            let __ret = match __val2 {
                                Ok(__val) => __val,
                                Err(__err) => {
                                    return Err(__err);
                                }
                            };
                            match {
                                let mut __rk: Option<_serde::__private::de::TagOrContentField> =
                                    None;
                                while let Some(__k) = match _serde::de::MapAccess::next_key_seed(
                                    &mut __map,
                                    _serde::__private::de::TagContentOtherFieldVisitor {
                                        tag: "id",
                                        content: "data",
                                    },
                                ) {
                                    Ok(__val) => __val,
                                    Err(__err) => {
                                        return Err(__err);
                                    }
                                } {
                                    match __k {
                                        _serde::__private::de::TagContentOtherField::Other => {
                                            match _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(
                                                &mut __map
                                            ) {
                                                Ok(__val) => __val,
                                                Err(__err) => {
                                                    return Err(__err);
                                                }
                                            };
                                            continue;
                                        }
                                        _serde::__private::de::TagContentOtherField::Tag => {
                                            __rk =
                                                Some(_serde::__private::de::TagOrContentField::Tag);
                                            break;
                                        }
                                        _serde::__private::de::TagContentOtherField::Content => {
                                            __rk = Some(
                                                _serde::__private::de::TagOrContentField::Content,
                                            );
                                            break;
                                        }
                                    }
                                }
                                __rk
                            } {
                                Some(_serde::__private::de::TagOrContentField::Tag) => {
                                    Err(<__A::Error as _serde::de::Error>::duplicate_field("id"))
                                }
                                Some(_serde::__private::de::TagOrContentField::Content) => {
                                    Err(<__A::Error as _serde::de::Error>::duplicate_field("data"))
                                }
                                None => Ok(__ret),
                            }
                        }
                        Some(_serde::__private::de::TagOrContentField::Content) => {
                            Err(<__A::Error as _serde::de::Error>::duplicate_field("data"))
                        }
                        None => Err(<__A::Error as _serde::de::Error>::missing_field("id")),
                    }
                }
                None => Err(<__A::Error as _serde::de::Error>::missing_field("id")),
            }
        }
        fn visit_seq<__A>(self, mut __seq: __A) -> Result<Self::Value, __A::Error>
        where
            __A: _serde::de::SeqAccess<'de>,
        {
            match match _serde::de::SeqAccess::next_element(&mut __seq) {
                Ok(__val) => __val,
                Err(__err) => {
                    return Err(__err);
                }
            } {
                Some(__field) => {
                    match match _serde::de::SeqAccess::next_element_seed(
                        &mut __seq,
                        __Seed::<K, T> {
                            field: __field,
                            marker: PhantomData,
                            lifetime: PhantomData,
                            map: self.map,
                        },
                    ) {
                        Ok(__val) => __val,
                        Err(__err) => {
                            return Err(__err);
                        }
                    } {
                        Some(__ret) => Ok(__ret),
                        None => Err(_serde::de::Error::invalid_length(1, &self)),
                    }
                }
                None => Err(_serde::de::Error::invalid_length(0, &self)),
            }
        }
    }
    const FIELDS: &'static [&'static str] = &["id", "data"];
    _serde::Deserializer::deserialize_struct(
        deserializer,
        "T",
        FIELDS,
        __Visitor::<K, T> {
            marker: PhantomData::<T>,
            lifetime: PhantomData,
            map: &deserialization_map,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Test {
        A(i32),
        B(String),
        C,
    }

    #[test]
    fn does_it_work() {
        let json1 = r#"{"id":"A","data":123}"#;
        let json2 = r#"{"id":"B","data":"BLAH"}"#;
        let json3 = r#"{"id":"C","data":null}"#;
        let json4 = r#"{"id":"C"}"#;

        let mut map = DeserializationMap::<String, Test>::new();
        map.insert(
            "A".to_string(),
            Box::new(|deserializer| i32::deserialize(deserializer).map(Test::A)),
        );
        map.insert(
            "B".to_string(),
            Box::new(|deserializer| String::deserialize(deserializer).map(Test::B)),
        );
        map.insert(
            "C".to_string(),
            Box::new(|deserializer| {
                extern crate serde as _serde;

                // apparently this is how you're supposed to deserialize a unit
                // struct
                struct __Visitor;
                impl<'de> _serde::de::Visitor<'de> for __Visitor {
                    type Value = ();
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "unit struct ()")
                    }
                    #[inline]
                    fn visit_unit<__E>(self) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        _serde::__private::Ok(())
                    }
                    #[inline]
                    fn visit_none<__E>(self) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        _serde::__private::Ok(())
                    }
                }
                _serde::Deserializer::deserialize_unit_struct(deserializer, "()", __Visitor)?;

                Ok(Test::C)
            }),
        );

        let mut deserializer = serde_json::Deserializer::from_str(json1);
        let result = deserialize_by_map(&mut deserializer, &map).unwrap();
        assert_eq!(result, Test::A(123));

        let mut deserializer = serde_json::Deserializer::from_str(json2);
        let result = deserialize_by_map(&mut deserializer, &map).unwrap();
        assert_eq!(result, Test::B("BLAH".to_string()));

        let mut deserializer = serde_json::Deserializer::from_str(json3);
        let result = deserialize_by_map(&mut deserializer, &map).unwrap();
        assert_eq!(result, Test::C);

        let mut deserializer = serde_json::Deserializer::from_str(json4);
        let result = deserialize_by_map(&mut deserializer, &map).unwrap();
        assert_eq!(result, Test::C);
    }

    #[test]
    fn missing_data_is_handled() {
        let json1 = r#"{"id":"A"}"#;
        let json2 = r#"{"id":"B"}"#;
        let json3 = r#"{"id":"C"}"#;

        let mut map = DeserializationMap::<String, Test>::new();
        map.insert(
            "A".to_string(),
            Box::new(|deserializer| i32::deserialize(deserializer).map(Test::A)),
        );
        map.insert(
            "B".to_string(),
            Box::new(|deserializer| String::deserialize(deserializer).map(Test::B)),
        );
        map.insert(
            "C".to_string(),
            Box::new(|deserializer| {
                extern crate serde as _serde;

                // apparently this is how you're supposed to deserialize a unit
                // struct
                struct __Visitor;
                impl<'de> _serde::de::Visitor<'de> for __Visitor {
                    type Value = ();
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "unit struct ()")
                    }
                    #[inline]
                    fn visit_unit<__E>(self) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        _serde::__private::Ok(())
                    }
                    #[inline]
                    fn visit_none<__E>(self) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        _serde::__private::Ok(())
                    }
                }
                _serde::Deserializer::deserialize_unit_struct(deserializer, "()", __Visitor)?;

                Ok(Test::C)
            }),
        );

        let mut deserializer = serde_json::Deserializer::from_str(json1);
        let result = deserialize_by_map(&mut deserializer, &map);
        assert!(result.is_err());

        let mut deserializer = serde_json::Deserializer::from_str(json2);
        let result = deserialize_by_map(&mut deserializer, &map);
        assert!(result.is_err());

        let mut deserializer = serde_json::Deserializer::from_str(json3);
        let result = deserialize_by_map(&mut deserializer, &map).unwrap();
        assert_eq!(result, Test::C);
    }

    #[test]
    fn returns_error_on_unknown_key_type() {
        let json = r#"{"id":"D","data":5.02}"#;

        let mut map = DeserializationMap::<String, Test>::new();
        map.insert(
            "A".to_string(),
            Box::new(|deserializer| i32::deserialize(deserializer).map(Test::A)),
        );
        map.insert(
            "B".to_string(),
            Box::new(|deserializer| String::deserialize(deserializer).map(Test::B)),
        );

        let mut deserializer = serde_json::Deserializer::from_str(json);
        let result = deserialize_by_map(&mut deserializer, &map);
        assert!(result.is_err());
    }
}
