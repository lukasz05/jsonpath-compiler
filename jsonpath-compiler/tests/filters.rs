use jsonpath_compiler::Target;
use jsonpath_compiler::test_helper::TestHelper;

#[test]
fn name_selector_after_filter_with_name_subquery() {
    TestHelper::new(
        r#"$[?@.a == 123].b"#,
        r#"[{"a": 123, "b": "x"}, {"a": 456, "b": "y"}]"#,
        r#"["x"]"#,
        Target::SimdjsonOndemand,
    ).run()
}

#[test]
fn index_selector_after_filter_with_index_subquery() {
    TestHelper::new(
        r#"$[?@[0] == 123][1]"#,
        r#"[[123, "x"], [456, "y"]]"#,
        r#"["x"]"#,
        Target::SimdjsonOndemand,
    ).run()
}

#[test]
fn multiple_selectors_after_filter() {
    TestHelper::new(
        r#"$[?@.a == 123].b.c[0]"#,
        r#"[{"a": 123, "b": {"c": ["x"]}}, {"a": 456, "b": {"c": ["y"]}}]"#,
        r#"["x"]"#,
        Target::SimdjsonOndemand,
    ).run()
}

static NESTED_DOCUMENT: &str = r#"[
    {
        "a": 123,
        "right_path": {
            "b": 456,
            "right_path": {
                "c": 789,
                "right_path": {
                    "d": 321,
                    "result": "ok"
                }
            },
            "wrong_path": {
                "c": 1,
                "wrong_path": {
                    "d": 321,
                    "result": "fail"
                }
            }
        },
        "wrong_path": {
            "b": 1,
            "wrong_path": {
                "c": 1,
                "wrong_path": {
                    "d": 321,
                    "result": "fail"
                }
            }
        }
    },
    {
        "a": 1,
        "wrong_path": {
            "b": 1,
            "wrong_path": {
                "c": 1,
                "wrong_path": {
                    "d": 321,
                    "result": "fail"
                }
            }
        }
    }
]"#;

#[test]
fn consecutive_filter_selectors() {
    TestHelper::new(
        r#"$[?@.a == 123][?@.b == 456][?@.c == 789][?@.d == 321].result"#,
        NESTED_DOCUMENT,
        r#"["ok"]"#,
        Target::SimdjsonOndemand,
    ).run()
}

#[test]
fn interleaved_filter_selectors() {
    TestHelper::new(
        r#"$[?@.a == 123].right_path.right_path[?@.d == 321].result"#,
        NESTED_DOCUMENT,
        r#"["ok"]"#,
        Target::SimdjsonOndemand,
    ).run()
}