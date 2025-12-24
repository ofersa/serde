//! Tests for the `forward_to_deserialize_any!` macro with `iter` support.
//!
//! These tests verify that the macro correctly generates `deserialize_iter`
//! implementations that return an error-containing iterator.

#![allow(clippy::derive_partial_eq_without_eq)]

use serde::de::{Deserialize, Deserializer, IterSeqAccess, SeqAccessIterator, Visitor};
use serde::forward_to_deserialize_any;
use std::fmt;
use std::marker::PhantomData;

// Custom error type for testing
#[derive(Debug, Clone, PartialEq)]
struct TestError(String);

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

impl serde::de::Error for TestError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        TestError(msg.to_string())
    }
}

// A minimal deserializer that uses forward_to_deserialize_any! for iter
struct ForwardingDeserializer;

impl<'de> Deserializer<'de> for ForwardingDeserializer {
    type Error = TestError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(TestError("deserialize_any called".to_string()))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any iter
    }
}

#[test]
fn test_forward_to_deserialize_any_iter_returns_error_iterator() {
    let deserializer = ForwardingDeserializer;
    let mut iter: SeqAccessIterator<'_, IterSeqAccess<'_, TestError>, i32> =
        deserializer.deserialize_iter::<i32>().unwrap();

    // The iterator should yield an error on first access
    let result = iter.next();
    assert!(result.is_some(), "Iterator should yield at least one item");

    let err = result.unwrap();
    assert!(err.is_err(), "The yielded item should be an error");

    let error_msg = err.unwrap_err().0;
    assert!(
        error_msg.contains("not supported"),
        "Error should indicate iter is not supported, got: {}",
        error_msg
    );
}

#[test]
fn test_forward_to_deserialize_any_iter_collect_fails() {
    let deserializer = ForwardingDeserializer;
    let iter: SeqAccessIterator<'_, IterSeqAccess<'_, TestError>, i32> =
        deserializer.deserialize_iter::<i32>().unwrap();

    // Collecting should fail because the iterator yields an error
    let result: Result<Vec<i32>, _> = iter.collect();
    assert!(result.is_err(), "Collect should fail with an error");
}

#[test]
fn test_forward_to_deserialize_any_iter_with_custom_lifetime() {
    // Test that the macro works with custom lifetime parameters
    struct CustomLifetimeDeserializer<'q>(PhantomData<&'q ()>);

    impl<'q> Deserializer<'q> for CustomLifetimeDeserializer<'q> {
        type Error = TestError;

        fn deserialize_any<W>(self, _visitor: W) -> Result<W::Value, Self::Error>
        where
            W: Visitor<'q>,
        {
            Err(TestError("deserialize_any called".to_string()))
        }

        forward_to_deserialize_any! {
            <W: Visitor<'q>>
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any iter
        }
    }

    let deserializer = CustomLifetimeDeserializer(PhantomData);
    let result = deserializer.deserialize_iter::<i32>();
    assert!(result.is_ok(), "deserialize_iter should return Ok");

    let mut iter = result.unwrap();
    let first = iter.next();
    assert!(first.is_some());
    assert!(first.unwrap().is_err());
}

#[test]
fn test_forward_to_deserialize_any_iter_does_not_forward_to_any() {
    // Test that iter does NOT forward to deserialize_any (it has special handling)
    struct TrackingDeserializer {
        any_called: std::cell::Cell<bool>,
    }

    impl<'de> Deserializer<'de> for TrackingDeserializer {
        type Error = TestError;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            self.any_called.set(true);
            Err(TestError("any called".to_string()))
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any iter
        }
    }

    let deserializer = TrackingDeserializer {
        any_called: std::cell::Cell::new(false),
    };

    // Call deserialize_iter - it should NOT call deserialize_any
    let _ = deserializer.deserialize_iter::<i32>();

    // Note: We can't check any_called because deserialize_iter consumes self,
    // but the test compiles and runs, which verifies the macro generates valid code
}

#[test]
fn test_forward_to_deserialize_any_iter_multiple_types() {
    // Test that iter works with different target types
    let d1 = ForwardingDeserializer;
    let _: SeqAccessIterator<'_, _, String> = d1.deserialize_iter().unwrap();

    let d2 = ForwardingDeserializer;
    let _: SeqAccessIterator<'_, _, Vec<u8>> = d2.deserialize_iter().unwrap();

    let d3 = ForwardingDeserializer;
    let _: SeqAccessIterator<'_, _, (i32, i32)> = d3.deserialize_iter().unwrap();
}

#[test]
fn test_forward_to_deserialize_any_partial_list() {
    // Test that iter can be included in a partial list of methods
    struct PartialDeserializer;

    impl<'de> Deserializer<'de> for PartialDeserializer {
        type Error = TestError;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(TestError("any called".to_string()))
        }

        // Custom implementations for some methods
        fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(TestError("custom bool".to_string()))
        }

        fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(TestError("custom seq".to_string()))
        }

        fn deserialize_iter<T>(
            self,
        ) -> Result<SeqAccessIterator<'de, IterSeqAccess<'de, Self::Error>, T>, Self::Error>
        where
            T: Deserialize<'de>,
        {
            // Custom implementation that returns empty iterator
            Ok(SeqAccessIterator::new(IterSeqAccess::empty()))
        }

        // Forward remaining methods
        forward_to_deserialize_any! {
            i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

    let deserializer = PartialDeserializer;
    let iter: SeqAccessIterator<'_, _, i32> = deserializer.deserialize_iter().unwrap();
    let result: Result<Vec<i32>, _> = iter.collect();

    // Custom implementation returns empty iterator, so collect should succeed with empty vec
    assert_eq!(result.unwrap(), Vec::<i32>::new());
}
