//! Extract typed information from the request's path.
//!
//! Thanks: <https://github.com/ntex-rs>

use std::{
    fmt::Display,
    iter::Peekable,
    ops::{Deref, DerefMut},
    slice::Iter,
    str::FromStr,
};

use serde::{
    de::{self, DeserializeOwned, Deserializer, Error as DeError, Visitor},
    forward_to_deserialize_any, Deserialize,
};

use viz_utils::{futures::future::BoxFuture, thiserror::Error as ThisError, tracing};

use crate::{http, Context, Extract, Response, Result};

/// Params Error
#[derive(ThisError, Debug, PartialEq)]
pub enum ParamsError {
    /// Failed to read single param
    #[error("failed to read param: {0:?}")]
    SingleRead(String),
    /// Failed to parse single param
    #[error("failed to parse param: {0:?}")]
    SingleParse(String),
    /// Failed to read params
    #[error("failed to read params")]
    Read,
    /// Failed to parse params
    #[error("failed to parse params")]
    Parse,
}

impl From<ParamsError> for Response {
    fn from(e: ParamsError) -> Self {
        (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into()
    }
}

/// Extract typed information from the request's path.
#[derive(Clone, Debug, Deserialize)]
pub struct Params<T = Vec<(String, String)>>(pub T);

impl Params {
    /// Gets single parameter by name.
    pub fn find<T: FromStr>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.iter()
            .find(|p| p.0 == name)
            .ok_or_else(|| ParamsError::SingleRead(name.to_string()))?
            .1
            .parse()
            .map_err(|e: T::Err| {
                tracing::debug!("Params deserialize error: {}", e);
                ParamsError::SingleParse(name.to_string())
            })
    }
}

impl<T> Params<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for Params<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Params<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Params<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl From<Vec<(String, String)>> for Params {
    fn from(v: Vec<(String, String)>) -> Self {
        Params(v)
    }
}

impl From<Vec<(&str, &str)>> for Params {
    fn from(v: Vec<(&str, &str)>) -> Self {
        Params(v.iter().map(|p| (p.0.to_string(), p.1.to_string())).collect())
    }
}

impl<T> Extract for Params<T>
where
    T: DeserializeOwned,
{
    type Error = ParamsError;

    #[inline]
    fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
        Box::pin(async move { cx.params() })
    }
}

impl Context {
    /// Gets parameters.
    pub fn params<T>(&self) -> Result<T, ParamsError>
    where
        T: DeserializeOwned,
    {
        de::Deserialize::deserialize(ParamsDeserializer::new(
            &self
                .extensions()
                .get::<Params>()
                .map(|ps| {
                    Params(ps.iter().map(|p| (p.0.as_str(), p.1.as_str())).collect::<Vec<_>>())
                })
                .ok_or(ParamsError::Read)?,
        ))
        .map_err(|e| {
            tracing::error!("Params deserialize error: {}", e);
            ParamsError::Parse
        })
    }

    /// Gets single parameter by name.
    pub fn param<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.extensions().get::<Params>().ok_or(ParamsError::Read)?.find(name)
    }
}

macro_rules! unsupported_type {
    ($trait_fn:ident, $name:expr) => {
        fn $trait_fn<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(de::value::Error::custom(concat!("unsupported type: ", $name)))
        }
    };
}

macro_rules! parse_single_value {
    ($trait_fn:ident, $visit_fn:ident, $tp:tt) => {
        fn $trait_fn<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            if self.len != 1 {
                Err(de::value::Error::custom(
                    format!("wrong number of parameters: {} expected 1", self.len).as_str(),
                ))
            } else {
                let v = self.params.nth(0).unwrap().1;
                visitor.$visit_fn(v.parse().map_err(|_| {
                    de::value::Error::custom(format!("can not parse {:?} to a {}", &v, $tp))
                })?)
            }
        }
    };
}

pub(crate) struct ParamsDeserializer<'de> {
    len: usize,
    params: Peekable<Iter<'de, (&'de str, &'de str)>>,
}

impl<'de> ParamsDeserializer<'de> {
    fn new(params: &'de Params<Vec<(&'de str, &'de str)>>) -> ParamsDeserializer<'de> {
        Self { len: params.len(), params: params.iter().peekable() }
    }
}

impl<'de> Deserializer<'de> for ParamsDeserializer<'de> {
    type Error = de::value::Error;

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.len < len {
            Err(de::value::Error::custom(
                format!("wrong number of parameters: {} expected {}", self.len, len).as_str(),
            ))
        } else {
            visitor.visit_seq(self)
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.len < len {
            Err(de::value::Error::custom(
                format!("wrong number of parameters: {} expected {}", self.len, len).as_str(),
            ))
        } else {
            visitor.visit_seq(self)
        }
    }

    fn deserialize_enum<V>(
        mut self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.len < 1 {
            Err(de::value::Error::custom("expeceted at least one parameters"))
        } else {
            visitor.visit_enum(ValueEnum { value: self.params.next().unwrap().1 })
        }
    }

    fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.len < 1 {
            Err(de::value::Error::custom(
                format!("wrong number of parameters: {} expected 1", self.len).as_str(),
            ))
        } else {
            visitor.visit_borrowed_str(self.params.next().unwrap().1)
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    unsupported_type!(deserialize_any, "'any'");
    unsupported_type!(deserialize_bytes, "bytes");
    unsupported_type!(deserialize_option, "Option<T>");
    unsupported_type!(deserialize_identifier, "identifier");
    unsupported_type!(deserialize_ignored_any, "ignored_any");

    parse_single_value!(deserialize_bool, visit_bool, "bool");
    parse_single_value!(deserialize_i8, visit_i8, "i8");
    parse_single_value!(deserialize_i16, visit_i16, "i16");
    parse_single_value!(deserialize_i32, visit_i32, "i32");
    parse_single_value!(deserialize_i64, visit_i64, "i64");
    parse_single_value!(deserialize_u8, visit_u8, "u8");
    parse_single_value!(deserialize_u16, visit_u16, "u16");
    parse_single_value!(deserialize_u32, visit_u32, "u32");
    parse_single_value!(deserialize_u64, visit_u64, "u64");
    parse_single_value!(deserialize_f32, visit_f32, "f32");
    parse_single_value!(deserialize_f64, visit_f64, "f64");
    parse_single_value!(deserialize_string, visit_string, "String");
    parse_single_value!(deserialize_byte_buf, visit_string, "String");
    parse_single_value!(deserialize_char, visit_char, "char");
}

impl<'de> de::MapAccess<'de> for ParamsDeserializer<'de> {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.params.peek() {
            Some(item) => Ok(Some(seed.deserialize(Key { key: item.0 })?)),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.params.next() {
            Some(item) => Ok(seed.deserialize(Value { value: item.1 })?),
            None => Err(de::value::Error::custom("unexpected item")),
        }
    }
}

impl<'de> de::SeqAccess<'de> for ParamsDeserializer<'de> {
    type Error = de::value::Error;

    fn next_element_seed<U>(&mut self, seed: U) -> Result<Option<U::Value>, Self::Error>
    where
        U: de::DeserializeSeed<'de>,
    {
        match self.params.next() {
            Some(item) => Ok(Some(seed.deserialize(Value { value: item.1 })?)),
            None => Ok(None),
        }
    }
}

struct Key<'de> {
    key: &'de str,
}

impl<'de> Deserializer<'de> for Key<'de> {
    type Error = de::value::Error;

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.key)
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::value::Error::custom("Unexpected"))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum ignored_any
    }
}

macro_rules! parse_value {
    ($trait_fn:ident, $visit_fn:ident, $tp:tt) => {
        fn $trait_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            visitor.$visit_fn(self.value.parse().map_err(|_| {
                de::value::Error::custom(format!("can not parse {:?} to a {}", self.value, $tp))
            })?)
        }
    };
}

struct Value<'de> {
    value: &'de str,
}

impl<'de> Deserializer<'de> for Value<'de> {
    type Error = de::value::Error;

    parse_value!(deserialize_bool, visit_bool, "bool");
    parse_value!(deserialize_i8, visit_i8, "i8");
    parse_value!(deserialize_i16, visit_i16, "i16");
    parse_value!(deserialize_i32, visit_i32, "i16");
    parse_value!(deserialize_i64, visit_i64, "i64");
    parse_value!(deserialize_u8, visit_u8, "u8");
    parse_value!(deserialize_u16, visit_u16, "u16");
    parse_value!(deserialize_u32, visit_u32, "u32");
    parse_value!(deserialize_u64, visit_u64, "u64");
    parse_value!(deserialize_f32, visit_f32, "f32");
    parse_value!(deserialize_f64, visit_f64, "f64");
    parse_value!(deserialize_string, visit_string, "String");
    parse_value!(deserialize_byte_buf, visit_string, "String");
    parse_value!(deserialize_char, visit_char, "char");

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.value.as_bytes())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.value)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(ValueEnum { value: self.value })
    }

    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, _: usize, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::value::Error::custom("unsupported type: tuple"))
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        _: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::value::Error::custom("unsupported type: struct"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        _: usize,
        _: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::value::Error::custom("unsupported type: tuple struct"))
    }

    unsupported_type!(deserialize_any, "any");
    unsupported_type!(deserialize_seq, "seq");
    unsupported_type!(deserialize_map, "map");
    unsupported_type!(deserialize_identifier, "identifier");
}

struct ValueEnum<'de> {
    value: &'de str,
}

impl<'de> de::EnumAccess<'de> for ValueEnum<'de> {
    type Error = de::value::Error;
    type Variant = UnitVariant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(Key { key: self.value })?, UnitVariant))
    }
}

struct UnitVariant;

impl<'de> de::VariantAccess<'de> for UnitVariant {
    type Error = de::value::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(de::value::Error::custom("not supported"))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::value::Error::custom("not supported"))
    }

    fn struct_variant<V>(self, _: &'static [&'static str], _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::value::Error::custom("not supported"))
    }
}

#[cfg(test)]
mod tests {
    use serde::{de, Deserialize};

    use super::{Params, ParamsDeserializer};

    #[derive(Debug, Deserialize)]
    struct MyStruct {
        key: String,
        value: String,
    }

    #[derive(Deserialize)]
    struct Id {
        _id: String,
    }

    #[derive(Debug, Deserialize)]
    struct Test1(String, u32);

    #[derive(Debug, Deserialize)]
    struct Test2 {
        key: String,
        value: u32,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum TestEnum {
        Val1,
        Val2,
    }

    #[derive(Debug, Deserialize)]
    struct Test3 {
        val: TestEnum,
    }

    #[allow(clippy::unit_cmp)]
    #[test]
    fn test_request_extract() {
        let params = Params(vec![("key", "name"), ("value", "user1")]);

        let s: () = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(s, ());

        let s: MyStruct = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(s.key, "name");
        assert_eq!(s.value, "user1");

        let s: (String, String) =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(s.0, "name");
        assert_eq!(s.1, "user1");

        let s: &str = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(s, "name");

        let params = Params(vec![("key", "name"), ("value", "32")]);

        let s: Test1 = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(s.0, "name");
        assert_eq!(s.1, 32);

        #[derive(Deserialize)]
        struct T(Test1);
        let s: T = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!((s.0).0, "name");
        assert_eq!((s.0).1, 32);

        let s: Test2 = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(s.key, "name");
        assert_eq!(s.value, 32);

        let s: Result<(Test2,), _> = de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_err());

        let s: (String, u8) =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(s.0, "name");
        assert_eq!(s.1, 32);

        let res: Vec<String> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(res[0], "name".to_owned());
        assert_eq!(res[1], "32".to_owned());

        #[derive(Debug, Deserialize)]
        struct S2(());
        let s: Result<S2, de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_ok());

        let s: Result<(), de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_ok());

        let s: Result<(String, ()), de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_ok());
    }

    #[test]
    fn test_extract_path_single() {
        let params = Params(vec![("value", "32")]);

        let i: i8 = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(i, 32);

        let i: (i8,) = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(i, (32,));

        let i: Result<(i8, i8), _> = de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(i.is_err());

        #[derive(Deserialize)]
        struct Test(i8);
        let i: Test = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(i.0, 32);
    }

    #[test]
    fn test_extract_enum() {
        let params = Params(vec![("val", "val1")]);

        let i: TestEnum = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(i, TestEnum::Val1);

        let params = Params(vec![("val1", "val1"), ("val2", "val2")]);

        let i: (TestEnum, TestEnum) =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(i, (TestEnum::Val1, TestEnum::Val2));
    }

    #[test]
    fn test_extract_enum_value() {
        let params = Params(vec![("val", "val1")]);

        let i: Test3 = de::Deserialize::deserialize(ParamsDeserializer::new(&params)).unwrap();
        assert_eq!(i.val, TestEnum::Val1);

        let params = Params(vec![("val", "val3")]);

        let i: Result<Test3, de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(i.is_err());
        assert!(format!("{:?}", i).contains("unknown variant"));
    }

    #[test]
    fn test_extract_errors() {
        let params = Params(vec![("value", "name")]);

        let s: Result<Test1, de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_err());
        assert!(format!("{:?}", s).contains("wrong number of parameters"));

        let s: Result<Test2, de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_err());
        assert!(format!("{:?}", s).contains("can not parse"));

        let s: Result<(String, String), de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_err());
        assert!(format!("{:?}", s).contains("wrong number of parameters"));

        let s: Result<u32, de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_err());
        assert!(format!("{:?}", s).contains("can not parse"));

        #[derive(Debug, Deserialize)]
        struct S {
            inner: (String,),
        }
        let s: Result<S, de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_err());
        assert!(format!("{:?}", s).contains("missing field `inner`"));

        let params = Params(vec![]);
        let s: Result<&str, de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_err());
        assert!(format!("{:?}", s).contains("wrong number of parameters: 0 expected 1"));

        let s: Result<TestEnum, de::value::Error> =
            de::Deserialize::deserialize(ParamsDeserializer::new(&params));
        assert!(s.is_err());
        assert!(format!("{:?}", s).contains("expeceted at least one parameters"));
    }
}
