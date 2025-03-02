use paste::paste;

use jsonpath_compiler::Target;
use jsonpath_compiler::test_helper::TestHelper;

macro_rules! cts {
    ($target:ident) => {
        paste! {
            #[test]
            fn [<$target:snake _basic_root>]() {
                TestHelper::new(r#"$"#, r#"["first", "second"]"#, r#"[["first", "second"]]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_name_shorthand>]() {
                TestHelper::new(r#"$.a"#, r#"{"a": "A", "b": "B"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_name_shorthand_extended_unicode_u263a>]() {
                TestHelper::new(r#"$.☺"#, r#"{"☺": "A", "b": "B"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_name_shorthand_underscore>]() {
                TestHelper::new(r#"$._"#, r#"{"_": "A", "_foo": "B"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_name_shorthand_absent_data>]() {
                TestHelper::new(r#"$.c"#, r#"{"a": "A", "b": "B"}"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_name_shorthand_array_data>]() {
                TestHelper::new(r#"$.a"#, r#"["first", "second"]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_wildcard_shorthand_object_data>]() {
                TestHelper::new(r#"$.*"#, r#"{"a": "A", "b": "B"}"#, r#"["A", "B"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_wildcard_shorthand_array_data>]() {
                TestHelper::new(r#"$.*"#, r#"["first", "second"]"#, r#"["first", "second"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_wildcard_selector_array_data>]() {
                TestHelper::new(r#"$[*]"#, r#"["first", "second"]"#, r#"["first", "second"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_wildcard_shorthand_then_name_shorthand>]() {
                TestHelper::new(r#"$.*.a"#, r#"{"x": {"a": "Ax", "b": "Bx"}, "y": {"a": "Ay", "b": "By"}}"#, r#"["Ax", "Ay"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_multiple_selectors>]() {
                TestHelper::new(r#"$[0,2]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[0, 2]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_multiple_selectors_name_and_index_array_data>]() {
                TestHelper::new(r#"$['a',1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[1]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_multiple_selectors_name_and_index_object_data>]() {
                TestHelper::new(r#"$['a',1]"#, r#"{"a": 1, "b": 2}"#, r#"[1]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _basic_multiple_selectors_index_and_slice>]() {
                TestHelper::new(r#"$[1,5:7]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[1, 5, 6]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _basic_multiple_selectors_index_and_slice_overlapping>]() {
                TestHelper::new(r#"$[1,0:3]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[1, 0, 1, 2]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_multiple_selectors_duplicate_index>]() {
                TestHelper::new(r#"$[1,1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[1, 1]"#, Target::$target)
                    .ignore_order_and_duplicates()
                    .run()
            }

            #[test]
            fn [<$target:snake _basic_multiple_selectors_wildcard_and_index>]() {
                TestHelper::new(r#"$[*,1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1]"#, Target::$target)
                    .ignore_order_and_duplicates()
                    .run()
            }

            #[test]
            fn [<$target:snake _basic_multiple_selectors_wildcard_and_name>]() {
                TestHelper::new(r#"$[*,'a']"#, r#"{"a": "A", "b": "B"}"#, r#"["A", "B", "A"]"#, Target::$target)
                    .ignore_order_and_duplicates()
                    .run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _basic_multiple_selectors_wildcard_and_slice>]() {
                TestHelper::new(r#"$[*,0:2]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_multiple_selectors_multiple_wildcards>]() {
                TestHelper::new(r#"$[*,*]"#, r#"[0, 1, 2]"#, r#"[0, 1, 2, 0, 1, 2]"#, Target::$target)
                    .ignore_order_and_duplicates()
                    .run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_index>]() {
                TestHelper::new(r#"$..[1]"#, r#"{"o": [0, 1, [2, 3]]}"#, r#"[1, 3]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_name_shorthand>]() {
                TestHelper::new(r#"$..a"#, r#"{"o": [{"a": "b"}, {"a": "c"}]}"#, r#"["b", "c"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_wildcard_shorthand_array_data>]() {
                TestHelper::new(r#"$..*"#, r#"[0, 1]"#, r#"[0, 1]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_wildcard_selector_array_data>]() {
                TestHelper::new(r#"$..[*]"#, r#"[0, 1]"#, r#"[0, 1]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_wildcard_selector_nested_arrays>]() {
                TestHelper::new(r#"$..[*]"#, r#"[[[1]], [2]]"#, r#"[[[1]], [2], [1], 1, 2]"#, Target::$target)
                    .ignore_order_and_duplicates()
                    .run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_wildcard_selector_nested_objects>]() {
                TestHelper::new(r#"$..[*]"#, r#"{"a": {"c": {"e": 1}}, "b": {"d": 2}}"#, r#"[{"c": {"e": 1}}, {"d": 2}, {"e": 1}, 1, 2]"#, Target::$target)
                    .ignore_order_and_duplicates()
                    .run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_wildcard_shorthand_object_data>]() {
                TestHelper::new(r#"$..*"#, r#"{"a": "b"}"#, r#"["b"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_wildcard_shorthand_nested_data>]() {
                TestHelper::new(r#"$..*"#, r#"{"o": [{"a": "b"}]}"#, r#"[[{"a": "b"}], {"a": "b"}, "b"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_multiple_selectors>]() {
                TestHelper::new(r#"$..['a','d']"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, r#"["b", "e", "c", "f"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _basic_descendant_segment_object_traversal_multiple_selectors>]() {
                TestHelper::new(r#"$..['a','d']"#, r#"{"x": {"a": "b", "d": "e"}, "y": {"a": "c", "d": "f"}}"#, r#"["b", "e", "c", "f"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_existence_without_segments>]() {
                TestHelper::new(r#"$[?@]"#, r#"{"a": 1, "b": null}"#, r#"[1, null]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_existence>]() {
                TestHelper::new(r#"$[?@.a]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_existence_present_with_null>]() {
                TestHelper::new(r#"$[?@.a]"#, r#"[{"a": null, "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": null, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_string_single_quotes>]() {
                TestHelper::new(r#"$[?@.a=='b']"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_numeric_string_single_quotes>]() {
                TestHelper::new(r#"$[?@.a=='1']"#, r#"[{"a": "1", "d": "e"}, {"a": 1, "d": "f"}]"#, r#"[{"a": "1", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_string_double_quotes>]() {
                TestHelper::new(r#"$[?@.a=="b"]"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_numeric_string_double_quotes>]() {
                TestHelper::new(r#"$[?@.a=="1"]"#, r#"[{"a": "1", "d": "e"}, {"a": 1, "d": "f"}]"#, r#"[{"a": "1", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number>]() {
                TestHelper::new(r#"$[?@.a==1]"#, r#"[{"a": 1, "d": "e"}, {"a": "c", "d": "f"}, {"a": 2, "d": "f"}, {"a": "1", "d": "f"}]"#, r#"[{"a": 1, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_null>]() {
                TestHelper::new(r#"$[?@.a==null]"#, r#"[{"a": null, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": null, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_null_absent_from_data>]() {
                TestHelper::new(r#"$[?@.a==null]"#, r#"[{"d": "e"}, {"a": "c", "d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_true>]() {
                TestHelper::new(r#"$[?@.a==true]"#, r#"[{"a": true, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": true, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_false>]() {
                TestHelper::new(r#"$[?@.a==false]"#, r#"[{"a": false, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": false, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_self>]() {
                TestHelper::new(r#"$[?@==@]"#, r#"[1, null, true, {"a": "b"}, [false]]"#, r#"[1, null, true, {"a": "b"}, [false]]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_deep_equality_arrays>]() {
                TestHelper::new(r#"$[?@.a==@.b]"#, r#"[{"a": false, "b": [1, 2]}, {"a": [[1, [2]]], "b": [[1, [2]]]}, {"a": [[1, [2]]], "b": [[[2], 1]]}, {"a": [[1, [2]]], "b": [[1, 2]]}]"#, r#"[{"a": [[1, [2]]], "b": [[1, [2]]]}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_deep_equality_objects>]() {
                TestHelper::new(r#"$[?@.a==@.b]"#, r#"[{"a": false, "b": {"x": 1, "y": {"z": 1}}}, {"a": {"x": 1, "y": {"z": 1}}, "b": {"x": 1, "y": {"z": 1}}}, {"a": {"x": 1, "y": {"z": 1}}, "b": {"y": {"z": 1}, "x": 1}}, {"a": {"x": 1, "y": {"z": 1}}, "b": {"x": 1}}, {"a": {"x": 1, "y": {"z": 1}}, "b": {"x": 1, "y": {"z": 2}}}]"#, r#"[{"a": {"x": 1, "y": {"z": 1}}, "b": {"x": 1, "y": {"z": 1}}}, {"a": {"x": 1, "y": {"z": 1}}, "b": {"y": {"z": 1}, "x": 1}}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_string_single_quotes>]() {
                TestHelper::new(r#"$[?@.a!='b']"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_numeric_string_single_quotes>]() {
                TestHelper::new(r#"$[?@.a!='1']"#, r#"[{"a": "1", "d": "e"}, {"a": 1, "d": "f"}]"#, r#"[{"a": 1, "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_string_single_quotes_different_type>]() {
                TestHelper::new(r#"$[?@.a!='b']"#, r#"[{"a": "b", "d": "e"}, {"a": 1, "d": "f"}]"#, r#"[{"a": 1, "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_string_double_quotes>]() {
                TestHelper::new(r#"$[?@.a!="b"]"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_numeric_string_double_quotes>]() {
                TestHelper::new(r#"$[?@.a!="1"]"#, r#"[{"a": "1", "d": "e"}, {"a": 1, "d": "f"}]"#, r#"[{"a": 1, "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_string_double_quotes_different_types>]() {
                TestHelper::new(r#"$[?@.a!="b"]"#, r#"[{"a": "b", "d": "e"}, {"a": 1, "d": "f"}]"#, r#"[{"a": 1, "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_number>]() {
                TestHelper::new(r#"$[?@.a!=1]"#, r#"[{"a": 1, "d": "e"}, {"a": 2, "d": "f"}, {"a": "1", "d": "f"}]"#, r#"[{"a": 2, "d": "f"}, {"a": "1", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_number_different_types>]() {
                TestHelper::new(r#"$[?@.a!=1]"#, r#"[{"a": 1, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_null>]() {
                TestHelper::new(r#"$[?@.a!=null]"#, r#"[{"a": null, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_null_absent_from_data>]() {
                TestHelper::new(r#"$[?@.a!=null]"#, r#"[{"d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"d": "e"}, {"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_true>]() {
                TestHelper::new(r#"$[?@.a!=true]"#, r#"[{"a": true, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_equals_false>]() {
                TestHelper::new(r#"$[?@.a!=false]"#, r#"[{"a": false, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_string_single_quotes>]() {
                TestHelper::new(r#"$[?@.a<'c']"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_string_double_quotes>]() {
                TestHelper::new(r#"$[?@.a<"c"]"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_number>]() {
                TestHelper::new(r#"$[?@.a<10]"#, r#"[{"a": 1, "d": "e"}, {"a": 10, "d": "e"}, {"a": "c", "d": "f"}, {"a": 20, "d": "f"}]"#, r#"[{"a": 1, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_null>]() {
                TestHelper::new(r#"$[?@.a<null]"#, r#"[{"a": null, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_true>]() {
                TestHelper::new(r#"$[?@.a<true]"#, r#"[{"a": true, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_false>]() {
                TestHelper::new(r#"$[?@.a<false]"#, r#"[{"a": false, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_or_equal_to_string_single_quotes>]() {
                TestHelper::new(r#"$[?@.a<='c']"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_or_equal_to_string_double_quotes>]() {
                TestHelper::new(r#"$[?@.a<="c"]"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_or_equal_to_number>]() {
                TestHelper::new(r#"$[?@.a<=10]"#, r#"[{"a": 1, "d": "e"}, {"a": 10, "d": "e"}, {"a": "c", "d": "f"}, {"a": 20, "d": "f"}]"#, r#"[{"a": 1, "d": "e"}, {"a": 10, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_or_equal_to_null>]() {
                TestHelper::new(r#"$[?@.a<=null]"#, r#"[{"a": null, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": null, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_or_equal_to_true>]() {
                TestHelper::new(r#"$[?@.a<=true]"#, r#"[{"a": true, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": true, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_less_than_or_equal_to_false>]() {
                TestHelper::new(r#"$[?@.a<=false]"#, r#"[{"a": false, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": false, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_string_single_quotes>]() {
                TestHelper::new(r#"$[?@.a>'c']"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_string_double_quotes>]() {
                TestHelper::new(r#"$[?@.a>"c"]"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_number>]() {
                TestHelper::new(r#"$[?@.a>10]"#, r#"[{"a": 1, "d": "e"}, {"a": 10, "d": "e"}, {"a": "c", "d": "f"}, {"a": 20, "d": "f"}]"#, r#"[{"a": 20, "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_null>]() {
                TestHelper::new(r#"$[?@.a>null]"#, r#"[{"a": null, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_true>]() {
                TestHelper::new(r#"$[?@.a>true]"#, r#"[{"a": true, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_false>]() {
                TestHelper::new(r#"$[?@.a>false]"#, r#"[{"a": false, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_or_equal_to_string_single_quotes>]() {
                TestHelper::new(r#"$[?@.a>='c']"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "c", "d": "f"}, {"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_or_equal_to_string_double_quotes>]() {
                TestHelper::new(r#"$[?@.a>="c"]"#, r#"[{"a": "b", "d": "e"}, {"a": "c", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "c", "d": "f"}, {"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_or_equal_to_number>]() {
                TestHelper::new(r#"$[?@.a>=10]"#, r#"[{"a": 1, "d": "e"}, {"a": 10, "d": "e"}, {"a": "c", "d": "f"}, {"a": 20, "d": "f"}]"#, r#"[{"a": 10, "d": "e"}, {"a": 20, "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_or_equal_to_null>]() {
                TestHelper::new(r#"$[?@.a>=null]"#, r#"[{"a": null, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": null, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_or_equal_to_true>]() {
                TestHelper::new(r#"$[?@.a>=true]"#, r#"[{"a": true, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": true, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_greater_than_or_equal_to_false>]() {
                TestHelper::new(r#"$[?@.a>=false]"#, r#"[{"a": false, "d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": false, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_exists_and_not_equals_null_absent_from_data>]() {
                TestHelper::new(r#"$[?@.a&&@.a!=null]"#, r#"[{"d": "e"}, {"a": "c", "d": "f"}]"#, r#"[{"a": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_exists_and_exists_data_false>]() {
                TestHelper::new(r#"$[?@.a&&@.b]"#, r#"[{"a": false, "b": false}, {"b": false}, {"c": false}]"#, r#"[{"a": false, "b": false}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_exists_or_exists_data_false>]() {
                TestHelper::new(r#"$[?@.a||@.b]"#, r#"[{"a": false, "b": false}, {"b": false}, {"c": false}]"#, r#"[{"a": false, "b": false}, {"b": false}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_and>]() {
                TestHelper::new(r#"$[?@.a>0&&@.a<10]"#, r#"[{"a": -10, "d": "e"}, {"a": 5, "d": "f"}, {"a": 20, "d": "f"}]"#, r#"[{"a": 5, "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_or>]() {
                TestHelper::new(r#"$[?@.a=='b'||@.a=='d']"#, r#"[{"a": "a", "d": "e"}, {"a": "b", "d": "f"}, {"a": "c", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "b", "d": "f"}, {"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_expression>]() {
                TestHelper::new(r#"$[?!(@.a=='b')]"#, r#"[{"a": "a", "d": "e"}, {"a": "b", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "a", "d": "e"}, {"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_exists>]() {
                TestHelper::new(r#"$[?!@.a]"#, r#"[{"a": "a", "d": "e"}, {"d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_not_exists_data_null>]() {
                TestHelper::new(r#"$[?!@.a]"#, r#"[{"a": null, "d": "e"}, {"d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"d": "f"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_non_singular_existence_wildcard>]() {
                TestHelper::new(r#"$[?@.*]"#, r#"[1, [], [2], {}, {"a": 3}]"#, r#"[[2], {"a": 3}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_non_singular_existence_multiple>]() {
                TestHelper::new(r#"$[?@[0, 0, 'a']]"#, r#"[1, [], [2], [2, 3], {"a": 3}, {"b": 4}, {"a": 3, "b": 4}]"#, r#"[[2], [2, 3], {"a": 3}, {"a": 3, "b": 4}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_non_singular_existence_slice>]() {
                TestHelper::new(r#"$[?@[0:2]]"#, r#"[1, [], [2], [2, 3, 4], {}, {"a": 3}]"#, r#"[[2], [2, 3, 4]]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_non_singular_existence_negated>]() {
                TestHelper::new(r#"$[?!@.*]"#, r#"[1, [], [2], {}, {"a": 3}]"#, r#"[1, [], {}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_nested>]() {
                TestHelper::new(r#"$[?@[?@>1]]"#, r#"[[0], [0, 1], [0, 1, 2], [42]]"#, r#"[[0, 1, 2], [42]]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_name_segment_on_primitive_selects_nothing>]() {
                TestHelper::new(r#"$[?@.a == 1]"#, r#"{"a": 1}"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_name_segment_on_array_selects_nothing>]() {
                TestHelper::new(r#"$[?@['0'] == 5]"#, r#"[[5, 6]]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_index_segment_on_object_selects_nothing>]() {
                TestHelper::new(r#"$[?@[0] == 5]"#, r#"[{"0": 5}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_multiple_selectors>]() {
                TestHelper::new(r#"$[?@.a,?@.b]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_multiple_selectors_comparison>]() {
                TestHelper::new(r#"$[?@.a=='b',?@.b=='x']"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_multiple_selectors_overlapping>]() {
                TestHelper::new(r#"$[?@.a,?@.d]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}, {"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, Target::$target)
                    .ignore_order_and_duplicates()
                    .run()
            }

            #[test]
            fn [<$target:snake _filter_multiple_selectors_filter_and_index>]() {
                TestHelper::new(r#"$[?@.a,1]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_multiple_selectors_filter_and_wildcard>]() {
                TestHelper::new(r#"$[?@.a,*]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}, {"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, Target::$target)
                    .ignore_order_and_duplicates()
                    .run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_multiple_selectors_filter_and_slice>]() {
                TestHelper::new(r#"$[?@.a,1:]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}, {"g": "h"}]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}, {"g": "h"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_multiple_selectors_comparison_filter_index_and_slice>]() {
                TestHelper::new(r#"$[1, ?@.a=='b', 1:]"#, r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"b": "c", "d": "f"}, {"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_zero_and_negative_zero>]() {
                TestHelper::new(r#"$[?@.a==0]"#, r#"[{"a": 0, "d": "e"}, {"a": 0.1, "d": "f"}, {"a": "0", "d": "g"}]"#, r#"[{"a": 0, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_negative_zero_and_zero>]() {
                TestHelper::new(r#"$[?@.a==-0]"#, r#"[{"a": 0, "d": "e"}, {"a": 0.1, "d": "f"}, {"a": "0", "d": "g"}]"#, r#"[{"a": 0, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_with_and_without_decimal_fraction>]() {
                TestHelper::new(r#"$[?@.a==1.0]"#, r#"[{"a": 1, "d": "e"}, {"a": 2, "d": "f"}, {"a": "1", "d": "g"}]"#, r#"[{"a": 1, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_exponent>]() {
                TestHelper::new(r#"$[?@.a==1e2]"#, r#"[{"a": 100, "d": "e"}, {"a": 100.1, "d": "f"}, {"a": "100", "d": "g"}]"#, r#"[{"a": 100, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_exponent_upper_e>]() {
                TestHelper::new(r#"$[?@.a==1E2]"#, r#"[{"a": 100, "d": "e"}, {"a": 100.1, "d": "f"}, {"a": "100", "d": "g"}]"#, r#"[{"a": 100, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_positive_exponent>]() {
                TestHelper::new(r#"$[?@.a==1e+2]"#, r#"[{"a": 100, "d": "e"}, {"a": 100.1, "d": "f"}, {"a": "100", "d": "g"}]"#, r#"[{"a": 100, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_negative_exponent>]() {
                TestHelper::new(r#"$[?@.a==1e-2]"#, r#"[{"a": 0.01, "d": "e"}, {"a": 0.02, "d": "f"}, {"a": "0.01", "d": "g"}]"#, r#"[{"a": 0.01, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_exponent_1>]() {
                TestHelper::new(r#"$[?@.a==1e0]"#, r#"[{"a": 1, "d": "e"}, {"a": 2, "d": "f"}, {"a": "1", "d": "g"}]"#, r#"[{"a": 1, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_exponent_2>]() {
                TestHelper::new(r#"$[?@.a==1e-0]"#, r#"[{"a": 1, "d": "e"}, {"a": 2, "d": "f"}, {"a": "1", "d": "g"}]"#, r#"[{"a": 1, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_exponent_plus0>]() {
                TestHelper::new(r#"$[?@.a==1e+0]"#, r#"[{"a": 1, "d": "e"}, {"a": 2, "d": "f"}, {"a": "1", "d": "g"}]"#, r#"[{"a": 1, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_exponent_leading_0>]() {
                TestHelper::new(r#"$[?@.a==1e-02]"#, r#"[{"a": 0.01, "d": "e"}, {"a": 0.02, "d": "f"}, {"a": "0.01", "d": "g"}]"#, r#"[{"a": 0.01, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_exponent_plus00>]() {
                TestHelper::new(r#"$[?@.a==1e+00]"#, r#"[{"a": 1, "d": "e"}, {"a": 2, "d": "f"}, {"a": "1", "d": "g"}]"#, r#"[{"a": 1, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_decimal_fraction>]() {
                TestHelper::new(r#"$[?@.a==1.1]"#, r#"[{"a": 1.1, "d": "e"}, {"a": 1, "d": "f"}, {"a": "1.1", "d": "g"}]"#, r#"[{"a": 1.1, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_decimal_fraction_trailing_0>]() {
                TestHelper::new(r#"$[?@.a==1.10]"#, r#"[{"a": 1.1, "d": "e"}, {"a": 1, "d": "f"}, {"a": "1.1", "d": "g"}]"#, r#"[{"a": 1.1, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_decimal_fraction_exponent>]() {
                TestHelper::new(r#"$[?@.a==1.1e2]"#, r#"[{"a": 110, "d": "e"}, {"a": 110.1, "d": "f"}, {"a": "110", "d": "g"}]"#, r#"[{"a": 110, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_decimal_fraction_positive_exponent>]() {
                TestHelper::new(r#"$[?@.a==1.1e+2]"#, r#"[{"a": 110, "d": "e"}, {"a": 110.1, "d": "f"}, {"a": "110", "d": "g"}]"#, r#"[{"a": 110, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_number_decimal_fraction_negative_exponent>]() {
                TestHelper::new(r#"$[?@.a==1.1e-2]"#, r#"[{"a": 0.011, "d": "e"}, {"a": 0.012, "d": "f"}, {"a": "0.011", "d": "g"}]"#, r#"[{"a": 0.011, "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_equals_special_nothing>]() {
                TestHelper::new(r#"$.values[?length(@.a) == value($..c)]"#, r#"{"c": "cd", "values": [{"a": "ab"}, {"c": "d"}, {"a": null}]}"#, r#"[{"c": "d"}, {"a": null}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_equals_empty_node_list_and_empty_node_list>]() {
                TestHelper::new(r#"$[?@.a == @.b]"#, r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"c": 3}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _filter_equals_empty_node_list_and_special_nothing>]() {
                TestHelper::new(r#"$[?@.a == length(@.b)]"#, r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"b": 2}, {"c": 3}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_object_data>]() {
                TestHelper::new(r#"$[?@<3]"#, r#"{"a": 1, "b": 2, "c": 3}"#, r#"[1, 2]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_and_binds_more_tightly_than_or>]() {
                TestHelper::new(r#"$[?@.a || @.b && @.c]"#, r#"[{"a": 1}, {"b": 2, "c": 3}, {"c": 3}, {"b": 2}, {"a": 1, "b": 2, "c": 3}]"#, r#"[{"a": 1}, {"b": 2, "c": 3}, {"a": 1, "b": 2, "c": 3}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_left_to_right_evaluation>]() {
                TestHelper::new(r#"$[?@.a && @.b || @.c]"#, r#"[{"a": 1}, {"b": 2}, {"a": 1, "b": 2}, {"a": 1, "c": 3}, {"b": 1, "c": 3}, {"c": 3}, {"a": 1, "b": 2, "c": 3}]"#, r#"[{"a": 1, "b": 2}, {"a": 1, "c": 3}, {"b": 1, "c": 3}, {"c": 3}, {"a": 1, "b": 2, "c": 3}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_group_terms_left>]() {
                TestHelper::new(r#"$[?(@.a || @.b) && @.c]"#, r#"[{"a": 1, "b": 2}, {"a": 1, "c": 3}, {"b": 2, "c": 3}, {"a": 1}, {"b": 2}, {"c": 3}, {"a": 1, "b": 2, "c": 3}]"#, r#"[{"a": 1, "c": 3}, {"b": 2, "c": 3}, {"a": 1, "b": 2, "c": 3}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_group_terms_right>]() {
                TestHelper::new(r#"$[?@.a && (@.b || @.c)]"#, r#"[{"a": 1}, {"a": 1, "b": 2}, {"a": 1, "c": 2}, {"b": 2}, {"c": 2}, {"a": 1, "b": 2, "c": 3}]"#, r#"[{"a": 1, "b": 2}, {"a": 1, "c": 2}, {"a": 1, "b": 2, "c": 3}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_string_literal_single_quote_in_double_quotes>]() {
                TestHelper::new(r#"$[?@ == "quoted' literal"]"#, r#"["quoted' literal", "a", "quoted\\' literal"]"#, r#"["quoted' literal"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_string_literal_double_quote_in_single_quotes>]() {
                TestHelper::new(r#"$[?@ == 'quoted" literal']"#, r#"["quoted\" literal", "a", "quoted\\\" literal", "'quoted\" literal'"]"#, r#"["quoted\" literal"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_string_literal_escaped_single_quote_in_single_quotes>]() {
                TestHelper::new(r#"$[?@ == 'quoted\' literal']"#, r#"["quoted' literal", "a", "quoted\\' literal", "'quoted\" literal'"]"#, r#"["quoted' literal"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _filter_string_literal_escaped_double_quote_in_double_quotes>]() {
                TestHelper::new(r#"$[?@ == "quoted\" literal"]"#, r#"["quoted\" literal", "a", "quoted\\\" literal", "'quoted\" literal'"]"#, r#"["quoted\" literal"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _index_selector_first_element>]() {
                TestHelper::new(r#"$[0]"#, r#"["first", "second"]"#, r#"["first"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _index_selector_second_element>]() {
                TestHelper::new(r#"$[1]"#, r#"["first", "second"]"#, r#"["second"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _index_selector_out_of_bound>]() {
                TestHelper::new(r#"$[2]"#, r#"["first", "second"]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _index_selector_min_exact_index>]() {
                TestHelper::new(r#"$[-9007199254740991]"#, r#"["first", "second"]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _index_selector_max_exact_index>]() {
                TestHelper::new(r#"$[9007199254740991]"#, r#"["first", "second"]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _index_selector_negative>]() {
                TestHelper::new(r#"$[-1]"#, r#"["first", "second"]"#, r#"["second"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _index_selector_more_negative>]() {
                TestHelper::new(r#"$[-2]"#, r#"["first", "second"]"#, r#"["first"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _index_selector_negative_out_of_bound>]() {
                TestHelper::new(r#"$[-3]"#, r#"["first", "second"]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _index_selector_on_object>]() {
                TestHelper::new(r#"$[0]"#, r#"{"foo": 1}"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes>]() {
                TestHelper::new(r#"$["a"]"#, r#"{"a": "A", "b": "B"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_absent_data>]() {
                TestHelper::new(r#"$["c"]"#, r#"{"a": "A", "b": "B"}"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_array_data>]() {
                TestHelper::new(r#"$["a"]"#, r#"["first", "second"]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_embedded_uplus0020>]() {
                TestHelper::new(r#"$[" "]"#, r#"{" ": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_embedded_uplus007f>]() {
                TestHelper::new(r#"$[""]"#, r#"{"": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_supplementary_plane_character>]() {
                TestHelper::new(r#"$["𝄞"]"#, r#"{"𝄞": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_double_quote>]() {
                TestHelper::new(r#"$["\""]"#, r#"{"\"": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_reverse_solidus>]() {
                TestHelper::new(r#"$["\\"]"#, r#"{"\\": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_solidus>]() {
                TestHelper::new(r#"$["\/"]"#, r#"{"/": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_backspace>]() {
                TestHelper::new(r#"$["\b"]"#, r#"{"\b": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_form_feed>]() {
                TestHelper::new(r#"$["\f"]"#, r#"{"\f": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_line_feed>]() {
                TestHelper::new(r#"$["\n"]"#, r#"{"\n": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_carriage_return>]() {
                TestHelper::new(r#"$["\r"]"#, r#"{"\r": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_tab>]() {
                TestHelper::new(r#"$["\t"]"#, r#"{"\t": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_u263a_upper_case_hex>]() {
                TestHelper::new(r#"$["\u263A"]"#, r#"{"☺": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_escaped_u263a_lower_case_hex>]() {
                TestHelper::new(r#"$["\u263a"]"#, r#"{"☺": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_surrogate_pair_u0001d11e>]() {
                TestHelper::new(r#"$["\uD834\uDD1E"]"#, r#"{"𝄞": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_surrogate_pair_u0001f600>]() {
                TestHelper::new(r#"$["\uD83D\uDE00"]"#, r#"{"😀": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_before_high_surrogates>]() {
                TestHelper::new(r#"$["\uD7FF\uD7FF"]"#, r#"{"퟿퟿": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_after_low_surrogates>]() {
                TestHelper::new(r#"$["\uE000\uE000"]"#, r#"{"": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes>]() {
                TestHelper::new(r#"$['a']"#, r#"{"a": "A", "b": "B"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_absent_data>]() {
                TestHelper::new(r#"$['c']"#, r#"{"a": "A", "b": "B"}"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_array_data>]() {
                TestHelper::new(r#"$['a']"#, r#"["first", "second"]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_embedded_uplus0020>]() {
                TestHelper::new(r#"$[' ']"#, r#"{" ": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_single_quote>]() {
                TestHelper::new(r#"$['\'']"#, r#"{"'": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_reverse_solidus>]() {
                TestHelper::new(r#"$['\\']"#, r#"{"\\": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_solidus>]() {
                TestHelper::new(r#"$['\/']"#, r#"{"/": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_backspace>]() {
                TestHelper::new(r#"$['\b']"#, r#"{"\b": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_form_feed>]() {
                TestHelper::new(r#"$['\f']"#, r#"{"\f": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_line_feed>]() {
                TestHelper::new(r#"$['\n']"#, r#"{"\n": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_carriage_return>]() {
                TestHelper::new(r#"$['\r']"#, r#"{"\r": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_tab>]() {
                TestHelper::new(r#"$['\t']"#, r#"{"\t": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_u263a_upper_case_hex>]() {
                TestHelper::new(r#"$['\u263A']"#, r#"{"☺": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_escaped_u263a_lower_case_hex>]() {
                TestHelper::new(r#"$['\u263a']"#, r#"{"☺": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_surrogate_pair_u0001d11e>]() {
                TestHelper::new(r#"$['\uD834\uDD1E']"#, r#"{"𝄞": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_surrogate_pair_u0001f600>]() {
                TestHelper::new(r#"$['\uD83D\uDE00']"#, r#"{"😀": "A"}"#, r#"["A"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_double_quotes_empty>]() {
                TestHelper::new(r#"$[""]"#, r#"{"a": "A", "b": "B", "": "C"}"#, r#"["C"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _name_selector_single_quotes_empty>]() {
                TestHelper::new(r#"$['']"#, r#"{"a": "A", "b": "B", "": "C"}"#, r#"["C"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_slice_selector>]() {
                TestHelper::new(r#"$[1:3]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[1, 2]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_slice_selector_with_step>]() {
                TestHelper::new(r#"$[1:6:2]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[1, 3, 5]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_slice_selector_with_everything_omitted_short_form>]() {
                TestHelper::new(r#"$[:]"#, r#"[0, 1, 2, 3]"#, r#"[0, 1, 2, 3]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_slice_selector_with_everything_omitted_long_form>]() {
                TestHelper::new(r#"$[::]"#, r#"[0, 1, 2, 3]"#, r#"[0, 1, 2, 3]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_slice_selector_with_start_omitted>]() {
                TestHelper::new(r#"$[:2]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[0, 1]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_slice_selector_with_start_and_end_omitted>]() {
                TestHelper::new(r#"$[::2]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[0, 2, 4, 6, 8]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_step_with_default_start_and_end>]() {
                TestHelper::new(r#"$[::-1]"#, r#"[0, 1, 2, 3]"#, r#"[3, 2, 1, 0]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_step_with_default_start>]() {
                TestHelper::new(r#"$[:0:-1]"#, r#"[0, 1, 2, 3]"#, r#"[3, 2, 1]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_step_with_default_end>]() {
                TestHelper::new(r#"$[2::-1]"#, r#"[0, 1, 2, 3]"#, r#"[2, 1, 0]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_larger_negative_step>]() {
                TestHelper::new(r#"$[::-2]"#, r#"[0, 1, 2, 3]"#, r#"[3, 1]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_range_with_default_step>]() {
                TestHelper::new(r#"$[-1:-3]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_range_with_negative_step>]() {
                TestHelper::new(r#"$[-1:-3:-1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[9, 8]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_range_with_larger_negative_step>]() {
                TestHelper::new(r#"$[-1:-6:-2]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[9, 7, 5]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_larger_negative_range_with_larger_negative_step>]() {
                TestHelper::new(r#"$[-1:-7:-2]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[9, 7, 5]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_from_positive_to>]() {
                TestHelper::new(r#"$[-5:7]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[5, 6]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_from>]() {
                TestHelper::new(r#"$[-2:]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[8, 9]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_positive_from_negative_to>]() {
                TestHelper::new(r#"$[1:-1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[1, 2, 3, 4, 5, 6, 7, 8]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_from_positive_to_negative_step>]() {
                TestHelper::new(r#"$[-1:1:-1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[9, 8, 7, 6, 5, 4, 3, 2]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_positive_from_negative_to_negative_step>]() {
                TestHelper::new(r#"$[7:-5:-1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[7, 6]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_zero_step>]() {
                TestHelper::new(r#"$[1:2:0]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_empty_range>]() {
                TestHelper::new(r#"$[2:2]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_slice_selector_with_everything_omitted_with_empty_array>]() {
                TestHelper::new(r#"$[:]"#, r#"[]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_negative_step_with_empty_array>]() {
                TestHelper::new(r#"$[::-1]"#, r#"[]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_maximal_range_with_positive_step>]() {
                TestHelper::new(r#"$[0:10]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_maximal_range_with_negative_step>]() {
                TestHelper::new(r#"$[9:0:-1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[9, 8, 7, 6, 5, 4, 3, 2, 1]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_excessively_large_to_value>]() {
                TestHelper::new(r#"$[2:113667776004]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[2, 3, 4, 5, 6, 7, 8, 9]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_excessively_small_from_value>]() {
                TestHelper::new(r#"$[-113667776004:1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[0]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_excessively_large_from_value_with_negative_step>]() {
                TestHelper::new(r#"$[113667776004:0:-1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[9, 8, 7, 6, 5, 4, 3, 2, 1]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_excessively_small_to_value_with_negative_step>]() {
                TestHelper::new(r#"$[3:-113667776004:-1]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[3, 2, 1, 0]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_excessively_large_step>]() {
                TestHelper::new(r#"$[1:10:113667776004]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[1]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_excessively_small_step>]() {
                TestHelper::new(r#"$[-1:-10:-113667776004]"#, r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#, r#"[9]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_start_min_exact>]() {
                TestHelper::new(r#"$[-9007199254740991::]"#, r#"[]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_start_max_exact>]() {
                TestHelper::new(r#"$[9007199254740991::]"#, r#"[]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_end_min_exact>]() {
                TestHelper::new(r#"$[:-9007199254740991:]"#, r#"[]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_end_max_exact>]() {
                TestHelper::new(r#"$[:9007199254740991:]"#, r#"[]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_step_min_exact>]() {
                TestHelper::new(r#"$[::-9007199254740991]"#, r#"[]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _slice_selector_step_max_exact>]() {
                TestHelper::new(r#"$[::9007199254740991]"#, r#"[]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_count_count_function>]() {
                TestHelper::new(r#"$[?count(@..*)>2]"#, r#"[{"a": [1, 2, 3]}, {"a": [1], "d": "f"}, {"a": 1, "d": "f"}]"#, r#"[{"a": [1, 2, 3]}, {"a": [1], "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_count_single_node_arg>]() {
                TestHelper::new(r#"$[?count(@.a)>1]"#, r#"[{"a": [1, 2, 3]}, {"a": [1], "d": "f"}, {"a": 1, "d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_count_multiple_selector_arg>]() {
                TestHelper::new(r#"$[?count(@['a','d'])>1]"#, r#"[{"a": [1, 2, 3]}, {"a": [1], "d": "f"}, {"a": 1, "d": "f"}]"#, r#"[{"a": [1], "d": "f"}, {"a": 1, "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_string_data>]() {
                TestHelper::new(r#"$[?length(@.a)>=2]"#, r#"[{"a": "ab"}, {"a": "d"}]"#, r#"[{"a": "ab"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_string_data_unicode>]() {
                TestHelper::new(r#"$[?length(@)==2]"#, r#"["☺", "☺☺", "☺☺☺", "ж", "жж", "жжж", "磨", "阿美", "形声字"]"#, r#"["☺☺", "жж", "阿美"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_array_data>]() {
                TestHelper::new(r#"$[?length(@.a)>=2]"#, r#"[{"a": [1, 2, 3]}, {"a": [1]}]"#, r#"[{"a": [1, 2, 3]}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_missing_data>]() {
                TestHelper::new(r#"$[?length(@.a)>=2]"#, r#"[{"d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_number_arg>]() {
                TestHelper::new(r#"$[?length(1)>=2]"#, r#"[{"d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_true_arg>]() {
                TestHelper::new(r#"$[?length(true)>=2]"#, r#"[{"d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_false_arg>]() {
                TestHelper::new(r#"$[?length(false)>=2]"#, r#"[{"d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_null_arg>]() {
                TestHelper::new(r#"$[?length(null)>=2]"#, r#"[{"d": "f"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_arg_is_a_function_expression>]() {
                TestHelper::new(r#"$.values[?length(@.a)==length(value($..c))]"#, r#"{"c": "cd", "values": [{"a": "ab"}, {"a": "d"}]}"#, r#"[{"a": "ab"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_length_arg_is_special_nothing>]() {
                TestHelper::new(r#"$[?length(value(@.a))>0]"#, r#"[{"a": "ab"}, {"c": "d"}, {"a": null}]"#, r#"[{"a": "ab"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_found_match>]() {
                TestHelper::new(r#"$[?match(@.a, 'a.*')]"#, r#"[{"a": "ab"}]"#, r#"[{"a": "ab"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_double_quotes>]() {
                TestHelper::new(r#"$[?match(@.a, "a.*")]"#, r#"[{"a": "ab"}]"#, r#"[{"a": "ab"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_regex_from_the_document>]() {
                TestHelper::new(r#"$.values[?match(@, $.regex)]"#, r#"{"regex": "b.?b", "values": ["abc", "bcd", "bab", "bba", "bbab", "b", true, [], {}]}"#, r#"["bab"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_dont_select_match>]() {
                TestHelper::new(r#"$[?!match(@.a, 'a.*')]"#, r#"[{"a": "ab"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_not_a_match>]() {
                TestHelper::new(r#"$[?match(@.a, 'a.*')]"#, r#"[{"a": "bc"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_select_non_match>]() {
                TestHelper::new(r#"$[?!match(@.a, 'a.*')]"#, r#"[{"a": "bc"}]"#, r#"[{"a": "bc"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_non_string_first_arg>]() {
                TestHelper::new(r#"$[?match(1, 'a.*')]"#, r#"[{"a": "bc"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_non_string_second_arg>]() {
                TestHelper::new(r#"$[?match(@.a, 1)]"#, r#"[{"a": "bc"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_filter_match_function_unicode_char_class_uppercase>]() {
                TestHelper::new(r#"$[?match(@, '\\p{Lu}')]"#, r#"["ж", "Ж", "1", "жЖ", true, [], {}]"#, r#"["Ж"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_filter_match_function_unicode_char_class_negated_uppercase>]() {
                TestHelper::new(r#"$[?match(@, '\\P{Lu}')]"#, r#"["ж", "Ж", "1", true, [], {}]"#, r#"["ж", "1"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_filter_match_function_unicode_surrogate_pair>]() {
                TestHelper::new(r#"$[?match(@, 'a.b')]"#, r#"["a𐄁b", "ab", "1", true, [], {}]"#, r#"["a𐄁b"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_dot_matcher_on_u2028>]() {
                TestHelper::new(r#"$[?match(@, '.')]"#, r#"[" ", "\r", "\n", true, [], {}]"#, r#"[" "]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_dot_matcher_on_u2029>]() {
                TestHelper::new(r#"$[?match(@, '.')]"#, r#"[" ", "\r", "\n", true, [], {}]"#, r#"[" "]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_arg_is_a_function_expression>]() {
                TestHelper::new(r#"$.values[?match(@.a, value($..['regex']))]"#, r#"{"regex": "a.*", "values": [{"a": "ab"}, {"a": "ba"}]}"#, r#"[{"a": "ab"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_dot_in_character_class>]() {
                TestHelper::new(r#"$[?match(@, 'a[.b]c')]"#, r#"["abc", "a.c", "axc"]"#, r#"["abc", "a.c"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_escaped_dot>]() {
                TestHelper::new(r#"$[?match(@, 'a\\.c')]"#, r#"["abc", "a.c", "axc"]"#, r#"["a.c"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_escaped_backslash_before_dot>]() {
                TestHelper::new(r#"$[?match(@, 'a\\\\.c')]"#, r#"["abc", "a.c", "axc", "a\\ c"]"#, r#"["a\\ c"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_escaped_left_square_bracket>]() {
                TestHelper::new(r#"$[?match(@, 'a\\[.c')]"#, r#"["abc", "a.c", "a[ c"]"#, r#"["a[ c"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_escaped_right_square_bracket>]() {
                TestHelper::new(r#"$[?match(@, 'a[\\].]c')]"#, r#"["abc", "a.c", "a c", "a]c"]"#, r#"["a.c", "a]c"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_explicit_caret>]() {
                TestHelper::new(r#"$[?match(@, '^ab.*')]"#, r#"["abc", "axc", "ab", "xab"]"#, r#"["abc", "ab"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_match_explicit_dollar>]() {
                TestHelper::new(r#"$[?match(@, '.*bc$')]"#, r#"["abc", "axc", "ab", "abcx"]"#, r#"["abc"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_at_the_end>]() {
                TestHelper::new(r#"$[?search(@.a, 'a.*')]"#, r#"[{"a": "the end is ab"}]"#, r#"[{"a": "the end is ab"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_double_quotes>]() {
                TestHelper::new(r#"$[?search(@.a, "a.*")]"#, r#"[{"a": "the end is ab"}]"#, r#"[{"a": "the end is ab"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_at_the_start>]() {
                TestHelper::new(r#"$[?search(@.a, 'a.*')]"#, r#"[{"a": "ab is at the start"}]"#, r#"[{"a": "ab is at the start"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_in_the_middle>]() {
                TestHelper::new(r#"$[?search(@.a, 'a.*')]"#, r#"[{"a": "contains two matches"}]"#, r#"[{"a": "contains two matches"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_regex_from_the_document>]() {
                TestHelper::new(r#"$.values[?search(@, $.regex)]"#, r#"{"regex": "b.?b", "values": ["abc", "bcd", "bab", "bba", "bbab", "b", true, [], {}]}"#, r#"["bab", "bba", "bbab"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_dont_select_match>]() {
                TestHelper::new(r#"$[?!search(@.a, 'a.*')]"#, r#"[{"a": "contains two matches"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_not_a_match>]() {
                TestHelper::new(r#"$[?search(@.a, 'a.*')]"#, r#"[{"a": "bc"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_select_non_match>]() {
                TestHelper::new(r#"$[?!search(@.a, 'a.*')]"#, r#"[{"a": "bc"}]"#, r#"[{"a": "bc"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_non_string_first_arg>]() {
                TestHelper::new(r#"$[?search(1, 'a.*')]"#, r#"[{"a": "bc"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_non_string_second_arg>]() {
                TestHelper::new(r#"$[?search(@.a, 1)]"#, r#"[{"a": "bc"}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_filter_search_function_unicode_char_class_uppercase>]() {
                TestHelper::new(r#"$[?search(@, '\\p{Lu}')]"#, r#"["ж", "Ж", "1", "жЖ", true, [], {}]"#, r#"["Ж", "жЖ"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_filter_search_function_unicode_char_class_negated_uppercase>]() {
                TestHelper::new(r#"$[?search(@, '\\P{Lu}')]"#, r#"["ж", "Ж", "1", true, [], {}]"#, r#"["ж", "1"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_filter_search_function_unicode_surrogate_pair>]() {
                TestHelper::new(r#"$[?search(@, 'a.b')]"#, r#"["a𐄁bc", "abc", "1", true, [], {}]"#, r#"["a𐄁bc"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_dot_matcher_on_u2028>]() {
                TestHelper::new(r#"$[?search(@, '.')]"#, r#"[" ", "\r \n", "\r", "\n", true, [], {}]"#, r#"[" ", "\r \n"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_dot_matcher_on_u2029>]() {
                TestHelper::new(r#"$[?search(@, '.')]"#, r#"[" ", "\r \n", "\r", "\n", true, [], {}]"#, r#"[" ", "\r \n"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_arg_is_a_function_expression>]() {
                TestHelper::new(r#"$.values[?search(@, value($..['regex']))]"#, r#"{"regex": "b.?b", "values": ["abc", "bcd", "bab", "bba", "bbab", "b", true, [], {}]}"#, r#"["bab", "bba", "bbab"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_dot_in_character_class>]() {
                TestHelper::new(r#"$[?search(@, 'a[.b]c')]"#, r#"["x abc y", "x a.c y", "x axc y"]"#, r#"["x abc y", "x a.c y"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_escaped_dot>]() {
                TestHelper::new(r#"$[?search(@, 'a\\.c')]"#, r#"["x abc y", "x a.c y", "x axc y"]"#, r#"["x a.c y"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_escaped_backslash_before_dot>]() {
                TestHelper::new(r#"$[?search(@, 'a\\\\.c')]"#, r#"["x abc y", "x a.c y", "x axc y", "x a\\ c y"]"#, r#"["x a\\ c y"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_escaped_left_square_bracket>]() {
                TestHelper::new(r#"$[?search(@, 'a\\[.c')]"#, r#"["x abc y", "x a.c y", "x a[ c y"]"#, r#"["x a[ c y"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_search_escaped_right_square_bracket>]() {
                TestHelper::new(r#"$[?search(@, 'a[\\].]c')]"#, r#"["x abc y", "x a.c y", "x a c y", "x a]c y"]"#, r#"["x a.c y", "x a]c y"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_value_single_value_nodelist>]() {
                TestHelper::new(r#"$[?value(@.*)==4]"#, r#"[[4], {"foo": 4}, [5], {"foo": 5}, 4]"#, r#"[[4], {"foo": 4}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _functions_value_multi_value_nodelist>]() {
                TestHelper::new(r#"$[?value(@.*)==4]"#, r#"[[4, 4], {"foo": 4, "bar": 4}]"#, r#"[]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_space_between_question_mark_and_expression>]() {
                TestHelper::new("$[? @.a]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_newline_between_question_mark_and_expression>]() {
                TestHelper::new("$[?\n@.a]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_tab_between_question_mark_and_expression>]() {
                TestHelper::new("$[?\t@.a]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_return_between_question_mark_and_expression>]() {
                TestHelper::new("$[?\r@.a]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_space_between_question_mark_and_parenthesized_expression>]() {
                TestHelper::new("$[? (@.a)]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_newline_between_question_mark_and_parenthesized_expression>]() {
                TestHelper::new("$[?\n(@.a)]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_tab_between_question_mark_and_parenthesized_expression>]() {
                TestHelper::new("$[?\t(@.a)]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_return_between_question_mark_and_parenthesized_expression>]() {
                TestHelper::new("$[?\r(@.a)]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_space_between_parenthesized_expression_and_bracket>]() {
                TestHelper::new("$[?(@.a) ]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_newline_between_parenthesized_expression_and_bracket>]() {
                TestHelper::new("$[?(@.a)\n]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_tab_between_parenthesized_expression_and_bracket>]() {
                TestHelper::new("$[?(@.a)\t]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_return_between_parenthesized_expression_and_bracket>]() {
                TestHelper::new("$[?(@.a)\r]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_space_between_bracket_and_question_mark>]() {
                TestHelper::new("$[ ?@.a]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_newline_between_bracket_and_question_mark>]() {
                TestHelper::new("$[\n?@.a]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_tab_between_bracket_and_question_mark>]() {
                TestHelper::new("$[\t?@.a]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_filter_return_between_bracket_and_question_mark>]() {
                TestHelper::new("$[\r?@.a]", r#"[{"a": "b", "d": "e"}, {"b": "c", "d": "f"}]"#, r#"[{"a": "b", "d": "e"}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_space_between_parenthesis_and_arg>]() {
                TestHelper::new("$[?count( @.*)==1]", r#"[{"a": 1}, {"b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_newline_between_parenthesis_and_arg>]() {
                TestHelper::new("$[?count(\n@.*)==1]", r#"[{"a": 1}, {"b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_tab_between_parenthesis_and_arg>]() {
                TestHelper::new("$[?count(\t@.*)==1]", r#"[{"a": 1}, {"b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_return_between_parenthesis_and_arg>]() {
                TestHelper::new("$[?count(\r@.*)==1]", r#"[{"a": 1}, {"b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_space_between_arg_and_comma>]() {
                TestHelper::new("$[?search(@ ,'[a-z]+')]", r#"["foo", "123"]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_newline_between_arg_and_comma>]() {
                TestHelper::new("$[?search(@\n,'[a-z]+')]", r#"["foo", "123"]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_tab_between_arg_and_comma>]() {
                TestHelper::new("$[?search(@\t,'[a-z]+')]", r#"["foo", "123"]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_return_between_arg_and_comma>]() {
                TestHelper::new("$[?search(@\r,'[a-z]+')]", r#"["foo", "123"]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_space_between_comma_and_arg>]() {
                TestHelper::new("$[?search(@, '[a-z]+')]", r#"["foo", "123"]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_newline_between_comma_and_arg>]() {
                TestHelper::new("$[?search(@,\n'[a-z]+')]", r#"["foo", "123"]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_tab_between_comma_and_arg>]() {
                TestHelper::new("$[?search(@,\t'[a-z]+')]", r#"["foo", "123"]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_return_between_comma_and_arg>]() {
                TestHelper::new("$[?search(@,\r'[a-z]+')]", r#"["foo", "123"]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_space_between_arg_and_parenthesis>]() {
                TestHelper::new("$[?count(@.* )==1]", r#"[{"a": 1}, {"b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_newline_between_arg_and_parenthesis>]() {
                TestHelper::new("$[?count(@.*\n)==1]", r#"[{"a": 1}, {"b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_tab_between_arg_and_parenthesis>]() {
                TestHelper::new("$[?count(@.*\t)==1]", r#"[{"a": 1}, {"b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_return_between_arg_and_parenthesis>]() {
                TestHelper::new("$[?count(@.*\r)==1]", r#"[{"a": 1}, {"b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_spaces_in_a_relative_singular_selector>]() {
                TestHelper::new("$[?length(@ .a .b) == 3]", r#"[{"a": {"b": "foo"}}, {}]"#, r#"[{"a": {"b": "foo"}}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_newlines_in_a_relative_singular_selector>]() {
                TestHelper::new("$[?length(@\n.a\n.b) == 3]", r#"[{"a": {"b": "foo"}}, {}]"#, r#"[{"a": {"b": "foo"}}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_tabs_in_a_relative_singular_selector>]() {
                TestHelper::new("$[?length(@\t.a\t.b) == 3]", r#"[{"a": {"b": "foo"}}, {}]"#, r#"[{"a": {"b": "foo"}}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_returns_in_a_relative_singular_selector>]() {
                TestHelper::new("$[?length(@\r.a\r.b) == 3]", r#"[{"a": {"b": "foo"}}, {}]"#, r#"[{"a": {"b": "foo"}}]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_spaces_in_an_absolute_singular_selector>]() {
                TestHelper::new("$..[?length(@)==length($ [0] .a)]", r#"[{"a": "foo"}, {}]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_newlines_in_an_absolute_singular_selector>]() {
                TestHelper::new("$..[?length(@)==length($\n[0]\n.a)]", r#"[{"a": "foo"}, {}]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_tabs_in_an_absolute_singular_selector>]() {
                TestHelper::new("$..[?length(@)==length($\t[0]\t.a)]", r#"[{"a": "foo"}, {}]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_functions_returns_in_an_absolute_singular_selector>]() {
                TestHelper::new("$..[?length(@)==length($\r[0]\r.a)]", r#"[{"a": "foo"}, {}]"#, r#"["foo"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_before_or>]() {
                TestHelper::new("$[?@.a ||@.b]", r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_before_or>]() {
                TestHelper::new("$[?@.a\n||@.b]", r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_before_or>]() {
                TestHelper::new("$[?@.a\t||@.b]", r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_before_or>]() {
                TestHelper::new("$[?@.a\r||@.b]", r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_after_or>]() {
                TestHelper::new("$[?@.a|| @.b]", r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_after_or>]() {
                TestHelper::new("$[?@.a||\n@.b]", r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_after_or>]() {
                TestHelper::new("$[?@.a||\t@.b]", r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_after_or>]() {
                TestHelper::new("$[?@.a||\r@.b]", r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#, r#"[{"a": 1}, {"b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_before_and>]() {
                TestHelper::new("$[?@.a &&@.b]", r#"[{"a": 1}, {"b": 2}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_before_and>]() {
                TestHelper::new("$[?@.a\n&&@.b]", r#"[{"a": 1}, {"b": 2}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_before_and>]() {
                TestHelper::new("$[?@.a\t&&@.b]", r#"[{"a": 1}, {"b": 2}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_before_and>]() {
                TestHelper::new("$[?@.a\r&&@.b]", r#"[{"a": 1}, {"b": 2}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_after_and>]() {
                TestHelper::new("$[?@.a&& @.b]", r#"[{"a": 1}, {"b": 2}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_after_and>]() {
                TestHelper::new("$[?@.a&& @.b]", r#"[{"a": 1}, {"b": 2}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_after_and>]() {
                TestHelper::new("$[?@.a&& @.b]", r#"[{"a": 1}, {"b": 2}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_after_and>]() {
                TestHelper::new("$[?@.a&& @.b]", r#"[{"a": 1}, {"b": 2}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_before_eq>]() {
                TestHelper::new("$[?@.a ==@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 1}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_before_eq>]() {
                TestHelper::new("$[?@.a\n==@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 1}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_before_eq>]() {
                TestHelper::new("$[?@.a\t==@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 1}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_before_eq>]() {
                TestHelper::new("$[?@.a\r==@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 1}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_after_eq>]() {
                TestHelper::new("$[?@.a== @.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 1}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_after_eq>]() {
                TestHelper::new("$[?@.a==\n@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 1}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_after_eq>]() {
                TestHelper::new("$[?@.a==\t@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 1}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_after_eq>]() {
                TestHelper::new("$[?@.a==\r@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 1}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_before_ne>]() {
                TestHelper::new("$[?@.a !=@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_before_ne>]() {
                TestHelper::new("$[?@.a\n!=@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_before_ne>]() {
                TestHelper::new("$[?@.a\t!=@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_before_ne>]() {
                TestHelper::new("$[?@.a\r!=@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_after_ne>]() {
                TestHelper::new("$[?@.a!= @.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_after_ne>]() {
                TestHelper::new("$[?@.a!=\n@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_after_ne>]() {
                TestHelper::new("$[?@.a!=\t@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_after_ne>]() {
                TestHelper::new("$[?@.a!=\r@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_before_lt>]() {
                TestHelper::new("$[?@.a <@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_before_lt>]() {
                TestHelper::new("$[?@.a\n<@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_before_lt>]() {
                TestHelper::new("$[?@.a\t<@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_before_lt>]() {
                TestHelper::new("$[?@.a\r<@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_after_lt>]() {
                TestHelper::new("$[?@.a< @.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_after_lt>]() {
                TestHelper::new("$[?@.a<\n@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_after_lt>]() {
                TestHelper::new("$[?@.a<\t@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_after_lt>]() {
                TestHelper::new("$[?@.a<\r@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_before_gt>]() {
                TestHelper::new("$[?@.b >@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_before_gt>]() {
                TestHelper::new("$[?@.b\n>@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_before_gt>]() {
                TestHelper::new("$[?@.b\t>@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_before_gt>]() {
                TestHelper::new("$[?@.b\r>@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_after_gt>]() {
                TestHelper::new("$[?@.b> @.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_after_gt>]() {
                TestHelper::new("$[?@.b>\n@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_after_gt>]() {
                TestHelper::new("$[?@.b>\t@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_after_gt>]() {
                TestHelper::new("$[?@.b>\r@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, r#"[{"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_before_lte>]() {
                TestHelper::new("$[?@.a <=@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_before_lte>]() {
                TestHelper::new("$[?@.a\n<=@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_before_lte>]() {
                TestHelper::new("$[?@.a\t<=@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_before_lte>]() {
                TestHelper::new("$[?@.a\r<=@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_after_lte>]() {
                TestHelper::new("$[?@.a<= @.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_after_lte>]() {
                TestHelper::new("$[?@.a<=\n@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_after_lte>]() {
                TestHelper::new("$[?@.a<=\t@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_after_lte>]() {
                TestHelper::new("$[?@.a<=\r@.b]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_before_gte>]() {
                TestHelper::new("$[?@.b >=@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_before_gte>]() {
                TestHelper::new("$[?@.b\n>=@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_before_gte>]() {
                TestHelper::new("$[?@.b\t>=@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_before_gte>]() {
                TestHelper::new("$[?@.b\r>=@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_after_gte>]() {
                TestHelper::new("$[?@.b>= @.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_after_gte>]() {
                TestHelper::new("$[?@.b>=\n@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_after_gte>]() {
                TestHelper::new("$[?@.b>=\t@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_after_gte>]() {
                TestHelper::new("$[?@.b>=\r@.a]", r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}, {"a": 2, "b": 1}]"#, r#"[{"a": 1, "b": 1}, {"a": 1, "b": 2}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_between_logical_not_and_test_expression>]() {
                TestHelper::new("$[?! @.a]", r#"[{"a": "a", "d": "e"}, {"d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_between_logical_not_and_test_expression>]() {
                TestHelper::new("$[?!\n@.a]", r#"[{"a": "a", "d": "e"}, {"d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_between_logical_not_and_test_expression>]() {
                TestHelper::new("$[?!\t@.a]", r#"[{"a": "a", "d": "e"}, {"d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_between_logical_not_and_test_expression>]() {
                TestHelper::new("$[?!\r@.a]", r#"[{"a": "a", "d": "e"}, {"d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_space_between_logical_not_and_parenthesized_expression>]() {
                TestHelper::new("$[?! (@.a=='b')]", r#"[{"a": "a", "d": "e"}, {"a": "b", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "a", "d": "e"}, {"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_newline_between_logical_not_and_parenthesized_expression>]() {
                TestHelper::new("$[?!\n(@.a=='b')]", r#"[{"a": "a", "d": "e"}, {"a": "b", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "a", "d": "e"}, {"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_tab_between_logical_not_and_parenthesized_expression>]() {
                TestHelper::new("$[?!\t(@.a=='b')]", r#"[{"a": "a", "d": "e"}, {"a": "b", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "a", "d": "e"}, {"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_operators_return_between_logical_not_and_parenthesized_expression>]() {
                TestHelper::new("$[?!\r(@.a=='b')]", r#"[{"a": "a", "d": "e"}, {"a": "b", "d": "f"}, {"a": "d", "d": "f"}]"#, r#"[{"a": "a", "d": "e"}, {"a": "d", "d": "f"}]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_space_between_root_and_bracket>]() {
                TestHelper::new("$ ['a']", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_newline_between_root_and_bracket>]() {
                TestHelper::new("$\n['a']", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_tab_between_root_and_bracket>]() {
                TestHelper::new("$\t['a']", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_return_between_root_and_bracket>]() {
                TestHelper::new("$\r['a']", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_space_between_bracket_and_bracket>]() {
                TestHelper::new("$['a'] ['b']", r#"{"a": {"b": "ab"}}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_newline_between_bracket_and_bracket>]() {
                TestHelper::new("$['a'] \n['b']", r#"{"a": {"b": "ab"}}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_tab_between_bracket_and_bracket>]() {
                TestHelper::new("$['a'] \t['b']", r#"{"a": {"b": "ab"}}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_return_between_bracket_and_bracket>]() {
                TestHelper::new("$['a'] \r['b']", r#"{"a": {"b": "ab"}}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_space_between_root_and_dot>]() {
                TestHelper::new("$ .a", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_newline_between_root_and_dot>]() {
                TestHelper::new("$\n.a", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_tab_between_root_and_dot>]() {
                TestHelper::new("$\t.a", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_return_between_root_and_dot>]() {
                TestHelper::new("$\r.a", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_space_between_bracket_and_selector>]() {
                TestHelper::new("$[ 'a']", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_newline_between_bracket_and_selector>]() {
                TestHelper::new("$[\n'a']", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_tab_between_bracket_and_selector>]() {
                TestHelper::new("$[\t'a']", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_return_between_bracket_and_selector>]() {
                TestHelper::new("$[\r'a']", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_space_between_selector_and_bracket>]() {
                TestHelper::new("$['a' ]", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_newline_between_selector_and_bracket>]() {
                TestHelper::new("$['a'\n]", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_tab_between_selector_and_bracket>]() {
                TestHelper::new("$['a'\t]", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_return_between_selector_and_bracket>]() {
                TestHelper::new("$['a'\r]", r#"{"a": "ab"}"#, r#"["ab"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_space_between_selector_and_comma>]() {
                TestHelper::new("$['a' ,'b']", r#"{"a": "ab", "b": "bc"}"#, r#"["ab", "bc"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_newline_between_selector_and_comma>]() {
                TestHelper::new("$['a'\n,'b']", r#"{"a": "ab", "b": "bc"}"#, r#"["ab", "bc"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_tab_between_selector_and_comma>]() {
                TestHelper::new("$['a'\t,'b']", r#"{"a": "ab", "b": "bc"}"#, r#"["ab", "bc"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_return_between_selector_and_comma>]() {
                TestHelper::new("$['a'\r,'b']", r#"{"a": "ab", "b": "bc"}"#, r#"["ab", "bc"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_space_between_comma_and_selector>]() {
                TestHelper::new("$['a', 'b']", r#"{"a": "ab", "b": "bc"}"#, r#"["ab", "bc"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_newline_between_comma_and_selector>]() {
                TestHelper::new("$['a',\n'b']", r#"{"a": "ab", "b": "bc"}"#, r#"["ab", "bc"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_tab_between_comma_and_selector>]() {
                TestHelper::new("$['a',\t'b']", r#"{"a": "ab", "b": "bc"}"#, r#"["ab", "bc"]"#, Target::$target).run()
            }

            #[test]
            fn [<$target:snake _whitespace_selectors_return_between_comma_and_selector>]() {
                TestHelper::new("$['a',\r'b']", r#"{"a": "ab", "b": "bc"}"#, r#"["ab", "bc"]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_space_between_start_and_colon>]() {
                TestHelper::new("$[1 :5:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_newline_between_start_and_colon>]() {
                TestHelper::new("$[1\n:5:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_tab_between_start_and_colon>]() {
                TestHelper::new("$[1\t:5:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_return_between_start_and_colon>]() {
                TestHelper::new("$[1\r:5:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_space_between_colon_and_end>]() {
                TestHelper::new("$[1: 5:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_newline_between_colon_and_end>]() {
                TestHelper::new("$[1:\n5:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_tab_between_colon_and_end>]() {
                TestHelper::new("$[1:\t5:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_return_between_colon_and_end>]() {
                TestHelper::new("$[1:\r5:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_space_between_end_and_colon>]() {
                TestHelper::new("$[1:5 :2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_newline_between_end_and_colon>]() {
                TestHelper::new("$[1:5\n:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_tab_between_end_and_colon>]() {
                TestHelper::new("$[1:5\t:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_return_between_end_and_colon>]() {
                TestHelper::new("$[1:5\r:2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_space_between_colon_and_step>]() {
                TestHelper::new("$[1:5: 2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_newline_between_colon_and_step>]() {
                TestHelper::new("$[1:5:\n2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_tab_between_colon_and_step>]() {
                TestHelper::new("$[1:5:\t2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }

            #[test]
            #[ignore]
            fn [<$target:snake _whitespace_slice_return_between_colon_and_step>]() {
                TestHelper::new("$[1:5:\r2]", r#"[1, 2, 3, 4, 5, 6]"#, r#"[2, 4]"#, Target::$target).run()
            }
        }
    }
}

cts!(SimdjsonOndemand);
//cts!(SimdjsonDom);