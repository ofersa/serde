//! Minimal test for `#[serde(serialize_owned)]` attribute.

#![allow(dead_code)]

use serde::ser::SerializeOwned;
use serde_derive::Serialize;

/// Verify that serialize_owned attribute generates SerializeOwned impl.
#[test]
fn test_serialize_owned_attribute() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct TestStruct {
        value: i32,
    }

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<TestStruct>();
}
