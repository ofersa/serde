//! Tests for SerializeOwned implementations on standard library types.

use serde::ser::{Serialize, SerializeOwned, Serializer};
use std::borrow::Cow;

/// Minimal test serializer for verifying SerializeOwned produces same output as Serialize.
struct TestSerializer;

#[derive(Debug)]
struct TestError;

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "test error")
    }
}

impl std::error::Error for TestError {}

impl serde::ser::Error for TestError {
    fn custom<T: std::fmt::Display>(_msg: T) -> Self {
        TestError
    }
}

impl Serializer for TestSerializer {
    type Ok = String;
    type Error = TestError;
    type SerializeSeq = serde::ser::Impossible<String, TestError>;
    type SerializeTuple = serde::ser::Impossible<String, TestError>;
    type SerializeTupleStruct = serde::ser::Impossible<String, TestError>;
    type SerializeTupleVariant = serde::ser::Impossible<String, TestError>;
    type SerializeMap = serde::ser::Impossible<String, TestError>;
    type SerializeStruct = serde::ser::Impossible<String, TestError>;
    type SerializeStructVariant = serde::ser::Impossible<String, TestError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> { Ok(format!("bool:{}", v)) }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> { Ok(format!("i8:{}", v)) }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> { Ok(format!("i16:{}", v)) }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> { Ok(format!("i32:{}", v)) }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> { Ok(format!("i64:{}", v)) }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> { Ok(format!("i128:{}", v)) }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> { Ok(format!("u8:{}", v)) }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> { Ok(format!("u16:{}", v)) }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> { Ok(format!("u32:{}", v)) }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> { Ok(format!("u64:{}", v)) }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> { Ok(format!("u128:{}", v)) }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> { Ok(format!("f32:{}", v)) }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> { Ok(format!("f64:{}", v)) }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> { Ok(format!("char:{}", v)) }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> { Ok(format!("str:{}", v)) }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> { Ok(format!("bytes:{:?}", v)) }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> { Ok("none".to_string()) }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<Self::Ok, Self::Error> {
        Ok(format!("some:{}", v.serialize(TestSerializer)?))
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> { Ok("unit".to_string()) }
    fn serialize_unit_struct(self, n: &'static str) -> Result<Self::Ok, Self::Error> { Ok(format!("unit_struct:{}", n)) }
    fn serialize_unit_variant(self, n: &'static str, i: u32, v: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(format!("unit_variant:{}::{}[{}]", n, v, i))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, n: &'static str, v: &T) -> Result<Self::Ok, Self::Error> {
        Ok(format!("newtype_struct:{}({})", n, v.serialize(TestSerializer)?))
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(self, n: &'static str, i: u32, var: &'static str, v: &T) -> Result<Self::Ok, Self::Error> {
        Ok(format!("newtype_variant:{}::{}[{}]({})", n, var, i, v.serialize(TestSerializer)?))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> { Err(TestError) }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> { Err(TestError) }
    fn serialize_tuple_struct(self, _n: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> { Err(TestError) }
    fn serialize_tuple_variant(self, _n: &'static str, _i: u32, _v: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> { Err(TestError) }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> { Err(TestError) }
    fn serialize_struct(self, _n: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> { Err(TestError) }
    fn serialize_struct_variant(self, _n: &'static str, _i: u32, _v: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> { Err(TestError) }
}

fn serialize_owned<T: SerializeOwned>(value: T) -> String {
    value.serialize_owned(TestSerializer).unwrap()
}

#[test]
fn test_string_serialize_owned() {
    assert_eq!(serialize_owned(String::from("hello")), "str:hello");
}

#[test]
fn test_box_serialize_owned() {
    assert_eq!(serialize_owned(Box::new(42i32)), "i32:42");
}

#[test]
fn test_option_serialize_owned() {
    assert_eq!(serialize_owned(Some(42i32)), "some:i32:42");
    assert_eq!(serialize_owned(Option::<i32>::None), "none");
}

#[test]
fn test_cow_serialize_owned() {
    let cow: Cow<str> = Cow::Owned(String::from("owned"));
    assert_eq!(serialize_owned(cow), "str:owned");
}

#[test]
fn test_rc_arc_serialize_owned() {
    use std::rc::Rc;
    use std::sync::Arc;
    assert_eq!(serialize_owned(Rc::new(42i32)), "i32:42");
    assert_eq!(serialize_owned(Arc::new(42i32)), "i32:42");
}
