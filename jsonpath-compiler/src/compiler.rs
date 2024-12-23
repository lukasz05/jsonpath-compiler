use askama::Template;
use clang_format::{clang_format_with_style, ClangFormatStyle};

use crate::ir::{Instruction, Procedure, Query};
use crate::ir::Instruction::{ForEachElement, ForEachMember,
                             IfCurrentIndexEquals, IfCurrentIndexFromEndEquals,
                             IfCurrentMemberNameEquals};

#[derive(Template)]
#[template(path = "ondemand/standalone.cpp", escape = "none")]
struct ToOnDemandStandaloneTemplate<'a> {
    logging: bool,
    mmap: bool,
    procedures: Vec<ProcedureTemplate<'a>>,
}

impl ToOnDemandStandaloneTemplate<'_> {
    fn new(query: &Query, logging: bool, mmap: bool) -> ToOnDemandStandaloneTemplate {
        ToOnDemandStandaloneTemplate {
            logging,
            mmap,
            procedures: query.procedures.iter()
                .map(|procedure| ProcedureTemplate::new(procedure))
                .collect(),
        }
    }
}


#[derive(Template)]
#[template(path = "ondemand/procedure.cpp", escape = "none")]
struct ProcedureTemplate<'a> {
    name: &'a str,
    instructions: Vec<InstructionTemplate<'a>>,
}

impl ProcedureTemplate<'_> {
    fn new(procedure: &Procedure) -> ProcedureTemplate {
        ProcedureTemplate {
            name: &procedure.name,
            instructions: procedure.instructions.iter()
                .map(|instruction| InstructionTemplate::new(instruction, "node"))
                .collect(),
        }
    }

    fn are_object_members_iterated(&self) -> bool
    {
        self.instructions.iter().any(|ins| ins.instruction.is_object_member_iteration())
    }

    fn are_array_elements_iterated(&self) -> bool
    {
        self.instructions.iter().any(|ins| ins.instruction.is_array_element_iteration())
    }
}


#[derive(Template)]
#[template(path = "ondemand/instruction.cpp", escape = "none")]
struct InstructionTemplate<'a> {
    instruction: &'a Instruction,
    current_node: &'a str,
}

impl InstructionTemplate<'_> {
    fn new<'a>(instruction: &'a Instruction, current_node: &'a str) -> InstructionTemplate<'a> {
        InstructionTemplate {
            instruction,
            current_node,
        }
    }
}

pub struct ToOnDemandCompiler<'a> {
    query: &'a Query,
    logging: bool,
    mmap: bool
}


impl ToOnDemandCompiler<'_> {
    pub fn new(query: &Query, logging: bool, mmap: bool) -> ToOnDemandCompiler {
        ToOnDemandCompiler {
            query,
            logging,
            mmap
        }
    }

    pub fn compile(self) -> String {
        let template = ToOnDemandStandaloneTemplate::new(
            self.query,
            self.logging,
            self.mmap,
        );
        let code = template.render().unwrap();
        clang_format_with_style(&code, &ClangFormatStyle::Microsoft).unwrap()
    }
}

static EMPTY_OBJECT_ITERATION: InstructionTemplate =
    InstructionTemplate { instruction: &ForEachMember { instructions: vec![] }, current_node: "node" };
static EMPTY_ARRAY_ITERATION: InstructionTemplate =
    InstructionTemplate { instruction: &ForEachElement { instructions: vec![] }, current_node: "node" };


fn is_array_length_needed(instructions: &Vec<Instruction>) -> bool {
    for instruction in instructions {
        let is_needed = match instruction {
            IfCurrentIndexFromEndEquals { .. } => true,
            IfCurrentIndexEquals { index: _index, instructions } => {
                is_array_length_needed(instructions)
            }
            IfCurrentMemberNameEquals { name: _name, instructions } => {
                is_array_length_needed(instructions)
            }
            ForEachMember { instructions } => {
                is_array_length_needed(instructions)
            }
            ForEachElement { instructions } => {
                is_array_length_needed(instructions)
            }
            _ => false
        };
        if is_needed {
            return true
        }
    }
    false
}


