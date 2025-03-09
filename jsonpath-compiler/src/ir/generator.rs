use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;

use crate::ir::{Instruction, Procedure, Query, SelectionCondition};
use crate::ir::filter_generator::{FilterGenerator, FilterSubqueryFinder, FilterUtils};
use crate::ir::Instruction::{Continue, EndFilterExecution, ExecuteProcedureOnChild, ForEachElement, ForEachMember, IfActiveFilterInstance, IfCurrentIndexEquals, IfCurrentIndexFromEndEquals, IfCurrentMemberNameEquals, SaveCurrentNodeDuringTraversal, StartFilterExecution, TraverseCurrentNodeSubtree, UpdateSubqueriesState};
use crate::ir::procedure_segments::{ProcedureSegments, ProcedureSegmentsData};

pub struct IRGenerator<'a> {
    query_syntax: &'a rsonpath_syntax::JsonPathQuery,
    generated_procedures: HashMap<ProcedureSegmentsData, Procedure>,
    procedures_to_generate: HashSet<ProcedureSegmentsData>,
    procedure_queue: VecDeque<ProcedureSegmentsData>,
    filter_generator: FilterGenerator,
    are_filters_in_query: bool,
}

impl IRGenerator<'_> {
    pub fn new(query_syntax: &rsonpath_syntax::JsonPathQuery) -> IRGenerator {
        IRGenerator {
            query_syntax,
            generated_procedures: HashMap::new(),
            procedures_to_generate: HashSet::new(),
            procedure_queue: VecDeque::new(),
            filter_generator: FilterGenerator::new(),
            are_filters_in_query: !FilterUtils::get_all_filters(query_syntax).is_empty(),
        }
    }

    pub fn generate(mut self) -> Query {
        let first_segment = ProcedureSegments::new(self.query_syntax, vec![0]);
        let first_segment_procedure_name = first_segment.name();
        if self.query_syntax.segments().is_empty() {
            Query {
                procedures: vec![Procedure {
                    name: first_segment_procedure_name,
                    instructions: vec![SaveCurrentNodeDuringTraversal {
                        instruction: Box::new(TraverseCurrentNodeSubtree),
                        condition: None,
                    }],
                }],
                filter_procedures: HashMap::new(),
                filter_subqueries: HashMap::new(),
                segments_count: 0,
            }
        } else {
            let segments_data = first_segment.segments_data();
            self.procedure_queue.push_back(segments_data.clone());
            self.procedures_to_generate.insert(segments_data);
            while let Some(procedure_segments) = self.procedure_queue.pop_front() {
                self.generate_procedure(&ProcedureSegments::new(
                    self.query_syntax,
                    procedure_segments.segments(),
                ));
            }
            let procedures: Vec<Procedure> = self
                .generated_procedures
                .into_values()
                .sorted_by(|a, b| a.name.cmp(&b.name))
                .collect();
            Query {
                procedures,
                filter_procedures: self
                    .filter_generator
                    .generate_filter_procedures(self.query_syntax),
                filter_subqueries: FilterSubqueryFinder::get_all_subqueries(self.query_syntax),
                segments_count: self.query_syntax.segments().len(),
            }
        }
    }

    fn generate_procedure(&mut self, segments: &ProcedureSegments) {
        let mut instructions = Vec::new();
        if self.are_filters_in_query {
            instructions.push(UpdateSubqueriesState);
        }
        instructions.append(&mut self.generate_object_selectors(&segments));
        instructions.append(&mut self.generate_array_selectors(&segments));
        let segments_data = segments.segments_data();
        self.procedures_to_generate.remove(&segments_data);
        self.generated_procedures.insert(
            segments_data,
            Procedure {
                name: segments.name(),
                instructions,
            },
        );
    }

    fn generate_start_filters_execution(
        &mut self,
        segments: &ProcedureSegments,
        instructions: &mut Vec<Instruction>,
    ) {
        for segment_index in segments.segments() {
            let filters = FilterUtils::get_filters_in_segment(self.query_syntax, segment_index);
            for (_, filter_id) in filters {
                instructions.push(StartFilterExecution { filter_id })
            }
        }
    }

    fn get_successors_segments_conditions(
        &self,
        successors_segments_excluding_filters: &ProcedureSegments,
        filters_segments: &ProcedureSegments,
    ) -> Vec<Option<SelectionCondition>> {
        let max_segment = successors_segments_excluding_filters
            .merge_with(&filters_segments.successors())
            .segments()
            .into_iter()
            .max();
        if max_segment.is_none() {
            return vec![];
        }
        let mut successors_segments_conditions = vec![None; max_segment.unwrap() + 1];
        for segment_index in filters_segments.segments().into_iter().sorted() {
            let successor_index = filters_segments.successor(segment_index);
            if successor_index.is_none() {
                continue;
            }
            let successor_index = successor_index.unwrap();
            if successors_segments_excluding_filters
                .segments()
                .contains(&successor_index)
            {
                continue;
            }
            let filters_in_segment =
                FilterUtils::get_filters_in_segment(self.query_syntax, segment_index);
            let conditions = filters_in_segment
                .into_iter()
                .map(|(_, filter_id)| Some(SelectionCondition::Filter { id: filter_id }))
                .collect();
            successors_segments_conditions[successor_index] = SelectionCondition::merge(conditions);
        }
        successors_segments_conditions
    }

    fn get_selection_condition(final_segments: &ProcedureSegments) -> Option<SelectionCondition> {
        let final_segments = final_segments.segments();
        if final_segments.len() > 0 {
            SelectionCondition::merge(
                final_segments
                    .into_iter()
                    .map(|segment_index| {
                        Some(SelectionCondition::RuntimeSegmentCondition { segment_index })
                    })
                    .collect(),
            )
        } else {
            None
        }
    }

    fn get_selection_condition_with_filters(
        &self,
        final_segments: &ProcedureSegments,
        final_filter_segments: &ProcedureSegments,
    ) -> Option<SelectionCondition> {
        let final_segments_excluding_filters = final_segments.segments();
        let final_filter_segments = final_filter_segments
            .segments()
            .into_iter()
            .filter(|segment_index| !final_segments_excluding_filters.contains(segment_index));
        let mut filter_selection_conditions = Vec::new();
        for segment_index in final_filter_segments {
            let filters = FilterUtils::get_filters_in_segment(self.query_syntax, segment_index);
            let conditions = filters
                .into_iter()
                .map(|(_, filter_id)| Some(SelectionCondition::Filter { id: filter_id }))
                .collect();
            filter_selection_conditions.push(SelectionCondition::And {
                lhs: Box::new(SelectionCondition::RuntimeSegmentCondition { segment_index }),
                rhs: Box::new(SelectionCondition::merge(conditions).unwrap()),
            });
        }
        let filter_selection_condition = SelectionCondition::merge(
            filter_selection_conditions
                .into_iter()
                .map(|condition| Some(condition))
                .collect(),
        );
        let non_filter_selection_condition = Self::get_selection_condition(final_segments);
        if non_filter_selection_condition.is_some() {
            if filter_selection_condition.is_some() {
                SelectionCondition::merge(vec![
                    filter_selection_condition,
                    non_filter_selection_condition,
                ])
            } else {
                non_filter_selection_condition
            }
        } else {
            filter_selection_condition
        }
    }

    fn generate_object_selectors(&mut self, segments: &ProcedureSegments) -> Vec<Instruction> {
        let descendant_segments = segments.descendants();
        let wildcards_segments = segments.wildcards();
        let filters_segments = segments.filters();
        let filters_successors = filters_segments.successors();
        let mut instructions = Vec::new();
        self.generate_start_filters_execution(segments, &mut instructions);
        for (name, occurrences) in segments.name_selectors() {
            let node_selected = !occurrences.finals().is_empty()
                || !wildcards_segments.finals().is_empty() || !filters_segments.finals().is_empty();
            let successors_segments_excluding_filters = ProcedureSegments::merge(
                self.query_syntax,
                vec![
                    descendant_segments.clone(),
                    wildcards_segments.successors(),
                    occurrences.successors(),
                ],
            );
            let successors_segments_conditions = self.get_successors_segments_conditions(
                &successors_segments_excluding_filters,
                &filters_segments,
            );
            let selection_condition = self.get_selection_condition_with_filters(
                &ProcedureSegments::merge(self.query_syntax, vec![
                    occurrences.finals(),
                    wildcards_segments.finals(),
                ]),
                &filters_segments.finals(),
            );
            let inner_instructions = self.generate_procedure_execution(
                segments,
                &successors_segments_excluding_filters.merge_with(&filters_successors),
                node_selected,
                selection_condition,
                successors_segments_conditions,
            );
            instructions.push(IfCurrentMemberNameEquals {
                name,
                instructions: inner_instructions,
            });
        }
        instructions.append(&mut self.generate_wildcard_filter_and_descendant_selectors(segments));
        self.generate_end_filter_execution_instructions(segments, &mut instructions);
        vec![ForEachMember { instructions }]
    }

    fn generate_array_selectors(&mut self, segments: &ProcedureSegments) -> Vec<Instruction> {
        let wildcards_segments = segments.wildcards();
        let filters_segments = segments.filters();
        let filters_successors = filters_segments.successors();
        let mut instructions = Vec::new();
        self.generate_start_filters_execution(segments, &mut instructions);
        for (index, occurrences) in segments.non_negative_index_selectors() {
            let node_selected = !occurrences.finals().is_empty()
                || !wildcards_segments.finals().is_empty() || !filters_segments.finals().is_empty();
            let successors_segments_excluding_filters = ProcedureSegments::merge(
                self.query_syntax,
                vec![
                    segments.descendants(),
                    wildcards_segments.successors(),
                    occurrences.successors(),
                ],
            );
            let mut inner_instructions = Vec::new();
            for (neg_index, neg_occurrences) in occurrences.negative_index_selectors() {
                let neg_node_selected = !neg_occurrences.finals().is_empty()
                    || !wildcards_segments.finals().is_empty()
                    || !filters_segments.finals().is_empty();
                let successors_segments_excluding_filters =
                    successors_segments_excluding_filters.merge_with(&neg_occurrences.successors());
                let successors_segments_conditions = self.get_successors_segments_conditions(
                    &successors_segments_excluding_filters,
                    &filters_segments,
                );
                let selection_condition = self.get_selection_condition_with_filters(
                    &ProcedureSegments::merge(self.query_syntax, vec![
                        occurrences.finals(),
                        neg_occurrences.finals(),
                        wildcards_segments.finals(),
                    ]),
                    &filters_segments.finals(),
                );
                let inner_inner_instructions = self.generate_procedure_execution(
                    segments,
                    &successors_segments_excluding_filters.merge_with(&filters_successors),
                    node_selected || neg_node_selected,
                    selection_condition,
                    successors_segments_conditions,
                );
                inner_instructions.push(IfCurrentIndexFromEndEquals {
                    index: u64::from_ne_bytes(i64::abs(neg_index).to_ne_bytes()),
                    instructions: inner_inner_instructions,
                })
            }
            let successors_segments_conditions = self.get_successors_segments_conditions(
                &successors_segments_excluding_filters,
                &filters_segments,
            );
            let selection_condition = self.get_selection_condition_with_filters(
                &ProcedureSegments::merge(self.query_syntax, vec![
                    occurrences.finals(),
                    wildcards_segments.finals(),
                ]),
                &filters_segments.finals(),
            );
            inner_instructions = inner_instructions
                .into_iter()
                .chain(self.generate_procedure_execution(
                    segments,
                    &successors_segments_excluding_filters.merge_with(&filters_successors),
                    node_selected,
                    selection_condition,
                    successors_segments_conditions,
                ))
                .collect();
            instructions.push(IfCurrentIndexEquals {
                index: u64::from_ne_bytes(index.to_ne_bytes()),
                instructions: inner_instructions,
            });
        }
        for (neg_index, occurrences) in segments.negative_index_selectors() {
            let node_selected = !occurrences.finals().is_empty()
                || !wildcards_segments.finals().is_empty() || !filters_segments.finals().is_empty();
            let successors_segments_excluding_filters = ProcedureSegments::merge(
                self.query_syntax,
                vec![
                    segments.descendants(),
                    wildcards_segments.successors(),
                    occurrences.successors(),
                ],
            );
            let successors_segments_conditions = self.get_successors_segments_conditions(
                &successors_segments_excluding_filters,
                &filters_segments,
            );
            let selection_condition = self.get_selection_condition_with_filters(
                &ProcedureSegments::merge(self.query_syntax, vec![
                    occurrences.finals(),
                    wildcards_segments.finals(),
                ]),
                &filters_segments.finals(),
            );
            let inner_instructions = self.generate_procedure_execution(
                segments,
                &successors_segments_excluding_filters.merge_with(&filters_successors),
                node_selected,
                selection_condition,
                successors_segments_conditions,
            );
            instructions.push(IfCurrentIndexFromEndEquals {
                index: u64::from_ne_bytes(i64::abs(neg_index).to_ne_bytes()),
                instructions: inner_instructions,
            });
        }
        instructions.append(&mut self.generate_wildcard_filter_and_descendant_selectors(segments));
        self.generate_end_filter_execution_instructions(segments, &mut instructions);
        vec![ForEachElement { instructions }]
    }

    fn generate_wildcard_filter_and_descendant_selectors(
        &mut self,
        segments: &ProcedureSegments,
    ) -> Vec<Instruction> {
        let descendants_segments = segments.descendants();
        let wildcards_segments = segments.wildcards();
        let filters_segments = segments.filters();
        let filters_successors = filters_segments.successors();
        let mut instructions = Vec::new();
        if !wildcards_segments.is_empty() || !filters_segments.is_empty() {
            let node_selected =
                !wildcards_segments.finals().is_empty() || !filters_segments.finals().is_empty();
            let selection_condition = self.get_selection_condition_with_filters(
                &wildcards_segments.finals(),
                &filters_segments.finals(),
            );
            let successors_segments_excluding_filters = ProcedureSegments::merge(
                self.query_syntax,
                vec![descendants_segments, wildcards_segments.successors()],
            );
            let successors_segments =
                successors_segments_excluding_filters.merge_with(&filters_successors);
            let successors_segments_conditions = self.get_successors_segments_conditions(
                &successors_segments_excluding_filters,
                &filters_segments,
            );
            instructions.append(&mut self.generate_procedure_execution(
                segments,
                &successors_segments,
                node_selected,
                selection_condition,
                successors_segments_conditions,
            ));
        } else if !descendants_segments.is_empty() {
            let procedure_name = self.get_or_create_procedure_for_segments(&descendants_segments);
            instructions.push(ExecuteProcedureOnChild {
                name: procedure_name,
                conditions: vec![None; descendants_segments.segments().len()],
            });
            instructions.push(Continue);
        } else {
            instructions.push(IfActiveFilterInstance {
                instructions: vec![TraverseCurrentNodeSubtree]
            });
        }
        instructions
    }

    fn generate_procedure_execution(
        &mut self,
        caller_segments: &ProcedureSegments,
        procedure_segments: &ProcedureSegments,
        node_selected: bool,
        node_selection_condition: Option<SelectionCondition>,
        segments_selection_conditions: Vec<Option<SelectionCondition>>,
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        if !procedure_segments.is_empty() {
            let procedure_name = self.get_or_create_procedure_for_segments(procedure_segments);
            instructions.push(
                Self::wrap_in_save_current_node_during_traversal_conditionally(
                    ExecuteProcedureOnChild {
                        name: procedure_name,
                        conditions: segments_selection_conditions,
                    },
                    node_selected,
                    node_selection_condition,
                ),
            );
            self.generate_end_filter_execution_instructions(caller_segments, &mut instructions);
            instructions.push(Continue);
        } else {
            instructions.push(
                Self::wrap_in_save_current_node_during_traversal_conditionally(
                    TraverseCurrentNodeSubtree,
                    node_selected,
                    node_selection_condition,
                ),
            );
            self.generate_end_filter_execution_instructions(caller_segments, &mut instructions);
            instructions.push(Continue);
        }
        instructions
    }

    fn get_or_create_procedure_for_segments(&mut self, segments: &ProcedureSegments) -> String {
        let segments_data = segments.segments_data();
        if !self.generated_procedures.contains_key(&segments_data)
            && !self.procedures_to_generate.contains(&segments_data)
        {
            self.procedures_to_generate.insert(segments_data.clone());
            self.procedure_queue.push_back(segments_data);
        }
        segments.name()
    }

    fn generate_end_filter_execution_instructions(
        &self,
        segments: &ProcedureSegments,
        instructions: &mut Vec<Instruction>,
    ) {
        let filters_in_caller_segments_count = segments
            .segments()
            .into_iter()
            .map(|segment_index| {
                FilterUtils::get_filters_in_segment(self.query_syntax, segment_index).len()
            })
            .sum();
        (0..filters_in_caller_segments_count).for_each(|_| instructions.push(EndFilterExecution));
    }

    fn wrap_in_save_current_node_during_traversal_conditionally(
        instruction: Instruction,
        condition: bool,
        selection_condition: Option<SelectionCondition>,
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
