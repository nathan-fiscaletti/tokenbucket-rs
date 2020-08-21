# tokenbucket

[![Documentation](https://img.shields.io/badge/docs.rs-view%20documentation-blue)](https://docs.rs/tokenbucket/)
[![GitHub license](https://img.shields.io/github/license/nathan-fiscaletti/tokenbucket-rs)](https://github.com/nathan-fiscaletti/tokenbucket-rs/blob/master/LICENSE)
[![Downloads](https://img.shields.io/crates/d/tokenbucket.svg)](https://crates.io/crates/tokenbucket)

This library provides a TokenBucket Algorithm implementation for the Rust programming language.

## Instalation

Add the following to your Cargo.toml

```toml
[dependencies]
tokenbucket = "0.1.2"
```

## Usage

```rust
use tokenbucket::{TokenBucket, TokenAcquisitionResult};

fn main() {
    let mut bucket = TokenBucket::new(5.0, 100.0);
    match bucket.acquire(1.0) {
        Ok(rate)  => println!("rate/allow: {}, true", rate),
        Err(rate) => println!("rate/allow: {}, false", rate),
    }
}
```

> See [the documentation](https://docs.rs/tokenbucket/) for more advanced usage examples.