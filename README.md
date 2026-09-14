# token_db

A small Rust library for storing unique tokens, tracking their frequency, assigning stable numeric IDs, merging databases, and persisting them in a compact binary format.

The API is intentionally small. There is no compatibility layer for the pre-0.2 API.

## Example

```rust
use token_db::TokenDb;

fn main() -> token_db::Result<()> {
    let mut db = TokenDb::new();

    let hello = db.insert("hello")?;
    db.insert("world")?;
    db.insert("hello")?;

    assert_eq!(hello.get(), 0);
    assert_eq!(db.get(hello).unwrap().count(), 2);

    db.save("tokens.tdb")?;
    let restored = TokenDb::load("tokens.tdb")?;

    assert_eq!(restored.get_by_token("hello").unwrap().count(), 2);
    Ok(())
}
```

## API

- `TokenDb::insert` inserts one occurrence and returns a stable `TokenId`.
- `TokenDb::get` safely looks up by ID.
- `TokenDb::id` and `TokenDb::get_by_token` look up by text without allocating.
- `TokenDb::iter` iterates in stable ID order.
- `TokenDb::merge` merges another database and returns an ID mapping.
- `TokenDb::{to_bytes, from_bytes, save, load}` provide versioned binary persistence.

## Design

The vector of entries is the source of truth. The hash map only maps token text to IDs and is rebuilt when persisted data is loaded. Counts and persisted IDs do not use `usize`, so files are independent of pointer width. Invalid and unsupported input is rejected instead of being silently repaired.

## Development

```sh
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

All crate tests live under `tests/`; `src/` contains production code only.

## License

Apache-2.0.
