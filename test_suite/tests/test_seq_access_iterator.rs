//! Tests for the SeqAccessIterator adapter.

#![allow(clippy::needless_pass_by_value)]

use serde::de::value::{Error, SeqDeserializer};
use serde::de::{Deserialize, Deserializer, SeqAccess, SeqAccessIterator, Visitor};
use std::fmt;

/// Helper struct that uses SeqAccessIterator to collect elements
struct CollectWithIterator<T>(Vec<T>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for CollectWithIterator<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CollectVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for CollectVisitor<T> {
            type Value = CollectWithIterator<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let iter = SeqAccessIterator::new(seq);
                let values: Result<Vec<T>, _> = iter.collect();
                Ok(CollectWithIterator(values?))
            }
        }

        deserializer.deserialize_seq(CollectVisitor(std::marker::PhantomData))
    }
}

/// A mock SeqAccess that can produce errors
struct MockSeqAccess {
    items: Vec<i32>,
    index: usize,
    fail_at: Option<usize>,
}

impl MockSeqAccess {
    fn new(items: Vec<i32>) -> Self {
        MockSeqAccess {
            items,
            index: 0,
            fail_at: None,
        }
    }

    fn with_error_at(items: Vec<i32>, fail_at: usize) -> Self {
        MockSeqAccess {
            items,
            index: 0,
            fail_at: Some(fail_at),
        }
    }
}

impl<'de> SeqAccess<'de> for MockSeqAccess {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if let Some(fail_at) = self.fail_at {
            if self.index == fail_at {
                return Err(serde::de::Error::custom("mock error"));
            }
        }

        if self.index < self.items.len() {
            let value = self.items[self.index];
            self.index += 1;
            use serde::de::IntoDeserializer;
            seed.deserialize(value.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.items.len() - self.index)
    }
}

#[test]
fn test_seq_access_iterator_empty() {
    let seq = MockSeqAccess::new(vec![]);
    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(seq);
    let collected: Vec<Result<i32, _>> = iter.collect();
    assert!(collected.is_empty());
}

#[test]
fn test_seq_access_iterator_single_element() {
    let seq = MockSeqAccess::new(vec![42]);
    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(seq);
    let collected: Vec<Result<i32, _>> = iter.collect();
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].as_ref().unwrap(), &42);
}

#[test]
fn test_seq_access_iterator_multiple_elements() {
    let seq = MockSeqAccess::new(vec![1, 2, 3, 4, 5]);
    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(seq);
    let collected: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_seq_access_iterator_size_hint() {
    let seq = MockSeqAccess::new(vec![1, 2, 3]);
    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(seq);

    // Initial size hint should be (3, Some(3))
    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn test_seq_access_iterator_error_propagation() {
    let seq = MockSeqAccess::with_error_at(vec![1, 2, 3], 1);
    let mut iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(seq);

    // First element should succeed
    let first = iter.next();
    assert!(first.is_some());
    assert!(first.unwrap().is_ok());

    // Second element should fail
    let second = iter.next();
    assert!(second.is_some());
    assert!(second.unwrap().is_err());
}

#[test]
fn test_seq_access_iterator_with_seq_deserializer() {
    // Use the real SeqDeserializer from serde::de::value
    let seq = SeqDeserializer::<_, Error>::new(vec![10i32, 20, 30].into_iter());
    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(seq);
    let collected: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(collected.unwrap(), vec![10, 20, 30]);
}

#[test]
fn test_seq_access_iterator_in_visitor() {
    // Test using SeqAccessIterator in a real deserialization scenario
    let deserializer = SeqDeserializer::<_, Error>::new(vec![1i32, 2, 3].into_iter());
    let result: CollectWithIterator<i32> = CollectWithIterator::deserialize(deserializer).unwrap();
    assert_eq!(result.0, vec![1, 2, 3]);
}

#[test]
fn test_seq_access_iterator_size_hint_no_size() {
    // Create a SeqAccess that returns None for size_hint
    struct NoSizeHintSeqAccess {
        items: Vec<i32>,
        index: usize,
    }

    impl<'de> SeqAccess<'de> for NoSizeHintSeqAccess {
        type Error = Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: serde::de::DeserializeSeed<'de>,
        {
            if self.index < self.items.len() {
                let value = self.items[self.index];
                self.index += 1;
                use serde::de::IntoDeserializer;
                seed.deserialize(value.into_deserializer()).map(Some)
            } else {
                Ok(None)
            }
        }

        fn size_hint(&self) -> Option<usize> {
            None
        }
    }

    let seq = NoSizeHintSeqAccess {
        items: vec![1, 2, 3],
        index: 0,
    };
    let iter: SeqAccessIterator<'_, _, i32> = SeqAccessIterator::new(seq);

    // When size_hint returns None, iterator should return (0, None)
    assert_eq!(iter.size_hint(), (0, None));
}
