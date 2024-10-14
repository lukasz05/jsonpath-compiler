use crate::compiler::code_generator::CodeGenerator;
use crate::compiler::subquery_finder::{find_subqueries, SubqueryToCompile};
use crate::ir::{
    Comparable, ComparisonOp, Index, Literal, LogicalExpression, Query, Segment, Selector,
    SingularSelector, Slice,
};
use rsonpath_syntax::str::{escape, EscapeMode};
use std::collections::HashMap;

mod code_generator;
mod subquery_finder;

const INPUT_NODELIST_VARIABLE_NAME: &str = "input_nodelist";
const OUTPUT_NODELIST_VARIABLE_NAME: &str = "output_nodelist";

pub fn compile_query(query: &Query) -> String {
    let mut code_generator = CodeGenerator::new();
    let mut subqueries_to_compile = find_subqueries(query);
    generate_prologue(&mut code_generator, &subqueries_to_compile);
    compile_segments(
        &mut code_generator,
        &mut subqueries_to_compile,
        query,
        "\"\"",
    );
    generate_epilogue(&mut code_generator);
    generate_subquery_functions(&mut code_generator, &subqueries_to_compile);
    return code_generator.get_code();
}

fn generate_subquery_functions_declarations(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile>,
) {
    for subquery_function in subqueries_to_compile.values() {
        let function_name = &subquery_function.function_name;
        let function_signature = get_subquery_function_signature(function_name);
        code_generator.write_line(&format!("{function_signature};"));
    }
}

fn generate_subquery_functions(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile>,
) {
    for subquery_function in subqueries_to_compile.values() {
        code_generator.write_line("");
        let subquery = subquery_function.subquery;
        let function_name = &subquery_function.function_name;
        let function_signature = get_subquery_function_signature(function_name);
        code_generator.write_line(&function_signature);
        code_generator.start_block();
        compile_segments(
            code_generator,
            subqueries_to_compile,
            subquery,
            "initial_node_path",
        );
        code_generator.write_line(&format!("return !{OUTPUT_NODELIST_VARIABLE_NAME}.empty();"));
        code_generator.end_block();
    }
}

fn get_subquery_function_signature(function_name: &str) -> String {
    return format!("bool {function_name}(padded_string &json, string initial_node_path)");
}

fn generate_prologue(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile>,
) {
    code_generator.write_lines(&[
        //"#define SIMDJSON_VERBOSE_LOGGING 1",
        "",
        "#include <iostream>",
        "#include <vector>",
        "#include <queue>",
        "#include <string>",
        "#include <algorithm>",
        "#include <simdjson.h>",
        "#include \"helpers.h\"",
        "",
        "using namespace std;",
        "using namespace simdjson;",
    ]);
    generate_subquery_functions_declarations(code_generator, &subqueries_to_compile);
    code_generator.write_lines(&["", "int main()"]);
    code_generator.start_block();
    code_generator.write_lines(&[
        "string input(istreambuf_iterator<char>(cin), {});",
        "auto json = padded_string(input);",
    ]);
}

fn generate_epilogue(code_generator: &mut CodeGenerator) {
    code_generator.write_lines(&[
        "",
        r#"cout << "[\n";"#,
        "bool first = true;",
        &format!("for (string node_path : {OUTPUT_NODELIST_VARIABLE_NAME})"),
    ]);
    code_generator.start_block();
    code_generator.write_line("ondemand::value node;");
    code_generator.write_line("root_node.at_pointer(node_path).get(node);");
    code_generator.write_line("if (!first) cout << \",\\n\";");
    code_generator.write_line(r#"cout << node.raw_json() << "\n";"#);
    code_generator.write_line("first = false;");
    code_generator.end_block();
    code_generator.write_lines(&[r#"cout << "]\n";"#, "", "return 0;"]);
    code_generator.end_block();
}

fn compile_segments<'a>(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile<'a>>,
    query: &'a Query,
    initial_node_path_code: &str,
) {
    code_generator.write_lines(&[
        "ondemand::parser parser;",
        "auto root_node = parser.iterate(json);",
        &format!("vector<string> {INPUT_NODELIST_VARIABLE_NAME}{{{initial_node_path_code}}};"),
        &format!("vector<string> {OUTPUT_NODELIST_VARIABLE_NAME};"),
        "simdjson::error_code error;",
        "",
    ]);
    if query.segments.is_empty() {
        code_generator.write_line(&format!(
            "{OUTPUT_NODELIST_VARIABLE_NAME} = {INPUT_NODELIST_VARIABLE_NAME};"
        ));
        return;
    }
    compile_segment(
        code_generator,
        subqueries_to_compile,
        &query.segments[0],
        true,
    );
    for segment in query.segments.iter().skip(1) {
        compile_segment(code_generator, subqueries_to_compile, segment, false);
    }
}

fn compile_segment<'a>(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile<'a>>,
    segment: &'a Segment,
    first_segment: bool,
) {
    if !first_segment {
        code_generator.write_lines(&[
            "",
            &format!("{INPUT_NODELIST_VARIABLE_NAME}.swap({OUTPUT_NODELIST_VARIABLE_NAME});"),
            &format!("{OUTPUT_NODELIST_VARIABLE_NAME}.clear();"),
        ]);
    }
    code_generator.write_line(&format!(
        "for (string node_path : {INPUT_NODELIST_VARIABLE_NAME})"
    ));
    code_generator.start_block();
    match segment {
        Segment::ChildSegment { selectors } => {
            compile_selectors(code_generator, subqueries_to_compile, selectors, false);
        }
        Segment::DescendantSegment { selectors } => {
            code_generator.write_lines(&[
                "queue<string> node_paths;",
                "node_paths.push(node_path);",
                "while (!node_paths.empty())",
            ]);
            code_generator.start_block();
            code_generator.write_lines(&[
                "node_path = node_paths.front();",
                "node_paths.pop();",
                "",
            ]);
            compile_selectors(code_generator, subqueries_to_compile, selectors, true);

            if !is_object_selector_present(selectors) {
                start_object_processing(code_generator);
                start_object_fields_iteration(code_generator);
                code_generator.write_line("node_paths.push(node_path + \"/\" + escaped_key);");
                end_object_fields_iteration(code_generator);
                end_object_processing(code_generator);
            }

            if !is_array_selector_present(selectors) {
                start_array_processing(code_generator);
                start_array_elements_iteration(code_generator);
                code_generator.write_line("node_paths.push(node_path + \"/\" + to_string(i));");
                end_array_elements_iteration(code_generator, false);
                end_array_processing(code_generator);
            }

            code_generator.end_block();
        }
    }
    code_generator.end_block();
}

fn compile_selectors<'a>(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile<'a>>,
    selectors: &'a Vec<Selector>,
    is_descendant_segment: bool,
) {
    for i in 0..selectors.len() {
        code_generator.write_line(&format!(
            "vector<string> {};",
            get_selector_results_vector_name(i)
        ));
    }
    code_generator.write_line("");
    code_generator.write_lines(&[
        "ondemand::value node;",
        "error = root_node.at_pointer(node_path).get(node);",
        "if (error) continue;",
        "",
    ]);

    if is_object_selector_present(selectors) {
        compile_object_selectors(
            code_generator,
            subqueries_to_compile,
            selectors,
            is_descendant_segment,
        );
    }
    if is_array_selector_present(selectors) {
        compile_array_selectors(
            code_generator,
            subqueries_to_compile,
            selectors,
            is_descendant_segment,
        );
    }

    for i in 0..selectors.len() {
        let selector_results_vector_name = get_selector_results_vector_name(i);
        code_generator.write_line(&format!(
            "{OUTPUT_NODELIST_VARIABLE_NAME}.insert({OUTPUT_NODELIST_VARIABLE_NAME}.end(), \
            {selector_results_vector_name}.begin(), {selector_results_vector_name}.end());"
        ));
    }
}

fn is_object_selector_present(selectors: &Vec<Selector>) -> bool {
    for selector in selectors {
        match selector {
            Selector::ChildByName { .. } | Selector::AllChildren | Selector::Filter { .. } => {
                return true
            }
            _ => {}
        }
    }
    return false;
}

fn is_array_selector_present(selectors: &Vec<Selector>) -> bool {
    for selector in selectors {
        match selector {
            Selector::ElementAtIndex { .. }
            | Selector::Slice { .. }
            | Selector::AllChildren
            | Selector::Filter { .. } => return true,
            _ => {}
        }
    }
    return false;
}

fn is_slice_selector_present(selectors: Vec<Selector>) -> bool {
    for selector in selectors {
        if let Selector::Slice { .. } = selector {
            return true;
        }
    }
    return false;
}

fn compile_object_selectors<'a>(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile<'a>>,
    selectors: &'a Vec<Selector>,
    is_descendant_segment: bool,
) {
    start_object_processing(code_generator);
    start_object_fields_iteration(code_generator);
    for (i, selector) in selectors.iter().enumerate() {
        let child_path_code = "node_path + \"/\" + escaped_key";
        match selector {
            Selector::AllChildren => {
                generate_add_child_path_to_output(code_generator, i, child_path_code, false);
            }
            Selector::ChildByName { name } => {
                let name_str = &name.0;
                let escaped_name_str = escape(name_str, EscapeMode::DoubleQuoted);
                code_generator.write_line(&format!("if (key == \"{escaped_name_str}\")"));
                generate_add_child_path_to_output(code_generator, i, child_path_code, true);
            }
            Selector::Filter { logical_expression } => {
                compile_filter_selector(
                    code_generator,
                    subqueries_to_compile,
                    i,
                    logical_expression,
                    child_path_code,
                );
            }
            _ => {}
        }
    }
    if is_descendant_segment {
        code_generator.write_line("node_paths.push(node_path + \"/\" + escaped_key);");
    }
    end_object_fields_iteration(code_generator);
    end_object_processing(code_generator);
}

fn start_object_processing(code_generator: &mut CodeGenerator) {
    code_generator.write_lines(&[
        "ondemand::object obj;",
        "error = node.get_object().get(obj);",
        "if (!error)",
    ]);
    code_generator.start_block();
}

fn end_object_processing(code_generator: &mut CodeGenerator) {
    code_generator.end_block();
}

fn start_object_fields_iteration(code_generator: &mut CodeGenerator) {
    code_generator.write_line("for (ondemand::field field : obj)");
    code_generator.start_block();
    code_generator.write_lines(&[
        "string_view escaped_key_view = field.escaped_key();",
        "string escaped_key = get_jsonpointer_encoded_string(escaped_key_view);",
        "string_view key_view = field.unescaped_key();",
        "string key = string(key_view);",
    ]);
}

fn end_object_fields_iteration(code_generator: &mut CodeGenerator) {
    code_generator.end_block();
}

fn compile_array_selectors<'a>(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile<'a>>,
    selectors: &'a Vec<Selector>,
    is_descendant_segment: bool,
) {
    start_array_processing(code_generator);
    start_array_elements_iteration(code_generator);
    let child_path_code = "node_path + \"/\" + to_string(i)";
    for (selector_index, selector) in selectors.iter().enumerate() {
        match selector {
            Selector::AllChildren => {
                generate_add_child_path_to_output(
                    code_generator,
                    selector_index,
                    child_path_code,
                    false,
                );
            }

            Selector::Filter { logical_expression } => {
                compile_filter_selector(
                    code_generator,
                    subqueries_to_compile,
                    selector_index,
                    logical_expression,
                    child_path_code,
                );
            }
            _ => {}
        }
    }
    if is_descendant_segment {
        code_generator.write_line("node_paths.push(node_path + \"/\" + to_string(i));");
    }
    end_array_elements_iteration(code_generator, true);
    for (selector_index, selector) in selectors.iter().enumerate() {
        match selector {
            Selector::ElementAtIndex { index } => {
                compile_element_at_index_selector(code_generator, selector_index, index)
            }
            Selector::Slice { slice } => {
                compile_slice_selector(code_generator, selector_index, slice)
            }
            _ => {}
        }
    }
    end_array_processing(code_generator);
}

fn start_array_processing(code_generator: &mut CodeGenerator) {
    code_generator.write_lines(&[
        "ondemand::array arr;",
        "error = node.get_array().get(arr);",
        "if (!error)",
    ]);
    code_generator.start_block();
    code_generator.write_line("size_t i = 0;");
}

fn end_array_processing(code_generator: &mut CodeGenerator) {
    code_generator.end_block();
}

fn start_array_elements_iteration(code_generator: &mut CodeGenerator) {
    code_generator.write_line("for (ondemand::value element : arr)");
    code_generator.start_block();
}

fn end_array_elements_iteration(code_generator: &mut CodeGenerator, save_element_count: bool) {
    code_generator.write_line("i++;");
    code_generator.end_block();
    if save_element_count {
        code_generator.write_line("int64_t element_count = i;");
    }
}

fn compile_element_at_index_selector(
    code_generator: &mut CodeGenerator,
    slice_index: usize,
    index: &Index,
) {
    let index_val = index.0;
    if index_val < 0 {
        code_generator.write_line(&format!("if (element_count >= {})", -index_val));
        generate_add_child_path_to_output(
            code_generator,
            slice_index,
            &format!(
                "node_path + \"/\" + to_string(element_count - {})",
                -index_val
            ),
            true,
        );
    } else {
        code_generator.write_line(&format!("if (element_count > {})", index_val));
        generate_add_child_path_to_output(
            code_generator,
            slice_index,
            &format!("node_path + \"/\" + to_string({index_val})"),
            true,
        );
    }
}

fn compile_slice_selector(
    code_generator: &mut CodeGenerator,
    selector_index: usize,
    slice: &Slice,
) {
    let step = slice.step;
    if step == 0 {
        return;
    }
    code_generator.write_line("if (element_count > 0)");
    code_generator.start_block();
    let start_str = if slice.start >= 0 {
        &format!("min({}ll, element_count - 1)", slice.start)
    } else {
        &format!("max(element_count - {}, 0ll)", -slice.start)
    };
    let for_condition = match slice.end {
        Some(end) => {
            if end >= 0 {
                if step >= 0 {
                    &format!("i < min({end}ll, element_count)")
                } else {
                    &format!("i > {end}")
                }
            } else {
                if step >= 0 {
                    &format!("i < min(max(element_count - {}, 0ll), element_count)", -end)
                } else {
                    &format!("i > max(element_count - {}, -1ll)", -end)
                }
            }
        }
        None => {
            if step >= 0 {
                "i < element_count"
            } else {
                "i >= 0"
            }
        }
    };
    code_generator.write_line(&format!(
        "for (int64_t i = {start_str}; {for_condition}; i += {step})"
    ));
    code_generator.start_block();
    generate_add_child_path_to_output(
        code_generator,
        selector_index,
        "node_path + \"/\" + to_string(i)",
        false,
    );
    code_generator.end_block();
    code_generator.end_block();
}

fn compile_filter_selector<'a>(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile<'a>>,
    selector_index: usize,
    logical_expression: &'a LogicalExpression,
    child_path_code: &str,
) {
    code_generator.indent();
    code_generator.write("if (");
    compile_logical_expression(
        code_generator,
        subqueries_to_compile,
        logical_expression,
        child_path_code,
    );
    code_generator.write(")\n");
    generate_add_child_path_to_output(code_generator, selector_index, child_path_code, true);
}

fn compile_logical_expression<'a>(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile<'a>>,
    logical_expression: &'a LogicalExpression,
    child_path_code: &str,
) {
    match logical_expression {
        LogicalExpression::Or { lhs, rhs } => {
            code_generator.write("(");
            compile_logical_expression(code_generator, subqueries_to_compile, lhs, child_path_code);
            code_generator.write(" || ");
            compile_logical_expression(code_generator, subqueries_to_compile, rhs, child_path_code);
            code_generator.write(")");
        }
        LogicalExpression::And { lhs, rhs } => {
            code_generator.write("(");
            compile_logical_expression(code_generator, subqueries_to_compile, lhs, child_path_code);
            code_generator.write(" && ");
            compile_logical_expression(code_generator, subqueries_to_compile, rhs, child_path_code);
            code_generator.write(")");
        }
        LogicalExpression::Not { expr } => {
            code_generator.write("!");
            compile_logical_expression(
                code_generator,
                subqueries_to_compile,
                expr,
                child_path_code,
            );
        }
        LogicalExpression::Comparison { lhs, rhs, op } => {
            code_generator.write("compare(");
            compile_comparable(code_generator, lhs, child_path_code);
            code_generator.write(", ");
            compile_comparable(code_generator, rhs, child_path_code);
            code_generator.write(", ");
            compile_comparison_op(code_generator, op);
            code_generator.write(")");
        }
        LogicalExpression::ExistenceTest { subquery, absolute } => {
            let initial_node_path_code = if *absolute { "\"\"" } else { child_path_code };
            compile_existence_test(
                code_generator,
                subqueries_to_compile,
                subquery,
                initial_node_path_code,
            );
        }
    }
}

fn compile_comparable(
    code_generator: &mut CodeGenerator,
    comparable: &Comparable,
    child_path_code: &str,
) {
    match comparable {
        Comparable::SingularQuery {
            selectors,
            absolute,
        } => {
            compile_singular_query(
                code_generator,
                selectors,
                if *absolute {
                    None
                } else {
                    Some(child_path_code)
                },
            );
        }
        Comparable::Literal { literal } => compile_literal(code_generator, literal),
    }
}

fn compile_singular_query(
    code_generator: &mut CodeGenerator,
    selectors: &Vec<SingularSelector>,
    child_path_code: Option<&str>,
) {
    code_generator.write("evaluate_singular_query({");
    let mut first_selector = true;
    for selector in selectors {
        if !first_selector {
            code_generator.write(", ");
        }
        first_selector = false;
        match selector {
            SingularSelector::ChildByName { name } => {
                let name_str = &name.0;
                code_generator.write(&format!("{{NAME, string(\"{name_str}\")}}"));
            }
            SingularSelector::ElementAtIndex { index } => {
                code_generator.write(&format!("{{INDEX, {index}ll}}"));
            }
        }
    }
    let base_path_code = child_path_code.unwrap_or("\"\"");
    code_generator.write(&format!("}}, {base_path_code}, json)"));
}

fn compile_literal(code_generator: &mut CodeGenerator, literal: &Literal) {
    match literal {
        Literal::String(value) => {
            let escaped_value = escape(value, EscapeMode::DoubleQuoted);
            code_generator.write(&format!("{{STRING, string(\"{escaped_value}\")}}"));
        }
        Literal::Int(value) => {
            code_generator.write(&format!("{{INT, {value}ll}}"));
        }
        Literal::Float(value) => {
            code_generator.write(&format!("{{FLOAT, {value}}}"));
        }
        Literal::Bool(value) => {
            code_generator.write(&format!("{{BOOL, {value}}}"));
        }
        Literal::Null => {
            code_generator.write("{_NULL, {}}");
        }
    }
}

fn compile_comparison_op(code_generator: &mut CodeGenerator, comparison_op: &ComparisonOp) {
    match comparison_op {
        ComparisonOp::EqualTo => code_generator.write("EQUAL_TO"),
        ComparisonOp::NotEqualTo => code_generator.write("NOT_EQUAL_TO"),
        ComparisonOp::LessOrEqualTo => code_generator.write("LESS_OR_EQUAL_TO"),
        ComparisonOp::GreaterOrEqualTo => code_generator.write("GREATER_OR_EQUAL_TO"),
        ComparisonOp::LessThan => code_generator.write("LESS_THAN"),
        ComparisonOp::GreaterThan => code_generator.write("GREATER_THAN"),
    };
    return;
}

fn compile_existence_test<'a>(
    code_generator: &mut CodeGenerator,
    subqueries_to_compile: &HashMap<*const Query, SubqueryToCompile<'a>>,
    subquery: &'a Query,
    initial_node_path_code: &str,
) {
    let subquery_ref = subquery as *const Query;
    let function_name = &subqueries_to_compile
        .get(&subquery_ref)
        .unwrap()
        .function_name;
    code_generator.write(&format!("{function_name}(json, {initial_node_path_code})"));
}

fn generate_add_child_path_to_output(
    code_generator: &mut CodeGenerator,
    selector_index: usize,
    child_path_code: &str,
    indented: bool,
) {
    let selector_results_vector_name = get_selector_results_vector_name(selector_index);
    let code_line = &format!("{selector_results_vector_name}.push_back({child_path_code});");
    if indented {
        code_generator.write_extra_indented_line(code_line);
    } else {
        code_generator.write_line(code_line);
    }
}

fn get_selector_results_vector_name(selector_index: usize) -> String {
    format!("selector{selector_index}_results")
}
