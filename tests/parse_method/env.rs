use config_manager::{config, ConfigInit, Error};

use crate::{assert_ok_and_compare, set_env, test_env};

fn single_var() {
    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__())]
    struct SingleVarConfig {
        #[source(env = "a")]
        a: i32,
    }

    set_env("a", 100);

    assert_ok_and_compare(&SingleVarConfig { a: 100 });
}

fn two_vars_one_is_default() {
    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__())]
    struct TwoVarConfig {
        #[source(env = "a")]
        a: String,
        #[source(env = "b", default = "\"default b\".into()")]
        b: String,
    }

    set_env("a", "foobar");

    assert_ok_and_compare(&TwoVarConfig {
        a: "foobar".into(),
        b: "default b".into(),
    })
}

fn optional_var() {
    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__())]
    struct OptionalVarConfig {
        #[source(env = "a")]
        a: Option<String>,
    }

    set_env("a", "foobar");

    assert_ok_and_compare(&OptionalVarConfig {
        a: Some("foobar".into()),
    })
}

fn optional_var_not_found() {
    #[config(__debug_cmd_input__())]
    struct OptionalVarConfig {
        #[allow(dead_code)]
        #[source(env = "a")]
        a: Option<String>,
    }

    let parsed = OptionalVarConfig::parse();
    assert!(matches!(parsed, Err(Error::MissingArgument(_))));
}

#[test]
fn env() {
    test_env(vec![
        single_var,
        two_vars_one_is_default,
        optional_var,
        optional_var_not_found,
    ])
}

#[test]
fn env_prefix() {
    test_env(vec![empty_prefix, no_prefix, some_prefix, binary_prefix])
}

fn empty_prefix() {
    #[derive(Debug, PartialEq)]
    #[config(env_prefix = "", __debug_cmd_input__())]
    struct EmptyPrefix {
        #[source(env = "fir")]
        first: i32,
        #[source(env)]
        second: i32,
    }

    set_env("fir", 1);
    set_env("second", 2);
    assert_ok_and_compare(&EmptyPrefix {
        first: 1,
        second: 2,
    });
}

fn some_prefix() {
    #[derive(Debug, PartialEq)]
    #[config(env_prefix = "some", __debug_cmd_input__())]
    struct SomePrefix {
        #[source(env = "fir")]
        first: i32,
        #[source(env)]
        second: i32,
    }

    set_env("fir", 1);
    set_env("some_second", 2);
    assert_ok_and_compare(&SomePrefix {
        first: 1,
        second: 2,
    });
}

fn no_prefix() {
    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__())]
    struct NoPrefix {
        #[source(env = "fir")]
        first: i32,
        #[source(env)]
        second: i32,
    }

    set_env("fir", 1);
    set_env("second", 2);
    assert_ok_and_compare(&NoPrefix {
        first: 1,
        second: 2,
    });
}

/// bin file is like config-manager/target/debug/deps/parse_method-b5e125d4f8a36dad
fn binary_prefix() {
    #[config(env_prefix, __debug_cmd_input__())]
    struct BinPrefix {
        #[allow(dead_code)]
        #[source(env)]
        first: String,
    }

    set_env("first", 1);

    let parsed = BinPrefix::parse();
    assert!(matches!(parsed, Err(Error::MissingArgument(_))));
}
