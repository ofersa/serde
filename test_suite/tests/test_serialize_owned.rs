//! Tests for the `#[serde(serialize_owned)]` attribute and `SerializeOwned` derive.
//!
//! This module tests that serde_derive correctly generates `SerializeOwned`
//! implementations when the `serialize_owned` attribute is present on a type.

#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::items_after_statements,
    dead_code
)]

use serde::ser::SerializeOwned;
use serde_derive::Serialize;
use serde_test::{assert_ser_tokens, Token};

//////////////////////////////////////////////////////////////////////////
// Basic struct tests
//////////////////////////////////////////////////////////////////////////

/// Test that a simple struct with `serialize_owned` attribute compiles
/// and generates both Serialize and SerializeOwned implementations.
#[test]
fn test_serialize_owned_simple_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct SimpleStruct {
        a: i32,
        b: String,
    }

    // Verify Serialize works
    let s = SimpleStruct {
        a: 42,
        b: "hello".to_string(),
    };
    assert_ser_tokens(
        &s,
        &[
            Token::Struct {
                name: "SimpleStruct",
                len: 2,
            },
            Token::Str("a"),
            Token::I32(42),
            Token::Str("b"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );

    // Verify that SerializeOwned is implemented (compile-time check)
    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<SimpleStruct>();
}

/// Test unit struct with serialize_owned attribute.
#[test]
fn test_serialize_owned_unit_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct UnitStruct;

    assert_ser_tokens(&UnitStruct, &[Token::UnitStruct { name: "UnitStruct" }]);

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<UnitStruct>();
}

/// Test tuple struct with serialize_owned attribute.
#[test]
fn test_serialize_owned_tuple_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct TupleStruct(i32, String, bool);

    let s = TupleStruct(1, "test".to_string(), true);
    assert_ser_tokens(
        &s,
        &[
            Token::TupleStruct {
                name: "TupleStruct",
                len: 3,
            },
            Token::I32(1),
            Token::Str("test"),
            Token::Bool(true),
            Token::TupleStructEnd,
        ],
    );

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<TupleStruct>();
}

/// Test newtype struct with serialize_owned attribute.
#[test]
fn test_serialize_owned_newtype_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct NewtypeStruct(String);

    let s = NewtypeStruct("inner".to_string());
    assert_ser_tokens(
        &s,
        &[
            Token::NewtypeStruct {
                name: "NewtypeStruct",
            },
            Token::Str("inner"),
        ],
    );

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<NewtypeStruct>();
}

//////////////////////////////////////////////////////////////////////////
// Enum tests
//////////////////////////////////////////////////////////////////////////

/// Test enum with serialize_owned attribute.
#[test]
fn test_serialize_owned_enum() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    enum MyEnum {
        Unit,
        Newtype(i32),
        Tuple(i32, i32),
        Struct { a: i32, b: String },
    }

    // Unit variant
    assert_ser_tokens(
        &MyEnum::Unit,
        &[Token::UnitVariant {
            name: "MyEnum",
            variant: "Unit",
        }],
    );

    // Newtype variant
    assert_ser_tokens(
        &MyEnum::Newtype(42),
        &[
            Token::NewtypeVariant {
                name: "MyEnum",
                variant: "Newtype",
            },
            Token::I32(42),
        ],
    );

    // Tuple variant
    assert_ser_tokens(
        &MyEnum::Tuple(1, 2),
        &[
            Token::TupleVariant {
                name: "MyEnum",
                variant: "Tuple",
                len: 2,
            },
            Token::I32(1),
            Token::I32(2),
            Token::TupleVariantEnd,
        ],
    );

    // Struct variant
    assert_ser_tokens(
        &MyEnum::Struct {
            a: 1,
            b: "test".to_string(),
        },
        &[
            Token::StructVariant {
                name: "MyEnum",
                variant: "Struct",
                len: 2,
            },
            Token::Str("a"),
            Token::I32(1),
            Token::Str("b"),
            Token::Str("test"),
            Token::StructVariantEnd,
        ],
    );

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<MyEnum>();
}

//////////////////////////////////////////////////////////////////////////
// Generics tests
//////////////////////////////////////////////////////////////////////////

/// Test generic struct with serialize_owned attribute.
#[test]
fn test_serialize_owned_generic_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct GenericStruct<T> {
        value: T,
    }

    let s = GenericStruct { value: 42i32 };
    assert_ser_tokens(
        &s,
        &[
            Token::Struct {
                name: "GenericStruct",
                len: 1,
            },
            Token::Str("value"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<GenericStruct<i32>>();
    assert_serialize_owned::<GenericStruct<String>>();
}

//////////////////////////////////////////////////////////////////////////
// Combined with other attributes
//////////////////////////////////////////////////////////////////////////

/// Test serialize_owned combined with rename attribute.
#[test]
fn test_serialize_owned_with_rename() {
    #[derive(Serialize)]
    #[serde(serialize_owned, rename = "RenamedStruct")]
    struct OriginalName {
        #[serde(rename = "renamed_field")]
        original_field: i32,
    }

    let s = OriginalName { original_field: 1 };
    assert_ser_tokens(
        &s,
        &[
            Token::Struct {
                name: "RenamedStruct",
                len: 1,
            },
            Token::Str("renamed_field"),
            Token::I32(1),
            Token::StructEnd,
        ],
    );

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<OriginalName>();
}

/// Test serialize_owned combined with skip_serializing.
#[test]
fn test_serialize_owned_with_skip() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct SkipFields {
        included: i32,
        #[serde(skip_serializing)]
        skipped: String,
    }

    let s = SkipFields {
        included: 42,
        skipped: "ignored".to_string(),
    };
    assert_ser_tokens(
        &s,
        &[
            Token::Struct {
                name: "SkipFields",
                len: 1,
            },
            Token::Str("included"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<SkipFields>();
}

/// Test serialize_owned combined with transparent attribute.
#[test]
fn test_serialize_owned_with_transparent() {
    #[derive(Serialize)]
    #[serde(serialize_owned, transparent)]
    struct Wrapper {
        inner: String,
    }

    let s = Wrapper {
        inner: "transparent".to_string(),
    };
    assert_ser_tokens(&s, &[Token::Str("transparent")]);

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<Wrapper>();
}

//////////////////////////////////////////////////////////////////////////
// Attribute presence tests (compile-time verification)
//////////////////////////////////////////////////////////////////////////

/// Verify that without serialize_owned, SerializeOwned is NOT implemented.
/// This is a compile-time check that relies on the fact that we can't call
/// a function requiring SerializeOwned on a type that doesn't implement it.
#[test]
fn test_without_serialize_owned_attribute() {
    #[derive(Serialize)]
    struct NoSerializeOwned {
        value: i32,
    }

    // This type only implements Serialize, not SerializeOwned
    let s = NoSerializeOwned { value: 42 };
    assert_ser_tokens(
        &s,
        &[
            Token::Struct {
                name: "NoSerializeOwned",
                len: 1,
            },
            Token::Str("value"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );

    // Note: We cannot add `assert_serialize_owned::<NoSerializeOwned>();`
    // here because it would fail to compile - which is the expected behavior!
}

//////////////////////////////////////////////////////////////////////////
// Internal tagging tests
//////////////////////////////////////////////////////////////////////////

/// Test serialize_owned with internally tagged enum.
#[test]
fn test_serialize_owned_internally_tagged_enum() {
    #[derive(Serialize)]
    #[serde(serialize_owned, tag = "type")]
    enum InternallyTagged {
        Unit,
        Struct { a: i32 },
    }

    assert_ser_tokens(
        &InternallyTagged::Unit,
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 1,
            },
            Token::Str("type"),
            Token::Str("Unit"),
            Token::StructEnd,
        ],
    );

    assert_ser_tokens(
        &InternallyTagged::Struct { a: 42 },
        &[
            Token::Struct {
                name: "InternallyTagged",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("Struct"),
            Token::Str("a"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<InternallyTagged>();
}

/// Test serialize_owned with adjacently tagged enum.
#[test]
fn test_serialize_owned_adjacently_tagged_enum() {
    #[derive(Serialize)]
    #[serde(serialize_owned, tag = "t", content = "c")]
    enum AdjacentlyTagged {
        Unit,
        Newtype(i32),
    }

    assert_ser_tokens(
        &AdjacentlyTagged::Unit,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 1,
            },
            Token::Str("t"),
            Token::Str("Unit"),
            Token::StructEnd,
        ],
    );

    assert_ser_tokens(
        &AdjacentlyTagged::Newtype(42),
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::Str("Newtype"),
            Token::Str("c"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<AdjacentlyTagged>();
}

/// Test serialize_owned with untagged enum.
#[test]
fn test_serialize_owned_untagged_enum() {
    #[derive(Serialize)]
    #[serde(serialize_owned, untagged)]
    enum Untagged {
        A(i32),
        B(String),
    }

    assert_ser_tokens(&Untagged::A(42), &[Token::I32(42)]);
    assert_ser_tokens(&Untagged::B("test".to_string()), &[Token::Str("test")]);

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<Untagged>();
}

//////////////////////////////////////////////////////////////////////////
// Struct with internal tag
//////////////////////////////////////////////////////////////////////////

/// Test serialize_owned with struct that has an internal tag.
#[test]
fn test_serialize_owned_struct_with_tag() {
    #[derive(Serialize)]
    #[serde(serialize_owned, tag = "type")]
    struct TaggedStruct {
        value: i32,
    }

    let s = TaggedStruct { value: 42 };
    assert_ser_tokens(
        &s,
        &[
            Token::Struct {
                name: "TaggedStruct",
                len: 2,
            },
            Token::Str("type"),
            Token::Str("TaggedStruct"),
            Token::Str("value"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );

    fn assert_serialize_owned<T: SerializeOwned>() {}
    assert_serialize_owned::<TaggedStruct>();
}
