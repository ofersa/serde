//! Tests for the SeqAccessIterator adapter.

#![allow(clippy::needless_pass_by_value)]

use serde::de::value::{Error, SeqDeserializer};
use serde::de::{DeserializeSeed, IntoDeserializer, SeqAccess, SeqAccessIterator};

/// Creates a SeqAccess from a vector of values for testing.
fn make_seq_access<T>(values: Vec<T>) -> SeqDeserializer<std::vec::IntoIter<T>, Error> {
    SeqDeserializer::new(values.into_iter())
}

#[test]
fn test_iterator_empty_sequence() {
    let seq = make_seq_access::<i32>(vec![]);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);
    let collected: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec![]);
}

#[test]
fn test_iterator_single_element() {
    let seq = make_seq_access(vec![42i32]);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);
    let collected: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec![42]);
}

#[test]
fn test_iterator_multiple_elements() {
    let seq = make_seq_access(vec![1i32, 2, 3, 4, 5]);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);
    let collected: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_iterator_strings() {
    let seq = make_seq_access(vec!["hello".to_string(), "world".to_string()]);
    let iter: SeqAccessIterator<_, String> = SeqAccessIterator::new(seq);
    let collected: Result<Vec<String>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec!["hello", "world"]);
}

#[test]
fn test_size_hint_known() {
    let seq = make_seq_access(vec![1i32, 2, 3]);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);
    // SeqDeserializer provides exact size hint via ExactSizeIterator
    let hint = iter.size_hint();
    assert_eq!(hint, (3, Some(3)));
}

#[test]
fn test_size_hint_empty() {
    let seq = make_seq_access::<i32>(vec![]);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);
    let hint = iter.size_hint();
    assert_eq!(hint, (0, Some(0)));
}

#[test]
fn test_size_hint_decreases() {
    let seq = make_seq_access(vec![1i32, 2, 3]);
    let mut iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    assert_eq!(iter.size_hint(), (3, Some(3)));
    let _ = iter.next();
    assert_eq!(iter.size_hint(), (2, Some(2)));
    let _ = iter.next();
    assert_eq!(iter.size_hint(), (1, Some(1)));
    let _ = iter.next();
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn test_into_inner() {
    let seq = make_seq_access(vec![1i32, 2, 3]);
    let mut iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    // Consume one element
    let first = iter.next();
    assert!(matches!(first, Some(Ok(1))));

    // Get the underlying SeqAccess back
    let mut inner = iter.into_inner();

    // The inner SeqAccess should still have remaining elements
    let next: Result<Option<i32>, _> = inner.next_element();
    assert_eq!(next.unwrap(), Some(2));
}

#[test]
fn test_partial_iteration() {
    let seq = make_seq_access(vec![1i32, 2, 3, 4, 5]);
    let mut iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    // Only take first 2 elements
    let first = iter.next().unwrap().unwrap();
    let second = iter.next().unwrap().unwrap();

    assert_eq!(first, 1);
    assert_eq!(second, 2);

    // Iterator should still have more elements
    let remaining: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(remaining.unwrap(), vec![3, 4, 5]);
}

#[test]
fn test_iterator_returns_none_after_exhaustion() {
    let seq = make_seq_access(vec![1i32]);
    let mut iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
    assert!(iter.next().is_none()); // Should continue returning None
}

#[test]
fn test_with_iterator_combinators() {
    let seq = make_seq_access(vec![1i32, 2, 3, 4, 5]);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    // Use filter and map
    let result: Result<Vec<i32>, _> = iter
        .filter_map(|r| match r {
            Ok(x) if x % 2 == 0 => Some(Ok(x * 2)),
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        })
        .collect();

    assert_eq!(result.unwrap(), vec![4, 8]); // 2*2=4, 4*2=8
}

#[test]
fn test_with_take() {
    let seq = make_seq_access(vec![1i32, 2, 3, 4, 5]);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    let result: Result<Vec<i32>, _> = iter.take(3).collect();
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_with_skip() {
    let seq = make_seq_access(vec![1i32, 2, 3, 4, 5]);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    let result: Result<Vec<i32>, _> = iter.skip(2).collect();
    assert_eq!(result.unwrap(), vec![3, 4, 5]);
}

/// A mock SeqAccess that yields some values then an error.
struct ErrorAfterSeqAccess<T> {
    values: Vec<T>,
    index: usize,
    error_at: usize,
}

impl<T> ErrorAfterSeqAccess<T> {
    fn new(values: Vec<T>, error_at: usize) -> Self {
        ErrorAfterSeqAccess {
            values,
            index: 0,
            error_at,
        }
    }
}

impl<'de, T> SeqAccess<'de> for ErrorAfterSeqAccess<T>
where
    T: Clone + Into<i32>,
{
    type Error = Error;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: DeserializeSeed<'de>,
    {
        if self.index >= self.values.len() {
            return Ok(None);
        }
        if self.index == self.error_at {
            return Err(serde::de::Error::custom("test error"));
        }
        let value = self.values[self.index].clone();
        self.index += 1;
        // We need to deserialize the value; use IntoDeserializer
        seed.deserialize(value.into().into_deserializer()).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        None // Unknown size to test that code path
    }
}

#[test]
fn test_error_propagation() {
    // Create a SeqAccess that errors on the 3rd element
    let seq = ErrorAfterSeqAccess::<i32>::new(vec![1, 2, 3, 4], 2);
    let mut iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    // First two elements should work
    assert_eq!(iter.next().unwrap().unwrap(), 1);
    assert_eq!(iter.next().unwrap().unwrap(), 2);

    // Third element should be an error
    let err = iter.next().unwrap();
    assert!(err.is_err());
}

#[test]
fn test_error_short_circuits_collect() {
    // Create a SeqAccess that errors on the 2nd element
    let seq = ErrorAfterSeqAccess::<i32>::new(vec![1, 2, 3], 1);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    // Collecting should stop at the error
    let result: Result<Vec<i32>, _> = iter.collect();
    assert!(result.is_err());
}

#[test]
fn test_size_hint_unknown() {
    // ErrorAfterSeqAccess returns None for size_hint
    let seq = ErrorAfterSeqAccess::<i32>::new(vec![1, 2, 3], 100);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    // Should return (0, None) for unknown size
    assert_eq!(iter.size_hint(), (0, None));
}
