mod parse_method {
    mod clap;
    mod default;
    mod deserialize_with;
    mod env;
    mod file;
    mod flatten;
    mod init_from;
    mod integration_test;
    mod layers;
    mod subcommand;
    mod tables;
    mod parse_options;
    mod get_command;
}

fn test_env(tests: Vec<fn()>) {
    for test in tests {
        let prev_env = envmnt::checkpoint();
        test();
        prev_env.restore();
    }
}

fn assert_ok_and_compare<T>(reference: &T)
where
    T: config_manager::ConfigInit + std::fmt::Debug + PartialEq,
{
    let cfg = T::parse().unwrap();
    assert_eq!(&cfg, reference)
}

fn set_env<T1, T2>(key: T1, value: T2)
where
    T1: std::fmt::Display,
    T2: std::fmt::Display,
{
    envmnt::set(
        std::ffi::OsString::from(key.to_string()),
        std::ffi::OsString::from(value.to_string()),
    );
}
