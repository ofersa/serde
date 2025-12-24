//! Integration tests for `deserialize_iter` demonstrating usage with various
//! collection types, custom deserializers, and early termination scenarios.

#![allow(
    clippy::needless_pass_by_value,
    clippy::derive_partial_eq_without_eq,
    clippy::uninlined_format_args
)]

use serde::de::value::{Error as ValueError, SeqDeserializer};
use serde::de::{
    Deserialize, DeserializeSeed, Deserializer, IntoDeserializer, SeqAccess, SeqAccessIterator,
    Visitor,
};
use serde::forward_to_deserialize_any;
use serde_derive::Deserialize;
use std::collections::{BTreeSet, HashSet, LinkedList, VecDeque};
use std::fmt;

// ============================================================================
// Custom collection type
// ============================================================================

/// A custom collection that wraps a Vec with additional behavior.
#[derive(Debug, PartialEq)]
struct CustomCollection<T> {
    items: Vec<T>,
    count: usize,
}

impl<T> CustomCollection<T> {
    fn new() -> Self {
        CustomCollection {
            items: Vec::new(),
            count: 0,
        }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
        self.count += 1;
    }

    fn from_iter<I: Iterator<Item = T>>(iter: I) -> Self {
        let mut coll = Self::new();
        for item in iter {
            coll.push(item);
        }
        coll
    }
}

// ============================================================================
// Type using deserialize_iter - demonstrates the pattern from the task
// ============================================================================

/// A type that uses `deserialize_iter` for its Vec field.
/// This demonstrates the ergonomic pattern that `deserialize_iter` enables.
#[derive(Debug, PartialEq)]
struct Foo(Vec<i32>);

impl<'de> Deserialize<'de> for Foo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Using deserialize_iter makes this much simpler than writing a custom Visitor
        let values = deserializer.deserialize_iter::<i32>()?;
        Ok(Foo(values))
    }
}

// ============================================================================
// JSON-like deserializer wrapper
// ============================================================================

/// A JSON-like deserializer that reports `is_human_readable() = true`.
struct JsonLikeDeserializer<D> {
    inner: D,
}

impl<D> JsonLikeDeserializer<D> {
    fn new(inner: D) -> Self {
        JsonLikeDeserializer { inner }
    }
}

impl<'de, D> Deserializer<'de> for JsonLikeDeserializer<D>
where
    D: Deserializer<'de>,
{
    type Error = D::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_any(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_seq(visitor)
    }

    fn is_human_readable(&self) -> bool {
        true
    }

    // Forward all other methods
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

// ============================================================================
// Binary-like deserializer wrapper
// ============================================================================

/// A binary-like deserializer that reports `is_human_readable() = false`.
struct BinaryLikeDeserializer<D> {
    inner: D,
}

impl<D> BinaryLikeDeserializer<D> {
    fn new(inner: D) -> Self {
        BinaryLikeDeserializer { inner }
    }
}

impl<'de, D> Deserializer<'de> for BinaryLikeDeserializer<D>
where
    D: Deserializer<'de>,
{
    type Error = D::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_any(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_seq(visitor)
    }

    fn is_human_readable(&self) -> bool {
        false
    }

    // Forward all other methods
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

// ============================================================================
// Tests for Vec with deserialize_iter
// ============================================================================

#[test]
fn test_deserialize_iter_empty_vec() {
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(vec![].into_iter());
    let result: Vec<i32> = seq.deserialize_iter().unwrap();
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_deserialize_iter_vec_of_integers() {
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(vec![1, 2, 3, 4, 5].into_iter());
    let result: Vec<i32> = seq.deserialize_iter().unwrap();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_deserialize_iter_vec_of_strings() {
    let data = vec!["hello".to_string(), "world".to_string(), "test".to_string()];
    let seq: SeqDeserializer<std::vec::IntoIter<String>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: Vec<String> = seq.deserialize_iter().unwrap();
    assert_eq!(result, vec!["hello", "world", "test"]);
}

#[test]
fn test_deserialize_iter_large_vec() {
    let data: Vec<i32> = (0..1000).collect();
    let expected = data.clone();
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: Vec<i32> = seq.deserialize_iter().unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_foo_with_deserialize_iter() {
    // Test the Foo type which demonstrates the ergonomic usage pattern from the task
    let data = vec![1, 2, 3, 4, 5];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: Foo = Foo::deserialize(seq).unwrap();
    assert_eq!(result, Foo(vec![1, 2, 3, 4, 5]));
}

// ============================================================================
// Tests for custom collection types
// ============================================================================

#[test]
fn test_deserialize_iter_into_custom_collection() {
    // Use SeqAccessIterator to manually collect into a custom collection
    struct CustomCollectionWrapper(CustomCollection<i32>);

    impl<'de> Deserialize<'de> for CustomCollectionWrapper {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct CustomVisitor;

            impl<'de> Visitor<'de> for CustomVisitor {
                type Value = CustomCollectionWrapper;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    let coll = CustomCollection::from_iter(
                        iter.map(|r| r.unwrap())
                    );
                    Ok(CustomCollectionWrapper(coll))
                }
            }

            deserializer.deserialize_seq(CustomVisitor)
        }
    }

    let data = vec![10, 20, 30];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: CustomCollectionWrapper = CustomCollectionWrapper::deserialize(seq).unwrap();

    assert_eq!(result.0.items, vec![10, 20, 30]);
    assert_eq!(result.0.count, 3);
}

#[test]
fn test_deserialize_iter_into_btreeset() {
    // Using SeqAccessIterator to collect into BTreeSet
    struct BTreeSetWrapper(BTreeSet<i32>);

    impl<'de> Deserialize<'de> for BTreeSetWrapper {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct SetVisitor;

            impl<'de> Visitor<'de> for SetVisitor {
                type Value = BTreeSetWrapper;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence of unique values")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    let set: Result<BTreeSet<i32>, _> = iter.collect();
                    Ok(BTreeSetWrapper(set?))
                }
            }

            deserializer.deserialize_seq(SetVisitor)
        }
    }

    let data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3]; // Duplicates
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: BTreeSetWrapper = BTreeSetWrapper::deserialize(seq).unwrap();

    // Duplicates should be removed
    let expected: BTreeSet<i32> = [1, 2, 3, 4, 5, 6, 9].into_iter().collect();
    assert_eq!(result.0, expected);
}

#[test]
fn test_deserialize_iter_into_hashset() {
    struct HashSetWrapper(HashSet<String>);

    impl<'de> Deserialize<'de> for HashSetWrapper {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct SetVisitor;

            impl<'de> Visitor<'de> for SetVisitor {
                type Value = HashSetWrapper;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence of strings")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, String> = SeqAccessIterator::new(seq);
                    let set: Result<HashSet<String>, _> = iter.collect();
                    Ok(HashSetWrapper(set?))
                }
            }

            deserializer.deserialize_seq(SetVisitor)
        }
    }

    let data = vec!["a".to_string(), "b".to_string(), "a".to_string(), "c".to_string()];
    let seq: SeqDeserializer<std::vec::IntoIter<String>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: HashSetWrapper = HashSetWrapper::deserialize(seq).unwrap();

    assert_eq!(result.0.len(), 3);
    assert!(result.0.contains("a"));
    assert!(result.0.contains("b"));
    assert!(result.0.contains("c"));
}

#[test]
fn test_deserialize_iter_into_vecdeque() {
    struct VecDequeWrapper(VecDeque<i32>);

    impl<'de> Deserialize<'de> for VecDequeWrapper {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct DequeVisitor;

            impl<'de> Visitor<'de> for DequeVisitor {
                type Value = VecDequeWrapper;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    let deque: Result<VecDeque<i32>, _> = iter.collect();
                    Ok(VecDequeWrapper(deque?))
                }
            }

            deserializer.deserialize_seq(DequeVisitor)
        }
    }

    let data = vec![1, 2, 3, 4, 5];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: VecDequeWrapper = VecDequeWrapper::deserialize(seq).unwrap();

    let expected: VecDeque<i32> = [1, 2, 3, 4, 5].into_iter().collect();
    assert_eq!(result.0, expected);
}

#[test]
fn test_deserialize_iter_into_linked_list() {
    struct LinkedListWrapper(LinkedList<i32>);

    impl<'de> Deserialize<'de> for LinkedListWrapper {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct ListVisitor;

            impl<'de> Visitor<'de> for ListVisitor {
                type Value = LinkedListWrapper;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    let list: Result<LinkedList<i32>, _> = iter.collect();
                    Ok(LinkedListWrapper(list?))
                }
            }

            deserializer.deserialize_seq(ListVisitor)
        }
    }

    let data = vec![10, 20, 30];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: LinkedListWrapper = LinkedListWrapper::deserialize(seq).unwrap();

    let expected: LinkedList<i32> = [10, 20, 30].into_iter().collect();
    assert_eq!(result.0, expected);
}

// ============================================================================
// Tests for early termination scenarios
// ============================================================================

#[test]
fn test_early_termination_with_take() {
    struct FirstThree(Vec<i32>);

    impl<'de> Deserialize<'de> for FirstThree {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct TakeVisitor;

            impl<'de> Visitor<'de> for TakeVisitor {
                type Value = FirstThree;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    // Only take first 3 elements, even if more are present
                    let result: Result<Vec<i32>, _> = iter.take(3).collect();
                    Ok(FirstThree(result?))
                }
            }

            deserializer.deserialize_seq(TakeVisitor)
        }
    }

    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: FirstThree = FirstThree::deserialize(seq).unwrap();

    assert_eq!(result.0, vec![1, 2, 3]);
}

#[test]
fn test_early_termination_with_take_while() {
    struct UntilZero(Vec<i32>);

    impl<'de> Deserialize<'de> for UntilZero {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct TakeWhileVisitor;

            impl<'de> Visitor<'de> for TakeWhileVisitor {
                type Value = UntilZero;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    // Take elements until we see a zero
                    let result: Vec<i32> = iter
                        .take_while(|r| r.as_ref().map(|&x| x != 0).unwrap_or(false))
                        .map(|r| r.unwrap())
                        .collect();
                    Ok(UntilZero(result))
                }
            }

            deserializer.deserialize_seq(TakeWhileVisitor)
        }
    }

    let data = vec![1, 2, 3, 0, 4, 5];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: UntilZero = UntilZero::deserialize(seq).unwrap();

    assert_eq!(result.0, vec![1, 2, 3]);
}

#[test]
fn test_early_termination_find() {
    struct FindFirst(Option<i32>);

    impl<'de> Deserialize<'de> for FindFirst {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct FindVisitor;

            impl<'de> Visitor<'de> for FindVisitor {
                type Value = FindFirst;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    // Find the first element greater than 50
                    let found = iter
                        .filter_map(|r| r.ok())
                        .find(|&x| x > 50);
                    Ok(FindFirst(found))
                }
            }

            deserializer.deserialize_seq(FindVisitor)
        }
    }

    let data = vec![10, 20, 30, 60, 70, 80];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: FindFirst = FindFirst::deserialize(seq).unwrap();

    assert_eq!(result.0, Some(60));
}

#[test]
fn test_early_termination_any() {
    struct HasNegative(bool);

    impl<'de> Deserialize<'de> for HasNegative {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct AnyVisitor;

            impl<'de> Visitor<'de> for AnyVisitor {
                type Value = HasNegative;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    let has_negative = iter
                        .filter_map(|r| r.ok())
                        .any(|x| x < 0);
                    Ok(HasNegative(has_negative))
                }
            }

            deserializer.deserialize_seq(AnyVisitor)
        }
    }

    let data_with_negative = vec![1, 2, -3, 4];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data_with_negative.into_iter());
    let result: HasNegative = HasNegative::deserialize(seq).unwrap();
    assert!(result.0);

    let data_all_positive = vec![1, 2, 3, 4];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data_all_positive.into_iter());
    let result: HasNegative = HasNegative::deserialize(seq).unwrap();
    assert!(!result.0);
}

#[test]
fn test_early_termination_into_inner() {
    // Test that we can get back the SeqAccess and continue from where we left off
    struct PartialConsumer;

    impl<'de> Deserialize<'de> for PartialConsumer {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct PartialVisitor;

            impl<'de> Visitor<'de> for PartialVisitor {
                type Value = PartialConsumer;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);

                    // Read first element
                    let first = iter.next();
                    assert!(matches!(first, Some(Ok(1))));

                    // Get the inner SeqAccess back
                    let mut inner = iter.into_inner();

                    // Continue reading
                    let second: Option<i32> = inner.next_element().unwrap();
                    assert_eq!(second, Some(2));

                    Ok(PartialConsumer)
                }
            }

            deserializer.deserialize_seq(PartialVisitor)
        }
    }

    let data = vec![1, 2, 3, 4];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let _ = PartialConsumer::deserialize(seq).unwrap();
}

// ============================================================================
// Tests with JSON-like deserializer
// ============================================================================

#[test]
fn test_deserialize_iter_json_like() {
    let data = vec![1, 2, 3, 4, 5];
    let inner: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let deserializer = JsonLikeDeserializer::new(inner);

    // Verify it's human readable
    assert!(deserializer.is_human_readable());

    let result: Vec<i32> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_deserialize_iter_json_like_with_strings() {
    let data = vec!["foo".to_string(), "bar".to_string()];
    let inner: SeqDeserializer<std::vec::IntoIter<String>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let deserializer = JsonLikeDeserializer::new(inner);

    let result: Vec<String> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec!["foo", "bar"]);
}

// ============================================================================
// Tests with binary-like deserializer
// ============================================================================

#[test]
fn test_deserialize_iter_binary_like() {
    let data = vec![1u8, 2, 3, 4, 5];
    let inner: SeqDeserializer<std::vec::IntoIter<u8>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let deserializer = BinaryLikeDeserializer::new(inner);

    // Verify it's not human readable
    assert!(!deserializer.is_human_readable());

    let result: Vec<u8> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_deserialize_iter_binary_like_compact_integers() {
    // Simulating compact binary encoding with larger integers
    let data: Vec<i64> = vec![100000, 200000, 300000];
    let inner: SeqDeserializer<std::vec::IntoIter<i64>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let deserializer = BinaryLikeDeserializer::new(inner);

    let result: Vec<i64> = deserializer.deserialize_iter().unwrap();
    assert_eq!(result, vec![100000, 200000, 300000]);
}

// ============================================================================
// Test error handling
// ============================================================================

/// A mock SeqAccess that produces an error mid-stream.
struct ErrorMidstreamSeqAccess {
    values: Vec<i32>,
    index: usize,
    error_at: usize,
}

impl ErrorMidstreamSeqAccess {
    fn new(values: Vec<i32>, error_at: usize) -> Self {
        ErrorMidstreamSeqAccess {
            values,
            index: 0,
            error_at,
        }
    }
}

impl<'de> SeqAccess<'de> for ErrorMidstreamSeqAccess {
    type Error = ValueError;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: DeserializeSeed<'de>,
    {
        if self.index >= self.values.len() {
            return Ok(None);
        }
        if self.index == self.error_at {
            return Err(serde::de::Error::custom("simulated error"));
        }
        let value = self.values[self.index];
        self.index += 1;
        seed.deserialize(value.into_deserializer()).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len() - self.index)
    }
}

#[test]
fn test_error_propagation_in_iter() {
    let seq = ErrorMidstreamSeqAccess::new(vec![1, 2, 3, 4, 5], 2);
    let mut iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    // First two elements succeed
    assert_eq!(iter.next().unwrap().unwrap(), 1);
    assert_eq!(iter.next().unwrap().unwrap(), 2);

    // Third element errors
    let err = iter.next().unwrap();
    assert!(err.is_err());
}

#[test]
fn test_error_stops_collect() {
    let seq = ErrorMidstreamSeqAccess::new(vec![1, 2, 3, 4, 5], 3);
    let iter: SeqAccessIterator<_, i32> = SeqAccessIterator::new(seq);

    let result: Result<Vec<i32>, _> = iter.collect();
    assert!(result.is_err());
}

// ============================================================================
// Test with nested types
// ============================================================================

#[test]
fn test_deserialize_iter_nested_vecs() {
    // Test deserializing Vec<Vec<i32>> using iterator approach
    struct NestedVecs(Vec<Vec<i32>>);

    impl<'de> Deserialize<'de> for NestedVecs {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let vecs = deserializer.deserialize_iter::<Vec<i32>>()?;
            Ok(NestedVecs(vecs))
        }
    }

    // Create nested sequence deserializer
    let inner_vecs: Vec<Vec<i32>> = vec![
        vec![1, 2, 3],
        vec![4, 5],
        vec![6, 7, 8, 9],
    ];

    // Use a custom approach since we can't easily nest SeqDeserializers
    struct NestedSeqDeserializer {
        vecs: Vec<Vec<i32>>,
        index: usize,
    }

    impl<'de> Deserializer<'de> for NestedSeqDeserializer {
        type Error = ValueError;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(serde::de::Error::custom("not supported"))
        }

        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            visitor.visit_seq(NestedSeqAccess {
                vecs: self.vecs,
                index: 0,
            })
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

    struct NestedSeqAccess {
        vecs: Vec<Vec<i32>>,
        index: usize,
    }

    impl<'de> SeqAccess<'de> for NestedSeqAccess {
        type Error = ValueError;

        fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
        where
            S: DeserializeSeed<'de>,
        {
            if self.index >= self.vecs.len() {
                return Ok(None);
            }
            let vec = self.vecs[self.index].clone();
            self.index += 1;

            let inner_seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
                SeqDeserializer::new(vec.into_iter());
            seed.deserialize(inner_seq).map(Some)
        }

        fn size_hint(&self) -> Option<usize> {
            Some(self.vecs.len() - self.index)
        }
    }

    let deserializer = NestedSeqDeserializer {
        vecs: inner_vecs.clone(),
        index: 0,
    };
    let result: NestedVecs = NestedVecs::deserialize(deserializer).unwrap();

    assert_eq!(result.0, inner_vecs);
}

// ============================================================================
// Test with derived types
// ============================================================================

#[test]
fn test_deserialize_iter_with_derived_types() {
    // Test that deserialize_iter works with serde_derive types
    #[derive(Debug, PartialEq, Deserialize)]
    struct Item {
        id: u32,
        name: String,
    }

    struct ItemList(Vec<Item>);

    impl<'de> Deserialize<'de> for ItemList {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct ItemVisitor;

            impl<'de> Visitor<'de> for ItemVisitor {
                type Value = ItemList;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence of items")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, Item> = SeqAccessIterator::new(seq);
                    let items: Result<Vec<Item>, _> = iter.collect();
                    Ok(ItemList(items?))
                }
            }

            deserializer.deserialize_seq(ItemVisitor)
        }
    }

    // For this test, we just verify the type compiles and the structure works
    // In a real scenario, you'd use a full deserializer like serde_json
}

// ============================================================================
// Test transformations with iterator combinators
// ============================================================================

#[test]
fn test_iter_with_map() {
    struct DoubledVec(Vec<i32>);

    impl<'de> Deserialize<'de> for DoubledVec {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct DoubleVisitor;

            impl<'de> Visitor<'de> for DoubleVisitor {
                type Value = DoubledVec;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    let doubled: Result<Vec<i32>, _> = iter
                        .map(|r| r.map(|x| x * 2))
                        .collect();
                    Ok(DoubledVec(doubled?))
                }
            }

            deserializer.deserialize_seq(DoubleVisitor)
        }
    }

    let data = vec![1, 2, 3, 4, 5];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: DoubledVec = DoubledVec::deserialize(seq).unwrap();

    assert_eq!(result.0, vec![2, 4, 6, 8, 10]);
}

#[test]
fn test_iter_with_filter() {
    struct EvenOnly(Vec<i32>);

    impl<'de> Deserialize<'de> for EvenOnly {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct FilterVisitor;

            impl<'de> Visitor<'de> for FilterVisitor {
                type Value = EvenOnly;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    let evens: Vec<i32> = iter
                        .filter_map(|r| match r {
                            Ok(x) if x % 2 == 0 => Some(Ok(x)),
                            Ok(_) => None,
                            Err(e) => Some(Err(e)),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(EvenOnly(evens))
                }
            }

            deserializer.deserialize_seq(FilterVisitor)
        }
    }

    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: EvenOnly = EvenOnly::deserialize(seq).unwrap();

    assert_eq!(result.0, vec![2, 4, 6, 8, 10]);
}

#[test]
fn test_iter_with_fold() {
    struct Sum(i32);

    impl<'de> Deserialize<'de> for Sum {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct SumVisitor;

            impl<'de> Visitor<'de> for SumVisitor {
                type Value = Sum;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut iter: SeqAccessIterator<A, i32> = SeqAccessIterator::new(seq);
                    let sum = iter
                        .try_fold(0i32, |acc, r| r.map(|x| acc + x))?;
                    Ok(Sum(sum))
                }
            }

            deserializer.deserialize_seq(SumVisitor)
        }
    }

    let data = vec![1, 2, 3, 4, 5];
    let seq: SeqDeserializer<std::vec::IntoIter<i32>, ValueError> =
        SeqDeserializer::new(data.into_iter());
    let result: Sum = Sum::deserialize(seq).unwrap();

    assert_eq!(result.0, 15);
}
