# Serde Architecture

## Tech Stack

- **Language**: Rust (Edition 2021)
- **Build System**: Cargo workspace
- **MSRV**: 1.56 for serde/serde_core, 1.68 for serde_derive
- **Key Dependencies**:
  - `proc-macro2`, `quote`, `syn` - For derive macro implementation
  - `serde_test` - Testing utilities (dev dependency)
  - `trybuild` - UI/compile-fail tests (dev dependency)

## Project Structure

```
serde/                      # Main crate (re-exports serde_core)
  src/
    lib.rs                  # Re-exports with derive feature support
    private/                # Internal helpers for derive codegen
      de.rs                 # Deserialization helpers
      ser.rs                # Serialization helpers

serde_core/                 # Core traits only (no derive support)
  src/
    lib.rs                  # Crate root, feature gating
    de/                     # Deserialization
      mod.rs                # Deserialize trait, Visitor, etc.
      impls.rs              # Impl for standard types
      value.rs              # Value deserializers
      ignored_any.rs        # IgnoredAny type
    ser/                    # Serialization
      mod.rs                # Serialize trait, Serializer
      impls.rs              # Impl for standard types
      fmt.rs                # Display-based serialization
      impossible.rs         # Impossible type for trait bounds
    private/                # Internal APIs (not semver stable)
      content.rs            # Buffered content for untagged enums
      seed.rs               # DeserializeSeed helpers

serde_derive/               # Proc-macro crate
  src/
    lib.rs                  # Entry point, #[proc_macro_derive]
    de.rs                   # Deserialize derive implementation
    de/
      enum_*.rs             # Enum deserialization strategies
      struct_.rs            # Struct deserialization
      tuple.rs              # Tuple deserialization
    ser.rs                  # Serialize derive implementation
    internals/              # Shared parsing/analysis
      ast.rs                # AST representation
      attr.rs               # Attribute parsing (#[serde(...)])
      check.rs              # Validation checks
      case.rs               # Rename case conversions

serde_derive_internals/     # Published internals for ecosystem crates

test_suite/                 # Integration tests
  no_std/                   # no_std compatibility tests
```

## Feature Flags

### serde / serde_core
- `std` (default) - Standard library support
- `alloc` - Alloc crate support (String, Vec, Box, etc.)
- `derive` - Re-export serde_derive macros
- `rc` - Rc/Arc serialization (opt-in due to semantics)
- `unstable` - Unstable/nightly features

### serde_derive
- `deserialize_in_place` - Generate in-place deserialization

## Code Conventions

### Error Handling
- Use `Result` with custom error types implementing `serde::de::Error` or `serde::ser::Error`
- Errors should provide descriptive messages via `Error::custom()`

### Derive Attributes
The derive macros support extensive customization via `#[serde(...)]`:
- Container: `rename_all`, `deny_unknown_fields`, `tag`, `content`, `untagged`
- Field: `rename`, `default`, `skip`, `flatten`, `with`, `deserialize_with`
- Variant: `rename`, `alias`, `skip`, `other`

### no_std Support
- Core traits work in `no_std` with `no_alloc`
- Most impls require `alloc` feature at minimum
- Test with `test_suite/no_std` crate

### Internal APIs
The `private` modules contain APIs used by derived code. These are:
- Not covered by semver
- Must stay in sync between serde and serde_derive versions
- Enforced via version pinning in Cargo.toml

## Testing

### Running Tests
```bash
# Full test suite (nightly required for some tests)
cd test_suite && cargo +nightly test --features unstable

# Core crate tests with miri
cd serde_core && cargo miri test --features rc,unstable

# UI/compile-fail tests
cd test_suite && cargo test --features unstable -- ui
```

### Test Categories
- Unit tests in each crate's `src/` via `#[cfg(test)]`
- Integration tests in `test_suite/tests/`
- UI tests via `trybuild` for compile-error messages
- no_std build verification in `test_suite/no_std`
- Miri for undefined behavior detection

## CI Requirements

All PRs must pass:
- Build on stable, beta, and nightly Rust
- Build on MSRV (1.56, 1.60, 1.68)
- Clippy with pedantic lints
- Documentation build
- Miri memory safety checks
- Minimal dependency version resolution
- Outdated dependency check (non-PR)

## Key Design Patterns

### Visitor Pattern
Deserialization uses the visitor pattern. Implement `Visitor` trait to define how to construct your type from various primitive and compound data representations.

### Data Model
Serde has a universal data model that all formats translate to/from:
- Primitives: bool, i8-i128, u8-u128, f32, f64, char
- String: str, String
- Bytes: &[u8], Vec<u8>
- Option, Unit, Unit struct, Unit variant
- Newtype struct, Newtype variant
- Seq, Tuple, Tuple struct, Tuple variant
- Map, Struct, Struct variant

### Stateless Serialization
Serializers and Deserializers should be stateless where possible. Complex state belongs in wrapper types, not the format implementation.

### Lifetime Bounds and the Visitor Trait
The `Visitor` trait methods like `visit_seq<A>` have bounds like `A: SeqAccess<'de>` but do NOT include `A: 'de`. However, in practice, all `SeqAccess` implementations satisfy `A: 'de` because they're created by the deserializer for the duration of a single call and only reference input data with lifetime `'de`.

**Important pattern**: When you need to store a `SeqAccess` in a struct that requires `'de` lifetime bounds:
1. You CANNOT add `+ 'de` to the impl's where clause because the trait doesn't have it (E0276: stricter requirements than trait)
2. You CANNOT remove the `+ 'de` from your struct because the borrow checker needs it (E0309: may not live long enough)
3. **Solution**: Use `unsafe` with a separate constructor like `new_unchecked` that doesn't require `A: 'de`, and document that serde's design guarantees the bound is satisfied in practice.

Example:
```rust
// Safe constructor requires the bound
fn new<A: SeqAccess<'de> + 'de>(seq: A) -> Self { ... }

// Unsafe constructor for use in Visitor impl
unsafe fn new_unchecked<A: SeqAccess<'de>>(seq: A) -> Self { ... }

// In Visitor impl:
fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<...> {
    // SAFETY: SeqAccess impls always satisfy A: 'de in practice
    unsafe { Ok(MyType::new_unchecked(seq)) }
}
```

This is a known limitation of serde's trait design. The Visitor trait was designed before Rust had GATs (Generic Associated Types), which would have allowed expressing this bound correctly.
