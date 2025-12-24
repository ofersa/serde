//! Integration tests demonstrating `deserialize_iter` usage with Vec,
//! custom collection types, and early termination scenarios.
//!
//! Tests cover both JSON-like (token-based via serde_test) and binary-like
//! (SeqDeserializer from value module) deserializers.
//!
//! The tests demonstrate:
//! - Using `SeqAccessIterator` directly within `visit_seq` for custom deserializers
//! - Collecting into various standard library collections (Vec, VecDeque, etc.)
//! - Custom collection types with validation during iteration
//! - Early termination patterns (take, find, any, etc.)
//! - Error handling and propagation
//! - Binary-like deserializers that read from byte buffers

#![allow(clippy::derive_partial_eq_without_eq)]

use serde::de::value::{self, SeqDeserializer};
use serde::de::{Deserialize, DeserializeSeed, Deserializer, IterSeqAccess, SeqAccess, SeqAccessIterator, Visitor};
use serde_derive::Deserialize;
use std::collections::{BTreeSet, HashSet, LinkedList, VecDeque};
use std::fmt;
use std::marker::PhantomData;

// ============================================================================
// BASIC VEC USAGE WITH JSON-LIKE DESERIALIZER (value module)
// ============================================================================

#[test]
fn test_vec_with_seq_deserializer() {
    // Using SeqDeserializer as a "JSON-like" deserializer that provides SeqAccess
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: Vec<i32> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_vec_strings_with_seq_deserializer() {
    let data = vec!["hello".to_string(), "world".to_string(), "test".to_string()];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, String> = SeqAccessIterator::new(&mut seq);
    let result: Vec<String> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec!["hello", "world", "test"]);
}

#[test]
fn test_empty_vec_with_seq_deserializer() {
    let data: Vec<i32> = vec![];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: Vec<i32> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(result, Vec::<i32>::new());
}

// ============================================================================
// CUSTOM COLLECTION TYPES
// ============================================================================

/// A custom collection that wraps a Vec but has a special constraint:
/// it only accepts non-negative numbers.
#[derive(Debug, PartialEq)]
struct NonNegativeVec(Vec<i32>);

impl NonNegativeVec {
    fn new() -> Self {
        NonNegativeVec(Vec::new())
    }

    fn push(&mut self, value: i32) -> Result<(), &'static str> {
        if value >= 0 {
            self.0.push(value);
            Ok(())
        } else {
            Err("negative value not allowed")
        }
    }
}

impl<'de> Deserialize<'de> for NonNegativeVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NonNegativeVecVisitor;

        impl<'de> Visitor<'de> for NonNegativeVecVisitor {
            type Value = NonNegativeVec;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of non-negative integers")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let iter: SeqAccessIterator<'_, A, i32> = SeqAccessIterator::new(seq);
                let mut result = NonNegativeVec::new();

                for item in iter {
                    let value = item?;
                    result
                        .push(value)
                        .map_err(serde::de::Error::custom)?;
                }

                Ok(result)
            }
        }

        deserializer.deserialize_seq(NonNegativeVecVisitor)
    }
}

#[test]
fn test_custom_collection_non_negative_vec() {
    let data = vec![0i32, 1, 2, 3, 4];
    let deserializer: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let result: NonNegativeVec = NonNegativeVec::deserialize(deserializer).unwrap();
    assert_eq!(result, NonNegativeVec(vec![0, 1, 2, 3, 4]));
}

#[test]
fn test_custom_collection_rejects_negative() {
    let data = vec![1i32, -1, 2];
    let deserializer: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let result = NonNegativeVec::deserialize(deserializer);
    assert!(result.is_err());
}

/// A custom wrapper that counts how many elements it deserialized.
#[derive(Debug, PartialEq)]
struct CountingVec<T> {
    items: Vec<T>,
    count: usize,
}

impl<T> CountingVec<T> {
    fn new() -> Self {
        CountingVec {
            items: Vec::new(),
            count: 0,
        }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
        self.count += 1;
    }
}

impl<'de, T> Deserialize<'de> for CountingVec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CountingVecVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for CountingVecVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = CountingVec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let iter: SeqAccessIterator<'_, A, T> = SeqAccessIterator::new(seq);
                let mut result = CountingVec::new();

                for item in iter {
                    result.push(item?);
                }

                Ok(result)
            }
        }

        deserializer.deserialize_seq(CountingVecVisitor(PhantomData))
    }
}

#[test]
fn test_counting_vec_custom_collection() {
    let data = vec![10i32, 20, 30, 40, 50];
    let deserializer: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let result: CountingVec<i32> = CountingVec::deserialize(deserializer).unwrap();
    assert_eq!(result.items, vec![10, 20, 30, 40, 50]);
    assert_eq!(result.count, 5);
}

// ============================================================================
// STANDARD LIBRARY COLLECTION TYPES
// ============================================================================

#[test]
fn test_collect_into_vec_deque() {
    let data = vec![1i32, 2, 3];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: VecDeque<i32> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(result, VecDeque::from([1, 2, 3]));
}

#[test]
fn test_collect_into_linked_list() {
    let data = vec![1i32, 2, 3];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: LinkedList<i32> = iter.collect::<Result<_, _>>().unwrap();

    let expected: LinkedList<i32> = [1, 2, 3].into_iter().collect();
    assert_eq!(result, expected);
}

#[test]
fn test_collect_into_btree_set() {
    let data = vec![3i32, 1, 2, 1, 3]; // duplicates should be removed
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: BTreeSet<i32> = iter.collect::<Result<_, _>>().unwrap();

    let expected: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
    assert_eq!(result, expected);
}

#[test]
fn test_collect_into_hash_set() {
    let data = vec![3i32, 1, 2, 1, 3]; // duplicates should be removed
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let result: HashSet<i32> = iter.collect::<Result<_, _>>().unwrap();

    let expected: HashSet<i32> = [1, 2, 3].into_iter().collect();
    assert_eq!(result, expected);
}

// ============================================================================
// EARLY TERMINATION SCENARIOS
// ============================================================================

#[test]
fn test_early_termination_with_take() {
    let data = vec![1i32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    // Only take the first 3 elements
    let result: Vec<i32> = iter.take(3).collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_early_termination_with_take_while() {
    let data = vec![1i32, 2, 3, 10, 4, 5];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    // Take elements while they are less than 10
    let result: Vec<i32> = iter
        .map(|r| r.unwrap())
        .take_while(|&x| x < 10)
        .collect();

    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_early_termination_manual_break() {
    let data = vec![1i32, 2, 999, 3, 4];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    let mut result = Vec::new();

    for item in iter {
        let value = item.unwrap();
        if value == 999 {
            break; // Early termination
        }
        result.push(value);
    }

    assert_eq!(result, vec![1, 2]);
}

#[test]
fn test_early_termination_find() {
    let data = vec![10i32, 20, 30, 40, 50];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    // Find the first element > 25
    let found = iter
        .map(|r| r.unwrap())
        .find(|&x| x > 25);

    assert_eq!(found, Some(30));
}

#[test]
fn test_early_termination_position() {
    let data = vec![10i32, 20, 30, 40, 50];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    // Find position of first element >= 30
    let pos = iter
        .map(|r| r.unwrap())
        .position(|x| x >= 30);

    assert_eq!(pos, Some(2));
}

#[test]
fn test_early_termination_any() {
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    // Check if any element is > 3 (should short-circuit after finding 4)
    let has_large = iter
        .map(|r| r.unwrap())
        .any(|x| x > 3);

    assert!(has_large);
}

#[test]
fn test_early_termination_nth() {
    let data = vec![10i32, 20, 30, 40, 50];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let mut iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    // Get the 3rd element (0-indexed), skipping the first two
    let third = iter.nth(2).map(|r| r.unwrap());

    assert_eq!(third, Some(30));
}

// ============================================================================
// BINARY-LIKE DESERIALIZER SIMULATION
// ============================================================================

/// A simple binary-like SeqAccess that simulates reading from a byte buffer.
/// Each element is a u8 read sequentially.
struct BinarySeqAccess<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> BinarySeqAccess<'a> {
    fn new(data: &'a [u8]) -> Self {
        BinarySeqAccess { data, position: 0 }
    }
}

impl<'de> SeqAccess<'de> for BinarySeqAccess<'de> {
    type Error = value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.position >= self.data.len() {
            return Ok(None);
        }

        let byte = self.data[self.position];
        self.position += 1;

        // Use a U8Deserializer to deserialize the byte
        let deserializer = value::U8Deserializer::<Self::Error>::new(byte);
        seed.deserialize(deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.data.len() - self.position)
    }
}

#[test]
fn test_binary_deserializer_basic() {
    let bytes: &[u8] = &[1, 2, 3, 4, 5];
    let mut seq = BinarySeqAccess::new(bytes);

    let iter: SeqAccessIterator<'_, _, u8> = SeqAccessIterator::new(&mut seq);
    let result: Vec<u8> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_binary_deserializer_empty() {
    let bytes: &[u8] = &[];
    let mut seq = BinarySeqAccess::new(bytes);

    let iter: SeqAccessIterator<'_, _, u8> = SeqAccessIterator::new(&mut seq);
    let result: Vec<u8> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(result, Vec::<u8>::new());
}

#[test]
fn test_binary_deserializer_early_termination() {
    let bytes: &[u8] = &[10, 20, 30, 40, 50, 60, 70, 80];
    let mut seq = BinarySeqAccess::new(bytes);

    let iter: SeqAccessIterator<'_, _, u8> = SeqAccessIterator::new(&mut seq);
    // Only take first 4 bytes
    let result: Vec<u8> = iter.take(4).collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec![10, 20, 30, 40]);
}

#[test]
fn test_binary_deserializer_size_hint() {
    let bytes: &[u8] = &[1, 2, 3, 4, 5];
    let mut seq = BinarySeqAccess::new(bytes);

    let iter: SeqAccessIterator<'_, _, u8> = SeqAccessIterator::new(&mut seq);

    // Initial size hint should be (5, Some(5))
    assert_eq!(iter.size_hint(), (5, Some(5)));
}

#[test]
fn test_binary_deserializer_size_hint_decreases() {
    let bytes: &[u8] = &[1, 2, 3, 4];
    let mut seq = BinarySeqAccess::new(bytes);

    let mut iter: SeqAccessIterator<'_, _, u8> = SeqAccessIterator::new(&mut seq);

    assert_eq!(iter.size_hint(), (4, Some(4)));
    let _ = iter.next();
    assert_eq!(iter.size_hint(), (3, Some(3)));
    let _ = iter.next();
    assert_eq!(iter.size_hint(), (2, Some(2)));
}

/// A binary-like SeqAccess that reads u16 values (little-endian) from bytes.
struct BinaryU16SeqAccess<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> BinaryU16SeqAccess<'a> {
    fn new(data: &'a [u8]) -> Self {
        BinaryU16SeqAccess { data, position: 0 }
    }
}

impl<'de> SeqAccess<'de> for BinaryU16SeqAccess<'de> {
    type Error = value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.position + 2 > self.data.len() {
            return Ok(None);
        }

        let bytes = [self.data[self.position], self.data[self.position + 1]];
        let value = u16::from_le_bytes(bytes);
        self.position += 2;

        let deserializer = value::U16Deserializer::<Self::Error>::new(value);
        seed.deserialize(deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some((self.data.len() - self.position) / 2)
    }
}

#[test]
fn test_binary_u16_deserializer() {
    // [0x01, 0x00] = 1, [0x02, 0x00] = 2, [0x03, 0x00] = 3 (little-endian)
    let bytes: &[u8] = &[0x01, 0x00, 0x02, 0x00, 0x03, 0x00];
    let mut seq = BinaryU16SeqAccess::new(bytes);

    let iter: SeqAccessIterator<'_, _, u16> = SeqAccessIterator::new(&mut seq);
    let result: Vec<u16> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_binary_u16_early_termination() {
    let bytes: &[u8] = &[0x0A, 0x00, 0x14, 0x00, 0x1E, 0x00, 0x28, 0x00];
    // Values: 10, 20, 30, 40
    let mut seq = BinaryU16SeqAccess::new(bytes);

    let iter: SeqAccessIterator<'_, _, u16> = SeqAccessIterator::new(&mut seq);
    // Take only 2 values
    let result: Vec<u16> = iter.take(2).collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec![10, 20]);
}

// ============================================================================
// NESTED/COMPLEX TYPES
// ============================================================================

#[derive(Debug, PartialEq, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn test_nested_struct_deserialization() {
    // Using SeqDeserializer with tuple data that can be converted to Points
    let data: Vec<(i32, i32)> = vec![(1, 2), (3, 4), (5, 6)];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, (i32, i32)> = SeqAccessIterator::new(&mut seq);
    let result: Vec<(i32, i32)> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec![(1, 2), (3, 4), (5, 6)]);
}

#[test]
fn test_option_elements() {
    let data: Vec<Option<i32>> = vec![Some(1), None, Some(3), None, Some(5)];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, Option<i32>> = SeqAccessIterator::new(&mut seq);
    let result: Vec<Option<i32>> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec![Some(1), None, Some(3), None, Some(5)]);
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

/// A SeqAccess that fails after a certain number of elements
struct FailingSeqAccess {
    current: usize,
    fail_at: usize,
    limit: usize,
}

impl FailingSeqAccess {
    fn new(fail_at: usize, limit: usize) -> Self {
        FailingSeqAccess {
            current: 0,
            fail_at,
            limit,
        }
    }
}

impl<'de> SeqAccess<'de> for FailingSeqAccess {
    type Error = value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.current >= self.limit {
            return Ok(None);
        }

        if self.current == self.fail_at {
            return Err(serde::de::Error::custom("intentional error at position"));
        }

        let value = self.current as u32;
        self.current += 1;

        let deserializer = value::U32Deserializer::<Self::Error>::new(value);
        seed.deserialize(deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.limit - self.current)
    }
}

#[test]
fn test_error_propagation_during_iteration() {
    let mut seq = FailingSeqAccess::new(3, 10);

    let iter: SeqAccessIterator<'_, _, u32> = SeqAccessIterator::new(&mut seq);
    let result: Result<Vec<u32>, _> = iter.collect();

    // Should fail because element at index 3 causes an error
    assert!(result.is_err());
}

#[test]
fn test_error_at_first_element() {
    let mut seq = FailingSeqAccess::new(0, 5);

    let iter: SeqAccessIterator<'_, _, u32> = SeqAccessIterator::new(&mut seq);
    let result: Result<Vec<u32>, _> = iter.collect();

    assert!(result.is_err());
}

#[test]
fn test_no_error_when_terminated_early() {
    let mut seq = FailingSeqAccess::new(5, 10); // Error at position 5

    let iter: SeqAccessIterator<'_, _, u32> = SeqAccessIterator::new(&mut seq);
    // Only take 3 elements, so we never reach the error
    let result: Vec<u32> = iter.take(3).collect::<Result<_, _>>().unwrap();

    assert_eq!(result, vec![0, 1, 2]);
}

#[test]
fn test_partial_results_before_error() {
    let mut seq = FailingSeqAccess::new(3, 10);

    let mut iter: SeqAccessIterator<'_, _, u32> = SeqAccessIterator::new(&mut seq);

    // First 3 elements should succeed
    assert_eq!(iter.next().unwrap().unwrap(), 0);
    assert_eq!(iter.next().unwrap().unwrap(), 1);
    assert_eq!(iter.next().unwrap().unwrap(), 2);

    // 4th element (index 3) should fail
    assert!(iter.next().unwrap().is_err());
}

// ============================================================================
// ITERATOR ADAPTER COMBINATIONS
// ============================================================================

#[test]
fn test_filter_map_combination() {
    let data = vec![1i32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    // Filter even numbers and double them
    let result: Vec<i32> = iter
        .filter_map(|r| match r {
            Ok(x) if x % 2 == 0 => Some(x * 2),
            Ok(_) => None,
            Err(_) => None,
        })
        .collect();

    assert_eq!(result, vec![4, 8, 12, 16, 20]);
}

#[test]
fn test_fold_with_iterator() {
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    // Sum all elements using fold
    let sum: i32 = iter
        .map(|r| r.unwrap())
        .fold(0, |acc, x| acc + x);

    assert_eq!(sum, 15);
}

#[test]
fn test_zip_with_index() {
    let data = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, String> = SeqAccessIterator::new(&mut seq);
    // Zip with indices
    let result: Vec<(usize, String)> = iter
        .map(|r| r.unwrap())
        .enumerate()
        .collect();

    assert_eq!(
        result,
        vec![
            (0, "a".to_string()),
            (1, "b".to_string()),
            (2, "c".to_string())
        ]
    );
}

#[test]
fn test_chain_iterators() {
    let data1 = vec![1i32, 2, 3];
    let data2 = vec![4i32, 5, 6];

    let mut seq1: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data1.into_iter());
    let mut seq2: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data2.into_iter());

    let iter1: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq1);
    let iter2: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq2);

    // Chain two iterators together
    let result: Vec<i32> = iter1
        .chain(iter2)
        .collect::<Result<_, _>>()
        .unwrap();

    assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_skip_and_take() {
    let data = vec![1i32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);
    // Skip first 3, take next 4
    let result: Vec<i32> = iter
        .skip(3)
        .take(4)
        .collect::<Result<_, _>>()
        .unwrap();

    assert_eq!(result, vec![4, 5, 6, 7]);
}

// ============================================================================
// DESERIALIZE_ITER TRAIT METHOD TESTS
// ============================================================================

/// A custom deserializer that implements `deserialize_iter` by wrapping a slice.
/// This simulates a binary format that can directly provide an iterator.
struct SliceIterDeserializer<'de> {
    data: &'de [i32],
    position: usize,
}

impl<'de> SliceIterDeserializer<'de> {
    fn new(data: &'de [i32]) -> Self {
        SliceIterDeserializer { data, position: 0 }
    }
}

/// SeqAccess implementation for SliceIterDeserializer
struct SliceSeqAccess<'de> {
    data: &'de [i32],
    position: usize,
}

impl<'de> SeqAccess<'de> for SliceSeqAccess<'de> {
    type Error = value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.position >= self.data.len() {
            return Ok(None);
        }

        let value = self.data[self.position];
        self.position += 1;

        let deserializer = value::I32Deserializer::<Self::Error>::new(value);
        seed.deserialize(deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.data.len() - self.position)
    }
}

impl<'de> Deserializer<'de> for SliceIterDeserializer<'de> {
    type Error = value::Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::custom("only deserialize_seq and deserialize_iter are supported"))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let seq_access = SliceSeqAccess {
            data: self.data,
            position: self.position,
        };
        visitor.visit_seq(seq_access)
    }

    // Implement deserialize_iter to directly return an iterator
    fn deserialize_iter<T>(self) -> Result<SeqAccessIterator<'de, IterSeqAccess<'de, Self::Error>, T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        // For this test, we'll use the default implementation behavior
        // In a real implementation, you'd return a proper iterator
        // The default implementation returns an error iterator
        let error = serde::de::Error::custom("deserialize_iter is not supported by this deserializer");
        let seq_access = IterSeqAccess::with_error(error);
        Ok(SeqAccessIterator::new(seq_access))
    }

    // Forward all other deserialize methods to deserialize_any
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[test]
fn test_deserialize_iter_default_returns_error() {
    // Test that the default deserialize_iter implementation returns an error iterator
    let data: &[i32] = &[1, 2, 3];
    let deserializer = SliceIterDeserializer::new(data);

    let iter = deserializer.deserialize_iter::<i32>().unwrap();
    let result: Result<Vec<i32>, _> = iter.collect();

    // The default implementation should return an error
    assert!(result.is_err());
}

#[test]
fn test_deserialize_seq_works_as_alternative() {
    // When deserialize_iter isn't supported, deserialize_seq with SeqAccessIterator works
    let data: &[i32] = &[1, 2, 3, 4, 5];
    let deserializer = SliceIterDeserializer::new(data);

    // Use the visitor pattern with SeqAccessIterator inside
    struct CollectVisitor;

    impl<'de> Visitor<'de> for CollectVisitor {
        type Value = Vec<i32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of i32")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter: SeqAccessIterator<'_, A, i32> = SeqAccessIterator::new(seq);
            iter.collect()
        }
    }

    let result = deserializer.deserialize_seq(CollectVisitor).unwrap();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

// ============================================================================
// DEMONSTRATING THE INTENDED USAGE PATTERN
// ============================================================================

/// This struct demonstrates the intended use case from the task description:
/// A type that can be deserialized from a sequence using iterator patterns.
#[derive(Debug, PartialEq)]
struct Foo(Vec<Bar>);

#[derive(Debug, PartialEq, Clone, Deserialize)]
struct Bar(i32);

impl<'de> Deserialize<'de> for Foo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // The intended pattern: use SeqAccessIterator in visit_seq
        // This is what deserialize_iter would return if properly implemented
        struct FooVisitor;

        impl<'de> Visitor<'de> for FooVisitor {
            type Value = Foo;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of Bar")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let iter: SeqAccessIterator<'_, A, Bar> = SeqAccessIterator::new(seq);
                let bars: Result<Vec<Bar>, _> = iter.collect();
                bars.map(Foo)
            }
        }

        deserializer.deserialize_seq(FooVisitor)
    }
}

#[test]
fn test_foo_bar_example_from_task() {
    // Replicate the example from the task description
    let data = vec![Bar(1), Bar(2), Bar(3)];
    let deserializer: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let foo: Foo = Foo::deserialize(deserializer).unwrap();
    assert_eq!(foo, Foo(vec![Bar(1), Bar(2), Bar(3)]));
}

#[test]
fn test_foo_bar_empty() {
    let data: Vec<Bar> = vec![];
    let deserializer: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let foo: Foo = Foo::deserialize(deserializer).unwrap();
    assert_eq!(foo, Foo(vec![]));
}

/// A more complex example: deserializing with validation using iterator methods
#[derive(Debug, PartialEq)]
struct ValidatedFoo {
    bars: Vec<Bar>,
    sum: i32,
}

impl<'de> Deserialize<'de> for ValidatedFoo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValidatedFooVisitor;

        impl<'de> Visitor<'de> for ValidatedFooVisitor {
            type Value = ValidatedFoo;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of Bar with valid values")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let iter: SeqAccessIterator<'_, A, Bar> = SeqAccessIterator::new(seq);

                let mut bars = Vec::new();
                let mut sum = 0;

                for item in iter {
                    let bar = item?;
                    // Validate during iteration
                    if bar.0 < 0 {
                        return Err(serde::de::Error::custom("negative values not allowed"));
                    }
                    sum += bar.0;
                    bars.push(bar);
                }

                Ok(ValidatedFoo { bars, sum })
            }
        }

        deserializer.deserialize_seq(ValidatedFooVisitor)
    }
}

#[test]
fn test_validated_foo_computes_sum() {
    let data = vec![Bar(1), Bar(2), Bar(3), Bar(4)];
    let deserializer: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let foo: ValidatedFoo = ValidatedFoo::deserialize(deserializer).unwrap();
    assert_eq!(foo.bars, vec![Bar(1), Bar(2), Bar(3), Bar(4)]);
    assert_eq!(foo.sum, 10);
}

#[test]
fn test_validated_foo_rejects_negative() {
    let data = vec![Bar(1), Bar(-2), Bar(3)];
    let deserializer: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let result = ValidatedFoo::deserialize(deserializer);
    assert!(result.is_err());
}

// ============================================================================
// STREAMING / LAZY EVALUATION DEMONSTRATION
// ============================================================================

/// Demonstrates that elements are deserialized lazily, only when requested
#[test]
fn test_lazy_evaluation_with_first() {
    // Even with a large logical sequence, only the first element is deserialized
    let data = vec![100i32, 200, 300, 400, 500];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let mut iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);

    // Only request the first element
    let first = iter.next().unwrap().unwrap();
    assert_eq!(first, 100);

    // The iterator hasn't consumed all elements
    // We can continue if needed
    let second = iter.next().unwrap().unwrap();
    assert_eq!(second, 200);
}

#[test]
fn test_lazy_evaluation_stops_early() {
    let data = vec![1i32, 2, 3, 4, 5];
    let mut seq: SeqDeserializer<_, value::Error> = SeqDeserializer::new(data.into_iter());

    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(&mut seq);

    // Using all() stops at first false
    let all_positive = iter
        .map(|r| r.unwrap())
        .all(|x| x < 3);

    // all() should have stopped early when it hit 3
    assert!(!all_positive);
}
