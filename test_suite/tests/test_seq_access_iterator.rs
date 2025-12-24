#![allow(clippy::derive_partial_eq_without_eq)]

use serde::de::value::{self, SeqDeserializer};
use serde::de::{DeserializeSeed, SeqAccess, SeqAccessIterator};
use std::fmt;

#[test]
fn test_basic_iteration() {
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: Result<Vec<i32>, _> = iter.collect();

    assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_empty_sequence() {
    let data: Vec<i32> = vec![];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: Result<Vec<i32>, _> = iter.collect();

    assert_eq!(result.unwrap(), Vec::<i32>::new());
}

#[test]
fn test_single_element() {
    let data = vec![42i32];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: Result<Vec<i32>, _> = iter.collect();

    assert_eq!(result.unwrap(), vec![42]);
}

#[test]
fn test_size_hint_with_known_size() {
    let data = vec![1i32, 2, 3];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);

    // SeqDeserializer's size_hint returns Some(len), so our iterator should return (len, Some(len))
    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn test_size_hint_updates_during_iteration() {
    let data = vec![1i32, 2, 3, 4];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let mut iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);

    assert_eq!(iter.size_hint(), (4, Some(4)));

    let _ = iter.next();
    assert_eq!(iter.size_hint(), (3, Some(3)));

    let _ = iter.next();
    assert_eq!(iter.size_hint(), (2, Some(2)));

    let _ = iter.next();
    assert_eq!(iter.size_hint(), (1, Some(1)));

    let _ = iter.next();
    assert_eq!(iter.size_hint(), (0, Some(0)));

    // After exhaustion
    assert!(iter.next().is_none());
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn test_next_returns_none_after_exhaustion() {
    let data = vec![1i32, 2];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let mut iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);

    assert!(iter.next().is_some());
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
    assert!(iter.next().is_none()); // calling next again should still return None
}

#[test]
fn test_collect_with_type_conversion() {
    // Test that the iterator works with types that need deserialization
    let data = vec!["hello".to_string(), "world".to_string()];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, String> = SeqAccessIterator::new(&mut seq);
    let result: Result<Vec<String>, _> = iter.collect();

    assert_eq!(result.unwrap(), vec!["hello", "world"]);
}

#[test]
fn test_with_tuple_deserialization() {
    // Test that the iterator works with tuple types that need deserialization
    let data = vec![(1i32, 2i32), (3, 4)];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, (i32, i32)> = SeqAccessIterator::new(&mut seq);
    let result: Result<Vec<(i32, i32)>, _> = iter.collect();

    assert_eq!(result.unwrap(), vec![(1, 2), (3, 4)]);
}

#[test]
fn test_take_partial() {
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: Result<Vec<i32>, _> = iter.take(3).collect();

    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

// Custom error type for testing error propagation
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

// A SeqAccess that yields some elements then errors
struct FailingSeqAccess {
    count: usize,
    fail_at: usize,
}

impl<'de> SeqAccess<'de> for FailingSeqAccess {
    type Error = TestError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.count >= self.fail_at {
            return Err(TestError("intentional failure".to_string()));
        }
        self.count += 1;
        // Deserialize the count value
        seed.deserialize(value::U32Deserializer::<TestError>::new(self.count as u32))
            .map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        None
    }
}

#[test]
fn test_error_propagation() {
    let mut seq = FailingSeqAccess {
        count: 0,
        fail_at: 3,
    };

    let mut iter: SeqAccessIterator<'_, _, u32> = SeqAccessIterator::new(&mut seq);

    // First three elements should succeed
    assert_eq!(iter.next().unwrap().unwrap(), 1);
    assert_eq!(iter.next().unwrap().unwrap(), 2);
    assert_eq!(iter.next().unwrap().unwrap(), 3);

    // Fourth should fail
    let result = iter.next();
    assert!(result.is_some());
    assert!(result.unwrap().is_err());
}

#[test]
fn test_error_stops_collect() {
    let mut seq = FailingSeqAccess {
        count: 0,
        fail_at: 2,
    };

    let iter: SeqAccessIterator<'_, _, u32> = SeqAccessIterator::new(&mut seq);
    let result: Result<Vec<u32>, _> = iter.collect();

    // The collect should fail because the iterator returns an error
    assert!(result.is_err());
}

#[test]
fn test_size_hint_with_unknown_size() {
    let mut seq = FailingSeqAccess {
        count: 0,
        fail_at: 10,
    };

    let iter: SeqAccessIterator<'_, _, u32> = SeqAccessIterator::new(&mut seq);

    // FailingSeqAccess returns None for size_hint, so iterator should return (0, None)
    assert_eq!(iter.size_hint(), (0, None));
}
