use paste::paste;

use jsonpath_compiler::test_helper::{TestHelper, TestTarget};

macro_rules! additional {
    ($target:ident) => {
        paste! {
            #[test]
            fn [<$target:snake _wildcard_after_descendant_name>]() {
                TestHelper::new(
                    r#"$..a[*]"#,
                    r#"[{"a": {"a": {"b": 1}, "c": 2, "d": 3}}]"#,
                    r#"[{"b": 1}, 1, 2, 3]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _wildcard_after_descendant_index>]() {
                TestHelper::new(
                    r#"$..[0][*]"#,
                    r#"[[[1], 2, 3]]"#,
                    r#"[[1], 1, 2, 3]"#,
                    TestTarget::$target,
                ).run()
            }

            #[test]
            fn [<$target:snake _wildcard_after_descendant_negative_index>]() {
                TestHelper::new(
                    r#"$..[-1][*]"#,
                    r#"[[1, 2, [3]]]"#,
                    r#"[1, 2, [3], 3]"#,
                    TestTarget::$target,
                ).run()
            }
        }
    }
}

additional!(SimdjsonOndemand);
additional!(SimdjsonOndemandEagerFilters);
additional!(SimdjsonDom);
