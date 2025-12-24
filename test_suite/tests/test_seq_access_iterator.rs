//! Tests for the SeqAccessIterator adapter.
//!
//! SeqAccessIterator wraps a SeqAccess and implements Iterator<Item = Result<T, E>>.

#![allow(clippy::needless_pass_by_value)]

use serde::de::value::{Error, SeqDeserializer};
use serde::de::{DeserializeSeed, IntoDeserializer, SeqAccess, SeqAccessIterator};

/// A mock SeqAccess that returns None for size_hint to test unknown size behavior.
struct UnknownSizeSeqAccess<I> {
    iter: I,
}

impl<I> UnknownSizeSeqAccess<I> {
    fn new(iter: I) -> Self {
        UnknownSizeSeqAccess { iter }
    }
}

impl<'de, I, T> SeqAccess<'de> for UnknownSizeSeqAccess<I>
where
    I: Iterator<Item = T>,
    T: IntoDeserializer<'de, Error>,
{
    type Error = Error;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        // Explicitly return None to test unknown size behavior
        None
    }
}

/// A mock SeqAccess that returns an error after a specified number of elements.
struct ErrorAfterSeqAccess<I> {
    iter: I,
    count: usize,
    error_after: usize,
}

impl<I> ErrorAfterSeqAccess<I> {
    fn new(iter: I, error_after: usize) -> Self {
        ErrorAfterSeqAccess {
            iter,
            count: 0,
            error_after,
        }
    }
}

impl<'de, I, T> SeqAccess<'de> for ErrorAfterSeqAccess<I>
where
    I: Iterator<Item = T>,
    T: IntoDeserializer<'de, Error>,
{
    type Error = Error;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: DeserializeSeed<'de>,
    {
        if self.count >= self.error_after {
            return Err(serde::de::Error::custom("intentional test error"));
        }
        match self.iter.next() {
            Some(value) => {
                self.count += 1;
                seed.deserialize(value.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        None
    }
}

/// Test basic iteration over sequence elements.
#[test]
fn test_basic_iteration() {
    let values = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    let collected: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec![1, 2, 3, 4, 5]);
}

/// Test iteration over an empty sequence.
#[test]
fn test_empty_sequence() {
    let values: Vec<i32> = vec![];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    let collected: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(collected.unwrap(), Vec::<i32>::new());
}

/// Test iteration over a single element.
#[test]
fn test_single_element() {
    let values = vec![42i32];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    let collected: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec![42]);
}

/// Test size_hint delegation when size is known.
#[test]
fn test_size_hint_known() {
    let values = vec![1i32, 2, 3];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    // SeqDeserializer derives size from ExactSizeIterator bounds
    let (lower, upper) = iter.size_hint();
    assert_eq!(lower, 3);
    assert_eq!(upper, Some(3));
}

/// Test size_hint updates as items are consumed.
#[test]
fn test_size_hint_after_consumption() {
    let values = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let mut iter = SeqAccessIterator::<_, i32>::new(seq);

    // Initial size
    assert_eq!(iter.size_hint(), (5, Some(5)));

    // Consume one element
    let _ = iter.next();
    assert_eq!(iter.size_hint(), (4, Some(4)));

    // Consume another
    let _ = iter.next();
    assert_eq!(iter.size_hint(), (3, Some(3)));
}

/// Test using iterator combinators (map, filter).
#[test]
fn test_iterator_combinators() {
    let values = vec![1i32, 2, 3, 4, 5, 6];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    // Filter even numbers and double them
    let result: Result<Vec<i32>, _> = iter
        .filter_map(|r| r.ok())
        .filter(|&x| x % 2 == 0)
        .map(|x| x * 2)
        .map(Ok)
        .collect();

    assert_eq!(result.unwrap(), vec![4, 8, 12]);
}

/// Test collect with early termination on error handling.
#[test]
fn test_collect_with_try() {
    let values = vec![1i32, 2, 3];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    // collect::<Result<Vec<_>, _>>() short-circuits on first error
    let result: Result<Vec<i32>, Error> = iter.collect();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

/// Test with string types.
#[test]
fn test_string_types() {
    let values = vec!["hello".to_string(), "world".to_string()];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, String>::new(seq);

    let collected: Result<Vec<String>, _> = iter.collect();
    assert_eq!(
        collected.unwrap(),
        vec!["hello".to_string(), "world".to_string()]
    );
}

/// Test iterator count() method.
#[test]
fn test_count() {
    let values = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    // count() consumes the iterator, counting successful elements
    // Since we're wrapping in Result, we need to count carefully
    let count = iter.filter(|r| r.is_ok()).count();
    assert_eq!(count, 5);
}

/// Test using take() combinator.
#[test]
fn test_take_combinator() {
    let values = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    let first_three: Result<Vec<i32>, _> = iter.take(3).collect();
    assert_eq!(first_three.unwrap(), vec![1, 2, 3]);
}

/// Test using skip() combinator.
#[test]
fn test_skip_combinator() {
    let values = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    let last_two: Result<Vec<i32>, _> = iter.skip(3).collect();
    assert_eq!(last_two.unwrap(), vec![4, 5]);
}

/// Test that the iterator properly terminates.
#[test]
fn test_iterator_terminates() {
    let values = vec![1i32, 2];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let mut iter = SeqAccessIterator::<_, i32>::new(seq);

    assert!(iter.next().unwrap().is_ok());
    assert!(iter.next().unwrap().is_ok());
    assert!(iter.next().is_none());
    // Should continue to return None after exhaustion
    assert!(iter.next().is_none());
}

/// Test with nested types (Vec of Vecs).
#[test]
fn test_nested_types() {
    let values = vec![vec![1i32, 2], vec![3, 4, 5]];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, Vec<i32>>::new(seq);

    let collected: Result<Vec<Vec<i32>>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec![vec![1, 2], vec![3, 4, 5]]);
}

/// Test sum via fold pattern.
#[test]
fn test_fold_sum() {
    let values = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    let sum: i32 = iter.filter_map(|r| r.ok()).sum();
    assert_eq!(sum, 15);
}

/// Test size_hint when SeqAccess returns None (unknown size).
#[test]
fn test_size_hint_unknown() {
    let values = vec![1i32, 2, 3];
    let seq = UnknownSizeSeqAccess::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    // When SeqAccess::size_hint() returns None, Iterator::size_hint() should be (0, None)
    let (lower, upper) = iter.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, None);
}

/// Test iteration with unknown size still works correctly.
#[test]
fn test_iteration_with_unknown_size() {
    let values = vec![10i32, 20, 30];
    let seq = UnknownSizeSeqAccess::new(values.into_iter());
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    let collected: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec![10, 20, 30]);
}

/// Test that errors are properly propagated through the iterator.
#[test]
fn test_error_propagation() {
    let values = vec![1i32, 2, 3, 4, 5];
    // Error will occur after 2 successful elements
    let seq = ErrorAfterSeqAccess::new(values.into_iter(), 2);
    let iter = SeqAccessIterator::<_, i32>::new(seq);

    // Collect should fail when it hits the error
    let result: Result<Vec<i32>, Error> = iter.collect();
    assert!(result.is_err());
}

/// Test that next() returns the error wrapped in Some(Err(...)).
#[test]
fn test_error_as_some_err() {
    let values = vec![1i32, 2, 3];
    // Error will occur after 1 successful element
    let seq = ErrorAfterSeqAccess::new(values.into_iter(), 1);
    let mut iter = SeqAccessIterator::<_, i32>::new(seq);

    // First element should succeed
    let first = iter.next();
    assert!(first.is_some());
    assert!(first.unwrap().is_ok());

    // Second call should return Some(Err(...))
    let second = iter.next();
    assert!(second.is_some());
    assert!(second.unwrap().is_err());
}
