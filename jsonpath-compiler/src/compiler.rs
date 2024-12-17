use crate::compiler::code_generator::CodeGenerator;
use crate::ir::{Instruction, Procedure, Query};
use crate::ir::Instruction::{ExecuteProcedureOnChild, ForEachElement, ForEachMember,
                             IfCurrentIndexEquals, IfCurrentIndexFromEndEquals,
                             IfCurrentMemberNameEquals, SaveCurrentNodeDuringTraversal,
                             TraverseCurrentNodeSubtree};

mod code_generator;

pub struct ToOnDemandCompiler<'a> {
    query: &'a Query,
    mmap: bool,
    code_generator: CodeGenerator,
}

impl ToOnDemandCompiler<'_> {
    pub fn new(query: &Query, mmap: bool) -> ToOnDemandCompiler {
        ToOnDemandCompiler {
            query,
            mmap,
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
            "void {}(ondemand::value &node, vector<string*> &results_in_progress, \
             vector<string*> &all_results)",
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
            "#include <fstream>",
            "#include <vector>",
            "#include <queue>",
            "#include <string>",
            "#include <algorithm>",
            "#include <fcntl.h>",
            "#include <sys/mman.h>",
            "#include <sys/stat.h>",
            "#include <simdjson.h>",
            "",
            "using namespace std;",
            "using namespace simdjson;",
            ""
        ]);
        self.generate_procedures_declarations(procedure_names);
        self.code_generator.write_lines(&["", "int main(int argc, char **argv)"]);
        self.code_generator.start_block();
        if self.mmap {
            self.code_generator.write_line("const auto input = map_input(argv[1]);");
        } else {
            self.code_generator.write_line("const auto input = read_input(argv[1]);");
        }
        self.code_generator.write_lines(&[
            "const auto json = padded_string(input);",
            "ondemand::parser parser;",
            "ondemand::value root_node = parser.iterate(json).get_value().value();",
            "vector<string*> results_in_progress;",
            "vector<string*> all_results;",
            "selectors_0(root_node, results_in_progress, all_results);",
            r#"cout << "[\n";"#,
            "bool first = true;",
            r#"for (const auto &buf_ptr : all_results)"#
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            r#"if (!first) cout << ",";"#,
            r#"cout << "  " << *buf_ptr;"#,
            "first = false;"
        ]);
        self.code_generator.end_block();
        self.code_generator.write_lines(&[
            r#"cout << "]\n";"#,
            "return 0;"
        ]);
        self.code_generator.end_block();
        self.code_generator.write_line("");
        if self.mmap {
            self.code_generator.write_line("string_view map_input(const char* filename)");
            self.code_generator.start_block();
            self.code_generator.write_lines(&[
                "const int fd = open(filename, O_RDONLY);",
                "if (fd == -1) exit(1);",
                "struct stat sb{};",
                "if (fstat(fd, &sb) == -1) exit(1);",
                "const size_t length = sb.st_size;",
                "const auto addr = static_cast<const char*>(mmap(nullptr, length, PROT_READ, MAP_PRIVATE, fd, 0u));",
                "if (addr == MAP_FAILED) exit(1);",
                "return {addr};"
            ]);
        } else {
            self.code_generator.write_line("string read_input(const char* filename)");
            self.code_generator.start_block();
            self.code_generator.write_lines(&[
                "ostringstream buf;",
                "ifstream input (filename);",
                "buf << input.rdbuf();",
                "return buf.str();"
            ]);
        }
        self.code_generator.end_block();
        self.code_generator.write_lines(&[
            "",
            "void add_to_all_bufs(const vector<string*> &bufs, const string_view str)"
        ]);
        self.code_generator.start_block();
        self.code_generator
            .write_line("for (const auto &buf_ptr : bufs) *buf_ptr += str;");
        self.code_generator.end_block();
        self.code_generator.write_lines(&[
            "",
            "void traverse_and_save_selected_nodes(ondemand::value &node, vector<string*> &results_in_progress)"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_line("if (!results_in_progress.empty())");
        self.code_generator.write_extra_indented_line("add_to_all_bufs(results_in_progress, node.raw_json());");
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
            ForEachElement { .. } => {
                self.compile_array_iteration(instruction);
            },
            ForEachMember { .. } => {
                self.compile_object_iteration(instruction);
            }
            IfCurrentIndexEquals { index, instructions } => {
                self.code_generator.write_line(&format!("if (index == {index})"));
                self.code_generator.start_block();
                self.compile_instructions(instructions, current_node);
                self.code_generator.end_block();
            }
            IfCurrentIndexFromEndEquals { index, instructions } => {
                self.code_generator.write_line(&format!("if (array_length - index == {index})"));
                self.code_generator.start_block();
                self.compile_instructions(instructions, current_node);
                self.code_generator.end_block();

            }
            IfCurrentMemberNameEquals { name, instructions } => {
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
            ExecuteProcedureOnChild { name } => {
                self.code_generator.write_line(
                    &format!(
                        "{}({current_node}, results_in_progress, all_results);",
                        name.to_lowercase()
                    )
                );
            }
            SaveCurrentNodeDuringTraversal { instruction } => {
                self.code_generator.write_lines(&[
                    "string* buf_ptr = new string();",
                    "all_results.push_back(buf_ptr);",
                    "results_in_progress.push_back(buf_ptr);"
                ]);
                self.compile_instruction(instruction, current_node);
                self.code_generator.write_line("results_in_progress.pop_back();");
            }
            Instruction::Continue => {
                self.code_generator.write_line("continue;");
            }
            TraverseCurrentNodeSubtree => {
                self.code_generator.write_line(
                    &format!(
                        "traverse_and_save_selected_nodes({current_node}, results_in_progress);"
                    )
                );
            }
        }
    }

    fn compile_array_iteration(&mut self, loop_instruction: &Instruction) {
        let ForEachElement { instructions } = loop_instruction else {
            panic!()
        };
        self.code_generator.write_lines(&[
            "ondemand::array array;",
            "if (!node.get_array().get(array))"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            r#"add_to_all_bufs(results_in_progress, string_view("["));"#,
            "bool first = true;",
            "size_t index = 0;"
        ]);
        if Self::is_array_length_needed(instructions) {
            self.code_generator.write_line("size_t array_length = array.count_elements();");
        }
        self.code_generator.write_line("for (ondemand::value element : array)");
        self.code_generator.start_block();
        self.code_generator.write_line("if (!first)");
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            "index++;",
            r#"add_to_all_bufs(results_in_progress, string_view(", "));"#,
        ]);
        self.code_generator.end_block();
        self.code_generator.write_line("first = false;");
        self.compile_instructions(instructions, "element");
        self.code_generator.end_block();
        self.code_generator.write_line(
            r#"add_to_all_bufs(results_in_progress, string_view("]"));"#,
        );
        self.code_generator.end_block();
    }

    fn compile_object_iteration(&mut self, loop_instruction: &Instruction) {
        let ForEachMember { instructions } = loop_instruction else {
            panic!()
        };
        self.code_generator.write_lines(&[
            "ondemand::object object;",
            "if (!node.get_object().get(object))"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            r#"add_to_all_bufs(results_in_progress, string_view("{"));"#,
            "bool first = true;",
            "for (ondemand::field field : object)"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            "string_view key = field.unescaped_key();",
            "for (const auto &buf_ptr : results_in_progress)"
        ]);
        self.code_generator.start_block();
        self.code_generator.write_lines(&[
            r#"if (!first) *buf_ptr += ", ";"#,
            r#"*buf_ptr += format("\"{}\": ", key);"#
        ]);
        self.code_generator.end_block();
        self.compile_instructions(instructions, "field.value()");
        self.code_generator.write_line("first = false;");
        self.code_generator.end_block();
        self.code_generator.write_line(
            r#"add_to_all_bufs(results_in_progress, string_view("}"));"#,
        );
        self.code_generator.end_block();
    }

    fn generate_procedures_declarations(&mut self, procedure_names: &Vec<&String>) {
        if self.mmap {
            self.code_generator.write_line("string_view map_input(const char* filename);");
        } else {
            self.code_generator.write_line("string read_input(const char* filename);");
        }
        self.code_generator.write_line("");
        for name in procedure_names {
            self.code_generator.write_line(
                &format!("{};", Self::get_procedure_signature(name))
            );
        }
    }

    fn is_array_length_needed(instructions: &Vec<Instruction>) -> bool {
        for instruction in instructions {
            let is_needed = match instruction {
                IfCurrentIndexFromEndEquals { .. } => true,
                IfCurrentIndexEquals { index: _index, instructions } => {
                    Self::is_array_length_needed(instructions)
                }
                IfCurrentMemberNameEquals { name: _name, instructions } => {
                    Self::is_array_length_needed(instructions)
                }
                ForEachMember { instructions } => {
                    Self::is_array_length_needed(instructions)
                }
                ForEachElement { instructions } => {
                    Self::is_array_length_needed(instructions)
                }
                _ => false
            };
            if is_needed {
                return true
            }
        }
        false
    }
}


