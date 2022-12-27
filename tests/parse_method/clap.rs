use std::collections::HashMap;

use config_manager::config;
use serde::Deserialize;

use crate::{assert_ok_and_compare, test_env};

fn simple_field() {
    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__("--plain=1", "--long=2", "--specified-long=3", "-u=4", "-q=5"))]
    struct Simple {
        #[source(clap)]
        plain: i32,
        #[source(clap)]
        long: i32,
        #[source(clap(long = "specified-long"))]
        long2: i32,
        #[source(clap(short))]
        unspecified_short: i32,
        #[source(clap(short = 'q'))]
        short2: i32,
    }

    assert_ok_and_compare(&Simple {
        plain: 1,
        long: 2,
        long2: 3,
        unspecified_short: 4,
        short2: 5,
    })
}

fn complex_field() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum Foo {
        Simple,
        Struct(Person),
        Map(HashMap<i32, bool>),
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Person {
        name: String,
        last_name: Option<String>,
        age: Option<i32>,
    }

    #[derive(Debug, PartialEq)]
    #[config(__debug_cmd_input__(
        "--field=[
            \"Simple\",
            {\"Struct\": {
                \"name\": \"Mike\",
                \"last_name\": null,
                \"age\": 22
            }},
            {\"Map\": {
                1: false,
                2: false,
                42: true
            }}
        ]"
    ))]
    struct Complex {
        #[source(clap)]
        field: Vec<Foo>,
    }

    assert_ok_and_compare(&Complex {
        field: vec![
            Foo::Simple,
            Foo::Struct(Person {
                name: "Mike".into(),
                last_name: None,
                age: Some(22),
            }),
            Foo::Map(HashMap::from_iter([(1, false), (2, false), (42, true)])),
        ],
    })
}

#[test]
fn clap() {
    test_env(vec![simple_field, complex_field]);
}
