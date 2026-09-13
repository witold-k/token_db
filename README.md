# token_db

A small, allocation-conscious token database for Rust.

`token_db` stores unique tokens, tracks their frequency, and assigns each token a stable numeric index.
It is designed for fast lookups and efficient merging of token databases.

## License

Apache-2.0 (C) Witold Kaminski

## current state

althogh feature complete test and coverage is low, this will be done in the neared future.

## Features

- Fast token insertion and lookup
- Token frequency counting
- Stable numeric indices
- Lookup by token or index
- Efficient database merging
- Index mapping when merging databases
- Compact binary persistence
- Automatic index/map reconstruction after loading
- `FxHashMap` for fast lookups

## Usage

please check [Howto](HOWTO.md)

