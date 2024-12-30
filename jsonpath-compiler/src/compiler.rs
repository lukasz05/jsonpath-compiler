use askama::Template;
use clang_format::{clang_format_with_style, ClangFormatStyle};

use crate::ir::{Instruction, Procedure, Query};
use crate::ir::Instruction::{ForEachElement, ForEachMember,
                             IfCurrentIndexEquals, IfCurrentIndexFromEndEquals,
                             IfCurrentMemberNameEquals};

type NamedQuery<'a> = (String, &'a Query);

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
#[template(path = "ondemand/header.cpp", escape = "none")]
struct ToOnDemandHeaderTemplate<'a> {
    filename: &'a str,
    logging: bool,
    procedures: Vec<ProcedureTemplate<'a>>,
    query_names: Vec<String>,
}

impl ToOnDemandHeaderTemplate<'_> {
    fn new<'a>(queries: Vec<NamedQuery<'a>>, logging: bool, filename: &'a str) -> ToOnDemandHeaderTemplate<'a> {
        let procedures = queries.iter()
            .flat_map(|(name, query)| query.procedures.iter().map(|p| (name.to_string(), p)))
            .map(|(name, procedure)|
            ProcedureTemplate::new_with_name(&procedure, format!("{name}_{}", procedure.name))
            )
            .collect();
        ToOnDemandHeaderTemplate {
            logging,
            filename,
            procedures,
            query_names: queries.iter().map(|(name, _)| name.to_string()).collect(),
        }
    }
}



#[derive(Template)]
#[template(path = "ondemand/procedure.cpp", escape = "none")]
struct ProcedureTemplate<'a> {
    name: String,
    instructions: Vec<InstructionTemplate<'a>>,
}

impl ProcedureTemplate<'_> {
    fn new(procedure: &Procedure) -> ProcedureTemplate {
        Self::new_with_name(procedure, procedure.name.clone())
    }

    fn new_with_name(procedure: &Procedure, name: String) -> ProcedureTemplate {
        ProcedureTemplate {
            name,
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
    queries: Vec<NamedQuery<'a>>,
    standalone: bool,
    logging: bool,
    mmap: bool,
    filename: Option<String>
}


impl ToOnDemandCompiler<'_> {
    pub fn new_standalone(
        query: NamedQuery,
        logging: bool,
        mmap: bool,
        filename: Option<String>,
    ) -> ToOnDemandCompiler {
        ToOnDemandCompiler {
            queries: vec![query],
            standalone: true,
            logging,
            mmap,
            filename
        }
    }

    pub fn new_header(
        queries: Vec<NamedQuery>,
        logging: bool,
        filename: Option<String>,
    ) -> ToOnDemandCompiler {
        ToOnDemandCompiler {
            queries,
            standalone: false,
            logging,
            mmap: false,
            filename,
        }
    }

    pub fn compile(self) -> String {
        let code: String;
        if self.standalone {
            let template = ToOnDemandStandaloneTemplate::new(
                self.queries[0].1,
                self.logging,
                self.mmap,
            );
            code = template.render().unwrap();
        } else {
            let filename = if self.filename.is_some() { self.filename.unwrap() } else {
                String::from("query.hpp")
            };
            let template = ToOnDemandHeaderTemplate::new(
                self.queries,
                self.logging,
                &filename,
            );
            code = template.render().unwrap();
        }
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


