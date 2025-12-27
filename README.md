# Xbox Network Transfer


## Dependencies

- Rust toolchain <https://rustup.rs>

## Client

This minimal client will fetch the first package from the first console discovered.

```
cargo run --bin client
```


## Server

This minimal server will pose as an Xbox (named: XBOXTEST) and announce files listed in [metadata.json](./metadata.json).

```
cargo run --bin server
```
