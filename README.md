> **Crate to build config from environment, command line and files**
# Motivation
Non-runtime data generally comes to a project from
command line, environment and configuration files.\
Sometimes it comes from each of the sources simultaneously,
so all of them must be handled.\
None of the popular crates (including [clap](https://docs.rs/clap/latest/clap/) and [config](https://docs.rs/config/latest/config/))
can't handle all 3 together, so this crate has been created to solve this problem.

# Basis
The Core of the crate is an attribute-macro `#[config]`. \
Annotate structure with this macro and a field of it with the `source` attribute,
so the field will be searched in one of the provided sources. The sources can be provided by using the following nested `source` attributes:
1. `clap`: command line argument
2. `env`: environment variable
3. `config`: configuration file key
4. `default`: default value

**Example**
```rust
use config_manager::config;

#[config]
struct ApplicationConfig {
    #[source(clap(long, short = 'p'), env = "APP_MODEL_PATH", config)]
    model_path: String,
    #[source(env, config, default = 0)]
    prediction_delay: u64,
}
```
In the example above, to set the value of the `model_path` field, a user may provide:
- command line argument `--model_path`
- environment variable named `model_path`
- configuration file containing field `model_path`

If the value is found in multiple provided sources, the value will be assigned according to the provided order
(the order for the `model_path` field is `clap -> env -> config` and `env -> config -> default` for the `prediction_delay`). \
If none of them (including the default value) isn't found, the program returns error `MissingArgument`.

**Note:** the default value is always assigned last.

# Attributes documentation
For further understanding of project syntax and features, it is recommended to visit [Cookbook](./cookbook.md).

# Complex example
```rust
use std::collections::HashMap;

use config_manager::config;

const SUFFIX: &str = "_env";

#[derive(Debug)]
#[config(
    clap(version, author),
    env_prefix = "demo",
    file(
        format = "toml",
        clap(long = "config", short, help = "path to configuration file"),
        env = "demo_config",
        optional = true,
        default = "./config.toml"
    ),
    global_name = "CFG"
)]
struct MethodConfig {
    a: i32,
    #[source(
        env(init_from = "format!(\"b{}\", SUFFIX)"),
        default = "\"abc\".to_string()"
    )]
    b: String,
    #[source(config = "bpm")]
    c: i32,
    #[source(default = "HashMap::new()")]
    d: HashMap<i32, String>,
}

fn main() {
    dbg!(&*CFG);
}
```
Run
```console
cargo run --package examples --bin demo -- --config="examples/config.toml" --a=5
```
Result must be:
```console
[examples/src/demo.rs:34] &*CFG = MethodConfig {
    a: 5,
    b: "qwerty",
    c: 165,
    d: {},
}
```
