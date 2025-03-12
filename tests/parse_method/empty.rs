use config_manager::config;

use crate::{assert_ok_and_compare, set_env, test_env};

fn empty_env() {
    fn set(x: &str) -> Result<i32, String> {
        if x.is_empty() {
            Ok(1)
        } else if x == "\"\"" {
            Ok(2)
        } else {
            Err(format!("not empty argument: {x}"))
        }
    }

    #[derive(Debug, PartialEq)]
    #[config(
        __debug_cmd_input__("--cli=\"\"", "--cli_deser=\"\""),
        file(format = "toml", default = "./tests/data/config.toml",)
    )]
    #[allow(dead_code)]
    struct EmptyConfig {
        #[source(clap)]
        cli: String,
        #[source(clap, deserialize_with = "set")]
        cli_deser: i32,
        #[source(env)]
        env: String,
        #[source(env, deserialize_with = "set")]
        env_deser: i32,
        #[source(config = "empty")]
        file: String,
        #[source(config = "empty", deserialize_with = "set")]
        file_deser: i32,
    }
    set_env("env", "");
    set_env("env_deser", "");
    assert_ok_and_compare(&EmptyConfig {
        cli: String::new(),
        cli_deser: 2,
        env: String::new(),
        env_deser: 1,
        file: String::new(),
        file_deser: 2,
    });
}

#[test]
fn test_empty_string() {
    test_env(vec![empty_env])
}
