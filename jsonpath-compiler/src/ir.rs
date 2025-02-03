use std::collections::HashMap;
use std::fmt::Debug;

pub mod generator;
mod procedure_segments;
mod filter_generator;

#[derive(Debug)]
pub struct Query {
    pub procedures: Vec<Procedure>,
    pub filter_procedures: HashMap<FilterId, FilterProcedure>,
    pub filter_subqueries: HashMap<FilterId, Vec<FilterSubquery>>
}

#[derive(Debug)]
pub struct Procedure {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    ForEachElement { instructions: Vec<Instruction> },
    ForEachMember { instructions: Vec<Instruction> },
    IfCurrentIndexEquals { index: u64, instructions: Vec<Instruction> },
    IfCurrentIndexFromEndEquals { index: u64, instructions: Vec<Instruction> },
    IfCurrentMemberNameEquals { name: String, instructions: Vec<Instruction> },
    ExecuteProcedureOnChild {
        conditions: Vec<Option<SelectionCondition>>,
        name: String,
    },
    SaveCurrentNodeDuringTraversal {
        condition: Option<SelectionCondition>,
        instruction: Box<Instruction>,
    },
    Continue,
    TraverseCurrentNodeSubtree,
    //RegisterSubqueryPath { subquery_path: FilterSubqueryPath },
    StartFilterExecution { filter_id: FilterId },
    EndFilterExecution,
    UpdateSubqueriesState,
}

impl Instruction {
    pub fn is_object_member_iteration(&self) -> bool {
        match self {
            Instruction::ForEachMember { .. } |
            Instruction::SaveCurrentNodeDuringTraversal { .. } => true,
            _ => false
        }
    }

    pub fn is_array_element_iteration(&self) -> bool {
        match self {
            Instruction::ForEachElement { .. } |
            Instruction::SaveCurrentNodeDuringTraversal { .. } => true,
            _ => false
        }
    }
}

#[derive(Debug)]
pub struct Name(pub String);

#[derive(Debug)]
pub struct Index(pub i64);

#[derive(Debug)]
pub struct FilterProcedure {
    pub name: String,
    pub arity: usize,
    pub expression: FilterExpression,
}


#[derive(Debug)]
pub enum FilterExpression {
    Or { lhs: Box<FilterExpression>, rhs: Box<FilterExpression> },
    And { lhs: Box<FilterExpression>, rhs: Box<FilterExpression> },
    Not { expr: Box<FilterExpression> },
    Comparison { lhs: Comparable, rhs: Comparable, op: ComparisonOp },
    BoolParam { id: usize },
}

#[derive(Debug)]
pub enum Comparable {
    Param { id: usize },
    Literal { value: LiteralValue },
}

#[derive(Debug)]
pub enum LiteralValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

#[derive(Debug)]
pub enum ComparisonOp {
    EqualTo,
    NotEqualTo,
    LesserOrEqualTo,
    GreaterOrEqualTo,
    LessThan,
    GreaterThan,
}

#[derive(Eq, PartialEq, Hash, Clone, Ord, PartialOrd, Debug)]
pub enum SelectionCondition {
    Filter { id: FilterId },
    RuntimeSegmentCondition { segment_index: SegmentIndex },
    Or { lhs: Box<SelectionCondition>, rhs: Box<SelectionCondition> },
    And { lhs: Box<SelectionCondition>, rhs: Box<SelectionCondition> },
}

impl SelectionCondition {
    fn merge_with(&self, other: &SelectionCondition, normalize: bool) -> SelectionCondition {
        let merged = SelectionCondition::Or {
            lhs: Box::new(self.clone()),
            rhs: Box::new(other.clone()),
        };
        if normalize { merged.normalize() } else { merged }
    }

    fn and(&self, other: &SelectionCondition) -> SelectionCondition {
        SelectionCondition::And { lhs: Box::new(self.clone()), rhs: Box::new(other.clone()) }
            .normalize()
    }

    fn normalize(&self) -> SelectionCondition {
        match self {
            SelectionCondition::Or { lhs, rhs } => {
                let (lhs, rhs) = Self::normalize_and_sort_subconditions(
                    *lhs.clone(),
                    *rhs.clone(),
                );
                if let Some(rhs) = rhs {
                    SelectionCondition::Or {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }
                } else {
                    lhs
                }
            }
            SelectionCondition::And { lhs, rhs } => {
                let (lhs, rhs) = Self::normalize_and_sort_subconditions(
                    *lhs.clone(),
                    *rhs.clone(),
                );
                if let Some(rhs) = rhs {
                    SelectionCondition::And {
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

    fn merge(conditions: Vec<Option<SelectionCondition>>) -> Option<SelectionCondition> {
        if conditions.is_empty() ||
            conditions.iter().any(|condition| condition.is_none()) {
            None
        } else {
            let conditions: Vec<SelectionCondition> = conditions.into_iter()
                .map(|condition| condition.unwrap())
                .collect();
            let merged_conditions = conditions.iter().skip(1).fold(
                conditions[0].clone(),
                |merged_conditions, condition| condition.merge_with(&merged_conditions, false),
            );
            Some(merged_conditions.normalize())
        }
    }

    fn normalize_and_sort_subconditions(
        lhs: SelectionCondition,
        rhs: SelectionCondition,
    ) -> (SelectionCondition, Option<SelectionCondition>) {
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

#[derive(Eq, PartialEq, Hash, Clone, Ord, PartialOrd, Debug)]
pub struct FilterId {
    pub segment_index: SegmentIndex,
    pub selector_index: SelectorIndex,
}

impl FilterId {
    pub fn new(segment_index: SegmentIndex, selector_index: SelectorIndex) -> FilterId {
        FilterId { segment_index, selector_index }
    }
}

#[derive(Debug)]
pub struct FilterSubquery {
    pub is_absolute: bool,
    pub segments: Vec<FilterSubquerySegment>,
}

#[derive(Debug)]
pub enum FilterSubquerySegment {
    Name(String),
    Index(i64),
}

pub type SegmentIndex = usize;
pub type SelectorIndex = usize;
