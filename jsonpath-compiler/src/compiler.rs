use crate::compiler::code_generator::CodeGenerator;
use crate::ir::{Index, Query, Segment, Selector, Slice};

mod code_generator;

const INPUT_NODELIST_VARIABLE_NAME: &str = "input_nodelist";
const OUTPUT_NODELIST_VARIABLE_NAME: &str = "output_nodelist";


pub fn compile_query(query: Query) -> String {
    let mut code_generator = CodeGenerator::new();
    generate_prologue(&mut code_generator);
    compile_segments(&mut code_generator, &query);
    generate_epilogue(&mut code_generator);
    return String::from(code_generator.get_code());
}

fn generate_prologue(code_generator: &mut CodeGenerator) {
    code_generator.write_lines(&[
        //"#define SIMDJSON_VERBOSE_LOGGING 1",
        "",
        "#include <iostream>",
        "#include <vector>",
        "#include <string>",
        "#include <simdjson.h>",
        "",
        "using namespace std;",
        "using namespace simdjson;",
        "",
        "int main()"
    ]);
    code_generator.start_block();
    code_generator.write_lines(&[
        "string input(istreambuf_iterator<char>(cin), {});",
        "auto json = padded_string(input);",
        "ondemand::parser parser;",
        "auto root_node = parser.iterate(json);",
        &format!("vector<string> {INPUT_NODELIST_VARIABLE_NAME}{{\"\"}};"),
        &format!("vector<string> {OUTPUT_NODELIST_VARIABLE_NAME};"),
        "simdjson::error_code error;",
        ""
    ]);
}

fn generate_epilogue(code_generator: &mut CodeGenerator) {
    code_generator.write_lines(&[
        "",
        "bool first = true;",
        &format!("for (string node_path : {OUTPUT_NODELIST_VARIABLE_NAME})")
    ]);
    code_generator.start_block();
    code_generator.write_line("ondemand::value node;");
    code_generator.write_line("root_node.at_pointer(node_path).get(node);");
    code_generator.write_line("if (!first) cout << \",\\n\";");
    code_generator.write_line(r#"cout << node.raw_json() << "\n";"#);
    code_generator.write_line("first = false;");
    code_generator.end_block();
    code_generator.write_lines(&[
        "",
        "return 0;"
    ]);
    code_generator.end_block();
}

fn compile_segments(code_generator: &mut CodeGenerator, query: &Query) {
    if query.segments.is_empty() {
        return;
    }
    compile_segment(code_generator, &query.segments[0], true);
    for segment in query.segments.iter().skip(1) {
        compile_segment(code_generator, &segment, false);
    }
}

fn compile_segment(code_generator: &mut CodeGenerator, segment: &Segment, first_segment: bool) {
    if !first_segment {
        code_generator.write_lines(&[
            "",
            &format!("{INPUT_NODELIST_VARIABLE_NAME}.swap({OUTPUT_NODELIST_VARIABLE_NAME});"),
            &format!("{OUTPUT_NODELIST_VARIABLE_NAME}.clear();")
        ]);
    }
    code_generator.write_line(&format!("for (string node_path : {INPUT_NODELIST_VARIABLE_NAME})"));
    code_generator.start_block();
    match segment {
        Segment::ChildSegment { selectors } => {
            compile_selectors(code_generator, selectors);
        }
        Segment::DescendantSegment { .. } => {
            unimplemented!()
        }
    }
    code_generator.end_block();
}

fn compile_selectors(code_generator: &mut CodeGenerator, selectors: &Vec<Selector>) {
    for i in 0..selectors.len() {
        code_generator.write_line(
            &format!("vector<string> {};", get_selector_results_vector_name(i))
        );
    }
    code_generator.write_line("");
    code_generator.write_lines(&[
        "ondemand::value node;",
        "error = root_node.at_pointer(node_path).get(node);",
        "if (error) continue;",
        ""
    ]);

    if is_object_selector_present(selectors) {
        compile_object_selectors(code_generator, selectors);
    }
    if is_array_selector_present(selectors) {
        compile_array_selectors(code_generator, selectors);
    }

    for i in 0..selectors.len() {
        let selector_results_vector_name = get_selector_results_vector_name(i);
        code_generator.write_line(&format!(
            "{OUTPUT_NODELIST_VARIABLE_NAME}.insert({OUTPUT_NODELIST_VARIABLE_NAME}.end(),\
            {selector_results_vector_name}.begin(), {selector_results_vector_name}.end());")
        );
    }
}

fn is_object_selector_present(selectors: &Vec<Selector>) -> bool {
    for selector in selectors {
        match selector {
            Selector::ChildByName { .. } |
            Selector::AllChildren |
            Selector::Filter { .. } => return true,
            _ => {}
        }
    }
    return false;
}

fn is_array_selector_present(selectors: &Vec<Selector>) -> bool {
    for selector in selectors {
        match selector {
            Selector::ElementAtIndex { .. } |
            Selector::Slice { .. } |
            Selector::AllChildren |
            Selector::Filter { .. } => return true,
            _ => {}
        }
    }
    return false;
}

fn compile_object_selectors(code_generator: &mut CodeGenerator, selectors: &Vec<Selector>) {
    code_generator.write_lines(&[
        "ondemand::object obj;",
        "error = node.get_object().get(obj);",
        "if (!error)"
    ]);
    code_generator.start_block();
    code_generator.write_line("for (ondemand::field field : obj)");
    code_generator.start_block();
    code_generator.write_line("string key = string(field.escaped_key());");
    for (i, selector) in selectors.iter().enumerate() {
        let selector_results_vector_name = get_selector_results_vector_name(i);
        let push_back_child_path_line =
            format!("{selector_results_vector_name}.push_back(node_path + \"/\" + key);");
        match selector {
            Selector::AllChildren => {
                code_generator.write_line(&push_back_child_path_line);
            }
            Selector::ChildByName { name } => {
                let name_str = &name.0;
                code_generator.write_line(&format!("if (key == \"{name_str}\")"));
                code_generator.write_extra_indented_line(&push_back_child_path_line);
            }
            Selector::Filter { .. } => unimplemented!(),
            _ => {}
        }
    }
    code_generator.end_block();
    code_generator.end_block();
    code_generator.write_line("");
}

fn compile_array_selectors(code_generator: &mut CodeGenerator, selectors: &Vec<Selector>) {
    code_generator.write_lines(&[
        "ondemand::array arr;",
        "error = node.get_array().get(arr);",
        "if (!error)"
    ]);
    code_generator.start_block();
    code_generator.write_line("size_t element_count = arr.count_elements();");
    for (i, selector) in selectors.iter().enumerate() {
        match selector {
            Selector::AllChildren => {
                code_generator.write_line("for (size_t i = 0; i < element_count; i++)");
                generate_add_child_path_to_output(
                    code_generator,
                    i,
                    "node_path + \"/\" + to_string(i)",
                );
            }
            Selector::ElementAtIndex { index } => {
                compile_element_at_index_selector(code_generator, i, index)
            }
            Selector::Slice { slice } => {
                compile_slice_selector(code_generator, i, slice)
            }
            Selector::Filter { .. } => unimplemented!(),
            _ => {}
        }
    }
    code_generator.end_block();
    code_generator.write_line("");
}

fn compile_element_at_index_selector(
    code_generator: &mut CodeGenerator,
    slice_index: usize,
    index: &Index,
) {
    let index_val = index.0;
    if index_val < 0 {
        code_generator.write_line(&format!(
            "if (element_count > {})", -index_val)
        );
        generate_add_child_path_to_output(
            code_generator,
            slice_index,
            &format!(
                "node_path + \"/\" + to_string(element_count - {})",
                -index_val
            ),
        );
    } else {
        code_generator.write_line(&format!(
            "if (element_count > {})", index_val
        ));
        generate_add_child_path_to_output(
            code_generator,
            slice_index,
            &format!("node_path + \"/\" + to_string({index_val})"),
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
    let start = slice.start;
    let end_value = match slice.end {
        Some(end) => &end.to_string(),
        None => if step >= 0 { "element_count" } else { "element_count - 1" }
    };
    // TODO: handle default start and end values for step < 0 properly
    code_generator.write_line(&format!(
        "for (size_t i = {start}; i < {end_value}; i += {step})"
    ));
    generate_add_child_path_to_output(
        code_generator,
        selector_index,
        "node_path + \"/\" + to_string(i)",
    );
}

fn generate_add_child_path_to_output(
    code_generator: &mut CodeGenerator,
    selector_index: usize,
    child_path_code: &str,
) {
    let selector_results_vector_name = get_selector_results_vector_name(selector_index);
    code_generator.write_extra_indented_line(&format!(
        "{selector_results_vector_name}.push_back({child_path_code});"
    ));
}

fn get_selector_results_vector_name(selector_index: usize) -> String {
    format!("selector{selector_index}_results")
}