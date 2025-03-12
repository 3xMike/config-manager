use config_manager::config;

use crate::{assert_ok_and_compare, set_env, test_env};

fn empty_env() {
    fn set_1(_x: &str) -> Result<i32, String> {
        Ok(1)
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
        #[source(clap, deserialize_with = "set_1")]
        cli_deser: i32,
        #[source(env)]
        env: String,
        #[source(env, deserialize_with = "set_1")]
        env_deser: i32,
        #[source(config = "empty")]
        file: String,
        #[source(config = "empty", deserialize_with = "set_1")]
        file_deser: i32,
    }
    set_env("env", "");
    assert_ok_and_compare(&EmptyConfig {
        cli: String::new(),
        cli_deser: 1,
        env: String::new(),
        env_deser: 1,
        file: String::new(),
        file_deser: 1,
    });
}

#[test]
fn test_empty_string() {
    test_env(vec![empty_env])
}
