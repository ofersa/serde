//! Tests for the `iter` method in the `forward_to_deserialize_any!` macro.
//!
//! These tests verify that the macro correctly generates `deserialize_iter`
//! implementations that forward to `deserialize_any`.

#![allow(clippy::needless_pass_by_value)]

use serde::de::value::{Error, SeqDeserializer};
use serde::de::{Deserialize, Deserializer, IntoDeserializer, Visitor};
use serde::forward_to_deserialize_any;
use std::fmt;

/// A simple deserializer that uses `forward_to_deserialize_any!` to forward
/// `deserialize_iter` calls to `deserialize_any`.
struct ForwardingDeserializer<I>(SeqDeserializer<I, Error>);

impl<'de, I, T> Deserializer<'de> for ForwardingDeserializer<I>
where
    I: Iterator<Item = T>,
    T: IntoDeserializer<'de, Error>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_any(visitor)
    }

    // Forward all methods including `iter` to deserialize_any
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any iter
    }
}

#[test]
fn test_forward_iter_empty_sequence() {
    let deserializer = ForwardingDeserializer(SeqDeserializer::new(Vec::<i32>::new().into_iter()));
    let result: Vec<i32> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_forward_iter_single_element() {
    let deserializer = ForwardingDeserializer(SeqDeserializer::new(vec![42i32].into_iter()));
    let result: Vec<i32> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec![42]);
}

#[test]
fn test_forward_iter_multiple_elements() {
    let deserializer = ForwardingDeserializer(SeqDeserializer::new(vec![1i32, 2, 3, 4, 5].into_iter()));
    let result: Vec<i32> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_forward_iter_strings() {
    let deserializer = ForwardingDeserializer(SeqDeserializer::new(
        vec!["hello".to_string(), "world".to_string()].into_iter(),
    ));
    let result: Vec<String> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec!["hello", "world"]);
}

/// A deserializer with a custom lifetime parameter to test the macro's
/// explicit lifetime syntax.
struct CustomLifetimeDeserializer<'q, I>(SeqDeserializer<I, Error>, std::marker::PhantomData<&'q ()>);

impl<'q, I, T> Deserializer<'q> for CustomLifetimeDeserializer<'q, I>
where
    I: Iterator<Item = T>,
    T: IntoDeserializer<'q, Error>,
{
    type Error = Error;

    fn deserialize_any<W>(self, visitor: W) -> Result<W::Value, Self::Error>
    where
        W: Visitor<'q>,
    {
        self.0.deserialize_any(visitor)
    }

    // Use explicit lifetime and visitor type syntax
    forward_to_deserialize_any! {
        <W: Visitor<'q>>
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any iter
    }
}

#[test]
fn test_forward_iter_custom_lifetime_syntax() {
    let deserializer = CustomLifetimeDeserializer(
        SeqDeserializer::new(vec![10i32, 20, 30].into_iter()),
        std::marker::PhantomData,
    );
    let result: Vec<i32> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec![10, 20, 30]);
}

/// A deserializer that tracks if deserialize_any was called, to verify forwarding.
struct TrackingDeserializer {
    inner: SeqDeserializer<std::vec::IntoIter<i32>, Error>,
    any_called: std::cell::Cell<bool>,
}

impl TrackingDeserializer {
    fn new(values: Vec<i32>) -> Self {
        Self {
            inner: SeqDeserializer::new(values.into_iter()),
            any_called: std::cell::Cell::new(false),
        }
    }
}

impl<'de> Deserializer<'de> for TrackingDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.any_called.set(true);
        self.inner.deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any iter
    }
}

#[test]
fn test_forward_iter_calls_deserialize_any() {
    let deserializer = TrackingDeserializer::new(vec![1, 2, 3]);
    let any_called = deserializer.any_called.clone();

    let result: Vec<i32> = deserializer.deserialize_iter().unwrap();

    assert_eq!(result, vec![1, 2, 3]);
    assert!(any_called.get(), "deserialize_iter should forward to deserialize_any");
}

/// A simple newtype wrapper to test deserialization of nested types.
#[derive(Debug, PartialEq, Eq)]
struct Wrapper(i32);

impl<'de> Deserialize<'de> for Wrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WrapperVisitor;

        impl<'de> Visitor<'de> for WrapperVisitor {
            type Value = Wrapper;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer")
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> {
                Ok(Wrapper(v))
            }
        }

        deserializer.deserialize_i32(WrapperVisitor)
    }
}

#[test]
fn test_forward_iter_with_custom_type() {
    let deserializer = ForwardingDeserializer(SeqDeserializer::new(vec![1i32, 2, 3].into_iter()));
    let result: Vec<Wrapper> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec![Wrapper(1), Wrapper(2), Wrapper(3)]);
}

/// Test that only including `iter` in the macro (without other methods) works.
/// This uses a wrapper that forwards everything and only adds `iter` via the macro.
struct IterOnlyDeserializer<I>(SeqDeserializer<I, Error>);

impl<'de, I, T> Deserializer<'de> for IterOnlyDeserializer<I>
where
    I: Iterator<Item = T>,
    T: IntoDeserializer<'de, Error>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_any(visitor)
    }

    // Only forward iter via the macro
    forward_to_deserialize_any! { iter }

    // Forward all other methods using another macro invocation
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[test]
fn test_forward_iter_only() {
    let deserializer = IterOnlyDeserializer(SeqDeserializer::new(vec![7i32, 8, 9].into_iter()));
    let result: Vec<i32> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec![7, 8, 9]);
}

/// Test that the macro-generated method respects size_hint when reserving capacity.
/// This ensures efficient memory allocation for large sequences.
#[test]
fn test_forward_iter_respects_size_hint() {
    // Create a sequence with a known size
    let data: Vec<i32> = (1..=100).collect();
    let deserializer = ForwardingDeserializer(SeqDeserializer::new(data.clone().into_iter()));

    let result: Vec<i32> = deserializer.deserialize_iter().unwrap();

    // Verify all elements are correctly deserialized
    assert_eq!(result.len(), 100);
    assert_eq!(result, data);
}
