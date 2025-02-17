use std::collections::HashMap;
use std::string::ToString;

use askama::Template;
use clang_format::{clang_format_with_style, ClangFormatStyle};

use crate::ir::{
    Comparable, FilterExpression, FilterId, FilterProcedure, FilterSubquery, FilterSubquerySegment,
    Instruction, LiteralValue, Procedure, Query, SelectionCondition,
};
use crate::ir::Instruction::{ForEachElement, ForEachMember};
use crate::targets::{NamedQuery, TargetCodeGenerator, TargetCodeGeneratorBase,
                     TargetCodeLibGenerator, TargetCodeLibGeneratorBase,
                     TargetCodeStandaloneProgGenerator, TargetCodeStandaloneProgGeneratorBase};

#[derive(Template)]
#[template(path = "simdjson/ondemand/standalone.cpp", escape = "none")]
struct OnDemandStandaloneProgTemplate<'a> {
    logging: bool,
    mmap: bool,
    procedures: Vec<ProcedureTemplate<'a>>,
    filter_procedures: Vec<FilterProcedureTemplate<'a>>,
    filter_subqueries: &'a HashMap<FilterId, Vec<FilterSubquery>>,
    segments_count: usize,
}

impl OnDemandStandaloneProgTemplate<'_> {
    fn new(query: &Query, logging: bool, mmap: bool) -> OnDemandStandaloneProgTemplate {
        OnDemandStandaloneProgTemplate {
            logging,
            mmap,
            procedures: query
                .procedures
                .iter()
                .map(|procedure| {
                    ProcedureTemplate::new(
                        "",
                        procedure,
                        &query.filter_subqueries,
                        !query.filter_procedures.is_empty(),
                    )
                })
                .collect(),
            filter_procedures: query
                .filter_procedures
                .values()
                .map(|filter_procedure| FilterProcedureTemplate::new(filter_procedure, ""))
                .collect(),
            filter_subqueries: &query.filter_subqueries,
            segments_count: query.segments_count,
        }
    }

    fn are_any_filters(&self) -> bool {
        !self.filter_procedures.is_empty()
    }

    fn max_subqueries_in_filter_count(&self) -> usize {
        self.filter_subqueries
            .values()
            .map(|subqueries| subqueries.len())
            .max()
            .unwrap_or(0)
    }
}

#[derive(Template)]
#[template(path = "simdjson/ondemand/lib.cpp", escape = "none")]
struct OnDemandLibTemplate<'a> {
    filename: &'a str,
    logging: bool,
    bindings: bool,
    procedures: HashMap<String, Vec<ProcedureTemplate<'a>>>,
    filter_procedures: HashMap<String, Vec<FilterProcedureTemplate<'a>>>,
    filter_subqueries: HashMap<String, &'a HashMap<FilterId, Vec<FilterSubquery>>>,
    query_segments_counts: HashMap<String, usize>,
}

impl OnDemandLibTemplate<'_> {
    fn new<'a>(
        queries: &'a Vec<NamedQuery>,
        logging: bool,
        bindings: bool,
        filename: &'a str,
    ) -> OnDemandLibTemplate<'a> {
        let mut procedures = HashMap::new();
        for (name, query) in queries.iter() {
            procedures.insert(
                name.to_string(),
                query
                    .procedures
                    .iter()
                    .map(|procedure| {
                        ProcedureTemplate::new(
                            name,
                            procedure,
                            &query.filter_subqueries,
                            !query.filter_procedures.is_empty(),
                        )
                    })
                    .collect::<Vec<ProcedureTemplate>>(),
            );
        }
        OnDemandLibTemplate {
            logging,
            bindings,
            filename,
            procedures,
            query_segments_counts: queries
                .iter()
                .map(|(name, query)| (name.to_string(), query.segments_count))
                .collect(),
            filter_procedures: queries
                .iter()
                .map(|(name, query)| {
                    (
                        name.to_string(),
                        query
                            .filter_procedures
                            .values()
                            .map(|fp| FilterProcedureTemplate::new(fp, name))
                            .collect(),
                    )
                })
                .collect(),
            filter_subqueries: queries
                .iter()
                .map(|(name, query)| (name.to_string(), &query.filter_subqueries))
                .collect(),
        }
    }

    fn query_names(&self) -> Vec<&String> {
        self.procedures.keys().collect()
    }

    fn all_procedures(&self) -> Vec<&ProcedureTemplate> {
        self.procedures.values().flatten().collect()
    }

    fn are_any_filters(&self) -> bool {
        !self.filter_procedures.is_empty()
    }

    fn are_any_filters_in_query(&self, query_name: &str) -> bool {
        self.filter_procedures.get(query_name).unwrap().len() > 0
    }

    fn max_subqueries_in_filter_count(&self) -> usize {
        self.filter_subqueries
            .values()
            .map(|v| v.values())
            .map(|subqueries| subqueries.len())
            .max()
            .unwrap_or(0)
    }

    fn query_filter_procedures(&self, query_name: &str) -> &Vec<FilterProcedureTemplate> {
        self.filter_procedures.get(query_name).unwrap()
    }

    fn query_filter_subqueries(&self, query_name: &str) -> &HashMap<FilterId, Vec<FilterSubquery>> {
        self.filter_subqueries.get(query_name).unwrap()
    }

    fn query_segments_count(&self, query_name: &str) -> usize {
        self.query_segments_counts.get(query_name).unwrap().clone()
    }

    fn all_filters_procedures(&self) -> Vec<&FilterProcedureTemplate> {
        self.filter_procedures.values().flatten().collect()
    }
}

#[derive(Template)]
#[template(path = "simdjson/ondemand/procedure.cpp", escape = "none")]
struct ProcedureTemplate<'a> {
    query_name: String,
    name: String,
    instructions: Vec<InstructionTemplate<'a>>,
    are_any_filters: bool,
}

impl ProcedureTemplate<'_> {
    fn new<'a>(
        query_name: &'a str,
        procedure: &'a Procedure,
        filter_subqueries: &'a HashMap<FilterId, Vec<FilterSubquery>>,
        are_any_filters: bool,
    ) -> ProcedureTemplate<'a> {
        ProcedureTemplate {
            query_name: query_name.to_string(),
            name: procedure.name.clone(),
            instructions: procedure
                .instructions
                .iter()
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
            are_any_filters,
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
#[template(path = "simdjson/ondemand/instruction.cpp", escape = "none")]
struct InstructionTemplate<'a> {
    instruction: &'a Instruction,
    query_name: &'a str,
    current_node: &'a str,
    filter_subqueries: Option<&'a HashMap<FilterId, Vec<FilterSubquery>>>,
    are_any_filters: bool,
}

impl InstructionTemplate<'_> {
    fn new<'a>(
        instruction: &'a Instruction,
        current_node: &'a str,
        query_name: &'a str,
        filter_subqueries: Option<&'a HashMap<FilterId, Vec<FilterSubquery>>>,
        are_any_filters: bool,
    ) -> InstructionTemplate<'a> {
        InstructionTemplate {
            instruction,
            current_node,
            query_name,
            filter_subqueries,
            are_any_filters,
        }
    }
}

#[derive(Template)]
#[template(path = "simdjson/ondemand/filter_procedure.cpp", escape = "none")]
struct FilterProcedureTemplate<'a> {
    name: String,
    filter_id: FilterId,
    expression: FilterExpressionTemplate<'a>,
    query_name: &'a str,
}

impl FilterProcedureTemplate<'_> {
    fn new<'a>(
        filter_procedure: &'a FilterProcedure,
        query_name: &'a str,
    ) -> FilterProcedureTemplate<'a> {
        FilterProcedureTemplate {
            name: filter_procedure.name.clone(),
            filter_id: filter_procedure.filter_id.clone(),
            expression: FilterExpressionTemplate::new(&filter_procedure.expression),
            query_name,
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
        SelectionConditionTemplate { condition }
    }
}

pub struct OnDemandCodeStandaloneProgGenerator {
    base: TargetCodeStandaloneProgGeneratorBase,
}

impl TargetCodeGenerator for OnDemandCodeStandaloneProgGenerator {
    fn base(&self) -> &TargetCodeGeneratorBase {
        &self.base.base
    }

    fn generate(&self) -> String {
        let template = OnDemandStandaloneProgTemplate::new(
            self.query(),
            self.logging(),
            self.mmap(),
        );
        let code = template.render().unwrap();
        //clang_format_with_style(&code, &ClangFormatStyle::Microsoft).unwrap()
        code
    }
}

impl TargetCodeStandaloneProgGenerator for OnDemandCodeStandaloneProgGenerator {
    fn new(query: Query, logging: bool, mmap: bool) -> impl TargetCodeStandaloneProgGenerator {
        OnDemandCodeStandaloneProgGenerator {
            base: TargetCodeStandaloneProgGeneratorBase::new(query, logging, mmap)
        }
    }

    fn base(&self) -> &TargetCodeStandaloneProgGeneratorBase {
        &self.base
    }
}

pub struct OnDemandCodeLibGenerator {
    base: TargetCodeLibGeneratorBase,
}

impl TargetCodeGenerator for OnDemandCodeLibGenerator {
    fn base(&self) -> &TargetCodeGeneratorBase {
        &self.base.base
    }

    fn generate(&self) -> String {
        let template = OnDemandLibTemplate::new(
            self.queries(),
            self.logging(),
            self.bindings(),
            self.filename(),
        );
        let code = template.render().unwrap();
        clang_format_with_style(&code, &ClangFormatStyle::Microsoft).unwrap()
    }
}

impl TargetCodeLibGenerator for OnDemandCodeLibGenerator {
    fn new(
        named_queries: Vec<NamedQuery>,
        filename: String,
        logging: bool,
        bindings: bool,
    ) -> impl TargetCodeLibGenerator {
        OnDemandCodeLibGenerator {
            base: TargetCodeLibGeneratorBase::new(
                named_queries,
                filename,
                logging,
                bindings,
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
    filter_subqueries: None,
    are_any_filters: false,
};
static EMPTY_ARRAY_ITERATION: InstructionTemplate = InstructionTemplate {
    instruction: &ForEachElement {
        instructions: vec![],
    },
    current_node: "node",
    query_name: "",
    filter_subqueries: None,
    are_any_filters: false,
};
