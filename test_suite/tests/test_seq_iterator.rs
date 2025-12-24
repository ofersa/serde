//! Minimal tests for SeqIterator struct.

#![allow(clippy::uninlined_format_args)]

use serde::de::{SeqAccess, SeqIterator, Visitor};
use std::fmt;

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

struct I32SeqAccess {
    elements: Vec<i32>,
    index: usize,
}

impl I32SeqAccess {
    fn new(elements: Vec<i32>) -> Self {
        I32SeqAccess { elements, index: 0 }
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

#[test]
fn test_seq_iterator_collect() {
    let seq = I32SeqAccess::new(vec![1, 2, 3]);
    let iter = SeqIterator::<i32, _>::new(seq);
    let values: Vec<i32> = iter.collect::<Result<_, _>>().unwrap();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn test_seq_iterator_size_hint() {
    let seq = I32SeqAccess::new(vec![1, 2, 3]);
    let iter = SeqIterator::<i32, _>::new(seq);
    assert_eq!(iter.size_hint(), (3, Some(3)));
}