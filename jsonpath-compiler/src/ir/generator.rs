use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;

use crate::ir::{Instruction, Procedure, Query, SelectionCondition};
use crate::ir::filter_generator::{FilterGenerator, FilterSubqueryFinder, FilterUtils};
use crate::ir::Instruction::{Continue, ExecuteProcedureOnChild, ForEachElement, ForEachMember, IfCurrentIndexEquals, IfCurrentIndexFromEndEquals, IfCurrentMemberNameEquals, RegisterSubqueryPath, SaveCurrentNodeDuringTraversal, TraverseCurrentNodeSubtree, TryUpdateSubqueries};
use crate::ir::procedure_segments::{ProcedureSegments, ProcedureSegmentsData};

pub struct IRGenerator<'a> {
    query_syntax: &'a rsonpath_syntax::JsonPathQuery,
    generated_procedures: HashMap<ProcedureSegmentsData, Procedure>,
    procedures_to_generate: HashSet<ProcedureSegmentsData>,
    procedure_queue: VecDeque<ProcedureSegmentsData>,
    filter_generator: FilterGenerator,
    filter_subquery_finder: FilterSubqueryFinder,
    are_filters_in_query: bool
}

impl IRGenerator<'_> {
    pub fn new(query_syntax: &rsonpath_syntax::JsonPathQuery) -> IRGenerator {
        IRGenerator {
            query_syntax,
            generated_procedures: HashMap::new(),
            procedures_to_generate: HashSet::new(),
            procedure_queue: VecDeque::new(),
            filter_generator: FilterGenerator::new(),
            filter_subquery_finder: FilterSubqueryFinder::new(),
            are_filters_in_query: !FilterUtils::get_all_filters(query_syntax).is_empty()
        }
    }

    pub fn generate(mut self) -> Query {
        let first_segment = ProcedureSegments::new(
            self.query_syntax,
            vec![(0, None)],
        );
        let first_segment_procedure_name = first_segment.name();
        if self.query_syntax.segments().is_empty() {
            Query {
                procedures: vec![
                    Procedure {
                        name: first_segment_procedure_name,
                        instructions: vec![
                            SaveCurrentNodeDuringTraversal {
                                instruction: Box::new(TraverseCurrentNodeSubtree),
                                condition: None
                            }
                        ],
                    }
                ],
                filter_procedures: vec![]
            }
        } else {
            let segments_data = first_segment.segments_data();
            self.procedure_queue.push_back(segments_data.clone());
            self.procedures_to_generate.insert(segments_data);
            while let Some(procedure_segments) = self.procedure_queue.pop_front() {
                self.generate_procedure(
                    &ProcedureSegments::new(self.query_syntax, procedure_segments.segments_with_conditions())
                );
            }
            let init_procedure = self.init_procedure();
            let procedures: Vec<Procedure> = if init_procedure.is_none() {
                vec![]
            } else {
                vec![init_procedure.unwrap()]
            };
            let procedures: Vec<Procedure> = procedures.into_iter()
                .chain(self.generated_procedures.into_values().sorted_by(|a, b| {
                    a.name.cmp(&b.name)
                }))
                .collect();
            Query {
                procedures,
                filter_procedures: self.filter_generator
                    .generate_filter_procedures(self.query_syntax),
            }
        }
    }

    fn init_procedure(&mut self) -> Option<Procedure> {
        let absolute_subqueries_paths = self.filter_subquery_finder
            .get_all_subqueries_paths(self.query_syntax).into_iter()
            .filter(|subquery_path| !subquery_path.is_relative);
        let register_subquery_instructions: Vec<Instruction> = absolute_subqueries_paths.into_iter()
            .map(|subquery_path| RegisterSubqueryPath { subquery_path })
            .collect();
        if register_subquery_instructions.is_empty() {
            None
        } else {
            Procedure {
                name: "Init".to_string(),
                instructions: register_subquery_instructions,
            }.into()
        }
    }

    fn generate_procedure(&mut self, segments: &ProcedureSegments) {
        let mut instructions = Vec::new();
        if self.are_filters_in_query {
            instructions.push(TryUpdateSubqueries);
        }
        instructions.append(&mut self.generate_object_selectors(&segments));
        instructions.append(&mut self.generate_array_selectors(&segments));
        let segments_data = segments.segments_data();
        self.procedures_to_generate.remove(&segments_data);
        self.generated_procedures.insert(segments_data, Procedure {
            name: segments.name(),
            instructions
        });
    }

    fn generate_register_relative_subqueries(
        &mut self,
        segments: &ProcedureSegments,
        instructions: &mut Vec<Instruction>,
    ) {
        for segment_index in segments.segments() {
            let filters = FilterUtils::get_all_filters_in_segment(self.query_syntax, segment_index);
            for (filter_expr, filter_id) in filters {
                let relative_subqueries_paths = self.filter_subquery_finder
                    .get_subqueries_paths_in_filter(filter_expr, &filter_id).into_iter()
                    .filter(|subquery_path| subquery_path.is_relative);
                for subquery_path in relative_subqueries_paths {
                    instructions.push(RegisterSubqueryPath { subquery_path })
                }
            }
        }
    }

    fn generate_object_selectors(
        &mut self,
        segments: &ProcedureSegments
    ) -> Vec<Instruction> {
        let descendants = segments.descendants();
        let wildcards = segments.wildcards();
        let filters_successors = segments.filters_successors();
        let mut instructions = Vec::new();
        self.generate_register_relative_subqueries(segments, &mut instructions);
        for (name, occurrences) in segments.name_selectors() {
            let node_selected = !occurrences.finals().is_empty();
            let procedure_segments = ProcedureSegments::merge(
                self.query_syntax,
                vec![
                    descendants.clone(),
                    wildcards.successors(),
                    filters_successors.clone(),
                    occurrences.successors(),
                ],
            );
            let inner_instructions = self.generate_procedure_execution(
                &procedure_segments,
                node_selected,
                occurrences.finals().any_segment_selection_condition()
            );
            instructions.push(IfCurrentMemberNameEquals {
                name,
                instructions: inner_instructions
            });
        }
        instructions.append(&mut self.generate_wildcard_filter_and_descendant_selectors(segments));
        vec![ForEachMember { instructions }]
    }

    fn generate_array_selectors(
        &mut self,
        segments: &ProcedureSegments,
    ) -> Vec<Instruction> {
        let wildcards = segments.wildcards();
        let filters_successors = segments.filters_successors();
        let mut instructions = Vec::new();
        self.generate_register_relative_subqueries(segments, &mut instructions);
        for (index, occurrences) in segments.non_negative_index_selectors() {
            let node_selected = !occurrences.finals().is_empty();
            let procedure_segments = ProcedureSegments::merge(
                self.query_syntax,
                vec![
                    segments.descendants(),
                    wildcards.successors(),
                    filters_successors.clone(),
                    occurrences.successors(),
                ],
            );
            let mut inner_instructions = Vec::new();
            for (neg_index, occurrences) in occurrences.negative_index_selectors() {
                let neg_node_selected = !occurrences.finals().is_empty();
                let inner_inner_instructions = self.generate_procedure_execution(
                    &procedure_segments.merge_with(&occurrences.successors()),
                    node_selected || neg_node_selected,
                    occurrences.finals().any_segment_selection_condition()
                );
                inner_instructions.push(IfCurrentIndexFromEndEquals {
                    index: u64::from_ne_bytes(i64::abs(neg_index).to_ne_bytes()),
                    instructions: inner_inner_instructions,
                })
            }
            inner_instructions = inner_instructions.into_iter().chain(
                self.generate_procedure_execution(
                    &procedure_segments,
                    node_selected,
                    occurrences.finals().any_segment_selection_condition(),
                )
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
                occurrences.finals().any_segment_selection_condition()
            );
            instructions.push(IfCurrentIndexFromEndEquals {
                index: u64::from_ne_bytes(i64::abs(neg_index).to_ne_bytes()),
                instructions: inner_instructions,
            });
        }
        instructions.append(&mut self.generate_wildcard_filter_and_descendant_selectors(segments));
        vec![ForEachElement { instructions }]
    }

    fn generate_wildcard_filter_and_descendant_selectors(
        &mut self,
        segments: &ProcedureSegments,
    ) -> Vec<Instruction> {
        let descendant_segments = segments.descendants();
        let wildcard_segments = segments.wildcards();
        let filter_segments = segments.filters();
        let filters_successors = segments.filters_successors();
        let mut instructions = Vec::new();
        if !wildcard_segments.is_empty() || !filter_segments.is_empty() {
            let node_selected = !wildcard_segments.finals().is_empty()
                || !filter_segments.finals().is_empty();
            let procedure_segments = ProcedureSegments::merge(
                self.query_syntax,
                vec![
                    descendant_segments,
                    wildcard_segments.successors(),
                    filters_successors,
                ]);
            let mut selection_conditions = Vec::new();
            if !wildcard_segments.finals().is_empty() {
                selection_conditions.push(
                    wildcard_segments.finals().any_segment_selection_condition()
                );
            }
            if !filter_segments.finals().is_empty() {
                selection_conditions.push(filter_segments.filters_any_final_selection_condition());
            }
            if !procedure_segments.is_empty() {
                let procedure_name = self.get_or_create_procedure_for_segments(
                    &procedure_segments
                );
                instructions.push(Self::wrap_in_save_current_node_during_traversal_conditionally(
                    ExecuteProcedureOnChild { name: procedure_name },
                    node_selected,
                    SelectionCondition::merge(selection_conditions).into()
                ));
            } else {
                instructions.push(Self::wrap_in_save_current_node_during_traversal_conditionally(
                    TraverseCurrentNodeSubtree,
                    node_selected,
                    SelectionCondition::merge(selection_conditions).into()
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
        node_selected: bool,
        selection_condition: Option<SelectionCondition>
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        if !procedure_segments.is_empty() {
            let procedure_name = self.get_or_create_procedure_for_segments(
                procedure_segments
            );
            instructions.push(Self::wrap_in_save_current_node_during_traversal_conditionally(
                ExecuteProcedureOnChild { name: procedure_name },
                node_selected,
                selection_condition,
            ));
            instructions.push(Continue);
        } else {
            instructions.push(Self::wrap_in_save_current_node_during_traversal_conditionally(
                TraverseCurrentNodeSubtree,
                node_selected,
                selection_condition,
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
        selection_condition: Option<SelectionCondition>
    ) -> Instruction {
        if condition {
            SaveCurrentNodeDuringTraversal {
                instruction: Box::new(instruction),
                condition: selection_condition,
            }
        } else {
            instruction
        }
    }
}
