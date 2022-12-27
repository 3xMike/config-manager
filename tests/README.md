Since tests use shared environment, in order to avoid conflicts tests must be executed with:
```console
cargo test -- --test-threads=1
```