//! Tests for forward_to_deserialize_any! macro support for deserialize_iter.

use serde::de::value::{Error, SeqDeserializer};
use serde::de::{Deserialize, Deserializer, Visitor};
use serde::forward_to_deserialize_any;

/// A deserializer that forwards all methods to deserialize_any,
/// including the new deserialize_iter method.
struct ForwardingDeserializer<I>(SeqDeserializer<I, Error>);

impl<'de, I, T> Deserializer<'de> for ForwardingDeserializer<I>
where
    I: Iterator<Item = T>,
    T: serde::de::IntoDeserializer<'de, Error>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_any(visitor)
    }

    // Forward all methods including the new iter method
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any iter
    }
}

#[test]
fn test_forward_deserialize_iter_to_any() {
    // Create a deserializer for a sequence of i32s
    let inner = SeqDeserializer::<_, Error>::new(vec![1i32, 2, 3].into_iter());
    let deserializer = ForwardingDeserializer(inner);

    // Use deserialize_iter - it should forward to deserialize_any
    let iter = deserializer.deserialize_iter::<i32>().unwrap();
    let result: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_forward_deserialize_iter_empty() {
    let inner = SeqDeserializer::<_, Error>::new(Vec::<i32>::new().into_iter());
    let deserializer = ForwardingDeserializer(inner);

    let iter = deserializer.deserialize_iter::<i32>().unwrap();
    let result: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(result.unwrap(), Vec::<i32>::new());
}

#[test]
fn test_forward_deserialize_iter_with_custom_lifetime() {
    // Test that the macro works with custom lifetime parameters
    struct CustomLifetimeDeserializer<'q, I>(SeqDeserializer<I, Error>, std::marker::PhantomData<&'q ()>);

    impl<'q, I, T> Deserializer<'q> for CustomLifetimeDeserializer<'q, I>
    where
        I: Iterator<Item = T>,
        T: serde::de::IntoDeserializer<'q, Error>,
    {
        type Error = Error;

        fn deserialize_any<W>(self, visitor: W) -> Result<W::Value, Self::Error>
        where
            W: Visitor<'q>,
        {
            self.0.deserialize_any(visitor)
        }

        // Use custom lifetime and visitor type parameter syntax
        forward_to_deserialize_any! {
            <W: Visitor<'q>>
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any iter
        }
    }

    let inner = SeqDeserializer::<_, Error>::new(vec![10i32, 20].into_iter());
    let deserializer = CustomLifetimeDeserializer(inner, std::marker::PhantomData);

    let iter = deserializer.deserialize_iter::<i32>().unwrap();
    let result: Result<Vec<i32>, _> = iter.collect();
    assert_eq!(result.unwrap(), vec![10, 20]);
}

#[test]
fn test_forward_deserialize_iter_with_strings() {
    // Test that it works with String types to ensure proper deserialization
    let inner = SeqDeserializer::<_, Error>::new(vec!["hello", "world"].into_iter());
    let deserializer = ForwardingDeserializer(inner);

    let iter = deserializer.deserialize_iter::<String>().unwrap();
    let result: Result<Vec<String>, _> = iter.collect();
    assert_eq!(
        result.unwrap(),
        vec!["hello".to_string(), "world".to_string()]
    );
}
