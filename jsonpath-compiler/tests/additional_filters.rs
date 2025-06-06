use paste::paste;

use jsonpath_compiler::test_helper::{TestHelper, TestTarget};

static NESTED_DOCUMENT_1: &str = r#"[
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

static NESTED_DOCUMENT_2: &str = r#"
[
    {
        "a": 123,
        "b": {
            "c": 1,
            "a": 123,
            "b": { "c": 2 }
        },
        "c": {
            "a": 0,
            "b": { "c": -1 }
        }
    },
    {
        "a": 123,
        "b": { "c": 3, "a": 111 }
    },
    {
        "a": 0,
        "b": { "c": -2 }
    }
]
"#;


fn wrap_in_object(json: &str) -> String {
    format!("{{ \"root\": {json} }}")
}

macro_rules! additional_filters {
    ($target:ident) => {
        paste! {
            #[test]
            fn [<$target:snake _name_selector_after_filter_with_name_subquery>]() {
                TestHelper::new(
                    r#"$[?@.a == 123].b"#,
                    r#"[{"a": 123, "b": "x"}, {"a": 456, "b": "y"}]"#,
                    r#"["x"]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _index_selector_after_filter_with_index_subquery>]() {
                TestHelper::new(
                    r#"$[?@[0] == 123][1]"#,
                    r#"[[123, "x"], [456, "y"]]"#,
                    r#"["x"]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _multiple_selectors_after_filter>]() {
                TestHelper::new(
                    r#"$[?@.a == 123].b.c[0]"#,
                    r#"[{"a": 123, "b": {"c": ["x"]}}, {"a": 456, "b": {"c": ["y"]}}]"#,
                    r#"["x"]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _consecutive_filter_selectors>]() {
                TestHelper::new(
                    r#"$[?@.a == 123][?@.b == 456][?@.c == 789][?@.d == 321].result"#,
                    NESTED_DOCUMENT_1,
                    r#"["ok"]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _interleaved_filter_selectors>]() {
                TestHelper::new(
                    r#"$[?@.a == 123].right_path.right_path[?@.d == 321].result"#,
                    NESTED_DOCUMENT_1,
                    r#"["ok"]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _name_selectors_after_descendant_filter_selector_1>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123].b.c"#,
                    NESTED_DOCUMENT_2,
                    r#"[1, 2, 3]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _name_selectors_after_descendant_filter_selector_2>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123].b.c"#,
                    &wrap_in_object(NESTED_DOCUMENT_2),
                    r#"[1, 2, 3]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _descendant_name_selector_after_descendant_filter_selector_1>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123]..c"#,
                    NESTED_DOCUMENT_2,
                    r#"[1, 2, {"a": 0, "b": {"c": -1}}, -1, 3]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _descendant_name_selector_after_descendant_filter_selector_2>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123]..c"#,
                    &wrap_in_object(NESTED_DOCUMENT_2),
                    r#"[1, 2, {"a": 0, "b": {"c": -1}}, -1, 3]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _name_selector_after_descendant_name_selector_after_descendant_filter_selector_1>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123].b..c"#,
                    NESTED_DOCUMENT_2,
                    r#"[1, 2, 3]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _name_selector_after_descendant_name_selector_after_descendant_filter_selector_2>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123].b..c"#,
                    &wrap_in_object(NESTED_DOCUMENT_2),
                    r#"[1, 2, 3]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _filter_selector_after_descendant_name_selector_1>]() {
                TestHelper::new(
                    r#"$..b[?@.c]"#,
                    NESTED_DOCUMENT_2,
                    r#"[{"c":2 }]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _filter_selector_after_descendant_name_selector_2>]() {
                TestHelper::new(
                    r#"$..b[?@.c]"#,
                    &wrap_in_object(NESTED_DOCUMENT_2),
                    r#"[{"c":2 }]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _filter_selector_after_descendant_index_selector_1>]() {
                TestHelper::new(
                    r#"$..[0][?@[0]]"#,
                    r#"[[[1], 2, 3]]"#,
                    r#"[[1]]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _filter_selector_after_descendant_index_selector_2>]() {
                TestHelper::new(
                    r#"$..[0][?@[0]]"#,
                    &wrap_in_object(r#"[[[1], 2, 3]]"#),
                    r#"[[1]]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _filter_selector_after_descendant_negative_index_1>]() {
                TestHelper::new(
                    r#"$..[-1][?@[0]]"#,
                    r#"[[1, 2, [3]]]"#,
                    r#"[[3]]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _filter_selector_after_descendant_negative_index_2>]() {
                TestHelper::new(
                    r#"$..[-1][?@[0]]"#,
                    &wrap_in_object(r#"[[1, 2, [3]]]"#),
                    r#"[[3]]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _consecutive_descendant_filter_selectors_1>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123]..[?@.c > 0]"#,
                    NESTED_DOCUMENT_2,
                    r#"[{"c": 1, "a": 123, "b": {"c": 2}}, {"c": 2}, {"c": 3, "a": 111}]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _consecutive_descendant_filter_selectors_2>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123]..[?@.c > 0]"#,
                    &wrap_in_object(NESTED_DOCUMENT_2),
                    r#"[{"c": 1, "a": 123, "b": {"c": 2}}, {"c": 2}, {"c": 3, "a": 111}]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _descendant_filter_selectors_with_name_selector_in_between_1>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123].b..[?@.c > 0]"#,
                    NESTED_DOCUMENT_2,
                    r#"[{"c": 2}]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _descendant_filter_selectors_with_name_selector_in_between_2>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123].b..[?@.c > 0]"#,
                    &wrap_in_object(NESTED_DOCUMENT_2),
                    r#"[{"c": 2}]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _filter_selector_after_descendant_filter_selector_1>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123][?@.c > 1]"#,
                    NESTED_DOCUMENT_2,
                    r#"[{"c":2 }, {"c":3, "a":111 }]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _filter_selector_after_descendant_filter_selector_2>]() {
                TestHelper::new(
                    r#"$..[?@.a == 123][?@.c > 1]"#,
                    &wrap_in_object(NESTED_DOCUMENT_2),
                    r#"[{"c":2 }, {"c":3, "a":111 }]"#,
                    TestTarget::$target,
                ).run()
            }


            #[test]
            fn [<$target:snake _descendant_filter_selector_after_filter_selector>]() {
                TestHelper::new(
                    r#"$[?@.a == 123]..[?@.c > 0]"#,
                    NESTED_DOCUMENT_2,
                    r#"[{"c": 1, "a": 123, "b": {"c": 2}}, {"c": 2}, {"c": 3, "a": 111}]"#,
                    TestTarget::$target,
                ).run()
            }
        }
    }
}

additional_filters!(SimdjsonOndemand);
additional_filters!(SimdjsonOndemandEagerFilters);