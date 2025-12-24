//! Integration tests demonstrating `deserialize_iter` usage.
//!
//! These tests show how the `deserialize_iter` method can be used to lazily
//! deserialize sequences without requiring a custom Visitor implementation.

#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args
)]

use serde::de::value::SeqDeserializer;
use serde::de::{
    Deserialize, DeserializeSeed, Deserializer, Error, IntoDeserializer, SeqAccess, Visitor,
};
use serde_derive::Deserialize;
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::fmt;
use std::marker::PhantomData;

//////////////////////////////////////////////////////////////////////////////
// Helper types for testing

/// An iterator adapter that wraps a SeqAccess and yields deserialized elements.
///
/// This demonstrates the pattern that `deserialize_iter` would provide
/// at the Deserializer level.
struct SeqIter<'de, A, T> {
    seq: A,
    _marker: PhantomData<(&'de (), T)>,
}

impl<'de, A, T> SeqIter<'de, A, T>
where
    A: SeqAccess<'de>,
{
    fn new(seq: A) -> Self {
        SeqIter {
            seq,
            _marker: PhantomData,
        }
    }
}

impl<'de, A, T> Iterator for SeqIter<'de, A, T>
where
    A: SeqAccess<'de>,
    T: Deserialize<'de>,
{
    type Item = Result<T, A::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.seq.next_element() {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.seq.size_hint() {
            Some(len) => (len, Some(len)),
            None => (0, None),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
// A simple JSON-like deserializer for testing
// This is a "self-describing" format

mod json_like {
    use super::*;
    use serde::de;

    #[derive(Debug)]
    pub struct Error(String);

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for Error {}

    impl de::Error for Error {
        fn custom<T: fmt::Display>(msg: T) -> Self {
            Error(msg.to_string())
        }
    }

    /// A simple deserializer that reads from a Vec of values.
    pub struct VecDeserializer<T> {
        values: Vec<T>,
    }

    impl<T> VecDeserializer<T> {
        pub fn new(values: Vec<T>) -> Self {
            VecDeserializer { values }
        }
    }

    impl<'de, T> de::Deserializer<'de> for VecDeserializer<T>
    where
        T: IntoDeserializer<'de, Error>,
    {
        type Error = Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            self.deserialize_seq(visitor)
        }

        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            let seq = SeqDeserializer::new(self.values.into_iter());
            visitor.visit_seq(seq)
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
// A simple binary-like deserializer for testing
// This is a "non-self-describing" format

mod binary_like {
    use super::*;
    use serde::de;

    #[derive(Debug)]
    pub struct Error(String);

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for Error {}

    impl de::Error for Error {
        fn custom<T: fmt::Display>(msg: T) -> Self {
            Error(msg.to_string())
        }
    }

    /// A binary-like deserializer that has length-prefixed sequences.
    pub struct BinaryDeserializer {
        /// The raw u32 values to deserialize
        values: Vec<u32>,
        /// Current position
        pos: usize,
    }

    impl BinaryDeserializer {
        pub fn new(values: Vec<u32>) -> Self {
            BinaryDeserializer { values, pos: 0 }
        }

        fn read_u32(&mut self) -> Result<u32, Error> {
            if self.pos < self.values.len() {
                let val = self.values[self.pos];
                self.pos += 1;
                Ok(val)
            } else {
                Err(Error::custom("unexpected end of input"))
            }
        }
    }

    impl<'de> de::Deserializer<'de> for &mut BinaryDeserializer {
        type Error = Error;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(Error::custom("binary format is not self-describing"))
        }

        fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            visitor.visit_u32(self.read_u32()?)
        }

        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            let len = self.read_u32()? as usize;
            visitor.visit_seq(BinarySeqAccess { de: self, remaining: len })
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct tuple
            tuple_struct map struct enum identifier ignored_any
        }

        fn is_human_readable(&self) -> bool {
            false
        }
    }

    struct BinarySeqAccess<'a> {
        de: &'a mut BinaryDeserializer,
        remaining: usize,
    }

    impl<'de, 'a> de::SeqAccess<'de> for BinarySeqAccess<'a> {
        type Error = Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: DeserializeSeed<'de>,
        {
            if self.remaining == 0 {
                return Ok(None);
            }
            self.remaining -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        }

        fn size_hint(&self) -> Option<usize> {
            Some(self.remaining)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
// Custom collection type for testing

/// A custom collection that wraps a VecDeque.
#[derive(Debug, PartialEq)]
struct RingBuffer<T> {
    inner: VecDeque<T>,
    capacity: usize,
}

#[allow(dead_code)]
impl<T> RingBuffer<T> {
    fn with_capacity(capacity: usize) -> Self {
        RingBuffer {
            inner: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, value: T) {
        if self.inner.len() >= self.capacity {
            self.inner.pop_front();
        }
        self.inner.push_back(value);
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }
}

impl<T> FromIterator<T> for RingBuffer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let inner: VecDeque<T> = iter.into_iter().collect();
        let capacity = inner.len();
        RingBuffer { inner, capacity }
    }
}

/// A sorted collection that maintains order.
#[derive(Debug, PartialEq)]
struct SortedVec<T: Ord>(Vec<T>);

#[allow(dead_code)]
impl<T: Ord> SortedVec<T> {
    fn new() -> Self {
        SortedVec(Vec::new())
    }

    fn insert(&mut self, value: T) {
        let pos = self.0.binary_search(&value).unwrap_or_else(|p| p);
        self.0.insert(pos, value);
    }
}

impl<T: Ord> FromIterator<T> for SortedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec: Vec<T> = iter.into_iter().collect();
        vec.sort();
        SortedVec(vec)
    }
}

//////////////////////////////////////////////////////////////////////////////
// Visitor that uses iterator pattern

struct IterCollectVisitor<T, C> {
    _marker: PhantomData<(T, C)>,
}

impl<T, C> IterCollectVisitor<T, C> {
    fn new() -> Self {
        IterCollectVisitor {
            _marker: PhantomData,
        }
    }
}

impl<'de, T, C> Visitor<'de> for IterCollectVisitor<T, C>
where
    T: Deserialize<'de>,
    C: FromIterator<T>,
{
    type Value = C;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Use the SeqIter adapter to convert SeqAccess into an Iterator
        let iter = SeqIter::<A, T>::new(seq);
        iter.collect::<Result<C, _>>()
    }
}

//////////////////////////////////////////////////////////////////////////////
// Tests with Vec

#[test]
fn test_iter_vec_integers() {
    // Test deserializing a sequence of integers into a Vec using iterator pattern
    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data.clone());

    let result: Vec<u32> = deserializer
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    assert_eq!(result, data);
}

#[test]
fn test_iter_vec_empty() {
    // Test deserializing an empty sequence
    let data: Vec<u32> = vec![];
    let deserializer = json_like::VecDeserializer::new(data.clone());

    let result: Vec<u32> = deserializer
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    assert_eq!(result, data);
}

#[test]
fn test_iter_vec_single_element() {
    // Test deserializing a single-element sequence
    let data = vec![42u32];
    let deserializer = json_like::VecDeserializer::new(data.clone());

    let result: Vec<u32> = deserializer
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    assert_eq!(result, data);
}

#[test]
fn test_iter_vec_strings() {
    // Test deserializing a sequence of strings
    let data = vec!["hello".to_string(), "world".to_string()];
    let deserializer = json_like::VecDeserializer::new(data.clone());

    let result: Vec<String> = deserializer
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    assert_eq!(result, data);
}

//////////////////////////////////////////////////////////////////////////////
// Tests with custom collection types

#[test]
fn test_iter_btreeset() {
    // Test deserializing into a BTreeSet (automatically deduplicates and sorts)
    let data = vec![3u32, 1, 4, 1, 5, 9, 2, 6];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: BTreeSet<u32> = deserializer
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    let expected: BTreeSet<u32> = [1, 2, 3, 4, 5, 6, 9].into_iter().collect();
    assert_eq!(result, expected);
}

#[test]
fn test_iter_hashset() {
    // Test deserializing into a HashSet
    let data = vec![1u32, 2, 3, 2, 1];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: HashSet<u32> = deserializer
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    let expected: HashSet<u32> = [1, 2, 3].into_iter().collect();
    assert_eq!(result, expected);
}

#[test]
fn test_iter_vecdeque() {
    // Test deserializing into a VecDeque
    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data.clone());

    let result: VecDeque<u32> = deserializer
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    let expected: VecDeque<u32> = data.into_iter().collect();
    assert_eq!(result, expected);
}

#[test]
fn test_iter_ring_buffer() {
    // Test deserializing into a custom RingBuffer collection
    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data.clone());

    let result: RingBuffer<u32> = deserializer
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    let expected: RingBuffer<u32> = data.into_iter().collect();
    assert_eq!(result, expected);
}

#[test]
fn test_iter_sorted_vec() {
    // Test deserializing into a custom SortedVec collection
    let data = vec![5u32, 2, 8, 1, 9, 3];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: SortedVec<u32> = deserializer
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    let expected = SortedVec(vec![1, 2, 3, 5, 8, 9]);
    assert_eq!(result, expected);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for early termination

struct TakeNVisitor<T> {
    n: usize,
    _marker: PhantomData<T>,
}

impl<T> TakeNVisitor<T> {
    fn new(n: usize) -> Self {
        TakeNVisitor {
            n,
            _marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for TakeNVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a sequence with at least {} elements", self.n)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Only take the first n elements, demonstrating early termination
        let iter = SeqIter::<A, T>::new(seq);
        iter.take(self.n).collect::<Result<Vec<T>, _>>()
    }
}

#[test]
fn test_iter_early_termination() {
    // Test that we can stop iterating before consuming all elements
    let data = vec![1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let deserializer = json_like::VecDeserializer::new(data);

    // Only take first 3 elements
    let result: Vec<u32> = deserializer
        .deserialize_seq(TakeNVisitor::new(3))
        .unwrap();

    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_iter_early_termination_empty() {
    // Test early termination on empty sequence
    let data: Vec<u32> = vec![];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Vec<u32> = deserializer
        .deserialize_seq(TakeNVisitor::new(5))
        .unwrap();

    assert_eq!(result, Vec::<u32>::new());
}

#[test]
fn test_iter_early_termination_take_more_than_available() {
    // Test taking more elements than available
    let data = vec![1u32, 2, 3];
    let deserializer = json_like::VecDeserializer::new(data.clone());

    let result: Vec<u32> = deserializer
        .deserialize_seq(TakeNVisitor::new(10))
        .unwrap();

    assert_eq!(result, data);
}

//////////////////////////////////////////////////////////////////////////////
// Tests with find/any patterns

struct FindFirstVisitor<T, F> {
    predicate: F,
    _marker: PhantomData<T>,
}

impl<T, F> FindFirstVisitor<T, F> {
    fn new(predicate: F) -> Self {
        FindFirstVisitor {
            predicate,
            _marker: PhantomData,
        }
    }
}

impl<'de, T, F> Visitor<'de> for FindFirstVisitor<T, F>
where
    T: Deserialize<'de>,
    F: Fn(&T) -> bool,
{
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let iter = SeqIter::<A, T>::new(seq);
        for item in iter {
            let item = item?;
            if (self.predicate)(&item) {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }
}

#[test]
fn test_iter_find_first() {
    // Test finding the first element matching a predicate
    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Option<u32> = deserializer
        .deserialize_seq(FindFirstVisitor::new(|&x| x > 3))
        .unwrap();

    assert_eq!(result, Some(4));
}

#[test]
fn test_iter_find_none() {
    // Test finding when no element matches
    let data = vec![1u32, 2, 3];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Option<u32> = deserializer
        .deserialize_seq(FindFirstVisitor::new(|&x| x > 10))
        .unwrap();

    assert_eq!(result, None);
}

//////////////////////////////////////////////////////////////////////////////
// Tests with binary-like (non-self-describing) deserializer

#[test]
fn test_iter_binary_like_vec() {
    // Test deserializing from a binary-like format
    // Format: [length, elem1, elem2, ...]
    let data = vec![5, 10, 20, 30, 40, 50]; // 5 elements: 10, 20, 30, 40, 50
    let mut deserializer = binary_like::BinaryDeserializer::new(data);

    let result: Vec<u32> = (&mut deserializer)
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    assert_eq!(result, vec![10, 20, 30, 40, 50]);
}

#[test]
fn test_iter_binary_like_empty() {
    // Test deserializing empty sequence from binary format
    let data = vec![0]; // 0 elements
    let mut deserializer = binary_like::BinaryDeserializer::new(data);

    let result: Vec<u32> = (&mut deserializer)
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    assert_eq!(result, Vec::<u32>::new());
}

#[test]
fn test_iter_binary_like_early_termination() {
    // Test early termination with binary format
    let data = vec![10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; // 10 elements
    let mut deserializer = binary_like::BinaryDeserializer::new(data);

    let result: Vec<u32> = (&mut deserializer)
        .deserialize_seq(TakeNVisitor::new(3))
        .unwrap();

    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_iter_binary_like_btreeset() {
    // Test deserializing into BTreeSet from binary format
    let data = vec![6, 3, 1, 4, 1, 5, 9]; // 6 elements with duplicates
    let mut deserializer = binary_like::BinaryDeserializer::new(data);

    let result: BTreeSet<u32> = (&mut deserializer)
        .deserialize_seq(IterCollectVisitor::new())
        .unwrap();

    let expected: BTreeSet<u32> = [1, 3, 4, 5, 9].into_iter().collect();
    assert_eq!(result, expected);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for size_hint usage

struct WithCapacityVisitor<T> {
    _marker: PhantomData<T>,
}

impl<T> WithCapacityVisitor<T> {
    fn new() -> Self {
        WithCapacityVisitor {
            _marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for WithCapacityVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let iter = SeqIter::<A, T>::new(seq);
        // Use size_hint to pre-allocate
        let (lower, _) = iter.size_hint();
        let mut vec = Vec::with_capacity(lower);
        for item in iter {
            vec.push(item?);
        }
        Ok(vec)
    }
}

#[test]
fn test_iter_size_hint_preallocate() {
    // Test that size_hint is available and can be used for pre-allocation
    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data.clone());

    let result: Vec<u32> = deserializer
        .deserialize_seq(WithCapacityVisitor::new())
        .unwrap();

    assert_eq!(result, data);
}

#[test]
fn test_iter_binary_size_hint() {
    // Binary format provides exact size_hint
    let data = vec![5, 10, 20, 30, 40, 50];
    let mut deserializer = binary_like::BinaryDeserializer::new(data);

    let result: Vec<u32> = (&mut deserializer)
        .deserialize_seq(WithCapacityVisitor::new())
        .unwrap();

    assert_eq!(result, vec![10, 20, 30, 40, 50]);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for chained/nested iteration

#[test]
fn test_iter_flatten_nested() {
    // Test deserializing nested sequences and flattening them
    struct FlattenVisitor;

    impl<'de> Visitor<'de> for FlattenVisitor {
        type Value = Vec<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of sequences")
        }

        fn visit_seq<A>(self, outer: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result = Vec::new();
            let iter = SeqIter::<A, Vec<u32>>::new(outer);
            for inner_result in iter {
                let inner = inner_result?;
                result.extend(inner);
            }
            Ok(result)
        }
    }

    // Nested vectors: [[1, 2], [3, 4, 5], [6]]
    let data = vec![vec![1u32, 2], vec![3, 4, 5], vec![6]];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Vec<u32> = deserializer.deserialize_seq(FlattenVisitor).unwrap();

    assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for error handling during iteration

#[test]
fn test_iter_collect_stops_on_error() {
    // When collecting an iterator, the first error should stop iteration
    // This is tested implicitly through the Result<_, _> collection pattern

    struct ErrorOnThirdVisitor;

    impl<'de> Visitor<'de> for ErrorOnThirdVisitor {
        type Value = Vec<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, u32>::new(seq);
            let mut result = Vec::new();
            for (i, item) in iter.enumerate() {
                if i == 2 {
                    return Err(A::Error::custom("simulated error on third element"));
                }
                result.push(item?);
            }
            Ok(result)
        }
    }

    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data);

    let result = deserializer.deserialize_seq(ErrorOnThirdVisitor);

    assert!(result.is_err());
}

//////////////////////////////////////////////////////////////////////////////
// Tests for filter/map transformations during iteration

#[test]
fn test_iter_filter_during_deserialization() {
    // Test filtering elements during deserialization
    struct FilterEvenVisitor;

    impl<'de> Visitor<'de> for FilterEvenVisitor {
        type Value = Vec<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, u32>::new(seq);
            iter.filter_map(|r| match r {
                Ok(v) if v % 2 == 0 => Some(Ok(v)),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect()
        }
    }

    let data = vec![1u32, 2, 3, 4, 5, 6, 7, 8];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Vec<u32> = deserializer.deserialize_seq(FilterEvenVisitor).unwrap();

    assert_eq!(result, vec![2, 4, 6, 8]);
}

#[test]
fn test_iter_map_during_deserialization() {
    // Test mapping elements during deserialization
    struct DoubleVisitor;

    impl<'de> Visitor<'de> for DoubleVisitor {
        type Value = Vec<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, u32>::new(seq);
            iter.map(|r| r.map(|v| v * 2)).collect()
        }
    }

    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Vec<u32> = deserializer.deserialize_seq(DoubleVisitor).unwrap();

    assert_eq!(result, vec![2, 4, 6, 8, 10]);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for take_while pattern (conditional early termination)

#[test]
fn test_iter_take_while() {
    // Test taking elements while a condition is met
    struct TakeWhilePositiveVisitor;

    impl<'de> Visitor<'de> for TakeWhilePositiveVisitor {
        type Value = Vec<i32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, i32>::new(seq);
            let mut result = Vec::new();
            for item in iter {
                let value = item?;
                if value <= 0 {
                    break;
                }
                result.push(value);
            }
            Ok(result)
        }
    }

    let data = vec![5i32, 3, 7, -1, 2, 4];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Vec<i32> = deserializer
        .deserialize_seq(TakeWhilePositiveVisitor)
        .unwrap();

    // Should stop at -1
    assert_eq!(result, vec![5, 3, 7]);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for skip pattern

#[test]
fn test_iter_skip_elements() {
    // Test skipping the first N elements
    struct SkipNVisitor {
        n: usize,
    }

    impl<'de> Visitor<'de> for SkipNVisitor {
        type Value = Vec<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a sequence")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, u32>::new(seq);
            iter.skip(self.n).collect()
        }
    }

    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Vec<u32> = deserializer
        .deserialize_seq(SkipNVisitor { n: 2 })
        .unwrap();

    assert_eq!(result, vec![3, 4, 5]);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for fold/reduce patterns

#[test]
fn test_iter_sum() {
    // Test summing all elements using fold
    struct SumVisitor;

    impl<'de> Visitor<'de> for SumVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of numbers")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, u32>::new(seq);
            let mut sum = 0u32;
            for item in iter {
                sum += item?;
            }
            Ok(sum)
        }
    }

    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: u32 = deserializer.deserialize_seq(SumVisitor).unwrap();

    assert_eq!(result, 15);
}

#[test]
fn test_iter_max() {
    // Test finding max element
    struct MaxVisitor;

    impl<'de> Visitor<'de> for MaxVisitor {
        type Value = Option<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of numbers")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, u32>::new(seq);
            let mut max = None;
            for item in iter {
                let value = item?;
                max = Some(max.map_or(value, |m: u32| m.max(value)));
            }
            Ok(max)
        }
    }

    let data = vec![3u32, 7, 2, 9, 1];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Option<u32> = deserializer.deserialize_seq(MaxVisitor).unwrap();

    assert_eq!(result, Some(9));
}

//////////////////////////////////////////////////////////////////////////////
// Tests for count pattern

#[test]
fn test_iter_count_elements() {
    // Test counting elements without storing them
    struct CountVisitor;

    impl<'de> Visitor<'de> for CountVisitor {
        type Value = usize;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, u32>::new(seq);
            let mut count = 0;
            for item in iter {
                let _ = item?;
                count += 1;
            }
            Ok(count)
        }
    }

    let data = vec![1u32, 2, 3, 4, 5, 6, 7];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: usize = deserializer.deserialize_seq(CountVisitor).unwrap();

    assert_eq!(result, 7);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for any/all patterns

#[test]
fn test_iter_any() {
    // Test checking if any element matches a predicate
    struct AnyGreaterThanVisitor {
        threshold: u32,
    }

    impl<'de> Visitor<'de> for AnyGreaterThanVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of numbers")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, u32>::new(seq);
            for item in iter {
                if item? > self.threshold {
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }

    // Should find element > 5
    let data = vec![1u32, 2, 3, 7, 4];
    let deserializer = json_like::VecDeserializer::new(data);
    let result: bool = deserializer
        .deserialize_seq(AnyGreaterThanVisitor { threshold: 5 })
        .unwrap();
    assert!(result);

    // Should not find element > 10
    let data = vec![1u32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data);
    let result: bool = deserializer
        .deserialize_seq(AnyGreaterThanVisitor { threshold: 10 })
        .unwrap();
    assert!(!result);
}

#[test]
fn test_iter_all() {
    // Test checking if all elements match a predicate
    struct AllPositiveVisitor;

    impl<'de> Visitor<'de> for AllPositiveVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of numbers")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, i32>::new(seq);
            for item in iter {
                if item? <= 0 {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }

    // All positive
    let data = vec![1i32, 2, 3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data);
    let result: bool = deserializer.deserialize_seq(AllPositiveVisitor).unwrap();
    assert!(result);

    // Contains non-positive
    let data = vec![1i32, 2, -3, 4, 5];
    let deserializer = json_like::VecDeserializer::new(data);
    let result: bool = deserializer.deserialize_seq(AllPositiveVisitor).unwrap();
    assert!(!result);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for partition pattern

#[test]
fn test_iter_partition() {
    // Test partitioning elements based on a predicate
    struct PartitionEvenOddVisitor;

    impl<'de> Visitor<'de> for PartitionEvenOddVisitor {
        type Value = (Vec<u32>, Vec<u32>);

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of numbers")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, u32>::new(seq);
            let mut evens = Vec::new();
            let mut odds = Vec::new();
            for item in iter {
                let value = item?;
                if value % 2 == 0 {
                    evens.push(value);
                } else {
                    odds.push(value);
                }
            }
            Ok((evens, odds))
        }
    }

    let data = vec![1u32, 2, 3, 4, 5, 6, 7, 8];
    let deserializer = json_like::VecDeserializer::new(data);

    let (evens, odds): (Vec<u32>, Vec<u32>) = deserializer
        .deserialize_seq(PartitionEvenOddVisitor)
        .unwrap();

    assert_eq!(evens, vec![2, 4, 6, 8]);
    assert_eq!(odds, vec![1, 3, 5, 7]);
}

//////////////////////////////////////////////////////////////////////////////
// Tests for zip pattern (with index)

#[test]
fn test_iter_enumerate() {
    // Test enumerating elements with their indices
    struct EnumerateVisitor;

    impl<'de> Visitor<'de> for EnumerateVisitor {
        type Value = Vec<(usize, String)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of strings")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let iter = SeqIter::<A, String>::new(seq);
            iter.enumerate()
                .map(|(i, r)| r.map(|s| (i, s)))
                .collect()
        }
    }

    let data = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let deserializer = json_like::VecDeserializer::new(data);

    let result: Vec<(usize, String)> = deserializer.deserialize_seq(EnumerateVisitor).unwrap();

    assert_eq!(
        result,
        vec![
            (0, "a".to_string()),
            (1, "b".to_string()),
            (2, "c".to_string())
        ]
    );
}

//////////////////////////////////////////////////////////////////////////////
// Integration test demonstrating the ideal usage pattern

/// This test demonstrates how the `deserialize_iter` API would simplify
/// custom deserialization for types with sequence fields.
#[test]
fn test_iter_ideal_usage_pattern() {
    // Foo contains a Vec<Bar> where Bar has custom deserialization
    #[derive(Debug, PartialEq, Deserialize)]
    struct Bar {
        value: u32,
    }

    // With the iterator pattern, deserializing into a custom collection
    // becomes much simpler - we just use FromIterator
    let data = vec![
        Bar { value: 1 },
        Bar { value: 2 },
        Bar { value: 3 },
    ];

    // Simulate serialized form
    let serialized: Vec<u32> = data.iter().map(|b| b.value).collect();

    struct BarVisitor;

    impl<'de> Visitor<'de> for BarVisitor {
        type Value = Vec<Bar>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of Bar values")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // Using iterator pattern to collect
            let iter = SeqIter::<A, u32>::new(seq);
            iter.map(|r| r.map(|v| Bar { value: v }))
                .collect::<Result<Vec<Bar>, _>>()
        }
    }

    let deserializer = json_like::VecDeserializer::new(serialized);
    let result: Vec<Bar> = deserializer.deserialize_seq(BarVisitor).unwrap();

    assert_eq!(result, data);
}
