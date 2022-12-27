use std::collections::HashMap;

use config_manager::{config, ConfigInit};
use serde::Deserialize;

use crate::{assert_ok_and_compare, set_env, test_env};

fn file_not_found() {
    #[config(
        file(
            format = "json",
            env = "config_1",
            clap(long = "config_1"),
            default = "./fake.json",
        ),
        __debug_cmd_input__()
    )]
    struct FileNotFound {}

    assert!(matches!(
        FileNotFound::parse(),
        Err(config_manager::Error::ExternalError(_))
    ));

    set_env("config_1", "fake");
    assert!(matches!(
        FileNotFound::parse(),
        Err(config_manager::Error::ExternalError(_))
    ));

    set_env("config_1", "./tests/data/config.json");
    assert!(FileNotFound::parse().is_ok());
}

fn file_found() {
    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "json", clap(long = "config1")),
        __debug_cmd_input__("--config1=./tests/data/config.json")
    )]
    struct ClapOnly {
        #[source(config)]
        int: i32,
    }

    assert_ok_and_compare(&ClapOnly { int: 1 });

    #[derive(Debug, PartialEq)]
    #[config(file(format = "json", env = "config_1"), __debug_cmd_input__())]
    struct EnvOnly {
        #[source(config)]
        int: i32,
    }

    set_env("config_1", "./tests/data/config.json");
    assert_ok_and_compare(&EnvOnly { int: 1 });

    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "json", default = "./tests/data/config.json"),
        __debug_cmd_input__()
    )]
    struct DefaultOnly {
        #[source(config)]
        int: i32,
    }

    assert_ok_and_compare(&DefaultOnly { int: 1 });

    #[derive(Debug, PartialEq)]
    #[config(
        file(
            format = "toml",
            clap(long = "config2"),
            env = "config_2",
            default = "./Cargo.toml",
        ),
        __debug_cmd_input__("--config2=./tests/data/config.toml")
    )]
    struct Both {
        #[source(config)]
        int: i32,
    }

    set_env("config_2", "./config-manager-proc/Cargo.toml");
    assert_ok_and_compare(&Both { int: 1 });
}

fn optional() {
    #[config(file(format = "json", env = "config_1",))]
    struct NotFound {}

    assert!(matches!(NotFound::parse(), Err(_)));

    #[config(
        file(format = "json", env = "config_1", optional = true),
        __debug_cmd_input__()
    )]
    struct Optional {}

    assert!(Optional::parse().is_ok());
}

fn solo_json() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum Foo {
        Var1,
        Var2(i32),
        Var3,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Bar {
        name: String,
        addr: String,
    }

    #[derive(Debug, PartialEq)]
    #[config(file(format = "json", env = "json_config"), __debug_cmd_input__())]
    struct Json {
        #[source(config)]
        int: i32,
        #[source(config = "name")]
        string: String,
        #[source(config = "10xPI")]
        float: f64,
        #[source(config)]
        none: Option<i64>,
        #[source(config)]
        some: Option<u64>,
        #[source(config = "debug_mode")]
        boolean: bool,
        #[source(config = "convenient_phone_map")]
        map: HashMap<String, u64>,
        #[source(config)]
        array_of_enums: Vec<Foo>,
        #[source(config)]
        class: Bar,
    }

    set_env("json_config", "./tests/data/config.json");
    assert_ok_and_compare(&Json {
        int: 1,
        string: "Mike".into(),
        float: 31.4159265,
        none: None,
        some: Some(999999999999999),
        boolean: true,
        map: HashMap::from([
            ("Mike".into(), 89650000000),
            ("James".into(), 79645553535),
            ("Money".into(), 89991113511),
        ]),
        array_of_enums: vec![Foo::Var1, Foo::Var3, Foo::Var3, Foo::Var2(5)],
        class: Bar {
            name: "Mike".into(),
            addr: "Moscow".into(),
        },
    });
}

fn toml_solo() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Fruit {
        name: String,
        physical: Option<HashMap<String, String>>,
        varieties: Vec<HashMap<String, String>>,
    }

    #[derive(Debug, PartialEq)]
    #[config(file(format = "toml", env = "toml_config"), __debug_cmd_input__())]
    struct Toml {
        #[source(config)]
        int: i32,
        #[source(config = "name")]
        string: String,
        #[source(config = "10xPI")]
        float: f64,
        #[source(config)]
        some: Option<u64>,
        #[source(config = "debug_mode")]
        boolean: bool,
        #[source(config)]
        fruits: Vec<Fruit>,
    }

    set_env("toml_config", "./tests/data/config.toml");
    assert_ok_and_compare(&Toml {
        int: 1,
        string: "Mike".into(),
        float: 31.4159265,
        some: Some(999999999999999),
        boolean: true,
        fruits: vec![
            Fruit {
                name: "apple".into(),
                physical: Some(HashMap::from([
                    ("color".into(), "red".into()),
                    ("shape".into(), "round".into()),
                ])),
                varieties: vec![
                    HashMap::from([("name".into(), "red delicious".into())]),
                    HashMap::from([("name".into(), "granny smith".into())]),
                ],
            },
            Fruit {
                name: "banana".into(),
                physical: None,
                varieties: vec![HashMap::from([("name".into(), "plantain".into())])],
            },
        ],
    });
}

fn yaml_solo() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct EmployeeInfo {
        name: String,
        job: String,
        skills: Vec<String>,
    }

    #[derive(Debug, PartialEq)]
    #[config(file(format = "yaml", env = "yaml_config"), __debug_cmd_input__())]
    struct Yaml {
        #[source(config)]
        none: Option<i32>,
        #[source(config = "bool_list")]
        array: Vec<bool>,
        #[source(config = "employees")]
        workers: HashMap<String, EmployeeInfo>,
    }

    set_env("yaml_config", "./tests/data/config.yml");
    assert_ok_and_compare(&Yaml {
        none: None,
        array: vec![true, false, true, false],
        workers: HashMap::from([
            (
                "martin".into(),
                EmployeeInfo {
                    name: "Martin D'vloper".into(),
                    job: "Developer".into(),
                    skills: vec!["python".into(), "perl".into(), "pascal".into()],
                },
            ),
            (
                "tabitha".into(),
                EmployeeInfo {
                    name: "Tabitha Bitumen".into(),
                    job: "Developer".into(),
                    skills: vec!["lisp".into(), "fortran".into(), "erlang".into()],
                },
            ),
        ]),
    });
}

fn merged_configs() {
    #[derive(Debug, Deserialize, PartialEq, Clone)]
    struct Person {
        name: String,
        surname: String,
    }

    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "toml", env = "toml_config"),
        file(format = "json", env = "json_config"),
        file(format = "yaml", env = "yaml_config"),
        __debug_cmd_input__()
    )]
    struct Merged {
        #[source(config = "json_person")]
        json: Person,
        #[source(config = "toml_person")]
        toml: Person,
        #[source(config = "yaml_person")]
        yml: Person,
        #[source(config)]
        json_toml: Person,
        #[source(config)]
        json_yml: Person,
        #[source(config)]
        toml_yml: Person,
        #[source(config = "person")]
        json_toml_yml: Person,
    }

    set_env("json_config", "./tests/data/config.json");
    set_env("toml_config", "./tests/data/config.toml");
    set_env("yaml_config", "./tests/data/config.yml");

    let chuck = Person {
        name: "Charles".into(),
        surname: "McGill".into(),
    };
    let jimmy = Person {
        name: "James".into(),
        surname: "McGill".into(),
    };
    let saul = Person {
        name: "Saul".into(),
        surname: "Goodman".into(),
    };

    assert_ok_and_compare(&Merged {
        json: chuck.clone(),
        toml: jimmy,
        yml: saul.clone(),
        json_toml: chuck,
        json_yml: saul.clone(),
        toml_yml: saul.clone(),
        json_toml_yml: saul,
    });
}

#[test]
fn field_from_file() {
    test_env(vec![solo_json, toml_solo, yaml_solo, merged_configs]);
}

#[test]
fn file_attribute() {
    test_env(vec![optional, file_not_found, file_found]);
}
