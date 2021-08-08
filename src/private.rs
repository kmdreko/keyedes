use std::marker::PhantomData;

// validated as of serde@1.0.126
use serde::__private::de::{
    Content, ContentDeserializer, TagContentOtherField, TagContentOtherFieldVisitor,
    TagOrContentField,
};
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct ErasedSerdeSerializeWrapper<'a, V: ?Sized>(pub &'a V);
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

struct MissingFieldDeserializer<E>(&'static str, PhantomData<E>);

impl<'de, E> Deserializer<'de> for MissingFieldDeserializer<E>
where
    E: Error,
{
    type Error = E;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        Err(Error::missing_field(self.0))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        visitor.visit_none()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ValueDeserializeSeed<'a, F, K, T>
where
    F: Fn(K, &mut dyn erased_serde::Deserializer) -> Result<T, erased_serde::Error>,
{
    field: K,
    deserialization_fn: &'a F,
    _dummy: PhantomData<fn(K) -> T>,
}

impl<'de, 'a, F, K, T> DeserializeSeed<'de> for ValueDeserializeSeed<'a, F, K, T>
where
    F: Fn(K, &mut dyn erased_serde::Deserializer) -> Result<T, erased_serde::Error>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        (self.deserialization_fn)(
            self.field,
            &mut <dyn erased_serde::Deserializer>::erase(deserializer),
        )
        .map_err(D::Error::custom)
    }
}

pub struct KeyValueVisitor<F, K, T>
where
    F: Fn(K, &mut dyn erased_serde::Deserializer) -> Result<T, erased_serde::Error>,
{
    pub deserialization_fn: F,
    pub key_name: &'static str,
    pub value_name: &'static str,
    pub _dummy: PhantomData<fn(K) -> T>,
}

impl<'de, F, K, T> Visitor<'de> for KeyValueVisitor<F, K, T>
where
    F: Fn(K, &mut dyn erased_serde::Deserializer) -> Result<T, erased_serde::Error>,
    K: Deserialize<'de>,
{
    type Value = T;
    fn expecting(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Formatter::write_str(__formatter, "adjacently tagged enum")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        match {
            let mut __rk: Option<TagOrContentField> = None;
            while let Some(__k) = map.next_key_seed(TagContentOtherFieldVisitor {
                tag: self.key_name,
                content: self.value_name,
            })? {
                match __k {
                    TagContentOtherField::Other => {
                        map.next_value::<IgnoredAny>()?;
                        continue;
                    }
                    TagContentOtherField::Tag => {
                        __rk = Some(TagOrContentField::Tag);
                        break;
                    }
                    TagContentOtherField::Content => {
                        __rk = Some(TagOrContentField::Content);
                        break;
                    }
                }
            }
            __rk
        } {
            Some(TagOrContentField::Tag) => {
                let __field = map.next_value()?;
                match {
                    let mut __rk: Option<TagOrContentField> = None;
                    while let Some(__k) = map.next_key_seed(TagContentOtherFieldVisitor {
                        tag: self.key_name,
                        content: self.value_name,
                    })? {
                        match __k {
                            TagContentOtherField::Other => {
                                map.next_value::<IgnoredAny>()?;
                                continue;
                            }
                            TagContentOtherField::Tag => {
                                __rk = Some(TagOrContentField::Tag);
                                break;
                            }
                            TagContentOtherField::Content => {
                                __rk = Some(TagOrContentField::Content);
                                break;
                            }
                        }
                    }
                    __rk
                } {
                    Some(TagOrContentField::Tag) => Err(
                        <A::Error as serde::de::Error>::duplicate_field(self.key_name),
                    ),
                    Some(TagOrContentField::Content) => {
                        let __ret = map.next_value_seed(ValueDeserializeSeed {
                            field: __field,
                            deserialization_fn: &self.deserialization_fn,
                            _dummy: PhantomData,
                        })?;
                        match {
                            let mut __rk: Option<TagOrContentField> = None;
                            while let Some(__k) =
                                map.next_key_seed(TagContentOtherFieldVisitor {
                                    tag: self.key_name,
                                    content: self.value_name,
                                })?
                            {
                                match __k {
                                    TagContentOtherField::Other => {
                                        map.next_value::<IgnoredAny>()?;
                                        continue;
                                    }
                                    TagContentOtherField::Tag => {
                                        __rk = Some(TagOrContentField::Tag);
                                        break;
                                    }
                                    TagContentOtherField::Content => {
                                        __rk = Some(TagOrContentField::Content);
                                        break;
                                    }
                                }
                            }
                            __rk
                        } {
                            Some(TagOrContentField::Tag) => Err(
                                <A::Error as serde::de::Error>::duplicate_field(self.key_name),
                            ),
                            Some(TagOrContentField::Content) => Err(
                                <A::Error as serde::de::Error>::duplicate_field(self.value_name),
                            ),
                            None => Ok(__ret),
                        }
                    }
                    None => {
                        let __deserializer =
                            MissingFieldDeserializer::<A::Error>(self.value_name, PhantomData);

                        (self.deserialization_fn)(
                            __field,
                            &mut <dyn erased_serde::Deserializer>::erase(__deserializer),
                        )
                        .map_err(A::Error::custom)
                    }
                }
            }
            Some(TagOrContentField::Content) => {
                let __content = map.next_value::<Content>()?;
                match {
                    let mut __rk: Option<TagOrContentField> = None;
                    while let Some(__k) = map.next_key_seed(TagContentOtherFieldVisitor {
                        tag: self.key_name,
                        content: self.value_name,
                    })? {
                        match __k {
                            TagContentOtherField::Other => {
                                map.next_value::<IgnoredAny>()?;
                                continue;
                            }
                            TagContentOtherField::Tag => {
                                __rk = Some(TagOrContentField::Tag);
                                break;
                            }
                            TagContentOtherField::Content => {
                                __rk = Some(TagOrContentField::Content);
                                break;
                            }
                        }
                    }
                    __rk
                } {
                    Some(TagOrContentField::Tag) => {
                        let __deserializer = ContentDeserializer::<A::Error>::new(__content);
                        let __val = map.next_value()?;

                        let __ret = (self.deserialization_fn)(
                            __val,
                            &mut <dyn erased_serde::Deserializer>::erase(__deserializer),
                        )
                        .map_err(A::Error::custom)?;

                        match {
                            let mut __rk: Option<TagOrContentField> = None;
                            while let Some(__k) =
                                map.next_key_seed(TagContentOtherFieldVisitor {
                                    tag: self.key_name,
                                    content: self.value_name,
                                })?
                            {
                                match __k {
                                    TagContentOtherField::Other => {
                                        map.next_value::<IgnoredAny>()?;
                                        continue;
                                    }
                                    TagContentOtherField::Tag => {
                                        __rk = Some(TagOrContentField::Tag);
                                        break;
                                    }
                                    TagContentOtherField::Content => {
                                        __rk = Some(TagOrContentField::Content);
                                        break;
                                    }
                                }
                            }
                            __rk
                        } {
                            Some(TagOrContentField::Tag) => Err(
                                <A::Error as serde::de::Error>::duplicate_field(self.key_name),
                            ),
                            Some(TagOrContentField::Content) => Err(
                                <A::Error as serde::de::Error>::duplicate_field(self.value_name),
                            ),
                            None => Ok(__ret),
                        }
                    }
                    Some(TagOrContentField::Content) => Err(
                        <A::Error as serde::de::Error>::duplicate_field(self.value_name),
                    ),
                    None => Err(<A::Error as serde::de::Error>::missing_field(self.key_name)),
                }
            }
            None => Err(<A::Error as serde::de::Error>::missing_field(self.key_name)),
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        match seq.next_element()? {
            Some(__field) => {
                match seq.next_element_seed(ValueDeserializeSeed {
                    field: __field,
                    deserialization_fn: &self.deserialization_fn,
                    _dummy: PhantomData,
                })? {
                    Some(__ret) => Ok(__ret),
                    None => Err(serde::de::Error::invalid_length(1, &self)),
                }
            }
            None => Err(serde::de::Error::invalid_length(0, &self)),
        }
    }
}
