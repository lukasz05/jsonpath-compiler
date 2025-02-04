use std::collections::{HashMap, HashSet};
use std::string::ToString;

use askama::Template;
use clang_format::{clang_format_with_style, ClangFormatStyle};

use crate::ir::{FilterExpression, FilterId, FilterProcedure, FilterSubquery, FilterSubquerySegment, Instruction, Procedure, Query, Comparable, LiteralValue, SelectionCondition};
use crate::ir::Instruction::{ForEachElement, ForEachMember};

type NamedQuery<'a> = (&'a str, &'a Query);

#[derive(Template)]
#[template(path = "simdjson/ondemand/standalone.cpp", escape = "none")]
struct ToOnDemandStandaloneTemplate<'a> {
    logging: bool,
    mmap: bool,
    procedures: Vec<ProcedureTemplate<'a>>,
    filter_procedures: Vec<FilterProcedureTemplate<'a>>,
    filter_subqueries: &'a HashMap<FilterId, Vec<FilterSubquery>>,
}

impl ToOnDemandStandaloneTemplate<'_> {
    fn new(query: &Query, logging: bool, mmap: bool) -> ToOnDemandStandaloneTemplate {
        ToOnDemandStandaloneTemplate {
            logging,
            mmap,
            procedures: query.procedures.iter()
                .map(|procedure| {
                    ProcedureTemplate::new(
                        procedure,
                        &query.filter_subqueries,
                        !query.filter_procedures.is_empty(),
                    )
                })
                .collect(),
            filter_procedures: query.filter_procedures.values()
                .map(|filter_procedure| {
                    FilterProcedureTemplate::new(filter_procedure)
                })
                .collect(),
            filter_subqueries: &query.filter_subqueries,
        }
    }

    fn are_any_filters(&self) -> bool {
        !self.filter_procedures.is_empty()
    }
}

#[derive(Template)]
#[template(path = "simdjson/ondemand/lib.cpp", escape = "none")]
struct ToOnDemandLibTemplate<'a> {
    filename: &'a str,
    logging: bool,
    bindings: bool,
    procedures: Vec<ProcedureTemplate<'a>>,
    query_names: Vec<String>,
}

impl ToOnDemandLibTemplate<'_> {
    fn new<'a>(
        queries: Vec<NamedQuery<'a>>,
        logging: bool,
        bindings: bool,
        filename: &'a str
    ) -> ToOnDemandLibTemplate<'a> {
        let mut procedures = Vec::new();
        let mut query_names = HashSet::new();
        for (name, query) in queries {
            for procedure in &query.procedures {
                procedures.push(ProcedureTemplate::new_with_query_name(
                    procedure,
                    &query.filter_subqueries,
                    !query.filter_procedures.is_empty(),
                    name)
                );
                query_names.insert(name.to_string());
            }
        }
        ToOnDemandLibTemplate {
            logging,
            bindings,
            filename,
            procedures,
            query_names: query_names.into_iter().collect()
        }
    }
}



#[derive(Template)]
#[template(path = "simdjson/ondemand/procedure.cpp", escape = "none")]
struct ProcedureTemplate<'a> {
    name: String,
    instructions: Vec<InstructionTemplate<'a>>,
    filter_subqueries: &'a HashMap<FilterId, Vec<FilterSubquery>>,
    are_any_filters: bool
}

impl ProcedureTemplate<'_> {
    fn new<'a>(
        procedure: &'a Procedure,
        filter_subqueries: &'a HashMap<FilterId, Vec<FilterSubquery>>,
        are_any_filters: bool,
    ) -> ProcedureTemplate<'a> {
        ProcedureTemplate {
            name: procedure.name.clone(),
            instructions: procedure.instructions.iter()
                .map(|instruction| {
                    InstructionTemplate::new(
                        instruction,
                        "node",
                        "",
                        Some(filter_subqueries),
                        are_any_filters,
                    )
                })
                .collect(),
            filter_subqueries,
            are_any_filters
        }
    }

    fn new_with_query_name<'a>(
        procedure: &'a Procedure,
        filter_subqueries: &'a HashMap<FilterId, Vec<FilterSubquery>>,
        are_any_filters: bool,
        query_name: &'a str,
    ) -> ProcedureTemplate<'a> {
        let procedure_name = format!("{}_{}", query_name, procedure.name);
        ProcedureTemplate {
            name: procedure_name,
            instructions: procedure.instructions.iter()
                .map(|instruction| {
                    InstructionTemplate::new(
                        instruction,
                        "node",
                        query_name,
                        Some(filter_subqueries),
                        are_any_filters,
                    )
                })
                .collect(),
            filter_subqueries,
            are_any_filters
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
#[template(path = "simdjson/ondemand/instruction.cpp", escape = "none")]
struct InstructionTemplate<'a> {
    instruction: &'a Instruction,
    current_node: &'a str,
    query_name: &'a str,
    filter_subqueries: Option<&'a HashMap<FilterId, Vec<FilterSubquery>>>,
    are_any_filters: bool
}

impl InstructionTemplate<'_> {
    fn new<'a>(
        instruction: &'a Instruction,
        current_node: &'a str,
        query_name: &'a str,
        filter_subqueries: Option<&'a HashMap<FilterId, Vec<FilterSubquery>>>,
        are_any_filters: bool
    ) -> InstructionTemplate<'a> {
        InstructionTemplate {
            instruction,
            current_node,
            query_name,
            filter_subqueries,
            are_any_filters
        }
    }
}

#[derive(Template)]
#[template(path = "simdjson/ondemand/filter_procedure.cpp", escape = "none")]
struct FilterProcedureTemplate<'a> {
    name: String,
    filter_id: FilterId,
    arity: usize,
    expression: FilterExpressionTemplate<'a>,
}

impl FilterProcedureTemplate<'_> {
    fn new<'a>(filter_procedure: &FilterProcedure) -> FilterProcedureTemplate {
        FilterProcedureTemplate {
            name: filter_procedure.name.clone(),
            filter_id: filter_procedure.filter_id.clone(),
            arity: filter_procedure.arity,
            expression: FilterExpressionTemplate::new(&filter_procedure.expression),
        }
    }
}

#[derive(Template)]
#[template(path = "simdjson/ondemand/filter_expression.cpp", escape = "none")]
struct FilterExpressionTemplate<'a> {
    expression: &'a FilterExpression,
}

impl FilterExpressionTemplate<'_> {
    fn new(expression: &FilterExpression) -> FilterExpressionTemplate {
        FilterExpressionTemplate { expression }
    }
}

#[derive(Template)]
#[template(path = "simdjson/ondemand/selection_condition.cpp", escape = "none")]
struct SelectionConditionTemplate<'a> {
    condition: &'a SelectionCondition,
}

impl SelectionConditionTemplate<'_> {
    fn new(condition: &SelectionCondition) -> SelectionConditionTemplate {
        SelectionConditionTemplate {condition}
    }
}

pub struct ToOnDemandCompiler<'a> {
    queries: Vec<NamedQuery<'a>>,
    standalone: bool,
    logging: bool,
    bindings: bool,
    mmap: bool,
    filename: Option<String>
}


impl ToOnDemandCompiler<'_> {
    pub fn new_standalone(
        query: NamedQuery,
        logging: bool,
        mmap: bool,
    ) -> ToOnDemandCompiler {
        ToOnDemandCompiler {
            queries: vec![query],
            standalone: true,
            logging,
            bindings: false,
            mmap,
            filename: None
        }
    }

    pub fn new_lib(
        queries: Vec<NamedQuery>,
        logging: bool,
        bindings: bool,
        filename: Option<String>,
    ) -> ToOnDemandCompiler {
        ToOnDemandCompiler {
            queries,
            standalone: false,
            logging,
            bindings,
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
            let template = ToOnDemandLibTemplate::new(
                self.queries,
                self.logging,
                self.bindings,
                &filename,
            );
            code = template.render().unwrap();
        }
        clang_format_with_style(&code, &ClangFormatStyle::Microsoft).unwrap()
        //code
    }
}

static EMPTY_OBJECT_ITERATION: InstructionTemplate =
    InstructionTemplate { instruction: &ForEachMember { instructions: vec![] }, current_node: "node", query_name: "", filter_subqueries: None, are_any_filters: false };
static EMPTY_ARRAY_ITERATION: InstructionTemplate =
    InstructionTemplate { instruction: &ForEachElement { instructions: vec![] }, current_node: "node", query_name: "", filter_subqueries: None, are_any_filters: false };
