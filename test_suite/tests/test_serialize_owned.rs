//! Minimal compile-time tests for the SerializeOwned trait.

use serde::ser::SerializeOwned;

/// Verify SerializeOwned trait exists and can be used as a bound.
fn assert_serialize_owned<T: SerializeOwned>() {}

/// Verify the blanket impl: &T implements SerializeOwned where T: Serialize.
#[test]
fn test_blanket_impl_compiles() {
    assert_serialize_owned::<&i32>();
    assert_serialize_owned::<&str>();
    assert_serialize_owned::<&bool>();
}
