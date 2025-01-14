use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

type SegmentIndex = usize;
type SelectorIndex = usize;

#[derive(Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
pub struct FilterId {
    segment_index: SegmentIndex,
    selector_index: SelectorIndex,
}

impl FilterId {
    pub fn new(segment_index: SegmentIndex, selector_index: SelectorIndex) -> FilterId {
        FilterId { segment_index, selector_index }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
pub enum SegmentCondition {
    Filter { id: FilterId },
    Or { lhs: Box<SegmentCondition>, rhs: Box<SegmentCondition> },
    And { lhs: Box<SegmentCondition>, rhs: Box<SegmentCondition> },
}

impl SegmentCondition {
    fn merge_with(&self, other: &SegmentCondition, normalize: bool) -> SegmentCondition {
        let merged = SegmentCondition::Or {
            lhs: Box::new(self.clone()),
            rhs: Box::new(other.clone()),
        };
        if normalize { merged.normalize() } else { merged }
    }

    fn normalize(&self) -> SegmentCondition {
        match self {
            SegmentCondition::Or { lhs, rhs } => {
                let (lhs, rhs) = Self::normalize_and_sort_subconditions(
                    *lhs.clone(),
                    *rhs.clone(),
                );
                if let Some(rhs) = rhs {
                    SegmentCondition::Or {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }
                } else {
                    lhs
                }
            }
            SegmentCondition::And { lhs, rhs } => {
                let (lhs, rhs) = Self::normalize_and_sort_subconditions(
                    *lhs.clone(),
                    *rhs.clone(),
                );
                if let Some(rhs) = rhs {
                    SegmentCondition::And {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }
                } else {
                    lhs
                }
            }
            _ => self.clone()
        }
    }

    fn merge(conditions: Vec<SegmentCondition>) -> SegmentCondition {
        let merged_conditions = conditions.iter().fold(
            conditions[0].clone(),
            |merged_conditions, condition| condition.merge_with(&merged_conditions, false),
        );
        merged_conditions.normalize()
    }

    fn normalize_and_sort_subconditions(
        lhs: SegmentCondition,
        rhs: SegmentCondition,
    ) -> (SegmentCondition, Option<SegmentCondition>) {
        let normalized_lhs = lhs.normalize();
        let normalized_rhs = rhs.normalize();
        if normalized_lhs == normalized_rhs {
            (normalized_lhs, None)
        } else if normalized_lhs < normalized_rhs {
            (lhs, Some(rhs))
        } else {
            (rhs, Some(lhs))
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ProcedureSegmentsData {
    segments_with_conditions: Vec<(SegmentIndex, Option<SegmentCondition>)>,
}

impl ProcedureSegmentsData {
    fn new(segments: Vec<SegmentIndex>) -> ProcedureSegmentsData {
        ProcedureSegmentsData {
            segments_with_conditions: segments.iter().map(|i| (*i, None)).collect()
        }
    }

    fn new_with_conditions(
        segments_with_conditions: Vec<(SegmentIndex, Option<SegmentCondition>)>
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

    pub fn segments_with_conditions(&self) -> Vec<(SegmentIndex, Option<SegmentCondition>)> {
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
        segments_with_conditions: Vec<(SegmentIndex, Option<SegmentCondition>)>,
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

    pub fn segments_with_conditions(&self) -> Vec<(SegmentIndex, Option<SegmentCondition>)> {
        self.segments_data.segments_with_conditions()
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
        let segment_count = self.query.segments().len();
        let segments_with_conditions = self.filter_and_map_segments(
            |s| s + 1 < segment_count,
            |s| s + 1,
        );
        ProcedureSegments::new(self.query, segments_with_conditions)
    }

    pub fn finals(&self) -> ProcedureSegments {
        let segment_count = self.query.segments().len();
        let segments_with_conditions = self.filter_and_map_segments(
            |s| s == segment_count - 1,
            |s| s,
        );
        ProcedureSegments::new(self.query, segments_with_conditions)
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

    pub fn merge_with(&self, other: &ProcedureSegments) -> ProcedureSegments {
        let mut segments_with_conditions: HashMap<SegmentIndex, Option<SegmentCondition>> =
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
    ) -> Vec<(SegmentIndex, Option<SegmentCondition>)> {
        self.segments_with_conditions().into_iter()
            .filter(|(segment_index, _)| f(*segment_index))
            .map(|(segment_index, condition)| (m(segment_index), condition))
            .collect()
    }

    fn selector_to_segments_map<T: Eq + Hash>(
        &self,
        get_key: impl Fn(&rsonpath_syntax::Selector) -> Option<T>,
    ) -> HashMap<T, ProcedureSegments> {
        let mut map: HashMap<T, Vec<(SegmentIndex, Option<SegmentCondition>)>> = HashMap::new();
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
}