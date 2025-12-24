//! Tests for SerializeOwned implementations on standard library types.
//!
//! These tests verify that the SerializeOwned trait is correctly implemented
//! for various standard library types (String, Vec, Box, etc.).

use serde::ser::{
    Serialize, SerializeMap, SerializeOwned, SerializeSeq, SerializeTuple, Serializer,
};
use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::ffi::CString;
use std::num::Wrapping;
use std::ops::Bound;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

/// A simple test serializer that captures the serialized output as a string.
/// Used to verify SerializeOwned implementations produce the same output as Serialize.
#[derive(Default)]
struct TestSerializer;

#[derive(Debug)]
struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

impl serde::ser::Error for TestError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        TestError(msg.to_string())
    }
}

impl Serializer for TestSerializer {
    type Ok = String;
    type Error = TestError;
    type SerializeSeq = TestSeqSerializer;
    type SerializeTuple = TestSeqSerializer;
    type SerializeTupleStruct = TestSeqSerializer;
    type SerializeTupleVariant = TestSeqSerializer;
    type SerializeMap = TestMapSerializer;
    type SerializeStruct = TestMapSerializer;
    type SerializeStructVariant = TestMapSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(format!("bool:{}", v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(format!("i8:{}", v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(format!("i16:{}", v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(format!("i32:{}", v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(format!("i64:{}", v))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(format!("i128:{}", v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(format!("u8:{}", v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(format!("u16:{}", v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(format!("u32:{}", v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(format!("u64:{}", v))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(format!("u128:{}", v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(format!("f32:{}", v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(format!("f64:{}", v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(format!("char:{}", v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(format!("str:{}", v))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(format!("bytes:{:?}", v))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok("none".to_string())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        let inner = value.serialize(TestSerializer::default())?;
        Ok(format!("some:{}", inner))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok("unit".to_string())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(format!("unit_struct:{}", name))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(format!("unit_variant:{}::{}[{}]", name, variant, variant_index))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let inner = value.serialize(TestSerializer::default())?;
        Ok(format!("newtype_struct:{}({})", name, inner))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let inner = value.serialize(TestSerializer::default())?;
        Ok(format!("newtype_variant:{}::{}[{}]({})", name, variant, variant_index, inner))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(TestSeqSerializer { items: Vec::new() })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(TestSeqSerializer { items: Vec::new() })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(TestSeqSerializer { items: Vec::new() })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(TestSeqSerializer { items: Vec::new() })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(TestMapSerializer {
            items: Vec::new(),
            pending_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(TestMapSerializer {
            items: Vec::new(),
            pending_key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(TestMapSerializer {
            items: Vec::new(),
            pending_key: None,
        })
    }
}

struct TestSeqSerializer {
    items: Vec<String>,
}

impl SerializeSeq for TestSeqSerializer {
    type Ok = String;
    type Error = TestError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let s = value.serialize(TestSerializer::default())?;
        self.items.push(s);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(format!("seq:[{}]", self.items.join(",")))
    }
}

impl SerializeTuple for TestSeqSerializer {
    type Ok = String;
    type Error = TestError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let s = value.serialize(TestSerializer::default())?;
        self.items.push(s);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(format!("tuple:({})", self.items.join(",")))
    }
}

impl serde::ser::SerializeTupleStruct for TestSeqSerializer {
    type Ok = String;
    type Error = TestError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let s = value.serialize(TestSerializer::default())?;
        self.items.push(s);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(format!("tuple_struct:({})", self.items.join(",")))
    }
}

impl serde::ser::SerializeTupleVariant for TestSeqSerializer {
    type Ok = String;
    type Error = TestError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let s = value.serialize(TestSerializer::default())?;
        self.items.push(s);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(format!("tuple_variant:({})", self.items.join(",")))
    }
}

struct TestMapSerializer {
    items: Vec<(String, String)>,
    pending_key: Option<String>,
}

impl SerializeMap for TestMapSerializer {
    type Ok = String;
    type Error = TestError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let s = key.serialize(TestSerializer::default())?;
        self.pending_key = Some(s);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self.pending_key.take().expect("serialize_value called without serialize_key");
        let v = value.serialize(TestSerializer::default())?;
        self.items.push((key, v));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let pairs: Vec<String> = self.items.into_iter().map(|(k, v)| format!("{}:{}", k, v)).collect();
        Ok(format!("map:{{{}}}", pairs.join(",")))
    }
}

impl serde::ser::SerializeStruct for TestMapSerializer {
    type Ok = String;
    type Error = TestError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let v = value.serialize(TestSerializer::default())?;
        self.items.push((key.to_string(), v));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let pairs: Vec<String> = self.items.into_iter().map(|(k, v)| format!("{}:{}", k, v)).collect();
        Ok(format!("struct:{{{}}}", pairs.join(",")))
    }
}

impl serde::ser::SerializeStructVariant for TestMapSerializer {
    type Ok = String;
    type Error = TestError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let v = value.serialize(TestSerializer::default())?;
        self.items.push((key.to_string(), v));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let pairs: Vec<String> = self.items.into_iter().map(|(k, v)| format!("{}:{}", k, v)).collect();
        Ok(format!("struct_variant:{{{}}}", pairs.join(",")))
    }
}

/// Helper to compare Serialize and SerializeOwned output
fn serialize_ref<T: Serialize>(value: &T) -> String {
    value.serialize(TestSerializer::default()).unwrap()
}

fn serialize_owned<T: SerializeOwned>(value: T) -> String {
    value.serialize_owned(TestSerializer::default()).unwrap()
}

// ============================================================================
// Tests for String
// ============================================================================

#[test]
fn test_string_serialize_owned() {
    let s = String::from("hello world");
    let expected = serialize_ref(&s);
    let actual = serialize_owned(s);
    assert_eq!(expected, actual);
    assert_eq!(actual, "str:hello world");
}

#[test]
fn test_string_empty() {
    let s = String::new();
    let expected = serialize_ref(&s);
    let actual = serialize_owned(s);
    assert_eq!(expected, actual);
    assert_eq!(actual, "str:");
}

// ============================================================================
// Tests for CString
// ============================================================================

#[test]
fn test_cstring_serialize_owned() {
    let cs = CString::new("test").unwrap();
    let expected = serialize_ref(&cs);
    let actual = serialize_owned(cs);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for Vec<T>
// ============================================================================

#[test]
fn test_vec_serialize_owned() {
    let v = vec![1i32, 2, 3];
    let expected = serialize_ref(&v);
    let actual = serialize_owned(v);
    assert_eq!(expected, actual);
}

#[test]
fn test_vec_empty() {
    let v: Vec<i32> = vec![];
    let expected = serialize_ref(&v);
    let actual = serialize_owned(v);
    assert_eq!(expected, actual);
}

#[test]
fn test_vec_strings() {
    let v = vec![String::from("a"), String::from("b")];
    let expected = serialize_ref(&v);
    let actual = serialize_owned(v);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for Box<T>
// ============================================================================

#[test]
fn test_box_serialize_owned() {
    let b = Box::new(42i32);
    let expected = serialize_ref(&b);
    let actual = serialize_owned(b);
    assert_eq!(expected, actual);
    assert_eq!(actual, "i32:42");
}

#[test]
fn test_box_string() {
    let b = Box::new(String::from("boxed"));
    let expected = serialize_ref(&b);
    let actual = serialize_owned(b);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for VecDeque<T>
// ============================================================================

#[test]
fn test_vecdeque_serialize_owned() {
    let mut vd = VecDeque::new();
    vd.push_back(1i32);
    vd.push_back(2);
    vd.push_back(3);
    let expected = serialize_ref(&vd);
    let actual = serialize_owned(vd);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for LinkedList<T>
// ============================================================================

#[test]
fn test_linkedlist_serialize_owned() {
    let mut ll = LinkedList::new();
    ll.push_back(1i32);
    ll.push_back(2);
    let expected = serialize_ref(&ll);
    let actual = serialize_owned(ll);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for BinaryHeap<T>
// ============================================================================

#[test]
fn test_binaryheap_serialize_owned() {
    let mut bh = BinaryHeap::new();
    bh.push(3i32);
    bh.push(1);
    bh.push(2);
    // Note: BinaryHeap order is not guaranteed when iterating
    // Just verify it produces output, not the exact content
    let result = serialize_owned(bh);
    assert!(result.starts_with("seq:["));
}

// ============================================================================
// Tests for BTreeSet<T>
// ============================================================================

#[test]
fn test_btreeset_serialize_owned() {
    let mut bs = BTreeSet::new();
    bs.insert(1i32);
    bs.insert(2);
    bs.insert(3);
    let expected = serialize_ref(&bs);
    let actual = serialize_owned(bs);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for HashSet<T>
// ============================================================================

#[test]
fn test_hashset_serialize_owned() {
    // Single element to avoid ordering issues
    let mut hs = HashSet::new();
    hs.insert(42i32);
    let expected = serialize_ref(&hs);
    let actual = serialize_owned(hs);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for BTreeMap<K, V>
// ============================================================================

#[test]
fn test_btreemap_serialize_owned() {
    let mut bm = BTreeMap::new();
    bm.insert("a", 1i32);
    bm.insert("b", 2);
    let expected = serialize_ref(&bm);
    let actual = serialize_owned(bm);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for HashMap<K, V>
// ============================================================================

#[test]
fn test_hashmap_serialize_owned() {
    // Single element to avoid ordering issues
    let mut hm = HashMap::new();
    hm.insert("key", 42i32);
    let expected = serialize_ref(&hm);
    let actual = serialize_owned(hm);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for Option<T>
// ============================================================================

#[test]
fn test_option_some_serialize_owned() {
    let opt = Some(42i32);
    let expected = serialize_ref(&opt);
    let actual = serialize_owned(opt);
    assert_eq!(expected, actual);
    assert_eq!(actual, "some:i32:42");
}

#[test]
fn test_option_none_serialize_owned() {
    let opt: Option<i32> = None;
    let expected = serialize_ref(&opt);
    let actual = serialize_owned(opt);
    assert_eq!(expected, actual);
    assert_eq!(actual, "none");
}

// ============================================================================
// Tests for Cow<'a, T>
// ============================================================================

#[test]
fn test_cow_borrowed_serialize_owned() {
    let cow: Cow<str> = Cow::Borrowed("borrowed");
    let expected = serialize_ref(&cow);
    let actual = serialize_owned(cow);
    assert_eq!(expected, actual);
}

#[test]
fn test_cow_owned_serialize_owned() {
    let cow: Cow<str> = Cow::Owned(String::from("owned"));
    let expected = serialize_ref(&cow);
    let actual = serialize_owned(cow);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for Rc<T>
// ============================================================================

#[test]
fn test_rc_serialize_owned() {
    let rc = Rc::new(42i32);
    let expected = serialize_ref(&rc);
    let actual = serialize_owned(rc);
    assert_eq!(expected, actual);
    assert_eq!(actual, "i32:42");
}

// ============================================================================
// Tests for Arc<T>
// ============================================================================

#[test]
fn test_arc_serialize_owned() {
    let arc = Arc::new(42i32);
    let expected = serialize_ref(&arc);
    let actual = serialize_owned(arc);
    assert_eq!(expected, actual);
    assert_eq!(actual, "i32:42");
}

// ============================================================================
// Tests for PathBuf
// ============================================================================

#[test]
fn test_pathbuf_serialize_owned() {
    let pb = PathBuf::from("/some/path");
    let expected = serialize_ref(&pb);
    let actual = serialize_owned(pb);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for Bound<T>
// ============================================================================

#[test]
fn test_bound_unbounded_serialize_owned() {
    let bound: Bound<i32> = Bound::Unbounded;
    let expected = serialize_ref(&bound);
    let actual = serialize_owned(bound);
    assert_eq!(expected, actual);
}

#[test]
fn test_bound_included_serialize_owned() {
    let bound = Bound::Included(42i32);
    let expected = serialize_ref(&bound);
    let actual = serialize_owned(bound);
    assert_eq!(expected, actual);
}

#[test]
fn test_bound_excluded_serialize_owned() {
    let bound = Bound::Excluded(42i32);
    let expected = serialize_ref(&bound);
    let actual = serialize_owned(bound);
    assert_eq!(expected, actual);
}

// ============================================================================
// Tests for Wrapping<T>
// ============================================================================

#[test]
fn test_wrapping_serialize_owned() {
    let w = Wrapping(42i32);
    let expected = serialize_ref(&w);
    let actual = serialize_owned(w);
    assert_eq!(expected, actual);
    assert_eq!(actual, "i32:42");
}

// ============================================================================
// Tests for Reverse<T>
// ============================================================================

#[test]
fn test_reverse_serialize_owned() {
    let r = Reverse(42i32);
    let expected = serialize_ref(&r);
    let actual = serialize_owned(r);
    assert_eq!(expected, actual);
    assert_eq!(actual, "i32:42");
}

// ============================================================================
// Tests for blanket implementation (&T where T: Serialize)
// ============================================================================

#[test]
fn test_reference_serialize_owned() {
    let value = 42i32;
    let reference = &value;
    // &T should implement SerializeOwned via blanket impl
    let result = serialize_owned(reference);
    assert_eq!(result, "i32:42");
}

#[test]
fn test_nested_types_serialize_owned() {
    // Test a complex nested type
    let nested: Vec<Option<Box<i32>>> = vec![Some(Box::new(1)), None, Some(Box::new(2))];
    let expected = serialize_ref(&nested);
    let actual = serialize_owned(nested);
    assert_eq!(expected, actual);
}
