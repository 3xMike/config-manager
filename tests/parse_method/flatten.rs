use std::collections::HashMap;

use config_manager::config;
use serde::Deserialize;

use crate::{assert_ok_and_compare, set_env, test_env};

fn simple_flatten() {
    #[derive(Debug, PartialEq, Deserialize, config_manager::Flatten)]
    struct Nested {
        #[source(clap, env, config)]
        i32: i32,
        #[source(clap, env, config)]
        string: String,
        #[source(config)]
        map_even: HashMap<String, bool>,
    }

    #[derive(Debug, PartialEq)]
    #[config(
        env_prefix = "",
        file(format = "json", default = "tests/data/config.json"),
        __debug_cmd_input__("--u8=255", "--i32=22",)
    )]
    struct Complex {
        #[source(clap, env, config)]
        u8: u8,
        #[flatten]
        nested: Nested,
    }

    set_env("string", "bazqux");

    assert_ok_and_compare(&Complex {
        u8: 255,
        nested: Nested {
            i32: 22,
            string: "bazqux".to_string(),
            map_even: HashMap::from_iter(
                [(1, false), (2, true), (42, true)].map(|x| (x.0.to_string(), x.1)),
            ),
        },
    })
}

fn complex_flatten() {
    use config_manager_proc::Flatten;

    #[derive(Debug, PartialEq, Deserialize, Flatten)]
    pub(super) struct Nested {
        #[source(clap)]
        pub(super) int: i32,
        #[source(clap(short))]
        pub(super) name: String,
        #[flatten]
        pub(super) twice: TwiceNested,
    }

    #[derive(Debug, PartialEq, Deserialize, Flatten)]
    pub(super) struct TwiceNested {
        #[source(clap)]
        pub(super) field: i32,
    }

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__("--int=1", "-n=Mike", "--field=2"))]
    pub(super) struct Main {
        #[flatten]
        pub(super) nested: Nested,
    }

    assert_ok_and_compare(&Main {
        nested: Nested {
            int: 1,
            name: "Mike".into(),
            twice: TwiceNested { field: 2 },
        },
    })
}

fn env_initialized_flatten() {
    use config_manager_proc::Flatten;

    #[derive(Debug, PartialEq, Deserialize, Flatten)]
    pub(super) struct Nested {
        #[source(env = "override_int")]
        pub(super) int: i32,
        #[source(clap(long, short), env, config)]
        pub(super) name: String,
        #[flatten]
        pub(super) twice: TwiceNested,
    }

    #[derive(Debug, PartialEq, Deserialize, Flatten)]
    pub(super) struct TwiceNested {
        #[source(clap, env, config)]
        pub(super) i32_field: i32,
        #[source(clap, env, config)]
        pub(super) string_field: String,
    }

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__("--i32_field=2"), env_prefix = "FLATTEN")]
    pub(super) struct Main {
        #[flatten]
        pub(super) nested: Nested,
    }

    set_env("flatten_INT", "will not be used");
    set_env("OVERRIDE_INT", 1);
    set_env("flatten_name", "Mike");
    set_env("flatten_i32_field", "will not be used");
    set_env("FLATTEN_STRING_FIELD", "foobar");

    assert_ok_and_compare(&Main {
        nested: Nested {
            int: 1,
            name: "Mike".into(),
            twice: TwiceNested {
                i32_field: 2,
                string_field: "foobar".to_string(),
            },
        },
    })
}

#[test]
fn flatten() {
    test_env(vec![
        simple_flatten,
        complex_flatten,
        env_initialized_flatten,
    ]);
}
