use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

use rsonpath_syntax::Selector;

use crate::ir::{Instruction, Procedure, Query};
use crate::ir::Instruction::{Continue, ExecuteProcedureOnChild, ForEachElement, ForEachMember,
                             IfCurrentIndexEquals, IfCurrentIndexFromEndEquals,
                             IfCurrentMemberNameEquals, SaveCurrentNodeDuringTraversal,
                             TraverseCurrentNodeSubtree};

pub struct IRGenerator<'a> {
    query_syntax: &'a rsonpath_syntax::JsonPathQuery,
    generated_procedures: HashMap<Vec<usize>, Procedure>,
    procedures_to_generate: HashSet<Vec<usize>>,
    procedure_queue: VecDeque<Vec<usize>>
}

impl IRGenerator<'_> {
    pub fn new(query_syntax: &rsonpath_syntax::JsonPathQuery) -> IRGenerator {
        IRGenerator {
            query_syntax,
            generated_procedures: HashMap::new(),
            procedures_to_generate: HashSet::new(),
            procedure_queue: VecDeque::new()
        }
    }

    pub fn generate(mut self) -> Query {
        let entry_procedure_sig = vec![0];
        let entry_procedure_name = Self::get_procedure_name(&entry_procedure_sig);
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
                ]
            }
        } else {
            self.procedure_queue.push_back(entry_procedure_sig.clone());
            self.procedures_to_generate.insert(entry_procedure_sig);
            while let Some(procedure_sig) = self.procedure_queue.pop_front() {
                self.generate_procedure(&procedure_sig);
            }
            let mut procedures: Vec<Procedure> = self.generated_procedures.into_values().collect();
            procedures.sort_by(|a, b| a.name.cmp(&b.name));
            Query { procedures }
        }
    }

    fn generate_procedure(&mut self, sig: &Vec<usize>) {
        let selectors: Vec<(&Selector, usize, usize)> = sig.iter()
            .map(|i| (self.query_syntax.segments()[*i].selectors(), i))
            .flat_map(
                |(selectors, segment)| selectors.iter().enumerate().map(
                    |(pos, selector)| (selector, *segment, pos)
                )
            )
            .collect();
        let object_selectors_instructions =
            self.generate_object_selectors(&selectors);
        let array_selectors_instructions =
            self.generate_array_selectors(&selectors);
        self.procedures_to_generate.remove(sig);
        self.generated_procedures.insert(sig.clone(), Procedure {
            name: Self::get_procedure_name(sig),
            instructions: object_selectors_instructions
                .into_iter().chain(array_selectors_instructions.into_iter()).collect()
        });
    }

    fn generate_object_selectors(
        &mut self,
        selectors: &Vec<(&Selector, usize, usize)>,
    ) -> Vec<Instruction> {
        let descendant_segments = self.get_descendant_segments(selectors);
        let wildcard_selectors = Self::get_wildcard_occurrences(selectors);
        let wildcard_segments = Self::get_segments_from_occurrences(&wildcard_selectors);
        let wildcard_next_segments = self.get_next_segments(&wildcard_segments);
        let name_selectors = Self::create_selector_occurrences_map(
            selectors,
            Self::extract_name_from_selector,
        );
        let mut instructions = Vec::new();
        for (name, occurrences) in name_selectors {
            let segments = Self::get_segments_from_occurrences(&occurrences);
            let next_segments = self.get_next_segments(&segments);
            let final_occurrences = self.get_final_segment_occurrences(&occurrences);
            let node_selected = !final_occurrences.is_empty();
            let procedure_segments = descendant_segments.clone().into_iter()
                .chain(wildcard_next_segments.clone())
                .chain(next_segments)
                .collect::<Vec<usize>>();
            let inner_instructions = self.generate_procedure_execution(procedure_segments, node_selected);
            instructions.push(IfCurrentMemberNameEquals {
                name,
                instructions: inner_instructions
            });
        }
        instructions.append(&mut self.generate_wildcard_and_descendant_selectors(
            wildcard_selectors,
            wildcard_segments,
            wildcard_next_segments,
            descendant_segments,
        ));
        vec![ForEachMember { instructions }]
    }

    fn generate_array_selectors(
        &mut self,
        selectors: &Vec<(&Selector, usize, usize)>,
    ) -> Vec<Instruction> {
        let descendant_segments = self.get_descendant_segments(selectors);
        let wildcard_selectors = Self::get_wildcard_occurrences(selectors);
        let wildcard_segments = Self::get_segments_from_occurrences(&wildcard_selectors);
        let wildcard_next_segments = self.get_next_segments(&wildcard_segments);
        let index_selectors = Self::create_selector_occurrences_map(
            selectors,
            Self::extract_index_from_selector,
        );
        let non_negative_index_selectors: HashMap<i64, Vec<(usize, usize)>> = index_selectors.iter()
            .filter(|(x, _)| **x >= 0)
            .map(|(x, occurrences)| (*x, occurrences.clone()))
            .collect();
        let negative_index_selectors: HashMap<i64, Vec<(usize, usize)>> = index_selectors.iter()
            .filter(|(x, _)| **x < 0)
            .map(|(x, occurrences)| (*x, occurrences.clone()))
            .collect();
        let mut instructions = Vec::new();
        for (index, occurrences) in non_negative_index_selectors {
            let segments = Self::get_segments_from_occurrences(&occurrences);
            let next_segments = self.get_next_segments(&segments);
            let final_occurrences = self.get_final_segment_occurrences(&occurrences);
            let node_selected = !final_occurrences.is_empty();
            let procedure_segments = descendant_segments.clone().into_iter()
                .chain(wildcard_next_segments.clone())
                .chain(next_segments)
                .collect::<Vec<usize>>();
            let mut inner_instructions = Vec::new();
            for (neg_index, occurrences) in &negative_index_selectors {
                let neg_segments = Self::get_segments_from_occurrences(&occurrences);
                let neg_next_segments = self.get_next_segments(&neg_segments);
                let neg_final_occurrences = self.get_final_segment_occurrences(&occurrences);
                let neg_node_selected = !neg_final_occurrences.is_empty();
                let inner_inner_instructions = self.generate_procedure_execution(
                    procedure_segments.clone().into_iter().chain(neg_next_segments).collect(),
                    node_selected || neg_node_selected,
                );
                inner_instructions.push(IfCurrentIndexFromEndEquals {
                    index: u64::from_ne_bytes(i64::abs(*neg_index).to_ne_bytes()),
                    instructions: inner_inner_instructions,
                })
            }
            inner_instructions = inner_instructions.into_iter().chain(
                self.generate_procedure_execution(procedure_segments, node_selected)
            ).collect();
            instructions.push(IfCurrentIndexEquals {
                index: u64::from_ne_bytes(index.to_ne_bytes()),
                instructions: inner_instructions,
            });
        }
        for (neg_index, occurrences) in negative_index_selectors {
            let segments = Self::get_segments_from_occurrences(&occurrences);
            let next_segments = self.get_next_segments(&segments);
            let final_occurrences = self.get_final_segment_occurrences(&occurrences);
            let node_selected = !final_occurrences.is_empty();
            let procedure_segments = descendant_segments.clone().into_iter()
                .chain(wildcard_next_segments.clone())
                .chain(next_segments)
                .collect::<Vec<usize>>();
            let inner_instructions = self.generate_procedure_execution(
                procedure_segments,
                node_selected,
            );
            instructions.push(IfCurrentIndexFromEndEquals {
                index: u64::from_ne_bytes(i64::abs(neg_index).to_ne_bytes()),
                instructions: inner_instructions,
            });
        }
        instructions.append(&mut self.generate_wildcard_and_descendant_selectors(
            wildcard_selectors,
            wildcard_segments,
            wildcard_next_segments,
            descendant_segments,
        ));
        vec![ForEachElement { instructions }]
    }

    fn generate_wildcard_and_descendant_selectors(
        &mut self,
        wildcard_selectors: Vec<(usize, usize)>,
        wildcard_segments: Vec<usize>,
        wildcard_next_segments: Vec<usize>,
        descendant_segments: Vec<usize>,
    ) -> Vec<Instruction> {
        let wildcard_final_occurrences =
            self.get_final_segment_occurrences(&wildcard_selectors);
        let mut instructions = Vec::new();
        if wildcard_segments.len() > 0 {
            let node_selected = !wildcard_final_occurrences.is_empty();
            let procedure_segments = descendant_segments.clone().into_iter()
                .chain(wildcard_next_segments.clone())
                .collect::<Vec<usize>>();
            if !procedure_segments.is_empty() {
                let procedure_name = self.get_or_create_procedure_for_segments(procedure_segments);
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
        } else if descendant_segments.len() > 0 {
            let procedure_name = self.get_or_create_procedure_for_segments(
                descendant_segments.clone().into_iter().collect::<Vec<usize>>()
            );
            instructions.push(ExecuteProcedureOnChild { name: procedure_name });
        }
        instructions
    }

    fn generate_procedure_execution(
        &mut self,
        procedure_segments: Vec<usize>,
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

    fn get_descendant_segments(&self, selectors: &Vec<(&Selector, usize, usize)>) -> Vec<usize> {
        selectors.iter()
            .filter(|(_, segment, _)| self.query_syntax.segments()[*segment].is_descendant())
            .map(|(_, segment, _)| *segment)
            .collect()
    }

    fn get_wildcard_occurrences(selectors: &Vec<(&Selector, usize, usize)>) -> Vec<(usize, usize)> {
        selectors.iter()
            .filter(|(selector, _, _)| selector.is_wildcard())
            .map(|(_, segment, pos)| (*segment, *pos))
            .collect()
    }

    fn get_segments_from_occurrences(occurrences: &Vec<(usize, usize)>) -> Vec<usize> {
        occurrences.iter().map(|(segment, _)| *segment).collect()
    }

    fn get_final_segment_occurrences(&self, occurrences: &Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        occurrences.iter()
            .filter(|(segment, _)| *segment == self.query_syntax.segments().len() - 1)
            .map(|(segment, pos)| (*segment, *pos))
            .collect()
    }

    fn get_next_segments(&self, segments: &Vec<usize>) -> Vec<usize> {
        segments.iter()
            .filter(|segment| *segment + 1 < self.query_syntax.segments().len())
            .map(|segment| segment + 1)
            .collect()
    }

    fn get_or_create_procedure_for_segments(&mut self, mut segments: Vec<usize>) -> String {
        segments = Self::sort_and_deduplicate(segments);
        if !self.generated_procedures.contains_key(&segments)
            && !self.procedures_to_generate.contains(&segments) {
            self.procedures_to_generate.insert(segments.clone());
            self.procedure_queue.push_back(segments.clone());
        }
        Self::get_procedure_name(&segments)
    }

    fn get_procedure_name(sig: &Vec<usize>) -> String {
        format!(
            "Selectors_{}",
            sig.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_")
        )
    }

    fn create_selector_occurrences_map<T: Eq + Hash>(
        selectors: &Vec<(&Selector, usize, usize)>,
        key: fn(&Selector) -> Option<T>,
    ) -> HashMap<T, Vec<(usize, usize)>> {
        let mut map: HashMap<T, Vec<(usize, usize)>> = HashMap::new();
        for (selector, segment, pos) in selectors.iter() {
            if let Some(key) = key(selector) {
                if let Some(occurrences) = map.get_mut(&key) {
                    occurrences.push((*segment, *pos));
                } else {
                    map.insert(key, vec![(*segment, *pos)]);
                }
            }
        }
        map
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

    fn extract_name_from_selector(selector: &Selector) -> Option<String> {
        if let Selector::Name(name_json_str) = selector {
            Some(name_json_str.unquoted().to_string())
        } else {
            None
        }
    }

    fn extract_index_from_selector(selector: &Selector) -> Option<i64> {
        if let Selector::Index(index_syntax) = selector {
            Some(Self::get_index_syntax_as_i64(index_syntax))
        } else {
            None
        }
    }

    fn get_index_syntax_as_i64(index_syntax: &rsonpath_syntax::Index) -> i64 {
        match index_syntax {
            rsonpath_syntax::Index::FromStart(num) => num.as_u64() as i64,
            rsonpath_syntax::Index::FromEnd(num) => -(num.as_u64() as i64),
        }
    }

    fn get_step_syntax_as_i64(step_syntax: &rsonpath_syntax::Step) -> i64 {
        match step_syntax {
            rsonpath_syntax::Step::Forward(num) => num.as_u64() as i64,
            rsonpath_syntax::Step::Backward(num) => -(num.as_u64() as i64),
        }
    }

    fn sort_and_deduplicate(mut v: Vec<usize>) -> Vec<usize> {
        v.sort();
        v.dedup();
        v
    }
}
