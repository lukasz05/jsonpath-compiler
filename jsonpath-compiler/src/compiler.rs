use crate::compiler::code_generator::CodeGenerator;
use crate::ir::{Instruction, Procedure, Query};
use crate::ir::Instruction::{ForEachElement, ForEachMember, TraverseCurrentNodeSubtree};

mod code_generator;

pub struct ToOnDemandCompiler<'a> {
    query: &'a Query,
    code_generator: CodeGenerator,
}

impl ToOnDemandCompiler<'_> {
    pub fn new(query: &Query) -> ToOnDemandCompiler {
        ToOnDemandCompiler {
            query,
            code_generator: CodeGenerator::new(),
        }
    }

    pub fn compile(mut self) -> String {
        let procedures = &self.query.procedures;
        let procedure_names: Vec<&String> = procedures.iter()
            .map(|proc| &proc.name)
            .collect();

        self.generate_prologue(&procedure_names);
        self.compile_procedures();

        self.code_generator.get_code()
    }

    fn get_procedure_signature(name: &String) -> String {
        format!(
            "void {}(ondemand::value node, vector<shared_ptr<ostringstream>> &results_in_progress, \
             vector<shared_ptr<ostringstream>> &all_results)",
            name.to_lowercase()
        )
    }

    fn generate_prologue(
        &mut self,
        procedure_names: &Vec<&String>,
    ) {
        self.code_generator.write_lines(&[
            //"#define SIMDJSON_VERBOSE_LOGGING 1",
            "",
            "#include <iostream>",
            "#include <vector>",
            "#include <queue>",
            "#include <string>",
            "#include <algorithm>",
            "#include <simdjson.h>",
            "",
            "using namespace std;",
            "using namespace simdjson;",
            ""
        ]);
        self.generate_procedures_declarations(procedure_names);
        self.code_generator.write_lines(&["", "int main()"]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            "string input(istreambuf_iterator<char>(cin), {});",
            "auto json = padded_string(input);",
            "ondemand::parser parser;",
            "auto root_node = parser.iterate(json);",
            "vector<shared_ptr<ostringstream>> results_in_progress;",
            "vector<shared_ptr<ostringstream>> all_results;",
            "selectors_0(root_node, results_in_progress, all_results);",
            r#"cout << "[\n";"#,
            "bool first = true;",
            r#"for (const auto &stream_ptr : all_results)"#
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            r#"if (!first) cout << ",";"#,
            r#"cout << "  " << stream_ptr->str();"#,
            "first = false;"
        ]);
        self.code_generator.end_block();

        self.code_generator.write_lines(&[
            r#"cout << "]\n";"#,
            "return 0;"
        ]);
        self.code_generator.end_block();
        self.code_generator.write_lines(&[
            "",
            "void add_to_all_streams(vector<shared_ptr<ostringstream>> &streams, string_view str)"
        ]);
        self.code_generator.start_block();
        self.code_generator
            .write_line("for (const auto &stream_ptr : streams) *stream_ptr << str;");
        self.code_generator.end_block();
        self.code_generator.write_lines(&[
            "",
            "void traverse_and_save_selected_nodes(ondemand::value node, vector<shared_ptr<ostringstream>> &results_in_progress)"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_line("if (!results_in_progress.empty())");
        self.code_generator.write_extra_indented_line("add_to_all_streams(results_in_progress, node.raw_json());");
        self.code_generator.end_block();
        self.code_generator.write_line("");
    }

    fn compile_procedures(&mut self) {
        let mut first = true;
        for procedure in &self.query.procedures {
            if !first {
                self.code_generator.write_line("");
            }
            first = false;
            self.compile_procedure(procedure);
        }
    }

    fn compile_procedure(&mut self, procedure: &Procedure) {
        self.code_generator.write_line(
            &Self::get_procedure_signature(&procedure.name)
        );
        self.code_generator.start_block();
        self.compile_instructions(&procedure.instructions, "node");
        if !procedure.instructions.iter().any(|ins| ins.is_object_member_iteration()) {
            self.compile_instruction(&ForEachMember { instructions: vec![] }, "node");
        }
        if !procedure.instructions.iter().any(|ins| ins.is_array_element_iteration()) {
            self.compile_instruction(&ForEachElement { instructions: vec![] }, "node");
        }
        self.code_generator.write_line("if (node.is_scalar())");
        self.code_generator.start_block();
        self.compile_instruction(&TraverseCurrentNodeSubtree, "node");
        self.code_generator.end_block();
        self.code_generator.end_block();
    }

    fn compile_instructions(&mut self, instructions: &Vec<Instruction>, current_node: &str) {
        for instruction in instructions {
            self.compile_instruction(instruction, current_node);
        }
    }

    fn compile_instruction(&mut self, instruction: &Instruction, current_node: &str) {
        match instruction {
            Instruction::ForEachElement { .. } => {
                self.compile_array_iteration(instruction);
            },
            Instruction::ForEachMember { .. } => {
                self.compile_object_iteration(instruction);
            }
            Instruction::IfCurrentIndexFromEndEquals { .. } => {
                todo!()
            }
            Instruction::IfCurrentIndexEquals { .. } => {
                todo!()
            }
            Instruction::IfCurrentMemberNameEquals { name, instructions } => {
                self.code_generator.write_line(&format!(
                    "if (key == \"{}\")",
                    rsonpath_syntax::str::escape(
                        name,
                        rsonpath_syntax::str::EscapeMode::DoubleQuoted,
                    )
                ));
                self.code_generator.start_block();
                self.compile_instructions(instructions, current_node);
                self.code_generator.end_block();
            }
            Instruction::ExecuteProcedureOnChild { name } => {
                self.code_generator.write_line(
                    &format!(
                        "{}({current_node}, results_in_progress, all_results);",
                        name.to_lowercase()
                    )
                );
            }
            Instruction::SaveCurrentNodeDuringTraversal { instruction } => {
                self.code_generator.write_lines(&[
                    "shared_ptr<ostringstream> stream_ptr = make_shared<ostringstream>();",
                    "all_results.push_back(stream_ptr);",
                    "results_in_progress.push_back(stream_ptr);"
                ]);
                self.compile_instruction(instruction, current_node);
                self.code_generator.write_line("results_in_progress.pop_back();");
            }
            Instruction::Continue => {
                self.code_generator.write_line("continue;");
            }
            Instruction::TraverseCurrentNodeSubtree => {
                self.code_generator.write_line(
                    &format!(
                        "traverse_and_save_selected_nodes({current_node}, results_in_progress);"
                    )
                );
            }
        }
    }

    fn compile_array_iteration(&mut self, loop_instruction: &Instruction) {
        let Instruction::ForEachElement { instructions } = loop_instruction else {
            panic!()
        };
        self.code_generator.write_lines(&[
            "ondemand::array array;",
            "if (!node.get_array().get(array))"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            r#"add_to_all_streams(results_in_progress, string_view("["));"#,
            "bool first = true;",
            "for (ondemand::value element : array)"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_line(
            r#"if (!first) add_to_all_streams(results_in_progress, string_view(", "));"#,
        );
        self.compile_instructions(instructions, "element");
        self.code_generator.write_line("first = false;");
        self.code_generator.end_block();
        self.code_generator.write_line(
            r#"add_to_all_streams(results_in_progress, string_view("]"));"#,
        );
        self.code_generator.end_block();
    }

    fn compile_object_iteration(&mut self, loop_instruction: &Instruction) {
        let Instruction::ForEachMember { instructions } = loop_instruction else {
            panic!()
        };
        self.code_generator.write_lines(&[
            "ondemand::object object;",
            "if (!node.get_object().get(object))"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            r#"add_to_all_streams(results_in_progress, string_view("{"));"#,
            "bool first = true;",
            "for (ondemand::field field : object)"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            "string_view key = field.escaped_key();",
            "for (const auto &stream_ptr : results_in_progress)"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            r#"if (!first) *stream_ptr << ", ";"#,
            r#"*stream_ptr << "\"" << key << "\": ";"#
        ]);
        self.code_generator.end_block();
        self.compile_instructions(instructions, "field.value()");
        self.code_generator.write_line("first = false;");
        self.code_generator.end_block();
        self.code_generator.write_line(
            r#"add_to_all_streams(results_in_progress, string_view("}"));"#,
        );
        self.code_generator.end_block();
    }

    fn generate_procedures_declarations(&mut self, procedure_names: &Vec<&String>) {
        for name in procedure_names {
            self.code_generator.write_line(
                &format!("{};", Self::get_procedure_signature(name))
            );
        }
    }
}


