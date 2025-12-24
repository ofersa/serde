//! Tests for SerializeOwned trait and Serializer::serialize_owned method.

use serde_core::ser::{Error, Impossible, Serialize, SerializeOwned, Serializer};
use std::fmt::Display;

/// A minimal test serializer that tracks what was serialized.
struct TestSerializer {
    output: String,
}

impl TestSerializer {
    fn new() -> Self {
        TestSerializer {
            output: String::new(),
        }
    }
}

/// Simple error type for the test serializer.
#[derive(Debug)]
struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

impl Error for TestError {
    fn custom<T: Display>(msg: T) -> Self {
        TestError(msg.to_string())
    }
}

impl Serializer for TestSerializer {
    type Ok = String;
    type Error = TestError;
    type SerializeSeq = Impossible<String, TestError>;
    type SerializeTuple = Impossible<String, TestError>;
    type SerializeTupleStruct = Impossible<String, TestError>;
    type SerializeTupleVariant = Impossible<String, TestError>;
    type SerializeMap = Impossible<String, TestError>;
    type SerializeStruct = Impossible<String, TestError>;
    type SerializeStructVariant = Impossible<String, TestError>;

    fn serialize_bool(mut self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("bool:{}", v));
        Ok(self.output)
    }

    fn serialize_i8(mut self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("i8:{}", v));
        Ok(self.output)
    }

    fn serialize_i16(mut self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("i16:{}", v));
        Ok(self.output)
    }

    fn serialize_i32(mut self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("i32:{}", v));
        Ok(self.output)
    }

    fn serialize_i64(mut self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("i64:{}", v));
        Ok(self.output)
    }

    fn serialize_u8(mut self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("u8:{}", v));
        Ok(self.output)
    }

    fn serialize_u16(mut self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("u16:{}", v));
        Ok(self.output)
    }

    fn serialize_u32(mut self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("u32:{}", v));
        Ok(self.output)
    }

    fn serialize_u64(mut self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("u64:{}", v));
        Ok(self.output)
    }

    fn serialize_f32(mut self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("f32:{}", v));
        Ok(self.output)
    }

    fn serialize_f64(mut self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("f64:{}", v));
        Ok(self.output)
    }

    fn serialize_char(mut self, v: char) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("char:{}", v));
        Ok(self.output)
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("str:{}", v));
        Ok(self.output)
    }

    fn serialize_bytes(mut self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("bytes:{:?}", v));
        Ok(self.output)
    }

    fn serialize_none(mut self) -> Result<Self::Ok, Self::Error> {
        self.output.push_str("none");
        Ok(self.output)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }

    fn serialize_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.output.push_str("unit");
        Ok(self.output)
    }

    fn serialize_unit_struct(mut self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&format!("unit_struct:{}", name));
        Ok(self.output)
    }

    fn serialize_unit_variant(
        mut self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.output
            .push_str(&format!("unit_variant:{}::{}", name, variant));
        Ok(self.output)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(TestError("not implemented".to_string()))
    }
}

/// Test that the blanket implementation of SerializeOwned for &T works.
/// When a reference to a Serialize type is used with serialize_owned,
/// it should delegate to the Serialize implementation.
#[test]
fn test_serialize_owned_blanket_impl_for_reference() {
    let value: i32 = 42;
    let serializer = TestSerializer::new();

    // Use the blanket impl: &T: SerializeOwned where T: Serialize
    let result = (&value).serialize_owned(serializer);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "i32:42");
}

/// Test that serialize_owned works with string references.
#[test]
fn test_serialize_owned_with_str_reference() {
    let value: &str = "hello";
    let serializer = TestSerializer::new();

    let result = (&value).serialize_owned(serializer);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "str:hello");
}

/// Test that the default serialize_owned implementation on Serializer panics.
/// This verifies the backwards-compatibility behavior where serializers that
/// don't override serialize_owned will panic when it's called.
#[test]
#[should_panic(expected = "serialize_owned is not implemented for this Serializer")]
fn test_default_serialize_owned_panics() {
    let serializer = TestSerializer::new();
    let value: i32 = 42;

    // This should panic because TestSerializer uses the default serialize_owned
    let _ = serializer.serialize_owned(&value);
}

/// Test that SerializeOwned trait can be used as a bound.
fn serialize_with_owned_bound<T, S>(value: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: SerializeOwned,
    S: Serializer,
{
    value.serialize_owned(serializer)
}

#[test]
fn test_serialize_owned_as_trait_bound() {
    let value: i32 = 123;
    let serializer = TestSerializer::new();

    // Pass a reference, which uses the blanket impl
    let result = serialize_with_owned_bound(&value, serializer);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "i32:123");
}

/// Test that SerializeOwned works with bool references.
#[test]
fn test_serialize_owned_with_bool() {
    let value: bool = true;
    let serializer = TestSerializer::new();

    let result = (&value).serialize_owned(serializer);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "bool:true");
}

/// Test that SerializeOwned works with unit type.
#[test]
fn test_serialize_owned_with_unit() {
    let value: () = ();
    let serializer = TestSerializer::new();

    let result = (&value).serialize_owned(serializer);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "unit");
}
