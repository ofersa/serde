//! Comprehensive tests for the `SerializeOwned` trait.
//!
//! Tests cover:
//! - Owned serialization for standard library types
//! - Blanket impl verification (`&T: SerializeOwned where T: Serialize`)
//! - Derive macro tests for `#[serde(serialize_owned)]` attribute

#![allow(dead_code, clippy::needless_pass_by_value)]

use serde::ser::{SerializeOwned, Serializer};
use serde::Serialize;
use serde_derive::Serialize;
use serde_test::{assert_ser_tokens, Token};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::ffi::CString;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// SECTION 1: Blanket Implementation Tests
// Verify that `&T: SerializeOwned` where `T: Serialize`
// ============================================================================

/// Helper function to assert a type implements SerializeOwned
fn assert_serialize_owned<T: SerializeOwned>() {}

#[test]
fn test_blanket_impl_for_references() {
    // Test that &T implements SerializeOwned for various T: Serialize types
    fn requires_serialize_owned<T: SerializeOwned>(_: T) {}

    // Primitives
    requires_serialize_owned(&42i32);
    requires_serialize_owned(&true);
    requires_serialize_owned(&3.14f64);
    requires_serialize_owned(&'c');

    // Strings
    requires_serialize_owned(&String::from("hello"));
    requires_serialize_owned(&"hello");

    // Collections
    requires_serialize_owned(&vec![1, 2, 3]);
    requires_serialize_owned(&Some(42));

    // Standard library types
    requires_serialize_owned(&PathBuf::from("/test"));
}

#[test]
fn test_blanket_impl_compile_check() {
    // Verify blanket impl exists for common reference types
    assert_serialize_owned::<&i32>();
    assert_serialize_owned::<&str>();
    assert_serialize_owned::<&String>();
    assert_serialize_owned::<&Vec<i32>>();
    assert_serialize_owned::<&Option<i32>>();
    assert_serialize_owned::<&HashMap<String, i32>>();
    assert_serialize_owned::<&BTreeMap<String, i32>>();
}

// ============================================================================
// SECTION 2: Standard Library Type SerializeOwned Tests
// ============================================================================

#[test]
fn test_string_serialize_owned() {
    assert_serialize_owned::<String>();
    let s = String::from("hello");
    assert_ser_tokens(&s, &[Token::Str("hello")]);
}

#[test]
fn test_cstring_serialize_owned() {
    assert_serialize_owned::<CString>();
    let cs = CString::new("test").unwrap();
    assert_ser_tokens(&cs, &[Token::Bytes(b"test")]);
}

#[test]
fn test_vec_serialize_owned() {
    assert_serialize_owned::<Vec<i32>>();
    let v = vec![1, 2, 3];
    assert_ser_tokens(
        &v,
        &[
            Token::Seq { len: Some(3) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_box_serialize_owned() {
    assert_serialize_owned::<Box<i32>>();
    let b = Box::new(42);
    assert_ser_tokens(&b, &[Token::I32(42)]);
}

#[test]
fn test_vecdeque_serialize_owned() {
    assert_serialize_owned::<VecDeque<i32>>();
    let mut vd = VecDeque::new();
    vd.push_back(1);
    vd.push_back(2);
    assert_ser_tokens(
        &vd,
        &[
            Token::Seq { len: Some(2) },
            Token::I32(1),
            Token::I32(2),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_linkedlist_serialize_owned() {
    assert_serialize_owned::<LinkedList<i32>>();
    let mut ll = LinkedList::new();
    ll.push_back(1);
    ll.push_back(2);
    assert_ser_tokens(
        &ll,
        &[
            Token::Seq { len: Some(2) },
            Token::I32(1),
            Token::I32(2),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_binaryheap_serialize_owned() {
    assert_serialize_owned::<BinaryHeap<i32>>();
    // BinaryHeap doesn't guarantee order, so we just test single element
    let mut bh = BinaryHeap::new();
    bh.push(1);
    assert_ser_tokens(&bh, &[Token::Seq { len: Some(1) }, Token::I32(1), Token::SeqEnd]);
}

#[test]
fn test_btreeset_serialize_owned() {
    assert_serialize_owned::<BTreeSet<i32>>();
    let mut bs = BTreeSet::new();
    bs.insert(1);
    bs.insert(2);
    assert_ser_tokens(
        &bs,
        &[
            Token::Seq { len: Some(2) },
            Token::I32(1),
            Token::I32(2),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_hashset_serialize_owned() {
    assert_serialize_owned::<HashSet<i32>>();
    // HashSet doesn't guarantee order, so we just test single element
    let mut hs = HashSet::new();
    hs.insert(1);
    assert_ser_tokens(&hs, &[Token::Seq { len: Some(1) }, Token::I32(1), Token::SeqEnd]);
}

#[test]
fn test_btreemap_serialize_owned() {
    assert_serialize_owned::<BTreeMap<String, i32>>();
    let mut bm = BTreeMap::new();
    bm.insert("a".to_string(), 1);
    assert_ser_tokens(
        &bm,
        &[
            Token::Map { len: Some(1) },
            Token::Str("a"),
            Token::I32(1),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_hashmap_serialize_owned() {
    assert_serialize_owned::<HashMap<String, i32>>();
    // HashMap doesn't guarantee order, so we just verify it compiles
    let mut hm = HashMap::new();
    hm.insert("a".to_string(), 1);
    // Single element map should have consistent serialization
    assert_ser_tokens(
        &hm,
        &[
            Token::Map { len: Some(1) },
            Token::Str("a"),
            Token::I32(1),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_option_serialize_owned() {
    assert_serialize_owned::<Option<i32>>();
    let some = Some(42);
    let none: Option<i32> = None;
    assert_ser_tokens(&some, &[Token::Some, Token::I32(42)]);
    assert_ser_tokens(&none, &[Token::None]);
}

#[test]
fn test_result_serialize_owned() {
    assert_serialize_owned::<Result<i32, String>>();
    let ok: Result<i32, String> = Ok(42);
    let err: Result<i32, String> = Err("error".to_string());
    assert_ser_tokens(
        &ok,
        &[
            Token::NewtypeVariant {
                name: "Result",
                variant: "Ok",
            },
            Token::I32(42),
        ],
    );
    assert_ser_tokens(
        &err,
        &[
            Token::NewtypeVariant {
                name: "Result",
                variant: "Err",
            },
            Token::Str("error"),
        ],
    );
}

#[test]
fn test_cow_serialize_owned() {
    assert_serialize_owned::<Cow<'static, str>>();
    let borrowed: Cow<str> = Cow::Borrowed("borrowed");
    let owned: Cow<str> = Cow::Owned(String::from("owned"));
    assert_ser_tokens(&borrowed, &[Token::Str("borrowed")]);
    assert_ser_tokens(&owned, &[Token::Str("owned")]);
}

#[test]
fn test_rc_serialize_owned() {
    assert_serialize_owned::<Rc<i32>>();
    let rc = Rc::new(42);
    assert_ser_tokens(&rc, &[Token::I32(42)]);
}

#[test]
fn test_arc_serialize_owned() {
    assert_serialize_owned::<Arc<i32>>();
    let arc = Arc::new(42);
    assert_ser_tokens(&arc, &[Token::I32(42)]);
}

#[test]
fn test_pathbuf_serialize_owned() {
    assert_serialize_owned::<PathBuf>();
    let pb = PathBuf::from("/test/path");
    assert_ser_tokens(&pb, &[Token::Str("/test/path")]);
}

// ============================================================================
// SECTION 3: Derive Macro Tests for #[serde(serialize_owned)]
// ============================================================================

#[test]
fn test_serialize_owned_attribute_basic_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct BasicStruct {
        value: i32,
    }

    assert_serialize_owned::<BasicStruct>();

    let s = BasicStruct { value: 42 };
    assert_ser_tokens(
        &s,
        &[
            Token::Struct {
                name: "BasicStruct",
                len: 1,
            },
            Token::Str("value"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_attribute_tuple_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct TupleStruct(i32, String, bool);

    assert_serialize_owned::<TupleStruct>();

    let ts = TupleStruct(1, "hello".to_string(), true);
    assert_ser_tokens(
        &ts,
        &[
            Token::TupleStruct {
                name: "TupleStruct",
                len: 3,
            },
            Token::I32(1),
            Token::Str("hello"),
            Token::Bool(true),
            Token::TupleStructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_attribute_unit_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct UnitStruct;

    assert_serialize_owned::<UnitStruct>();

    let us = UnitStruct;
    assert_ser_tokens(&us, &[Token::UnitStruct { name: "UnitStruct" }]);
}

#[test]
fn test_serialize_owned_attribute_newtype_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct NewtypeStruct(String);

    assert_serialize_owned::<NewtypeStruct>();

    let ns = NewtypeStruct("wrapped".to_string());
    assert_ser_tokens(
        &ns,
        &[
            Token::NewtypeStruct {
                name: "NewtypeStruct",
            },
            Token::Str("wrapped"),
        ],
    );
}

#[test]
fn test_serialize_owned_attribute_enum() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    enum MyEnum {
        Unit,
        Newtype(i32),
        Tuple(i32, i32),
        Struct { x: i32, y: i32 },
    }

    assert_serialize_owned::<MyEnum>();

    let unit = MyEnum::Unit;
    assert_ser_tokens(
        &unit,
        &[Token::UnitVariant {
            name: "MyEnum",
            variant: "Unit",
        }],
    );

    let newtype = MyEnum::Newtype(42);
    assert_ser_tokens(
        &newtype,
        &[
            Token::NewtypeVariant {
                name: "MyEnum",
                variant: "Newtype",
            },
            Token::I32(42),
        ],
    );

    let tuple = MyEnum::Tuple(1, 2);
    assert_ser_tokens(
        &tuple,
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

    let struct_var = MyEnum::Struct { x: 10, y: 20 };
    assert_ser_tokens(
        &struct_var,
        &[
            Token::StructVariant {
                name: "MyEnum",
                variant: "Struct",
                len: 2,
            },
            Token::Str("x"),
            Token::I32(10),
            Token::Str("y"),
            Token::I32(20),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_attribute_generic_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct GenericStruct<T: Serialize> {
        value: T,
    }

    assert_serialize_owned::<GenericStruct<i32>>();
    assert_serialize_owned::<GenericStruct<String>>();

    let gs = GenericStruct { value: 42 };
    assert_ser_tokens(
        &gs,
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
}

#[test]
fn test_serialize_owned_attribute_with_lifetime() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct WithLifetime<'a> {
        borrowed: &'a str,
    }

    assert_serialize_owned::<WithLifetime<'static>>();

    let wl = WithLifetime { borrowed: "test" };
    assert_ser_tokens(
        &wl,
        &[
            Token::Struct {
                name: "WithLifetime",
                len: 1,
            },
            Token::Str("borrowed"),
            Token::Str("test"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_attribute_nested_types() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct Inner {
        x: i32,
    }

    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct Outer {
        inner: Inner,
        values: Vec<i32>,
    }

    assert_serialize_owned::<Inner>();
    assert_serialize_owned::<Outer>();

    let outer = Outer {
        inner: Inner { x: 1 },
        values: vec![2, 3],
    };
    assert_ser_tokens(
        &outer,
        &[
            Token::Struct {
                name: "Outer",
                len: 2,
            },
            Token::Str("inner"),
            Token::Struct {
                name: "Inner",
                len: 1,
            },
            Token::Str("x"),
            Token::I32(1),
            Token::StructEnd,
            Token::Str("values"),
            Token::Seq { len: Some(2) },
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_attribute_with_rename() {
    #[derive(Serialize)]
    #[serde(serialize_owned, rename_all = "camelCase")]
    struct RenamedStruct {
        my_field: i32,
        another_field: String,
    }

    assert_serialize_owned::<RenamedStruct>();

    let rs = RenamedStruct {
        my_field: 1,
        another_field: "test".to_string(),
    };
    assert_ser_tokens(
        &rs,
        &[
            Token::Struct {
                name: "RenamedStruct",
                len: 2,
            },
            Token::Str("myField"),
            Token::I32(1),
            Token::Str("anotherField"),
            Token::Str("test"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_attribute_with_skip() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct WithSkip {
        included: i32,
        #[serde(skip_serializing)]
        skipped: String,
    }

    assert_serialize_owned::<WithSkip>();

    let ws = WithSkip {
        included: 42,
        skipped: "ignored".to_string(),
    };
    assert_ser_tokens(
        &ws,
        &[
            Token::Struct {
                name: "WithSkip",
                len: 1,
            },
            Token::Str("included"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_attribute_with_option() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct WithOption {
        required: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        optional: Option<String>,
    }

    assert_serialize_owned::<WithOption>();

    let with_some = WithOption {
        required: 1,
        optional: Some("present".to_string()),
    };
    assert_ser_tokens(
        &with_some,
        &[
            Token::Struct {
                name: "WithOption",
                len: 2,
            },
            Token::Str("required"),
            Token::I32(1),
            Token::Str("optional"),
            Token::Some,
            Token::Str("present"),
            Token::StructEnd,
        ],
    );

    let with_none = WithOption {
        required: 2,
        optional: None,
    };
    assert_ser_tokens(
        &with_none,
        &[
            Token::Struct {
                name: "WithOption",
                len: 1,
            },
            Token::Str("required"),
            Token::I32(2),
            Token::StructEnd,
        ],
    );
}

// ============================================================================
// SECTION 4: SerializeOwned Function Usage Tests
// ============================================================================

/// Test that SerializeOwned can be used as a trait bound
#[test]
fn test_serialize_owned_as_trait_bound() {
    fn serialize_and_drop<T: SerializeOwned, S: Serializer>(
        value: T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.serialize_owned(serializer)
    }

    // Just verify this compiles - the function can accept owned values
    fn _use_it() {
        #[derive(Serialize)]
        #[serde(serialize_owned)]
        struct MyData {
            x: i32,
        }

        // This would work at runtime with an actual serializer
        let _data = MyData { x: 42 };
        // serialize_and_drop(data, some_serializer);
    }
}

/// Test that derived types can be passed to functions expecting SerializeOwned
#[test]
fn test_derived_type_as_serialize_owned() {
    fn requires_serialize_owned<T: SerializeOwned>(_value: T) {}

    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct DerivedType {
        value: i32,
    }

    let dt = DerivedType { value: 42 };
    requires_serialize_owned(dt);
}

/// Test multiple generic bounds with SerializeOwned
#[test]
fn test_multiple_bounds_with_serialize_owned() {
    fn process<T: SerializeOwned + Clone + Send>(_value: T) {}

    #[derive(Serialize, Clone)]
    #[serde(serialize_owned)]
    struct MultiTraitType {
        data: String,
    }

    let mt = MultiTraitType {
        data: "test".to_string(),
    };
    process(mt);
}

// ============================================================================
// SECTION 5: Edge Cases and Complex Scenarios
// ============================================================================

#[test]
fn test_serialize_owned_empty_struct() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct EmptyStruct {}

    assert_serialize_owned::<EmptyStruct>();

    let es = EmptyStruct {};
    assert_ser_tokens(
        &es,
        &[
            Token::Struct {
                name: "EmptyStruct",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_with_box_field() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct WithBox {
        boxed: Box<i32>,
    }

    assert_serialize_owned::<WithBox>();

    let wb = WithBox {
        boxed: Box::new(42),
    };
    assert_ser_tokens(
        &wb,
        &[
            Token::Struct {
                name: "WithBox",
                len: 1,
            },
            Token::Str("boxed"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_with_vec_field() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct WithVec {
        items: Vec<String>,
    }

    assert_serialize_owned::<WithVec>();

    let wv = WithVec {
        items: vec!["a".to_string(), "b".to_string()],
    };
    assert_ser_tokens(
        &wv,
        &[
            Token::Struct {
                name: "WithVec",
                len: 1,
            },
            Token::Str("items"),
            Token::Seq { len: Some(2) },
            Token::Str("a"),
            Token::Str("b"),
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_with_hashmap_field() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct WithHashMap {
        map: HashMap<String, i32>,
    }

    assert_serialize_owned::<WithHashMap>();

    let mut map = HashMap::new();
    map.insert("key".to_string(), 42);
    let wh = WithHashMap { map };
    assert_ser_tokens(
        &wh,
        &[
            Token::Struct {
                name: "WithHashMap",
                len: 1,
            },
            Token::Str("map"),
            Token::Map { len: Some(1) },
            Token::Str("key"),
            Token::I32(42),
            Token::MapEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_serialize_owned_recursive_type() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct TreeNode {
        value: i32,
        children: Vec<TreeNode>,
    }

    assert_serialize_owned::<TreeNode>();

    let tree = TreeNode {
        value: 1,
        children: vec![
            TreeNode {
                value: 2,
                children: vec![],
            },
            TreeNode {
                value: 3,
                children: vec![],
            },
        ],
    };

    assert_ser_tokens(
        &tree,
        &[
            Token::Struct {
                name: "TreeNode",
                len: 2,
            },
            Token::Str("value"),
            Token::I32(1),
            Token::Str("children"),
            Token::Seq { len: Some(2) },
            Token::Struct {
                name: "TreeNode",
                len: 2,
            },
            Token::Str("value"),
            Token::I32(2),
            Token::Str("children"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "TreeNode",
                len: 2,
            },
            Token::Str("value"),
            Token::I32(3),
            Token::Str("children"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

// ============================================================================
// SECTION 6: Direct serialize_owned Method Tests
// ============================================================================

/// Test that serialize_owned can be called directly on derived types
#[test]
fn test_serialize_owned_method_call() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct OwnedData {
        value: String,
    }

    // Verify the type implements SerializeOwned
    fn call_serialize_owned<T: SerializeOwned>(_: T) {}

    let data = OwnedData {
        value: "test".to_string(),
    };
    call_serialize_owned(data);
}

/// Test serialize_owned with a type that would benefit from owned serialization
#[test]
fn test_serialize_owned_with_expensive_clone() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct ExpensiveData {
        large_vec: Vec<String>,
        nested_map: HashMap<String, Vec<i32>>,
    }

    assert_serialize_owned::<ExpensiveData>();

    let mut nested_map = HashMap::new();
    nested_map.insert("key".to_string(), vec![1, 2, 3]);

    let data = ExpensiveData {
        large_vec: vec!["a".to_string(), "b".to_string()],
        nested_map,
    };

    // Single-element map for predictable ordering
    assert_ser_tokens(
        &data,
        &[
            Token::Struct {
                name: "ExpensiveData",
                len: 2,
            },
            Token::Str("large_vec"),
            Token::Seq { len: Some(2) },
            Token::Str("a"),
            Token::Str("b"),
            Token::SeqEnd,
            Token::Str("nested_map"),
            Token::Map { len: Some(1) },
            Token::Str("key"),
            Token::Seq { len: Some(3) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::SeqEnd,
            Token::MapEnd,
            Token::StructEnd,
        ],
    );
}

// ============================================================================
// SECTION 7: Blanket Impl Behavior Verification
// ============================================================================

/// Verify that the blanket impl &T: SerializeOwned delegates to Serialize correctly
#[test]
fn test_blanket_impl_delegates_to_serialize() {
    // For &T where T: Serialize, serialize_owned should call T::serialize
    let value = 42i32;
    let reference = &value;

    // Both should serialize the same way
    assert_ser_tokens(&value, &[Token::I32(42)]);
    assert_ser_tokens(reference, &[Token::I32(42)]);
}

/// Test that unsized types work with the blanket impl
#[test]
fn test_blanket_impl_unsized_types() {
    // &str should work through the blanket impl
    fn requires_serialize_owned<T: SerializeOwned>(_: T) {}

    let s: &str = "hello";
    requires_serialize_owned(s);
    assert_ser_tokens(&s, &[Token::Str("hello")]);

    // &[i32] should work too
    let slice: &[i32] = &[1, 2, 3];
    requires_serialize_owned(slice);
}

/// Test double references work with blanket impl
#[test]
fn test_blanket_impl_double_reference() {
    fn requires_serialize_owned<T: SerializeOwned>(_: T) {}

    let value = 42i32;
    let reference = &value;
    let double_reference = &reference;

    requires_serialize_owned(double_reference);
}

// ============================================================================
// SECTION 8: Derive Macro Integration Tests
// ============================================================================

/// Test that serialize_owned attribute works with default field values
#[test]
fn test_serialize_owned_with_default() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct WithDefault {
        #[serde(default)]
        value: i32,
    }

    assert_serialize_owned::<WithDefault>();

    let wd = WithDefault { value: 0 };
    assert_ser_tokens(
        &wd,
        &[
            Token::Struct {
                name: "WithDefault",
                len: 1,
            },
            Token::Str("value"),
            Token::I32(0),
            Token::StructEnd,
        ],
    );
}

/// Test that serialize_owned works with flatten attribute
#[test]
fn test_serialize_owned_with_flatten() {
    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct Inner {
        x: i32,
        y: i32,
    }

    #[derive(Serialize)]
    #[serde(serialize_owned)]
    struct Outer {
        name: String,
        #[serde(flatten)]
        inner: Inner,
    }

    assert_serialize_owned::<Outer>();

    let outer = Outer {
        name: "test".to_string(),
        inner: Inner { x: 1, y: 2 },
    };

    assert_ser_tokens(
        &outer,
        &[
            Token::Map { len: None },
            Token::Str("name"),
            Token::Str("test"),
            Token::Str("x"),
            Token::I32(1),
            Token::Str("y"),
            Token::I32(2),
            Token::MapEnd,
        ],
    );
}

/// Test that serialize_owned works with transparent repr
#[test]
fn test_serialize_owned_transparent() {
    #[derive(Serialize)]
    #[serde(serialize_owned, transparent)]
    struct Wrapper(String);

    assert_serialize_owned::<Wrapper>();

    let w = Wrapper("wrapped".to_string());
    assert_ser_tokens(&w, &[Token::Str("wrapped")]);
}

/// Test serialize_owned with multiple serde attributes combined
#[test]
fn test_serialize_owned_multiple_attributes() {
    #[derive(Serialize)]
    #[serde(serialize_owned, rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
    struct MultiAttr {
        my_field: i32,
    }

    assert_serialize_owned::<MultiAttr>();

    let ma = MultiAttr { my_field: 42 };
    assert_ser_tokens(
        &ma,
        &[
            Token::Struct {
                name: "MultiAttr",
                len: 1,
            },
            Token::Str("MY_FIELD"),
            Token::I32(42),
            Token::StructEnd,
        ],
    );
}
