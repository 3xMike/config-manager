use serde::Deserialize;

#[test]
fn test_private_method() {
    use config::*;
    use config_manager::__private::find_field_in_table;

    let cfg = r#"
        b = "qwerty"
        bpm = 165
        source.env = "no-env"
        [input]
        b = "mama"
        [input.data]
        frame = 5
        [input.data.img]
        frame_rate = 16000
    "#;
    let cfg = File::from_str(cfg, FileFormat::Toml).collect().unwrap();

    assert_eq!(
        find_field_in_table(&cfg, None, "b".into()).unwrap(),
        Some("\"qwerty\"".into())
    );
    assert_eq!(
        find_field_in_table(&cfg, Some("source".into()), "env".into()).unwrap(),
        Some("\"no-env\"".into())
    );
    assert_eq!(
        find_field_in_table(&cfg, None, "source.env".into()).unwrap(),
        Some("\"no-env\"".into())
    );
    assert_eq!(
        find_field_in_table(&cfg, Some("input".into()), "b".into()).unwrap(),
        Some("\"mama\"".into())
    );
    assert_eq!(
        find_field_in_table(&cfg, None, "input.b".into()).unwrap(),
        Some("\"mama\"".into())
    );
    assert_eq!(
        find_field_in_table(&cfg, Some("input.data".into()), "frame".into()).unwrap(),
        Some("5".into())
    );
    assert_eq!(
        find_field_in_table(&cfg, Some("input.data".into()), "img.frame_rate".into()).unwrap(),
        Some("16000".into())
    );

    assert_eq!(find_field_in_table(&cfg, None, "a".into()).unwrap(), None);
    assert_eq!(
        find_field_in_table(&cfg, Some("source".into()), "a".into()).unwrap(),
        None
    );
    assert_eq!(
        find_field_in_table(&cfg, None, "source.a".into()).unwrap(),
        None
    );
    assert_eq!(
        find_field_in_table(&cfg, Some("input".into()), "data.a".into()).unwrap(),
        None
    );

    assert_eq!(
        find_field_in_table(&cfg, Some("a".into()), "a".into()).unwrap(),
        None
    );
    assert_eq!(find_field_in_table(&cfg, None, "c.a".into()).unwrap(), None);
    assert_eq!(
        find_field_in_table(&cfg, Some("input".into()), "c.a".into()).unwrap(),
        None
    );
}

#[test]
fn test_struct_tables() {
    use crate::assert_ok_and_compare;
    use config_manager::{config, Flatten};

    #[derive(Debug, PartialEq)]
    #[config(
        file(format = "toml", default = "./tests/data/config.toml"),
        table = "input",
        __debug_cmd_input__()
    )]
    struct Cfg {
        #[source(config)]
        int: i32,
        #[source(config = "data.frame")]
        frame_id: i32,
        #[flatten]
        img: Images,
    }

    #[derive(Debug, Deserialize, Flatten, PartialEq)]
    #[table = "input.data"]
    struct Images {
        #[source(config)]
        frame: i32,
        #[source(config = "img.frame_rate")]
        rate: i32,
        #[source(default)]
        not_found: i32,
    }

    assert_ok_and_compare(&Cfg {
        int: 5,
        frame_id: 10,
        img: Images {
            frame: 10,
            rate: 16000,
            not_found: 0,
        },
    })
}
