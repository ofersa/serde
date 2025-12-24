//! Tests for SeqAccessIterator adapter.

#![allow(clippy::needless_pass_by_value)]

use serde::de::value::{Error, SeqDeserializer};
use serde::de::{Deserializer, SeqAccess, SeqAccessIterator, Visitor};
use std::fmt;

#[test]
fn test_seq_access_iterator_basic() {
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    let result: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_seq_access_iterator_empty() {
    let data: Vec<i32> = vec![];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    let result: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(result.unwrap(), Vec::<i32>::new());
}

#[test]
fn test_seq_access_iterator_single_element() {
    let data = vec![42i32];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    let result: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(result.unwrap(), vec![42]);
}

#[test]
fn test_seq_access_iterator_size_hint_with_known_size() {
    let data = vec![1i32, 2, 3];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    let (lower, upper) = iter.size_hint();
    // SeqDeserializer provides size_hint from the underlying iterator
    assert_eq!(lower, 3);
    assert_eq!(upper, Some(3));
}

#[test]
fn test_seq_access_iterator_size_hint_empty() {
    let data: Vec<i32> = vec![];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    let (lower, upper) = iter.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(0));
}

#[test]
fn test_seq_access_iterator_strings() {
    let data = vec!["hello".to_string(), "world".to_string()];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, String> = SeqAccessIterator::new(&mut seq);

    let result: Result<Vec<String>, _> = iter.collect();
    assert_eq!(result.unwrap(), vec!["hello", "world"]);
}

#[test]
fn test_seq_access_iterator_next_manually() {
    let data = vec![10i32, 20, 30];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let mut iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    assert_eq!(iter.next().unwrap().unwrap(), 10);
    assert_eq!(iter.next().unwrap().unwrap(), 20);
    assert_eq!(iter.next().unwrap().unwrap(), 30);
    assert!(iter.next().is_none());
}

#[test]
fn test_seq_access_iterator_partial_consumption() {
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let mut iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    // Consume only first two elements
    assert_eq!(iter.next().unwrap().unwrap(), 1);
    assert_eq!(iter.next().unwrap().unwrap(), 2);

    // Collect the rest
    let rest: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(rest.unwrap(), vec![3, 4, 5]);
}

#[test]
fn test_seq_access_iterator_with_option() {
    let data = vec![Some(1i32), None, Some(3)];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, Option<i32>> = SeqAccessIterator::new(&mut seq);

    let result: Result<Vec<Option<i32>>, _> = iter.collect();
    assert_eq!(result.unwrap(), vec![Some(1), None, Some(3)]);
}

#[test]
fn test_seq_access_iterator_chained_with_map() {
    let data = vec![1i32, 2, 3];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    // Use with other iterator adapters
    let result: Result<Vec<i32>, _> = iter
        .map(|r| r.map(|x| x * 2))
        .collect();
    assert_eq!(result.unwrap(), vec![2, 4, 6]);
}

#[test]
fn test_seq_access_iterator_with_take() {
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    let result: Result<Vec<i32>, _> = iter.take(3).collect();
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_seq_access_iterator_with_filter() {
    let data = vec![1i32, 2, 3, 4, 5, 6];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    // Filter to only even numbers
    let result: Result<Vec<i32>, _> = iter
        .filter_map(|r| match r {
            Ok(x) if x % 2 == 0 => Some(Ok(x)),
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        })
        .collect();
    assert_eq!(result.unwrap(), vec![2, 4, 6]);
}

#[test]
fn test_seq_access_iterator_with_fold() {
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(&mut seq);

    // Sum all elements using fold
    let result: Result<i32, _> = iter.try_fold(0, |acc, r| r.map(|x| acc + x));
    assert_eq!(result.unwrap(), 15);
}

/// Test using SeqAccessIterator within a Visitor, which is the primary use case.
#[test]
fn test_seq_access_iterator_in_visitor() {
    struct SumVisitor;

    impl<'de> Visitor<'de> for SumVisitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of integers")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
            iter.try_fold(0, |acc, r| r.map(|x| acc + x))
        }
    }

    let data = vec![1i32, 2, 3, 4, 5];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let result = seq.deserialize_seq(SumVisitor);
    assert_eq!(result.unwrap(), 15);
}

/// Test using SeqAccessIterator to collect into a custom type.
#[test]
fn test_seq_access_iterator_custom_collection() {
    struct CollectVisitor;

    impl<'de> Visitor<'de> for CollectVisitor {
        type Value = Vec<i32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of integers")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
            iter.collect()
        }
    }

    let data = vec![10i32, 20, 30];
    let seq: SeqDeserializer<_, Error> = SeqDeserializer::new(data.into_iter());
    let result = seq.deserialize_seq(CollectVisitor);
    assert_eq!(result.unwrap(), vec![10, 20, 30]);
}
