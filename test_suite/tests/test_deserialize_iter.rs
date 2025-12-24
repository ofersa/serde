//! Tests for the `Deserializer::deserialize_iter` method and `DeserializeIter` type.

#![allow(clippy::needless_pass_by_value)]

use serde::de::value::SeqDeserializer;
use serde::de::{Deserialize, Deserializer, IntoDeserializer};

/// Test basic usage of deserialize_iter with primitive types.
#[test]
fn test_deserialize_iter_primitives() {
    let data = vec![1i32, 2, 3, 4, 5];
    let deserializer: SeqDeserializer<_, serde::de::value::Error> = data.into_deserializer();

    let iter = deserializer.deserialize_iter::<i32>().unwrap();
    let collected: Vec<i32> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(collected, vec![1, 2, 3, 4, 5]);
}

/// Test deserialize_iter with an empty sequence.
#[test]
fn test_deserialize_iter_empty() {
    let data: Vec<i32> = vec![];
    let deserializer: SeqDeserializer<_, serde::de::value::Error> = data.into_deserializer();

    let iter = deserializer.deserialize_iter::<i32>().unwrap();
    let collected: Vec<i32> = iter.collect::<Result<_, _>>().unwrap();

    assert!(collected.is_empty());
}

/// Test deserialize_iter with unsigned integers.
#[test]
fn test_deserialize_iter_unsigned() {
    let data = vec![10u32, 20, 30];
    let deserializer: SeqDeserializer<_, serde::de::value::Error> = data.into_deserializer();

    let iter = deserializer.deserialize_iter::<u32>().unwrap();
    let collected: Vec<u32> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(collected, vec![10, 20, 30]);
}

/// Test that deserialize_iter's size_hint is correct.
#[test]
fn test_deserialize_iter_size_hint() {
    let data = vec![1i32, 2, 3];
    let deserializer: SeqDeserializer<_, serde::de::value::Error> = data.into_deserializer();

    let iter = deserializer.deserialize_iter::<i32>().unwrap();

    // Size hint should reflect remaining elements
    assert_eq!(iter.size_hint(), (3, Some(3)));
}

/// Test that iterating updates the size_hint.
#[test]
fn test_deserialize_iter_size_hint_updates() {
    let data = vec![1i32, 2, 3];
    let deserializer: SeqDeserializer<_, serde::de::value::Error> = data.into_deserializer();

    let mut iter = deserializer.deserialize_iter::<i32>().unwrap();

    assert_eq!(iter.size_hint(), (3, Some(3)));

    let _ = iter.next();
    assert_eq!(iter.size_hint(), (2, Some(2)));

    let _ = iter.next();
    assert_eq!(iter.size_hint(), (1, Some(1)));

    let _ = iter.next();
    assert_eq!(iter.size_hint(), (0, Some(0)));

    // After exhausting, should remain at zero
    let _ = iter.next();
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

/// Test deserialize_iter works in a custom Deserialize implementation.
#[test]
fn test_deserialize_iter_in_custom_impl() {
    struct MyCollection(Vec<i32>);

    impl<'de> Deserialize<'de> for MyCollection {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let items: Vec<i32> = deserializer.deserialize_iter()?.collect::<Result<_, _>>()?;
            Ok(MyCollection(items))
        }
    }

    let data = vec![100i32, 200, 300];
    let deserializer: SeqDeserializer<_, serde::de::value::Error> = data.into_deserializer();

    let collection = MyCollection::deserialize(deserializer).unwrap();
    assert_eq!(collection.0, vec![100, 200, 300]);
}

/// Test deserialize_iter with string data.
#[test]
fn test_deserialize_iter_strings() {
    let data = vec!["hello".to_string(), "world".to_string()];
    let deserializer: SeqDeserializer<_, serde::de::value::Error> = data.into_deserializer();

    let iter = deserializer.deserialize_iter::<String>().unwrap();
    let collected: Vec<String> = iter.collect::<Result<_, _>>().unwrap();

    assert_eq!(collected, vec!["hello".to_string(), "world".to_string()]);
}

/// Test that partial iteration works correctly.
#[test]
fn test_deserialize_iter_partial() {
    let data = vec![1i32, 2, 3, 4, 5];
    let deserializer: SeqDeserializer<_, serde::de::value::Error> = data.into_deserializer();

    let mut iter = deserializer.deserialize_iter::<i32>().unwrap();

    // Only consume first two elements
    assert_eq!(iter.next().unwrap().unwrap(), 1);
    assert_eq!(iter.next().unwrap().unwrap(), 2);

    // Iterator should still have 3 more elements
    assert_eq!(iter.size_hint(), (3, Some(3)));
}

/// Test deserialize_iter with take adapter.
#[test]
fn test_deserialize_iter_with_take() {
    let data = vec![1i32, 2, 3, 4, 5];
    let deserializer: SeqDeserializer<_, serde::de::value::Error> = data.into_deserializer();

    let iter = deserializer.deserialize_iter::<i32>().unwrap();
    let collected: Vec<i32> = iter.take(3).collect::<Result<_, _>>().unwrap();

    assert_eq!(collected, vec![1, 2, 3]);
}
