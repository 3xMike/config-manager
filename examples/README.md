**Brief glance at `examples/main.rs` and `tests/integration_test.rs` should make crate's usage clear.**

### Example
To run example from project page Readme, type:
```bash
cargo run --package examples --bin demo -- --config="config.toml" --a=5
```

Next example shows sjson-syntax to use if you want to pass params with comand line or env:
```bash
cargo run --package examples --bin json_syntax -- --int=5 --string="Hello World!" \
--array="[Error, {\"Ok\":true}, {\"Ok\":false}]" --map_of_optionals="{1:null, 2: 2, 3: 10}" --class="{\"value\": 0, \"is_approved\": false}"
```
The result must be: 
```rust
cfg = Ok(
    Config {
        int: 5,
        string: "Hello World!",
        array: [
            Error,
            Ok(
                true,
            ),
            Ok(
                false,
            ),
        ],
        map_of_optionals: {
            2: Some(
                2,
            ),
            1: None,
            3: Some(
                10,
            ),
        },
        class: ApprovedInt {
            value: 0,
            is_approved: false,
        },
    },
)
```
Or you can pass any of these params with env keys; feel free to play with the examples.

Also, there are lots of examples in the `../tests` folder.