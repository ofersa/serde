//! Tests for IntoSeqIterator trait and SeqAccessIterator.

use serde::de::value::{Error, SeqDeserializer};
use serde::de::{IntoSeqIterator, SeqAccess, Visitor};
use std::fmt;

/// Test that IntoSeqIterator::into_seq_iter produces correct values
#[test]
fn test_into_seq_iter_basic() {
    let values = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let result: Vec<i32> = seq.into_seq_iter().collect::<Result<_, _>>().unwrap();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

/// Test empty sequence iteration
#[test]
fn test_into_seq_iter_empty() {
    let values: Vec<i32> = vec![];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let result: Vec<i32> = seq.into_seq_iter().collect::<Result<_, _>>().unwrap();
    assert!(result.is_empty());
}

/// Test single element sequence
#[test]
fn test_into_seq_iter_single_element() {
    let values = vec![42i32];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let result: Vec<i32> = seq.into_seq_iter().collect::<Result<_, _>>().unwrap();
    assert_eq!(result, vec![42]);
}

/// Test that size_hint is propagated from SeqAccess
#[test]
fn test_into_seq_iter_size_hint() {
    let values = vec![1i32, 2, 3];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter = seq.into_seq_iter::<i32>();
    let (lower, upper) = iter.size_hint();
    // The SeqDeserializer knows the exact size from the underlying iterator
    assert_eq!(lower, 3);
    assert_eq!(upper, Some(3));
}

/// Test that the iterator works with different types
#[test]
fn test_into_seq_iter_strings() {
    let values = vec!["hello".to_string(), "world".to_string()];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let result: Vec<String> = seq.into_seq_iter().collect::<Result<_, _>>().unwrap();
    assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
}

/// Test using into_seq_iter in a custom visitor implementation
struct VecVisitor;

impl<'de> Visitor<'de> for VecVisitor {
    type Value = Vec<i32>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of integers")
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        seq.into_seq_iter().collect()
    }
}

/// Test that IntoSeqIterator works in a visitor context
#[test]
fn test_into_seq_iter_in_visitor() {
    use serde::de::Deserializer;

    let values = vec![10i32, 20, 30];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let result = seq.deserialize_seq(VecVisitor).unwrap();
    assert_eq!(result, vec![10, 20, 30]);
}

/// Test partial iteration (not consuming all elements)
#[test]
fn test_into_seq_iter_partial() {
    let values = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let mut iter = seq.into_seq_iter::<i32>();

    assert_eq!(iter.next(), Some(Ok(1)));
    assert_eq!(iter.next(), Some(Ok(2)));
    // Stop early - should still work fine
    let (lower, _) = iter.size_hint();
    assert_eq!(lower, 3); // 3 remaining
}

/// Test that boolean sequences work
#[test]
fn test_into_seq_iter_bools() {
    let values = vec![true, false, true];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let result: Vec<bool> = seq.into_seq_iter().collect::<Result<_, _>>().unwrap();
    assert_eq!(result, vec![true, false, true]);
}

/// Test with floating point numbers
#[test]
fn test_into_seq_iter_floats() {
    let values = vec![1.5f64, 2.5, 3.5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let result: Vec<f64> = seq.into_seq_iter().collect::<Result<_, _>>().unwrap();
    assert_eq!(result, vec![1.5, 2.5, 3.5]);
}

/// Test chaining iterator adapters
#[test]
fn test_into_seq_iter_filter_map() {
    let values = vec![1i32, 2, 3, 4, 5, 6];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    // Filter even numbers and double them
    let result: Result<Vec<i32>, _> = seq
        .into_seq_iter::<i32>()
        .filter_map(|r| match r {
            Ok(n) if n % 2 == 0 => Some(Ok(n * 2)),
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        })
        .collect();

    assert_eq!(result.unwrap(), vec![4, 8, 12]);
}

/// Test take adapter with the iterator
#[test]
fn test_into_seq_iter_take() {
    let values = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let result: Vec<i32> = seq
        .into_seq_iter()
        .take(3)
        .collect::<Result<_, _>>()
        .unwrap();

    assert_eq!(result, vec![1, 2, 3]);
}
