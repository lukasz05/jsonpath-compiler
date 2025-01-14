use std::collections::{HashMap, HashSet, VecDeque};

use crate::ir::{Comparable, ComparisonOp, FilterExpression, FilterProcedure, Instruction, LiteralValue, Procedure, Query};
use crate::ir::Comparable::{Literal, Param};
use crate::ir::FilterExpression::{And, BoolParam, Comparison, Not, Or};
use crate::ir::Instruction::{Continue, ExecuteProcedureOnChild, ForEachElement, ForEachMember,
                             IfCurrentIndexEquals, IfCurrentIndexFromEndEquals,
                             IfCurrentMemberNameEquals, SaveCurrentNodeDuringTraversal,
                             TraverseCurrentNodeSubtree};
use crate::ir::LiteralValue::{Bool, Float, Int, Null};
use crate::ir::procedure_segments::{ProcedureSegments, ProcedureSegmentsData};

pub struct IRGenerator<'a> {
    query_syntax: &'a rsonpath_syntax::JsonPathQuery,
    generated_procedures: HashMap<ProcedureSegmentsData, Procedure>,
    procedures_to_generate: HashSet<ProcedureSegmentsData>,
    procedure_queue: VecDeque<ProcedureSegmentsData>,
    filter_generator: FilterGenerator
}

impl IRGenerator<'_> {
    pub fn new(query_syntax: &rsonpath_syntax::JsonPathQuery) -> IRGenerator {
        IRGenerator {
            query_syntax,
            generated_procedures: HashMap::new(),
            procedures_to_generate: HashSet::new(),
            procedure_queue: VecDeque::new(),
            filter_generator: FilterGenerator::new()
        }
    }

    pub fn generate(mut self) -> Query {
        let entry_procedure_segments = ProcedureSegments::new(
            self.query_syntax,
            vec![(0, None)],
        );
        let entry_procedure_name = entry_procedure_segments.name();
        if self.query_syntax.segments().is_empty() {
            Query {
                procedures: vec![
                    Procedure {
                        name: entry_procedure_name,
                        instructions: vec![
                            SaveCurrentNodeDuringTraversal {
                                instruction: Box::new(TraverseCurrentNodeSubtree)
                            }
                        ],
                    }
                ],
                filter_procedures: vec![]
            }
        } else {
            let segments_data = entry_procedure_segments.segments_data();
            self.procedure_queue.push_back(segments_data.clone());
            self.procedures_to_generate.insert(segments_data);
            while let Some(procedure_segments) = self.procedure_queue.pop_front() {
                self.generate_procedure(
                    &ProcedureSegments::new(self.query_syntax, procedure_segments.segments_with_conditions())
                );
            }
            let mut procedures: Vec<Procedure> = self.generated_procedures.into_values().collect();
            procedures.sort_by(|a, b| a.name.cmp(&b.name));
            Query {
                procedures,
                filter_procedures: self.filter_generator
                    .generate_filter_procedures(self.query_syntax),
            }
        }
    }

    fn generate_procedure(&mut self, segments: &ProcedureSegments) {
        let object_selectors_instructions =
            self.generate_object_selectors(&segments);
        let array_selectors_instructions =
            self.generate_array_selectors(&segments);
        let segments_data = segments.segments_data();
        self.procedures_to_generate.remove(&segments_data);
        self.generated_procedures.insert(segments_data, Procedure {
            name: segments.name(),
            instructions: object_selectors_instructions
                .into_iter().chain(array_selectors_instructions.into_iter()).collect()
        });
    }


    fn generate_object_selectors(
        &mut self,
        segments: &ProcedureSegments
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        for (name, occurrences) in segments.name_selectors() {
            let node_selected = !occurrences.finals().is_empty();
            let descendants = segments.descendants();
            let wildcards = segments.wildcards();
            let procedure_segments = ProcedureSegments::merge(
                self.query_syntax,
                vec![
                    descendants,
                    wildcards.successors(),
                    occurrences.successors(),
                ],
            );
            let inner_instructions = self.generate_procedure_execution(
                &procedure_segments,
                node_selected,
            );
            instructions.push(IfCurrentMemberNameEquals {
                name,
                instructions: inner_instructions
            });
        }
        instructions.append(&mut self.generate_wildcard_and_descendant_selectors(segments));
        vec![ForEachMember { instructions }]
    }

    fn generate_array_selectors(
        &mut self,
        segments: &ProcedureSegments,
    ) -> Vec<Instruction> {
        let wildcards = segments.wildcards();
        let mut instructions = Vec::new();
        for (index, occurrences) in segments.non_negative_index_selectors() {
            let node_selected = !occurrences.finals().is_empty();
            let procedure_segments = ProcedureSegments::merge(
                self.query_syntax,
                vec![
                    segments.descendants(),
                    wildcards.successors(),
                    occurrences.successors(),
                ],
            );
            let mut inner_instructions = Vec::new();
            for (neg_index, occurrences) in occurrences.negative_index_selectors() {
                let neg_node_selected = !occurrences.finals().is_empty();
                let inner_inner_instructions = self.generate_procedure_execution(
                    &procedure_segments.merge_with(&occurrences.successors()),
                    node_selected || neg_node_selected,
                );
                inner_instructions.push(IfCurrentIndexFromEndEquals {
                    index: u64::from_ne_bytes(i64::abs(neg_index).to_ne_bytes()),
                    instructions: inner_inner_instructions,
                })
            }
            inner_instructions = inner_instructions.into_iter().chain(
                self.generate_procedure_execution(&procedure_segments, node_selected)
            ).collect();
            instructions.push(IfCurrentIndexEquals {
                index: u64::from_ne_bytes(index.to_ne_bytes()),
                instructions: inner_instructions,
            });
        }
        for (neg_index, occurrences) in segments.negative_index_selectors() {
            let node_selected = !occurrences.finals().is_empty();
            let procedure_segments = ProcedureSegments::merge(
                self.query_syntax,
                vec![
                    segments.descendants(),
                    wildcards.successors(),
                    occurrences.successors(),
                ],
            );
            let inner_instructions = self.generate_procedure_execution(
                &procedure_segments,
                node_selected,
            );
            instructions.push(IfCurrentIndexFromEndEquals {
                index: u64::from_ne_bytes(i64::abs(neg_index).to_ne_bytes()),
                instructions: inner_instructions,
            });
        }
        instructions.append(&mut self.generate_wildcard_and_descendant_selectors(segments));
        vec![ForEachElement { instructions }]
    }

    fn generate_wildcard_and_descendant_selectors(
        &mut self,
        segments: &ProcedureSegments,
    ) -> Vec<Instruction> {
        let descendant_segments = segments.descendants();
        let wildcard_segments = segments.wildcards();
        let mut instructions = Vec::new();
        if !wildcard_segments.is_empty() {
            let node_selected = !wildcard_segments.finals().is_empty();
            let procedure_segments = descendant_segments
                .merge_with(&wildcard_segments.successors());
            if !procedure_segments.is_empty() {
                let procedure_name = self.get_or_create_procedure_for_segments(
                    &procedure_segments
                );
                instructions.push(Self::wrap_in_save_current_node_during_traversal_conditionally(
                    ExecuteProcedureOnChild { name: procedure_name },
                    node_selected,
                ));
            } else {
                instructions.push(Self::wrap_in_save_current_node_during_traversal_conditionally(
                    TraverseCurrentNodeSubtree,
                    node_selected,
                ));
            }
        } else if !descendant_segments.is_empty() {
            let procedure_name = self.get_or_create_procedure_for_segments(
                &descendant_segments
            );
            instructions.push(ExecuteProcedureOnChild { name: procedure_name });
        }
        instructions
    }

    fn generate_procedure_execution(
        &mut self,
        procedure_segments: &ProcedureSegments,
        node_selected: bool
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        if !procedure_segments.is_empty() {
            let procedure_name = self.get_or_create_procedure_for_segments(
                procedure_segments
            );
            instructions.push(Self::wrap_in_save_current_node_during_traversal_conditionally(
                ExecuteProcedureOnChild { name: procedure_name },
                node_selected,
            ));
            instructions.push(Continue);
        } else {
            instructions.push(Self::wrap_in_save_current_node_during_traversal_conditionally(
                TraverseCurrentNodeSubtree,
                node_selected,
            ));
            instructions.push(Continue);
        }
        instructions
    }

    fn get_or_create_procedure_for_segments(&mut self, segments: &ProcedureSegments) -> String {
        let segments_data = segments.segments_data();
        if !self.generated_procedures.contains_key(&segments_data)
            && !self.procedures_to_generate.contains(&segments_data) {
            self.procedures_to_generate.insert(segments_data.clone());
            self.procedure_queue.push_back(segments_data);
        }
        segments.name()
    }

    fn wrap_in_save_current_node_during_traversal_conditionally(
        instruction: Instruction,
        condition: bool,
    ) -> Instruction {
        if condition {
            SaveCurrentNodeDuringTraversal { instruction: Box::new(instruction) }
        } else {
            instruction
        }
    }
}

struct FilterGenerator {
    subquery_count: usize,
}

impl FilterGenerator {
    pub fn new() -> FilterGenerator {
        FilterGenerator {
            subquery_count: 0
        }
    }

    pub fn generate_filter_procedures(
        &mut self,
        query_syntax: &rsonpath_syntax::JsonPathQuery,
    ) -> Vec<FilterProcedure> {
        let mut filter_procedures = Vec::new();
        for (filter_expression, id) in Self::get_all_filters(query_syntax) {
            filter_procedures.push(self.generate_filter_procedure(filter_expression, id));
        }
        filter_procedures
    }

    fn get_all_filters(
        query_syntax: &rsonpath_syntax::JsonPathQuery
    ) -> Vec<(&rsonpath_syntax::LogicalExpr, (usize, usize))> {
        let mut filters = Vec::new();
        for (i, segment) in query_syntax.segments().iter().enumerate() {
            let selectors = segment.selectors();
            for (j, selector) in selectors.iter().enumerate() {
                if let rsonpath_syntax::Selector::Filter(filter_expression) = selector {
                    filters.push((filter_expression, (i, j)));
                }
            }
        }
        filters
    }

    fn generate_filter_procedure(
        &mut self,
        filter_expression: &rsonpath_syntax::LogicalExpr,
        id: (usize, usize),
    ) -> FilterProcedure {
        self.subquery_count = 0;
        let expression = self.generate_filter_expr(filter_expression);
        FilterProcedure {
            name: format!("Filter_{}_{}", id.0, id.1),
            arity: self.subquery_count,
            expression,
        }
    }

    fn generate_filter_expr(&mut self, filter_expr: &rsonpath_syntax::LogicalExpr) -> FilterExpression {
        match filter_expr {
            rsonpath_syntax::LogicalExpr::Or(lhs, rhs) => {
                Or {
                    lhs: Box::new(self.generate_filter_expr(lhs)),
                    rhs: Box::new(self.generate_filter_expr(rhs)),
                }
            }
            rsonpath_syntax::LogicalExpr::And(lhs, rhs) => {
                And {
                    lhs: Box::new(self.generate_filter_expr(lhs)),
                    rhs: Box::new(self.generate_filter_expr(rhs)),
                }
            }
            rsonpath_syntax::LogicalExpr::Not(expr) => {
                Not { expr: Box::new(self.generate_filter_expr(expr)) }
            }
            rsonpath_syntax::LogicalExpr::Comparison(comparison_expr) => {
                Comparison {
                    lhs: self.generate_comparable(comparison_expr.lhs()),
                    rhs: self.generate_comparable(comparison_expr.rhs()),
                    op: self.generate_comparison_op(comparison_expr.op()),
                }
            }
            rsonpath_syntax::LogicalExpr::Test { .. } => {
                self.subquery_count += 1;
                BoolParam { id: self.subquery_count }
            }
        }
    }

    fn generate_comparable(&mut self, comparable: &rsonpath_syntax::Comparable) -> Comparable {
        match comparable {
            rsonpath_syntax::Comparable::Literal(literal) => {
                Literal { value: self.generate_literal(literal) }
            }
            rsonpath_syntax::Comparable::AbsoluteSingularQuery { .. }
            | rsonpath_syntax::Comparable::RelativeSingularQuery { .. } => {
                self.subquery_count += 1;
                Param { id: self.subquery_count }
            }
        }
    }

    fn generate_literal(&self, literal_syntax: &rsonpath_syntax::Literal) -> LiteralValue {
        match literal_syntax {
            rsonpath_syntax::Literal::String(json_str) => {
                LiteralValue::String(json_str.unquoted().to_string())
            },
            rsonpath_syntax::Literal::Number(json_num) => {
                match json_num {
                    rsonpath_syntax::num::JsonNumber::Int(json_int) => {
                        Int(json_int.as_i64())
                    }
                    rsonpath_syntax::num::JsonNumber::Float(json_float) => {
                        Float(json_float.as_f64())
                    }
                }
            }
            rsonpath_syntax::Literal::Bool(bool_value) => Bool(*bool_value),
            rsonpath_syntax::Literal::Null => Null
        }
    }

    fn generate_comparison_op(
        &self,
        comparison_op_syntax: rsonpath_syntax::ComparisonOp,
    ) -> ComparisonOp {
        match comparison_op_syntax {
            rsonpath_syntax::ComparisonOp::EqualTo => ComparisonOp::EqualTo,
            rsonpath_syntax::ComparisonOp::NotEqualTo => ComparisonOp::NotEqualTo,
            rsonpath_syntax::ComparisonOp::LesserOrEqualTo => ComparisonOp::LesserOrEqualTo,
            rsonpath_syntax::ComparisonOp::GreaterOrEqualTo => ComparisonOp::GreaterOrEqualTo,
            rsonpath_syntax::ComparisonOp::LessThan => ComparisonOp::LessThan,
            rsonpath_syntax::ComparisonOp::GreaterThan => ComparisonOp::GreaterThan
        }
    }
}