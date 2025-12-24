//! Minimal test for SerializeOwned trait on standard library types.

use serde::ser::SerializeOwned;

/// Verify SerializeOwned is implemented for common types.
#[test]
fn test_serialize_owned_trait_bounds() {
    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<String>();
    assert_serialize_owned::<Box<i32>>();
    assert_serialize_owned::<Option<i32>>();
}
