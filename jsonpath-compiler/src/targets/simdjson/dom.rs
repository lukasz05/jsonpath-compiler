use std::collections::HashSet;
use std::string::ToString;

use askama::Template;
use clang_format::{clang_format_with_style, ClangFormatStyle};

use crate::ir::{Instruction, Procedure, Query};
use crate::ir::Instruction::{ForEachElement, ForEachMember};
use crate::NamedQuery;
use crate::targets::{TargetCodeGenerator, TargetCodeGeneratorBase, TargetCodeLibGenerator, TargetCodeLibGeneratorBase, TargetCodeStandaloneProgGenerator, TargetCodeStandaloneProgGeneratorBase};

#[derive(Template)]
#[template(path = "simdjson/dom/standalone.cpp", escape = "none")]
struct DomStandaloneProgTemplate<'a> {
    logging: bool,
    mmap: bool,
    procedures: Vec<ProcedureTemplate<'a>>,
}

impl DomStandaloneProgTemplate<'_> {
    fn new(query: &Query, logging: bool, mmap: bool) -> DomStandaloneProgTemplate {
        DomStandaloneProgTemplate {
            logging,
            mmap,
            procedures: query
                .procedures
                .iter()
                .map(|procedure| ProcedureTemplate::new(procedure))
                .collect(),
        }
    }
}

#[derive(Template)]
#[template(path = "simdjson/dom/lib.cpp", escape = "none")]
struct DomLibTemplate<'a> {
    filename: &'a str,
    logging: bool,
    bindings: bool,
    procedures: Vec<ProcedureTemplate<'a>>,
    query_names: Vec<String>,
}

impl DomLibTemplate<'_> {
    fn new<'a>(
        queries: &'a Vec<NamedQuery>,
        logging: bool,
        bindings: bool,
        filename: &'a str,
    ) -> DomLibTemplate<'a> {
        let mut procedures = Vec::new();
        let mut query_names = HashSet::new();
        for (name, query) in queries {
            for procedure in &query.procedures {
                procedures.push(ProcedureTemplate::new_with_query_name(procedure, name));
                query_names.insert(name.to_string());
            }
        }
        DomLibTemplate {
            logging,
            bindings,
            filename,
            procedures,
            query_names: query_names.into_iter().collect(),
        }
    }
}

#[derive(Template)]
#[template(path = "simdjson/dom/procedure.cpp", escape = "none")]
struct ProcedureTemplate<'a> {
    name: String,
    instructions: Vec<InstructionTemplate<'a>>,
}

impl ProcedureTemplate<'_> {
    fn new(procedure: &Procedure) -> ProcedureTemplate {
        ProcedureTemplate {
            name: procedure.name.clone(),
            instructions: procedure
                .instructions
                .iter()
                .map(|instruction| InstructionTemplate::new(instruction, "node", ""))
                .collect(),
        }
    }

    fn new_with_query_name<'a>(
        procedure: &'a Procedure,
        query_name: &'a str,
    ) -> ProcedureTemplate<'a> {
        let procedure_name = format!("{}_{}", query_name, procedure.name);
        ProcedureTemplate {
            name: procedure_name,
            instructions: procedure
                .instructions
                .iter()
                .map(|instruction| InstructionTemplate::new(instruction, "node", query_name))
                .collect(),
        }
    }

    fn are_object_members_iterated(&self) -> bool {
        self.instructions
            .iter()
            .any(|ins| ins.instruction.is_object_member_iteration())
    }

    fn are_array_elements_iterated(&self) -> bool {
        self.instructions
            .iter()
            .any(|ins| ins.instruction.is_array_element_iteration())
    }
}

#[derive(Template)]
#[template(path = "simdjson/dom/instruction.cpp", escape = "none")]
struct InstructionTemplate<'a> {
    instruction: &'a Instruction,
    current_node: &'a str,
    query_name: &'a str,
}

impl InstructionTemplate<'_> {
    fn new<'a>(
        instruction: &'a Instruction,
        current_node: &'a str,
        query_name: &'a str,
    ) -> InstructionTemplate<'a> {
        InstructionTemplate {
            instruction,
            current_node,
            query_name,
        }
    }
}

pub struct DomCodeStandaloneProgGenerator {
    base: TargetCodeStandaloneProgGeneratorBase,
}

impl TargetCodeGenerator for DomCodeStandaloneProgGenerator {
    fn base(&self) -> &TargetCodeGeneratorBase {
        &self.base.base
    }

    fn generate(&self) -> String {
        let template = DomStandaloneProgTemplate::new(
            self.query(),
            self.logging(),
            self.mmap(),
        );
        let code = template.render().unwrap();
        clang_format_with_style(&code, &ClangFormatStyle::Microsoft).unwrap()
    }
}

impl TargetCodeStandaloneProgGenerator for DomCodeStandaloneProgGenerator {
    fn new(
        query: Query,
        logging: bool,
        mmap: bool,
        eager_filter_evaluation: bool,
    ) -> impl TargetCodeStandaloneProgGenerator {
        DomCodeStandaloneProgGenerator {
            base: TargetCodeStandaloneProgGeneratorBase::new(
                query,
                logging,
                mmap,
                eager_filter_evaluation,
            )
        }
    }

    fn base(&self) -> &TargetCodeStandaloneProgGeneratorBase {
        &self.base
    }
}

pub struct DomCodeLibGenerator {
    base: TargetCodeLibGeneratorBase,
}

impl TargetCodeGenerator for DomCodeLibGenerator {
    fn base(&self) -> &TargetCodeGeneratorBase {
        &self.base.base
    }

    fn generate(&self) -> String {
        let template = DomLibTemplate::new(
            self.queries(),
            self.logging(),
            self.bindings(),
            self.filename(),
        );
        let code = template.render().unwrap();
        clang_format_with_style(&code, &ClangFormatStyle::Microsoft).unwrap()
    }
}

impl TargetCodeLibGenerator for DomCodeLibGenerator {
    fn new(
        named_queries: Vec<NamedQuery>,
        filename: String,
        logging: bool,
        bindings: bool,
        eager_filter_evaluation: bool,
    ) -> impl TargetCodeLibGenerator {
        DomCodeLibGenerator {
            base: TargetCodeLibGeneratorBase::new(
                named_queries,
                filename,
                logging,
                bindings,
                eager_filter_evaluation,
            )
        }
    }

    fn base(&self) -> &TargetCodeLibGeneratorBase {
        &self.base
    }
}

static EMPTY_OBJECT_ITERATION: InstructionTemplate = InstructionTemplate {
    instruction: &ForEachMember {
        instructions: vec![],
    },
    current_node: "node",
    query_name: "",
};
static EMPTY_ARRAY_ITERATION: InstructionTemplate = InstructionTemplate {
    instruction: &ForEachElement {
        instructions: vec![],
    },
    current_node: "node",
    query_name: "",
};
