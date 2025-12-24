# Serde

A framework for serializing and deserializing Rust data structures efficiently and generically.

## Build and Test

```bash
# Run the full test suite (requires nightly)
cd test_suite && cargo +nightly test --features unstable

# Build main crate with common features
cd serde && cargo build --features rc

# Build without std (no_std)
cd serde && cargo build --no-default-features

# Run clippy (pedantic)
cd serde && cargo clippy --features rc,unstable -- -Dclippy::all -Dclippy::pedantic
cd serde_derive && cargo clippy -- -Dclippy::all -Dclippy::pedantic

# Generate documentation
cargo docs-rs -p serde
```

## Key Conventions

- MSRV: serde requires Rust 1.56+, serde_derive requires Rust 1.68+
- All warnings are treated as errors (`RUSTFLAGS=-Dwarnings`)
- Clippy runs with `--all` and `--pedantic` flags
- Support `no_std` environments via feature flags
- serde_derive version must stay in lockstep with serde (uses internal APIs)

## Workspace Crates

- `serde` - Main crate, re-exports serde_core with derive support
- `serde_core` - Core traits (Serialize, Deserialize) without derive
- `serde_derive` - Proc-macro for `#[derive(Serialize, Deserialize)]`
- `serde_derive_internals` - Shared internals for derive macros
- `test_suite` - Integration tests and UI tests
