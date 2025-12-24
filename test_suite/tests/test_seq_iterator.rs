#![allow(clippy::uninlined_format_args)]

use serde::de::{SeqAccess, SeqIterator, Visitor};
use std::fmt;

/// A simple error type for testing.
#[derive(Debug, Clone, PartialEq)]
struct MockError(String);

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MockError {}

impl serde::de::Error for MockError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        MockError(msg.to_string())
    }
}

/// A SeqAccess that returns i32 values directly by implementing next_element.
struct I32SeqAccess {
    elements: Vec<i32>,
    index: usize,
}

impl I32SeqAccess {
    fn new(elements: Vec<i32>) -> Self {
        I32SeqAccess { elements, index: 0 }
    }
}

/// Simple i32 deserializer for testing.
struct I32Deserializer(i32);

impl<'de> serde::Deserializer<'de> for I32Deserializer {
    type Error = MockError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.0)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for I32SeqAccess {
    type Error = MockError;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: serde::de::DeserializeSeed<'de>,
    {
        if self.index >= self.elements.len() {
            return Ok(None);
        }
        let value = self.elements[self.index];
        self.index += 1;
        seed.deserialize(I32Deserializer(value)).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.elements.len() - self.index)
    }
}

/// A SeqAccess that can produce errors on specific indices.
struct ErrorSeqAccess {
    elements: Vec<i32>,
    error_at: usize,
    index: usize,
}

impl ErrorSeqAccess {
    fn new(elements: Vec<i32>, error_at: usize) -> Self {
        ErrorSeqAccess {
            elements,
            error_at,
            index: 0,
        }
    }
}

impl<'de> SeqAccess<'de> for ErrorSeqAccess {
    type Error = MockError;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: serde::de::DeserializeSeed<'de>,
    {
        if self.index >= self.elements.len() {
            return Ok(None);
        }
        if self.index == self.error_at {
            self.index += 1;
            return Err(MockError("intentional error".to_string()));
        }
        let value = self.elements[self.index];
        self.index += 1;
        seed.deserialize(I32Deserializer(value)).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.elements.len() - self.index)
    }
}

/// A SeqAccess with no size hint.
struct NoSizeHintSeqAccess {
    elements: Vec<i32>,
    index: usize,
}

impl NoSizeHintSeqAccess {
    fn new(elements: Vec<i32>) -> Self {
        NoSizeHintSeqAccess { elements, index: 0 }
    }
}

impl<'de> SeqAccess<'de> for NoSizeHintSeqAccess {
    type Error = MockError;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: serde::de::DeserializeSeed<'de>,
    {
        if self.index >= self.elements.len() {
            return Ok(None);
        }
        let value = self.elements[self.index];
        self.index += 1;
        seed.deserialize(I32Deserializer(value)).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        None
    }
}

#[test]
fn test_seq_iterator_empty() {
    let seq = I32SeqAccess::new(vec![]);
    let mut iter = SeqIterator::<i32, _>::new(seq);

    assert!(iter.next().is_none());
    // Should continue to return None
    assert!(iter.next().is_none());
}

#[test]
fn test_seq_iterator_single_element() {
    let seq = I32SeqAccess::new(vec![42]);
    let mut iter = SeqIterator::<i32, _>::new(seq);

    let first = iter.next();
    assert!(first.is_some());
    assert_eq!(first.unwrap().unwrap(), 42);

    assert!(iter.next().is_none());
}

#[test]
fn test_seq_iterator_multiple_elements() {
    let seq = I32SeqAccess::new(vec![1, 2, 3, 4, 5]);
    let iter = SeqIterator::<i32, _>::new(seq);

    let values: Vec<i32> = iter.collect::<Result<_, _>>().unwrap();
    assert_eq!(values, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_seq_iterator_collect_result() {
    let seq = I32SeqAccess::new(vec![10, 20, 30]);
    let iter = SeqIterator::<i32, _>::new(seq);

    let result: Result<Vec<i32>, _> = iter.collect();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![10, 20, 30]);
}

#[test]
fn test_seq_iterator_error_propagation() {
    let seq = ErrorSeqAccess::new(vec![1, 2, 3], 1); // Error at index 1
    let iter = SeqIterator::<i32, _>::new(seq);

    let result: Result<Vec<i32>, _> = iter.collect();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), MockError("intentional error".to_string()));
}

#[test]
fn test_seq_iterator_partial_iteration_before_error() {
    let seq = ErrorSeqAccess::new(vec![1, 2, 3], 2); // Error at index 2
    let mut iter = SeqIterator::<i32, _>::new(seq);

    // First element should succeed
    let first = iter.next();
    assert!(first.is_some());
    assert_eq!(first.unwrap().unwrap(), 1);

    // Second element should succeed
    let second = iter.next();
    assert!(second.is_some());
    assert_eq!(second.unwrap().unwrap(), 2);

    // Third element should be an error
    let third = iter.next();
    assert!(third.is_some());
    assert!(third.unwrap().is_err());
}

#[test]
fn test_seq_iterator_size_hint_with_known_size() {
    let seq = I32SeqAccess::new(vec![1, 2, 3, 4, 5]);
    let iter = SeqIterator::<i32, _>::new(seq);

    // Initial size hint should be (5, Some(5))
    assert_eq!(iter.size_hint(), (5, Some(5)));
}

#[test]
fn test_seq_iterator_size_hint_decreases() {
    let seq = I32SeqAccess::new(vec![1, 2, 3]);
    let mut iter = SeqIterator::<i32, _>::new(seq);

    assert_eq!(iter.size_hint(), (3, Some(3)));

    iter.next();
    assert_eq!(iter.size_hint(), (2, Some(2)));

    iter.next();
    assert_eq!(iter.size_hint(), (1, Some(1)));

    iter.next();
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn test_seq_iterator_size_hint_unknown() {
    let seq = NoSizeHintSeqAccess::new(vec![1, 2, 3]);
    let iter = SeqIterator::<i32, _>::new(seq);

    // When size_hint is None, Iterator should return (0, None)
    assert_eq!(iter.size_hint(), (0, None));
}

#[test]
fn test_seq_iterator_size_hint_seq_method() {
    let seq = I32SeqAccess::new(vec![1, 2, 3, 4]);
    let iter = SeqIterator::<i32, _>::new(seq);

    // Test the size_hint_seq method
    assert_eq!(iter.size_hint_seq(), Some(4));
}

#[test]
fn test_seq_iterator_size_hint_seq_none() {
    let seq = NoSizeHintSeqAccess::new(vec![1, 2, 3]);
    let iter = SeqIterator::<i32, _>::new(seq);

    assert_eq!(iter.size_hint_seq(), None);
}

#[test]
fn test_seq_iterator_with_map() {
    let seq = I32SeqAccess::new(vec![1, 2, 3]);
    let iter = SeqIterator::<i32, _>::new(seq);

    // Use map to transform results
    let doubled: Result<Vec<i32>, _> = iter.map(|r| r.map(|x| x * 2)).collect();

    assert!(doubled.is_ok());
    assert_eq!(doubled.unwrap(), vec![2, 4, 6]);
}

#[test]
fn test_seq_iterator_with_filter() {
    let seq = I32SeqAccess::new(vec![1, 2, 3, 4, 5, 6]);
    let iter = SeqIterator::<i32, _>::new(seq);

    // Filter to keep only even numbers (need to handle Result)
    let evens: Result<Vec<i32>, _> = iter
        .filter(|r| match r {
            Ok(x) => x % 2 == 0,
            Err(_) => true, // Keep errors
        })
        .collect();

    assert!(evens.is_ok());
    assert_eq!(evens.unwrap(), vec![2, 4, 6]);
}

#[test]
fn test_seq_iterator_with_take() {
    let seq = I32SeqAccess::new(vec![1, 2, 3, 4, 5]);
    let iter = SeqIterator::<i32, _>::new(seq);

    let first_three: Result<Vec<i32>, _> = iter.take(3).collect();

    assert!(first_three.is_ok());
    assert_eq!(first_three.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_seq_iterator_with_skip() {
    let seq = I32SeqAccess::new(vec![1, 2, 3, 4, 5]);
    let iter = SeqIterator::<i32, _>::new(seq);

    let last_two: Result<Vec<i32>, _> = iter.skip(3).collect();

    assert!(last_two.is_ok());
    assert_eq!(last_two.unwrap(), vec![4, 5]);
}

#[test]
fn test_seq_iterator_fused_behavior() {
    let seq = I32SeqAccess::new(vec![1]);
    let mut iter = SeqIterator::<i32, _>::new(seq);

    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
    // Iterator should stay None after exhaustion
    assert!(iter.next().is_none());
    assert!(iter.next().is_none());
}

#[test]
fn test_seq_iterator_error_at_first_element() {
    let seq = ErrorSeqAccess::new(vec![1, 2, 3], 0); // Error at index 0
    let mut iter = SeqIterator::<i32, _>::new(seq);

    // First element should be an error
    let first = iter.next();
    assert!(first.is_some());
    assert!(first.unwrap().is_err());
}

#[test]
fn test_seq_iterator_error_at_last_element() {
    let seq = ErrorSeqAccess::new(vec![1, 2, 3], 2); // Error at last index
    let iter = SeqIterator::<i32, _>::new(seq);

    let result: Result<Vec<i32>, _> = iter.collect();
    assert!(result.is_err());
}