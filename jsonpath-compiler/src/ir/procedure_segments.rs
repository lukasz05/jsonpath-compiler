use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use itertools::Itertools;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ProcedureSegmentsData {
    segments: Vec<usize>,
}

impl ProcedureSegmentsData {
    fn new(segments: Vec<usize>) -> ProcedureSegmentsData {
        ProcedureSegmentsData { segments }
    }

    fn name(&self) -> String {
        format!(
            "Selectors_{}",
            self.segments.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_")
        )
    }

    fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn segments(&self) -> Vec<usize> {
        self.segments.clone()
    }
}

#[derive(Clone)]
pub struct ProcedureSegments<'a> {
    query: &'a rsonpath_syntax::JsonPathQuery,
    segments_data: ProcedureSegmentsData,
}

impl ProcedureSegments<'_> {
    pub fn new(query: &rsonpath_syntax::JsonPathQuery, segments: Vec<usize>) -> ProcedureSegments {
        ProcedureSegments { query, segments_data: ProcedureSegmentsData::new(segments) }
    }

    pub fn name(&self) -> String {
        self.segments_data.name()
    }

    pub fn segments(&self) -> &Vec<usize> {
        &self.segments_data.segments
    }

    pub fn segments_data(&self) -> ProcedureSegmentsData {
        self.segments_data.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.segments_data.is_empty()
    }

    pub fn descendants(&self) -> ProcedureSegments {
        let segments = self.filter_and_map_segments(
            |s| self.query.segments()[s].is_descendant(),
            |s| s,
        );
        ProcedureSegments::new(self.query, segments)
    }

    pub fn successors(&self) -> ProcedureSegments {
        let segment_count = self.query.segments().len();
        let segments = self.filter_and_map_segments(
            |s| s + 1 < segment_count,
            |s| s + 1,
        );
        ProcedureSegments::new(self.query, segments)
    }

    pub fn finals(&self) -> ProcedureSegments {
        let segment_count = self.query.segments().len();
        let segments = self.filter_and_map_segments(
            |s| s == segment_count - 1,
            |s| s,
        );
        ProcedureSegments::new(self.query, segments)
    }

    pub fn wildcards(&self) -> ProcedureSegments {
        let segments = self.filter_and_map_segments(
            |s| self.query.segments()[s].selectors().iter().any(|sel| sel.is_wildcard()),
            |s| s,
        );
        ProcedureSegments::new(self.query, segments)
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
        let descendants = self.descendants();
        let other_descendants = other.descendants();
        let max_descendant_segment = descendants.segments().iter()
            .chain(other_descendants.segments())
            .max();
        let segments_set: HashSet<usize> = HashSet::from_iter(
            self.segments().iter().chain(other.segments()).map(|s| *s)
        );
        let segments: Vec<usize> = segments_set.into_iter().sorted().collect();
        if max_descendant_segment.is_none() {
            ProcedureSegments::new(self.query, segments)
        } else {
            let max_descendant_segment = max_descendant_segment.unwrap();
            let segments = segments.into_iter().filter(
                |s| *s == *max_descendant_segment || !self.query.segments()[*s].is_descendant()
            );
            ProcedureSegments::new(self.query, segments.collect())
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
        ProcedureSegments::new(query, merged_segments.segments().clone())
    }

    fn filter_and_map_segments<T>(
        &self,
        f: impl Fn(usize) -> bool,
        m: impl Fn(usize) -> T,
    ) -> Vec<T> {
        self.segments().iter().filter(|s| f(**s)).map(|s| m(*s)).collect()
    }

    fn selector_to_segments_map<T: Eq + Hash>(
        &self,
        get_key: impl Fn(&rsonpath_syntax::Selector) -> Option<T>,
    ) -> HashMap<T, ProcedureSegments> {
        let mut map: HashMap<T, Vec<usize>> = HashMap::new();
        for segment in self.segments() {
            let selectors = self.query.segments()[*segment].selectors();
            for selector in selectors.iter() {
                if let Some(key) = get_key(selector) {
                    match map.get_mut(&key) {
                        Some(segments) => segments.push(*segment),
                        None => {
                            map.insert(key, vec![*segment]);
                        }
                    }
                }
            }
        }
        map.into_iter()
            .map(|(key, segments)| (key, ProcedureSegments::new(self.query, segments)))
            .collect()
    }
}