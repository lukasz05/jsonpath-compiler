use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use itertools::Itertools;

use crate::ir::{FilterId, SegmentIndex};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ProcedureSegmentsData {
    segments: Vec<SegmentIndex>,
}

impl ProcedureSegmentsData {
    fn new(segments: Vec<SegmentIndex>) -> ProcedureSegmentsData {
        ProcedureSegmentsData {
            segments: segments.into_iter().sorted().collect(),
        }
    }

    fn name(&self) -> String {
        let segments_str = self
            .segments
            .iter()
            .sorted()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join("_");
        format!("Selectors_{segments_str}")
    }

    fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn segments(&self) -> Vec<SegmentIndex> {
        self.segments.clone()
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
        segments: Vec<SegmentIndex>,
    ) -> ProcedureSegments {
        ProcedureSegments {
            query,
            segments_data: ProcedureSegmentsData::new(segments),
        }
    }

    pub fn name(&self) -> String {
        self.segments_data.name()
    }

    pub fn segments(&self) -> Vec<SegmentIndex> {
        self.segments_data.segments()
    }

    pub fn segments_data(&self) -> ProcedureSegmentsData {
        self.segments_data.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.segments_data.is_empty()
    }

    pub fn descendants(&self) -> ProcedureSegments {
        let segments =
            self.filter_and_map_segments(|s| self.query.segments()[s].is_descendant(), |s| s);
        ProcedureSegments::new(self.query, segments)
    }

    pub fn successors(&self) -> ProcedureSegments {
        let segments =
            self.filter_and_map_segments(|s| s + 1 < self.query.segments().len(), |s| s + 1);
        ProcedureSegments::new(self.query, segments)
    }

    pub fn successor(&self, segment_index: SegmentIndex) -> Option<SegmentIndex> {
        if segment_index + 1 < self.query.segments().len() {
            Some(segment_index + 1)
        } else {
            None
        }
    }

    pub fn finals(&self) -> ProcedureSegments {
        let segment_count = self.query.segments().len();
        let segments = self.filter_and_map_segments(|s| s == segment_count - 1, |s| s);
        ProcedureSegments::new(self.query, segments)
    }

    pub fn wildcards(&self) -> ProcedureSegments {
        let segments = self.filter_and_map_segments(
            |s| {
                self.query.segments()[s]
                    .selectors()
                    .iter()
                    .any(|sel| sel.is_wildcard())
            },
            |s| s,
        );
        ProcedureSegments::new(self.query, segments)
    }

    pub fn name_selectors(&self) -> HashMap<String, ProcedureSegments> {
        self.selector_to_segments_map(|selector| {
            if let rsonpath_syntax::Selector::Name(name) = selector {
                Some(name.unquoted().to_string())
            } else {
                None
            }
        })
    }

    pub fn index_selectors(&self) -> HashMap<i64, ProcedureSegments> {
        self.selector_to_segments_map(|selector| {
            if let rsonpath_syntax::Selector::Index(index) = selector {
                Some(match index {
                    rsonpath_syntax::Index::FromStart(num) => num.as_u64() as i64,
                    rsonpath_syntax::Index::FromEnd(num) => -(num.as_u64() as i64),
                })
            } else {
                None
            }
        })
    }

    pub fn non_negative_index_selectors(&self) -> HashMap<i64, ProcedureSegments> {
        self.index_selectors()
            .into_iter()
            .filter(|(key, _)| *key >= 0)
            .collect()
    }

    pub fn negative_index_selectors(&self) -> HashMap<i64, ProcedureSegments> {
        self.index_selectors()
            .into_iter()
            .filter(|(key, _)| *key < 0)
            .collect()
    }

    pub fn filters(&self) -> ProcedureSegments {
        let segments = self.filter_and_map_segments(
            |s| {
                self.query.segments()[s]
                    .selectors()
                    .iter()
                    .any(|sel| sel.is_filter())
            },
            |s| s,
        );
        ProcedureSegments::new(self.query, segments)
    }

    pub fn merge_with(&self, other: &ProcedureSegments) -> ProcedureSegments {
        let mut segments: HashSet<SegmentIndex> = self.segments().iter().map(|i| *i).collect();
        for segment_id in other.segments() {
            if !segments.contains(&segment_id) {
                segments.insert(segment_id);
            }
        }
        let max_descendant_segment = segments
            .iter()
            .filter(|segment_id| self.query.segments()[**segment_id].is_descendant())
            .max();
        if max_descendant_segment.is_none() {
            ProcedureSegments::new(self.query, segments.iter().map(|i| *i).collect())
        } else {
            // TODO: is this allowed with filters
            let max_descendant_segment = *max_descendant_segment.unwrap();
            let segments = segments.iter().filter(|i| {
                **i == max_descendant_segment || !self.query.segments()[**i].is_descendant()
            });
            ProcedureSegments::new(self.query, segments.map(|i| *i).collect())
        }
    }

    pub fn merge<'a>(
        query: &'a rsonpath_syntax::JsonPathQuery,
        segments: Vec<ProcedureSegments<'a>>,
    ) -> ProcedureSegments<'a> {
        let merged_segments = segments.iter().fold(
            ProcedureSegments::new(query, vec![]),
            |segments, new_segments| new_segments.merge_with(&segments),
        );
        ProcedureSegments::new(query, merged_segments.segments().clone())
    }

    fn filter_and_map_segments(
        &self,
        f: impl Fn(SegmentIndex) -> bool,
        m: impl Fn(SegmentIndex) -> SegmentIndex,
    ) -> Vec<SegmentIndex> {
        self.segments()
            .into_iter()
            .filter(|segment_index| f(*segment_index))
            .map(|segment_index| m(segment_index))
            .collect()
    }

    fn selector_to_segments_map<T: Eq + Hash>(
        &self,
        get_key: impl Fn(&rsonpath_syntax::Selector) -> Option<T>,
    ) -> HashMap<T, ProcedureSegments> {
        let mut map: HashMap<T, Vec<SegmentIndex>> = HashMap::new();
        for segment_index in self.segments() {
            let selectors = self.query.segments()[segment_index].selectors();
            for selector in selectors.iter() {
                if let Some(key) = get_key(selector) {
                    match map.get_mut(&key) {
                        Some(segments) => segments.push(segment_index),
                        None => {
                            map.insert(key, vec![segment_index]);
                        }
                    }
                }
            }
        }
        map.into_iter()
            .map(|(key, segments)| (key, ProcedureSegments::new(self.query, segments)))
            .collect()
    }

    fn filter_selectors(&self) -> Vec<FilterId> {
        self.segments()
            .into_iter()
            .flat_map(|segment_id| {
                self.query.segments()[segment_id]
                    .selectors()
                    .into_iter()
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
