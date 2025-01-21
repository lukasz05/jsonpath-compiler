use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use itertools::Itertools;

use crate::ir::{FilterId, SegmentIndex, SelectionCondition};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ProcedureSegmentsData {
    segments_with_conditions: Vec<(SegmentIndex, Option<SelectionCondition>)>,
}

impl ProcedureSegmentsData {
    fn new(segments: Vec<SegmentIndex>) -> ProcedureSegmentsData {
        ProcedureSegmentsData {
            segments_with_conditions: segments.iter().map(|i| (*i, None)).collect()
        }
    }

    fn new_with_conditions(
        segments_with_conditions: Vec<(SegmentIndex, Option<SelectionCondition>)>
    ) -> ProcedureSegmentsData {
        ProcedureSegmentsData {
            segments_with_conditions
        }
    }

    fn name(&self) -> String {
        let hash_str = if self.segments_with_conditions.iter()
            .any(|(_, condition)| condition.is_some()) {
            let mut hasher = DefaultHasher::default();
            self.hash(&mut hasher);
            hasher.finish().to_string()
        } else { "".to_string() };
        let segments_str = self.segments_with_conditions.iter()
            .map(|(i, condition)| i.to_string() + if condition.is_some() { "c" } else { "" })
            .collect::<Vec<String>>()
            .join("_");
        let mut name = format!("Selectors_{segments_str}");
        if !hash_str.is_empty() {
            name = name + "_" + &hash_str;
        }
        name
    }

    fn is_empty(&self) -> bool {
        self.segments_with_conditions.is_empty()
    }

    pub fn segments(&self) -> Vec<SegmentIndex> {
        self.segments_with_conditions.iter().map(|(i, _)| *i).collect()
    }

    pub fn segments_with_conditions(&self) -> Vec<(SegmentIndex, Option<SelectionCondition>)> {
        self.segments_with_conditions.clone()
    }
}

#[derive(Clone)]
pub struct ProcedureSegments<'a> {
    query: &'a rsonpath_syntax::JsonPathQuery,
    segments_data: ProcedureSegmentsData,
}

impl ProcedureSegments<'_> {
    pub fn new(
        query: &rsonpath_syntax::JsonPathQuery,
        segments_with_conditions: Vec<(SegmentIndex, Option<SelectionCondition>)>,
    ) -> ProcedureSegments {
        ProcedureSegments {
            query,
            segments_data: ProcedureSegmentsData::new_with_conditions(segments_with_conditions),
        }
    }

    pub fn name(&self) -> String {
        self.segments_data.name()
    }

    pub fn segments(&self) -> Vec<SegmentIndex> {
        self.segments_data.segments()
    }

    pub fn segments_with_conditions(&self) -> Vec<(SegmentIndex, Option<SelectionCondition>)> {
        self.segments_data.segments_with_conditions()
    }

    pub fn segment_selection_condition(&self, segment_index: SegmentIndex) -> Option<SelectionCondition> {
        let (_, condition) = self.segments_data.segments_with_conditions().into_iter()
            .find(|(index, _)| *index == segment_index)
            .unwrap();
        condition
    }

    pub fn any_segment_selection_condition(&self) -> Option<SelectionCondition> {
        let conditions: Vec<SelectionCondition> = self.segments_with_conditions().into_iter()
            .filter(|(_, condition)| condition.is_some())
            .map(|(_, condition)| condition.unwrap())
            .collect();
        if conditions.is_empty() {
            None
        } else {
            SelectionCondition::merge(conditions.into_iter().map_into().collect())
        }
    }

    pub fn unconditional_segments(&self) -> Vec<SegmentIndex> {
        self.segments_with_conditions().iter()
            .filter(|(_, condition)| condition.is_none())
            .map(|(i, _)| *i)
            .collect()
    }

    pub fn segments_data(&self) -> ProcedureSegmentsData {
        self.segments_data.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.segments_data.is_empty()
    }

    pub fn descendants(&self) -> ProcedureSegments {
        let segments_with_conditions = self.filter_and_map_segments(
            |s| self.query.segments()[s].is_descendant(),
            |s| s,
        );
        ProcedureSegments::new(self.query, segments_with_conditions)
    }

    pub fn successors(&self) -> ProcedureSegments {
        let segments_with_conditions = self.filter_and_map_segments(
            |s| s + 1 < self.query.segments().len(),
            |s| s + 1,
        );
        ProcedureSegments::new(self.query, segments_with_conditions)
    }

    pub fn filters_successors(&self) -> ProcedureSegments {
        let filters_ids = self.filter_selectors();
        let mut segments_with_conditions: HashMap<SegmentIndex, Option<SelectionCondition>> = HashMap::new();
        for filter_id in &filters_ids {
            if filter_id.segment_index + 1 >= self.query.segments().len() {
                continue
            }
            let segment_condition = self.segment_selection_condition(filter_id.segment_index);
            let filter_condition = SelectionCondition::Filter { id: filter_id.clone() };
            let successor_condition = if let Some(segment_condition) = segment_condition {
                segment_condition.and(&filter_condition)
            } else {
                filter_condition.clone()
            };
            if let Some(Some(existing_condition)) = segments_with_conditions.get(&(filter_id.segment_index + 1)) {
                segments_with_conditions.insert(
                    filter_id.segment_index + 1,
                    existing_condition.merge_with(&successor_condition, true).into(),
                );
            } else {
                segments_with_conditions.insert(filter_id.segment_index + 1, successor_condition.into());
            }
        }
        ProcedureSegments::new(
            self.query,
            segments_with_conditions.into_iter()
                .map(|(key, value)| (key, value))
                .collect(),
        )
    }

    pub fn finals(&self) -> ProcedureSegments {
        let segment_count = self.query.segments().len();
        let segments_with_conditions = self.filter_and_map_segments(
            |s| s == segment_count - 1,
            |s| s,
        );
        ProcedureSegments::new(self.query, segments_with_conditions)
    }

    pub fn filters_any_final_selection_condition(&self) -> Option<SelectionCondition> {
        let filters_ids: Vec<FilterId> = self.filter_selectors().into_iter()
            .filter(|filter_id| filter_id.segment_index + 1 == self.query.segments().len())
            .collect();
        let mut selection_conditions = Vec::new();
        for filter_id in &filters_ids {
            let segment_condition = self.segment_selection_condition(filter_id.segment_index);
            let filter_condition = SelectionCondition::Filter { id: filter_id.clone() };
            selection_conditions.push(if let Some(segment_condition) = segment_condition {
                segment_condition.and(&filter_condition)
            } else {
                filter_condition
            });
        }
        SelectionCondition::merge(selection_conditions.into_iter().map_into().collect())
    }

    pub fn wildcards(&self) -> ProcedureSegments {
        let segments_with_conditions = self.filter_and_map_segments(
            |s| self.query.segments()[s].selectors().iter().any(|sel| sel.is_wildcard()),
            |s| s,
        );
        ProcedureSegments::new(self.query, segments_with_conditions)
    }

    pub fn name_selectors(&self) -> HashMap<String, ProcedureSegments> {
        self.selector_to_segments_map(
            |selector| {
                if let rsonpath_syntax::Selector::Name(name) = selector {
                    Some(name.unquoted().to_string())
                } else {
                    None
                }
            }
        )
    }

    pub fn index_selectors(&self) -> HashMap<i64, ProcedureSegments> {
        self.selector_to_segments_map(
            |selector| {
                if let rsonpath_syntax::Selector::Index(index) = selector {
                    Some(match index {
                        rsonpath_syntax::Index::FromStart(num) => num.as_u64() as i64,
                        rsonpath_syntax::Index::FromEnd(num) => -(num.as_u64() as i64),
                    })
                } else {
                    None
                }
            }
        )
    }

    pub fn non_negative_index_selectors(&self) -> HashMap<i64, ProcedureSegments> {
        self.index_selectors().into_iter().filter(|(key, _)| *key >= 0).collect()
    }

    pub fn negative_index_selectors(&self) -> HashMap<i64, ProcedureSegments> {
        self.index_selectors().into_iter().filter(|(key, _)| *key < 0).collect()
    }

    pub fn filters(&self) -> ProcedureSegments {
        let segments_with_conditions = self.filter_and_map_segments(
            |s| self.query.segments()[s].selectors().iter().any(|sel| sel.is_filter()),
            |s| s,
        );
        ProcedureSegments::new(self.query, segments_with_conditions)
    }

    pub fn merge_with(&self, other: &ProcedureSegments) -> ProcedureSegments {
        let mut segments_with_conditions: HashMap<SegmentIndex, Option<SelectionCondition>> =
            self.segments_with_conditions().into_iter().collect();
        for (segment_id, condition) in other.segments_with_conditions() {
            if !segments_with_conditions.contains_key(&segment_id) {
                segments_with_conditions.insert(segment_id, condition);
                continue;
            }
            let current_condition = segments_with_conditions[&segment_id].clone();
            if current_condition.is_none() {
                continue;
            }
            let current_condition = current_condition.unwrap();
            let new_condition = if condition.is_none() { None } else {
                Some(condition.unwrap().merge_with(&current_condition, true))
            };
            segments_with_conditions.insert(segment_id, new_condition);
        }
        let max_descendant_segment = segments_with_conditions.iter()
            .filter(|(_, condition)| condition.is_none())
            .map(|(segment_id, _)| *segment_id)
            .filter(|segment_id| self.query.segments()[*segment_id].is_descendant())
            .max();
        if max_descendant_segment.is_none() {
            ProcedureSegments::new(self.query, segments_with_conditions.into_iter().collect())
        } else {
            let max_descendant_segment = max_descendant_segment.unwrap();
            let segments_with_conditions = segments_with_conditions.into_iter().filter(
                |(i, condition)| {
                    *i == max_descendant_segment || !self.query.segments()[*i].is_descendant()
                        || (*i > max_descendant_segment && condition.is_some())
                }
            );
            ProcedureSegments::new(self.query, segments_with_conditions.collect())
        }
    }

    pub fn merge<'a>(
        query: &'a rsonpath_syntax::JsonPathQuery,
        segments_vec: Vec<ProcedureSegments<'a>>,
    ) -> ProcedureSegments<'a> {
        let merged_segments = segments_vec.iter().fold(
            ProcedureSegments::new(query, vec![]),
            |segments, new_segments| new_segments.merge_with(&segments),
        );
        ProcedureSegments::new(query, merged_segments.segments_with_conditions().clone())
    }

    fn filter_and_map_segments(
        &self,
        f: impl Fn(SegmentIndex) -> bool,
        m: impl Fn(SegmentIndex) -> SegmentIndex,
    ) -> Vec<(SegmentIndex, Option<SelectionCondition>)> {
        self.segments_with_conditions().into_iter()
            .filter(|(segment_index, _)| f(*segment_index))
            .map(|(segment_index, condition)| (m(segment_index), condition))
            .collect()
    }

    fn selector_to_segments_map<T: Eq + Hash>(
        &self,
        get_key: impl Fn(&rsonpath_syntax::Selector) -> Option<T>,
    ) -> HashMap<T, ProcedureSegments> {
        let mut map: HashMap<T, Vec<(SegmentIndex, Option<SelectionCondition>)>> = HashMap::new();
        for (segment_index, condition) in self.segments_with_conditions() {
            let selectors = self.query.segments()[segment_index].selectors();
            for selector in selectors.iter() {
                if let Some(key) = get_key(selector) {
                    match map.get_mut(&key) {
                        Some(segments_with_conditions) => {
                            segments_with_conditions.push((segment_index, condition.clone()))
                        },
                        None => {
                            map.insert(key, vec![(segment_index, condition.clone())]);
                        }
                    }
                }
            }
        }
        map.into_iter()
            .map(|(key, segments)| {
                (key, ProcedureSegments::new(self.query, segments))
            })
            .collect()
    }

    fn successors_segments_with_conditions(&self) -> Vec<(SegmentIndex, Option<SelectionCondition>)> {
        let segment_count = self.query.segments().len();
        self.filter_and_map_segments(
            |s| s + 1 < segment_count,
            |s| s + 1,
        )
    }

    fn filter_selectors(&self) -> Vec<FilterId> {
        self.segments().into_iter()
            .flat_map(|segment_id| {
                self.query.segments()[segment_id].selectors().into_iter()
                    .enumerate()
                    .map(move |(index, selector)| ((segment_id, index), selector))
            })
            .filter(|(_, selector)| selector.is_filter())
            .map(|((segment_index, selector_index), _)| {
                FilterId::new(segment_index, selector_index)
            })
            .collect()
    }
}