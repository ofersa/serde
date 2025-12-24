//! Tests for the SerializeOwned trait and its blanket implementation.

#![allow(clippy::float_cmp)]

use serde::ser::{Serialize, SerializeOwned};

mod test_serializer {
    use serde::ser::{
        Error as SerError, Serialize, SerializeMap, SerializeSeq, SerializeStruct,
        SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
        Serializer,
    };
    use std::fmt::{self, Display};

    #[derive(Debug)]
    pub struct Error;

    impl Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "TestError")
        }
    }

    impl std::error::Error for Error {}

    impl SerError for Error {
        fn custom<T: Display>(_msg: T) -> Self {
            Error
        }
    }

    /// A simple serializer that captures the serialized value as a string representation.
    pub struct StringSerializer;

    impl Serializer for StringSerializer {
        type Ok = String;
        type Error = Error;
        type SerializeSeq = Impossible;
        type SerializeTuple = Impossible;
        type SerializeTupleStruct = Impossible;
        type SerializeTupleVariant = Impossible;
        type SerializeMap = Impossible;
        type SerializeStruct = Impossible;
        type SerializeStructVariant = Impossible;

        fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
            Ok("bytes".to_string())
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Ok("None".to_string())
        }

        fn serialize_some<T: ?Sized + Serialize>(
            self,
            value: &T,
        ) -> Result<Self::Ok, Self::Error> {
            value.serialize(self)
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Ok("()".to_string())
        }

        fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
            Ok(name.to_string())
        }

        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Ok(variant.to_string())
        }

        fn serialize_newtype_struct<T: ?Sized + Serialize>(
            self,
            _name: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error> {
            value.serialize(self)
        }

        fn serialize_newtype_variant<T: ?Sized + Serialize>(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error> {
            value.serialize(self)
        }

        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            Err(Error)
        }

        fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Err(Error)
        }

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(Error)
        }

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(Error)
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            Err(Error)
        }

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Err(Error)
        }

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(Error)
        }
    }

    pub struct Impossible;

    impl SerializeSeq for Impossible {
        type Ok = String;
        type Error = Error;

        fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Error> {
            Err(Error)
        }

        fn end(self) -> Result<String, Error> {
            Err(Error)
        }
    }

    impl SerializeTuple for Impossible {
        type Ok = String;
        type Error = Error;

        fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Error> {
            Err(Error)
        }

        fn end(self) -> Result<String, Error> {
            Err(Error)
        }
    }

    impl SerializeTupleStruct for Impossible {
        type Ok = String;
        type Error = Error;

        fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Error> {
            Err(Error)
        }

        fn end(self) -> Result<String, Error> {
            Err(Error)
        }
    }

    impl SerializeTupleVariant for Impossible {
        type Ok = String;
        type Error = Error;

        fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Error> {
            Err(Error)
        }

        fn end(self) -> Result<String, Error> {
            Err(Error)
        }
    }

    impl SerializeMap for Impossible {
        type Ok = String;
        type Error = Error;

        fn serialize_key<T: ?Sized + Serialize>(&mut self, _key: &T) -> Result<(), Error> {
            Err(Error)
        }

        fn serialize_value<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Error> {
            Err(Error)
        }

        fn end(self) -> Result<String, Error> {
            Err(Error)
        }
    }

    impl SerializeStruct for Impossible {
        type Ok = String;
        type Error = Error;

        fn serialize_field<T: ?Sized + Serialize>(
            &mut self,
            _key: &'static str,
            _value: &T,
        ) -> Result<(), Error> {
            Err(Error)
        }

        fn end(self) -> Result<String, Error> {
            Err(Error)
        }
    }

    impl SerializeStructVariant for Impossible {
        type Ok = String;
        type Error = Error;

        fn serialize_field<T: ?Sized + Serialize>(
            &mut self,
            _key: &'static str,
            _value: &T,
        ) -> Result<(), Error> {
            Err(Error)
        }

        fn end(self) -> Result<String, Error> {
            Err(Error)
        }
    }
}

use test_serializer::StringSerializer;

/// Test that the blanket impl `&T: SerializeOwned where T: Serialize` works.
#[test]
fn test_blanket_impl_for_reference() {
    let value: i32 = 42;
    let reference: &i32 = &value;

    // Use serialize_owned on the reference
    let result = reference.serialize_owned(StringSerializer).unwrap();
    assert_eq!(result, "42");
}

/// Test that serialize_owned produces the same result as serialize.
#[test]
fn test_serialize_owned_equals_serialize() {
    let value: &str = "hello world";

    // Serialize via Serialize::serialize
    let result1 = value.serialize(StringSerializer).unwrap();

    // Serialize via SerializeOwned::serialize_owned (blanket impl)
    let result2 = value.serialize_owned(StringSerializer).unwrap();

    assert_eq!(result1, result2);
    assert_eq!(result1, "hello world");
}

/// Test using SerializeOwned as a trait bound.
fn generic_serialize<T: SerializeOwned>(value: T) -> String {
    value.serialize_owned(StringSerializer).unwrap()
}

#[test]
fn test_serialize_owned_as_trait_bound() {
    // Pass a reference - works because &T: SerializeOwned where T: Serialize
    let value: u64 = 12345;
    let result = generic_serialize(&value);
    assert_eq!(result, "12345");
}

/// Test that various primitive reference types work with serialize_owned.
#[test]
fn test_primitive_references() {
    assert_eq!((&true).serialize_owned(StringSerializer).unwrap(), "true");
    assert_eq!((&false).serialize_owned(StringSerializer).unwrap(), "false");
    assert_eq!((&42i8).serialize_owned(StringSerializer).unwrap(), "42");
    assert_eq!((&42i16).serialize_owned(StringSerializer).unwrap(), "42");
    assert_eq!((&42i32).serialize_owned(StringSerializer).unwrap(), "42");
    assert_eq!((&42i64).serialize_owned(StringSerializer).unwrap(), "42");
    assert_eq!((&42u8).serialize_owned(StringSerializer).unwrap(), "42");
    assert_eq!((&42u16).serialize_owned(StringSerializer).unwrap(), "42");
    assert_eq!((&42u32).serialize_owned(StringSerializer).unwrap(), "42");
    assert_eq!((&42u64).serialize_owned(StringSerializer).unwrap(), "42");
    assert_eq!((&3.14f32).serialize_owned(StringSerializer).unwrap(), "3.14");
    assert_eq!((&3.14f64).serialize_owned(StringSerializer).unwrap(), "3.14");
    assert_eq!((&'x').serialize_owned(StringSerializer).unwrap(), "x");
}

/// Test that str references work with serialize_owned.
#[test]
fn test_str_reference() {
    let s: &str = "test string";
    let result = s.serialize_owned(StringSerializer).unwrap();
    assert_eq!(result, "test string");
}

/// Test that unit type reference works.
#[test]
fn test_unit_reference() {
    let unit: () = ();
    let result = (&unit).serialize_owned(StringSerializer).unwrap();
    assert_eq!(result, "()");
}

/// Test Option references.
#[test]
fn test_option_reference() {
    let some: Option<i32> = Some(42);
    let none: Option<i32> = None;

    let result_some = (&some).serialize_owned(StringSerializer).unwrap();
    let result_none = (&none).serialize_owned(StringSerializer).unwrap();

    assert_eq!(result_some, "42");
    assert_eq!(result_none, "None");
}
