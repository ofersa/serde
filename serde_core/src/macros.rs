// Super explicit first paragraph because this shows up at the top level and
// trips up people who are just looking for basic Serialize / Deserialize
// documentation.
/// Helper macro when implementing the `Deserializer` part of a new data format
/// for Serde.
///
/// Some [`Deserializer`] implementations for self-describing formats do not
/// care what hint the [`Visitor`] gives them, they just want to blindly call
/// the [`Visitor`] method corresponding to the data they can tell is in the
/// input. This requires repetitive implementations of all the [`Deserializer`]
/// trait methods.
///
/// ```edition2021
/// # use serde::forward_to_deserialize_any;
/// # use serde::de::{value, Deserializer, Visitor};
/// #
/// # struct MyDeserializer;
/// #
/// # impl<'de> Deserializer<'de> for MyDeserializer {
/// #     type Error = value::Error;
/// #
/// #     fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error>
/// #     where
/// #         V: Visitor<'de>,
/// #     {
/// #         unimplemented!()
/// #     }
/// #
/// #[inline]
/// fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
/// where
///     V: Visitor<'de>,
/// {
///     self.deserialize_any(visitor)
/// }
/// #
/// #     forward_to_deserialize_any! {
/// #         i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
/// #         bytes byte_buf option unit unit_struct newtype_struct seq tuple
/// #         tuple_struct map struct enum identifier ignored_any iter
/// #     }
/// # }
/// ```
///
/// The `forward_to_deserialize_any!` macro implements these simple forwarding
/// methods so that they forward directly to [`Deserializer::deserialize_any`].
/// You can choose which methods to forward.
///
/// ```edition2021
/// # use serde::forward_to_deserialize_any;
/// # use serde::de::{value, Deserializer, Visitor};
/// #
/// # struct MyDeserializer;
/// #
/// impl<'de> Deserializer<'de> for MyDeserializer {
/// #   type Error = value::Error;
/// #
///     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
///     where
///         V: Visitor<'de>,
///     {
///         /* ... */
/// #       let _ = visitor;
/// #       unimplemented!()
///     }
///
///     forward_to_deserialize_any! {
///         bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
///         bytes byte_buf option unit unit_struct newtype_struct seq tuple
///         tuple_struct map struct enum identifier ignored_any iter
///     }
/// }
/// ```
///
/// The macro assumes the convention that your `Deserializer` lifetime parameter
/// is called `'de` and that the `Visitor` type parameters on each method are
/// called `V`. A different type parameter and a different lifetime can be
/// specified explicitly if necessary.
///
/// ```edition2021
/// # use serde::forward_to_deserialize_any;
/// # use serde::de::{value, Deserializer, Visitor};
/// # use std::marker::PhantomData;
/// #
/// # struct MyDeserializer<V>(PhantomData<V>);
/// #
/// # impl<'q, V> Deserializer<'q> for MyDeserializer<V> {
/// #     type Error = value::Error;
/// #
/// #     fn deserialize_any<W>(self, visitor: W) -> Result<W::Value, Self::Error>
/// #     where
/// #         W: Visitor<'q>,
/// #     {
/// #         unimplemented!()
/// #     }
/// #
/// forward_to_deserialize_any! {
///     <W: Visitor<'q>>
///     bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
///     bytes byte_buf option unit unit_struct newtype_struct seq tuple
///     tuple_struct map struct enum identifier ignored_any iter
/// }
/// # }
/// ```
///
/// [`Deserializer`]: crate::Deserializer
/// [`Visitor`]: crate::de::Visitor
/// [`Deserializer::deserialize_any`]: crate::Deserializer::deserialize_any
#[macro_export(local_inner_macros)]
macro_rules! forward_to_deserialize_any {
    (<$visitor:ident: Visitor<$lifetime:tt>> $($func:ident)*) => {
        $(forward_to_deserialize_any_helper!{$func<$lifetime, $visitor>})*
    };
    // This case must be after the previous one.
    ($($func:ident)*) => {
        $(forward_to_deserialize_any_helper!{$func<'de, V>})*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! forward_to_deserialize_any_method {
    ($func:ident<$l:tt, $v:ident>($($arg:ident : $ty:ty),*)) => {
        #[inline]
        fn $func<$v>(self, $($arg: $ty,)* visitor: $v) -> $crate::__private::Result<$v::Value, <Self as $crate::de::Deserializer<$l>>::Error>
        where
            $v: $crate::de::Visitor<$l>,
        {
            $(
                let _ = $arg;
            )*
            self.deserialize_any(visitor)
        }
    };
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! forward_to_deserialize_any_helper {
    (bool<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_bool<$l, $v>()}
    };
    (i8<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_i8<$l, $v>()}
    };
    (i16<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_i16<$l, $v>()}
    };
    (i32<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_i32<$l, $v>()}
    };
    (i64<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_i64<$l, $v>()}
    };
    (i128<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_i128<$l, $v>()}
    };
    (u8<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_u8<$l, $v>()}
    };
    (u16<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_u16<$l, $v>()}
    };
    (u32<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_u32<$l, $v>()}
    };
    (u64<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_u64<$l, $v>()}
    };
    (u128<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_u128<$l, $v>()}
    };
    (f32<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_f32<$l, $v>()}
    };
    (f64<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_f64<$l, $v>()}
    };
    (char<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_char<$l, $v>()}
    };
    (str<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_str<$l, $v>()}
    };
    (string<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_string<$l, $v>()}
    };
    (bytes<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_bytes<$l, $v>()}
    };
    (byte_buf<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_byte_buf<$l, $v>()}
    };
    (option<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_option<$l, $v>()}
    };
    (unit<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_unit<$l, $v>()}
    };
    (unit_struct<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_unit_struct<$l, $v>(name: &'static str)}
    };
    (newtype_struct<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_newtype_struct<$l, $v>(name: &'static str)}
    };
    (seq<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_seq<$l, $v>()}
    };
    (tuple<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_tuple<$l, $v>(len: usize)}
    };
    (tuple_struct<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_tuple_struct<$l, $v>(name: &'static str, len: usize)}
    };
    (map<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_map<$l, $v>()}
    };
    (struct<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_struct<$l, $v>(name: &'static str, fields: &'static [&'static str])}
    };
    (enum<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_enum<$l, $v>(name: &'static str, variants: &'static [&'static str])}
    };
    (identifier<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_identifier<$l, $v>()}
    };
    (ignored_any<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_method!{deserialize_ignored_any<$l, $v>()}
    };
    (iter<$l:tt, $v:ident>) => {
        forward_to_deserialize_any_iter_method!{deserialize_iter<$l>}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! forward_to_deserialize_any_iter_method {
    (deserialize_iter<$l:tt>) => {
        #[cfg(any(feature = "std", feature = "alloc"))]
        #[inline]
        fn deserialize_iter<T>(self) -> $crate::__private::Result<$crate::lib::vec::IntoIter<$crate::__private::Result<T, <Self as $crate::de::Deserializer<$l>>::Error>>, <Self as $crate::de::Deserializer<$l>>::Error>
        where
            T: $crate::Deserialize<$l>,
        {
            // Create a visitor that collects into a Vec, delegating to deserialize_any
            struct IterVisitor<$l, T: $crate::Deserialize<$l>> {
                marker: $crate::__private::PhantomData<(&$l (), T)>,
            }

            impl<$l, T: $crate::Deserialize<$l>> $crate::de::Visitor<$l> for IterVisitor<$l, T> {
                type Value = $crate::lib::Vec<$crate::__private::Result<T, $crate::lib::String>>;

                fn expecting(&self, formatter: &mut $crate::__private::Formatter) -> $crate::__private::fmt::Result {
                    formatter.write_str("a sequence")
                }

                fn visit_seq<A>(self, seq: A) -> $crate::__private::Result<Self::Value, A::Error>
                where
                    A: $crate::de::SeqAccess<$l>,
                {
                    let iter: $crate::de::SeqAccessIterator<A, T> = $crate::de::SeqAccessIterator::new(seq);
                    // Collect all results, converting errors to strings for type erasure
                    let results: $crate::lib::Vec<$crate::__private::Result<T, $crate::lib::String>> = iter
                        .map(|r| r.map_err(|e| {
                            use $crate::__private::fmt::Write;
                            let mut buf = $crate::lib::String::new();
                            let _ = $crate::__private::write!(buf, "{}", e);
                            buf
                        }))
                        .collect();
                    $crate::__private::Ok(results)
                }
            }

            let results = match self.deserialize_any(IterVisitor::<T> {
                marker: $crate::__private::PhantomData,
            }) {
                $crate::__private::Ok(v) => v,
                $crate::__private::Err(e) => return $crate::__private::Err(e),
            };

            // Convert String errors back to Self::Error
            let converted: $crate::lib::Vec<$crate::__private::Result<T, <Self as $crate::de::Deserializer<$l>>::Error>> = results
                .into_iter()
                .map(|r| r.map_err(|s| $crate::de::Error::custom(s)))
                .collect();

            $crate::__private::Ok(converted.into_iter())
        }
    };
}
