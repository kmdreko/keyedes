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

// { "data": "", "id": "" }

type DeserializationMap<K, T> = HashMap<
    K,
    Box<dyn Fn(&mut dyn erased_serde::Deserializer) -> Result<Box<T>, erased_serde::Error>>,
>;

pub fn deserialize_boxed_trait<'de, D, K, T>(
    deserializer: D,
    deserialization_map: &DeserializationMap<K, T>,
) -> Result<Box<T>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + Eq + Hash,
{
    extern crate serde as _serde;

    use _serde::de::Error;

    struct __Seed<'de, 'a, K, T> {
        field: K,
        marker: PhantomData<Box<T>>,
        lifetime: PhantomData<&'de ()>,
        map: &'a DeserializationMap<K, T>,
    }
    impl<'de, 'a, K, T> _serde::de::DeserializeSeed<'de> for __Seed<'de, 'a, K, T>
    where
        K: Eq + Hash,
    {
        type Value = Box<T>;
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
        marker: PhantomData<Box<T>>,
        lifetime: PhantomData<&'de ()>,
        map: &'a DeserializationMap<K, T>,
    }
    impl<'de, 'a, K, T> _serde::de::Visitor<'de> for __Visitor<'de, 'a, K, T>
    where
        K: Deserialize<'de> + Eq + Hash,
    {
        type Value = Box<T>;
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
                        None => match __field {
                            // TODO: figure out deserializing from
                            // missing data field, some trait objects
                            // might not have data and thats ok, we
                            // should handle that case
                            _ => todo!(),
                            // __Field::__field0 => {
                            //     _serde::__private::de::missing_field("data").map(TempEnum::A)
                            // }
                            // __Field::__field1 => {
                            //     _serde::__private::de::missing_field("data").map(TempEnum::B)
                            // }
                            // __Field::__field2 => {
                            //     _serde::__private::de::missing_field("data").map(TempEnum::C)
                            // }
                        },
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
        "Box<T>",
        FIELDS,
        __Visitor::<K, T> {
            marker: PhantomData::<Box<T>>,
            lifetime: PhantomData,
            map: &deserialization_map,
        },
    )
}
