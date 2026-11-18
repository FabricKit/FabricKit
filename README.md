# Fabrickit

Composable Rust crates for building distributed systems.

## Crates

| Crate | Purpose |
| --- | --- |
| `fabrickit-core` | Shared types used by every other crate |
| `fabrickit-net` | Connect machines and exchange data |
| `fabrickit-storage` | Persistent, reliable data storage |
| `fabrickit-consensus` | Agreement between nodes |
| `fabrickit-testkit` | Deterministic test helpers |

Dependencies point one way: every crate may depend on `fabrickit-core`;
`fabrickit-consensus` will also depend on `fabrickit-net` and `fabrickit-storage`.

## Build

```sh
cargo build --workspace
cargo test --workspace
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
