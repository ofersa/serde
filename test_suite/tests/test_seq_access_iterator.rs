//! Tests for the SeqAccessIterator adapter struct.

#![allow(clippy::needless_pass_by_value)]

use serde::de::value::{Error, SeqDeserializer};
use serde::de::SeqAccessIterator;

/// Test that SeqAccessIterator correctly iterates over elements.
#[test]
fn test_iterator_basic() {
    let values = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let collected: Result<Vec<i32>, _> = iter.collect();

    assert_eq!(collected.unwrap(), vec![1, 2, 3, 4, 5]);
}

/// Test that SeqAccessIterator handles empty sequences.
#[test]
fn test_iterator_empty() {
    let values: Vec<i32> = vec![];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let collected: Result<Vec<i32>, _> = iter.collect();

    assert_eq!(collected.unwrap(), Vec::<i32>::new());
}

/// Test that SeqAccessIterator handles single element sequences.
#[test]
fn test_iterator_single_element() {
    let values = vec![42i32];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let collected: Result<Vec<i32>, _> = iter.collect();

    assert_eq!(collected.unwrap(), vec![42]);
}

/// Test that size_hint is properly delegated when the underlying SeqAccess provides a size.
#[test]
fn test_size_hint_with_known_size() {
    let values = vec![1i32, 2, 3];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);

    // Vec iterator provides ExactSizeIterator, so size_hint should be (3, Some(3))
    let (lower, upper) = iter.size_hint();
    assert_eq!(lower, 3);
    assert_eq!(upper, Some(3));
}

/// Test that size_hint returns (0, None) when the underlying SeqAccess has no size.
#[test]
fn test_size_hint_empty() {
    let values: Vec<i32> = vec![];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);

    let (lower, upper) = iter.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(0));
}

/// Test that SeqAccessIterator works with string elements.
#[test]
fn test_iterator_with_strings() {
    let values = vec!["hello".to_string(), "world".to_string()];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter: SeqAccessIterator<'_, _, String> = SeqAccessIterator::new(&mut seq);
    let collected: Result<Vec<String>, _> = iter.collect();

    assert_eq!(
        collected.unwrap(),
        vec!["hello".to_string(), "world".to_string()]
    );
}

/// Test that SeqAccessIterator properly handles partial iteration.
#[test]
fn test_iterator_partial_consumption() {
    let values = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let mut iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);

    // Take only first two elements
    let first = iter.next();
    let second = iter.next();

    assert_eq!(first, Some(Ok(1)));
    assert_eq!(second, Some(Ok(2)));

    // The iterator should still have more elements
    let rest: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(rest.unwrap(), vec![3, 4, 5]);
}

/// Test using SeqAccessIterator with take() adapter.
#[test]
fn test_iterator_with_take() {
    let values = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let collected: Result<Vec<i32>, _> = iter.take(3).collect();

    assert_eq!(collected.unwrap(), vec![1, 2, 3]);
}

/// Test using SeqAccessIterator with filter_map().
#[test]
fn test_iterator_with_filter_map() {
    let values = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);

    // Filter to only even numbers and double them
    let collected: Vec<i32> = iter
        .filter_map(|r| r.ok())
        .filter(|n| n % 2 == 0)
        .map(|n| n * 2)
        .collect();

    assert_eq!(collected, vec![4, 8]);
}

/// Test that SeqAccessIterator works with bool elements.
#[test]
fn test_iterator_with_bools() {
    let values = vec![true, false, true];
    let mut seq: SeqDeserializer<_, Error> = SeqDeserializer::new(values.into_iter());

    let iter: SeqAccessIterator<'_, _, bool> = SeqAccessIterator::new(&mut seq);
    let collected: Result<Vec<bool>, _> = iter.collect();

    assert_eq!(collected.unwrap(), vec![true, false, true]);
}
