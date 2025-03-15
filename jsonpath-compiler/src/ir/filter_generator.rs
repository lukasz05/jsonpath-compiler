use std::collections::HashMap;

use itertools::chain;
use rsonpath_syntax::Comparable::{AbsoluteSingularQuery, RelativeSingularQuery};
use rsonpath_syntax::TestExpr::{Absolute, Relative};

use crate::ir::{
    Comparable, ComparisonOp, FilterExpression, FilterId, FilterProcedure, FilterSubquery,
    LiteralValue, SegmentIndex,
};
use crate::ir::Comparable::{Literal, Param};
use crate::ir::FilterExpression::{And, Comparison, ExistenceTest, Not, Or};
use crate::ir::FilterSubquerySegment::{Index, Name};
use crate::ir::LiteralValue::{Bool, Float, Int, Null};

pub struct FilterUtils {}

impl FilterUtils {
    pub fn get_all_filters(
        query_syntax: &rsonpath_syntax::JsonPathQuery,
    ) -> Vec<(&rsonpath_syntax::LogicalExpr, FilterId)> {
        let mut filters = Vec::new();
        for i in 0..query_syntax.segments().len() {
            filters.append(&mut Self::get_filters_in_segment(query_syntax, i));
        }
        filters
    }

    pub fn get_filters_in_segment(
        query_syntax: &rsonpath_syntax::JsonPathQuery,
        segment_index: SegmentIndex,
    ) -> Vec<(&rsonpath_syntax::LogicalExpr, FilterId)> {
        let mut filters = Vec::new();
        let selectors = query_syntax.segments()[segment_index].selectors();
        for (selector_index, selector) in selectors.iter().enumerate() {
            if let rsonpath_syntax::Selector::Filter(filter_expression) = selector {
                filters.push((
                    filter_expression,
                    FilterId {
                        segment_index,
                        selector_index,
                    },
                ));
            }
        }
        filters
    }
}

pub struct FilterSubqueryFinder {}

impl FilterSubqueryFinder {
    pub fn get_all_subqueries(
        query_syntax: &rsonpath_syntax::JsonPathQuery,
    ) -> HashMap<FilterId, Vec<FilterSubquery>> {
        let mut result = HashMap::new();
        for (filter_expr, filter_id) in FilterUtils::get_all_filters(query_syntax) {
            result.insert(filter_id, Self::get_subqueries_in_filter(filter_expr));
        }
        result
    }

    pub fn get_subqueries_in_filter(
        filter_expr: &rsonpath_syntax::LogicalExpr,
    ) -> Vec<FilterSubquery> {
        Self::get_all_subqueries_paths_in_expr(filter_expr)
    }

    fn get_all_subqueries_paths_in_expr(
        filter_expr: &rsonpath_syntax::LogicalExpr,
        //filter_id: &FilterId,
    ) -> Vec<FilterSubquery> {
        match filter_expr {
            rsonpath_syntax::LogicalExpr::Or(lhs, rhs) => chain![
                Self::get_all_subqueries_paths_in_expr(lhs),
                Self::get_all_subqueries_paths_in_expr(rhs)
            ]
                .collect(),
            rsonpath_syntax::LogicalExpr::And(lhs, rhs) => chain![
                Self::get_all_subqueries_paths_in_expr(lhs),
                Self::get_all_subqueries_paths_in_expr(rhs)
            ]
                .collect(),
            rsonpath_syntax::LogicalExpr::Not(expr) => Self::get_all_subqueries_paths_in_expr(expr),
            rsonpath_syntax::LogicalExpr::Comparison(comparison_expr) => vec![
                Self::get_subquery_from_comparable(comparison_expr.lhs()),
                Self::get_subquery_from_comparable(comparison_expr.rhs()),
            ]
                .into_iter()
                .filter(|subquery| subquery.is_some())
                .map(|subquery| subquery.unwrap())
                .collect(),
            rsonpath_syntax::LogicalExpr::Test(test_expr) => {
                let subquery = match test_expr {
                    Relative(subquery) => Self::convert_subquery(subquery, false, true),
                    Absolute(subquery) => Self::convert_subquery(subquery, true, true),
                };
                vec![subquery]
            }
        }
    }

    fn get_subquery_from_comparable(
        comparison_expr: &rsonpath_syntax::Comparable,
    ) -> Option<FilterSubquery> {
        let subquery = match comparison_expr {
            RelativeSingularQuery(subquery) => {
                Some(Self::convert_singular_subquery(subquery, false, false))
            }
            AbsoluteSingularQuery(subquery) => {
                Some(Self::convert_singular_subquery(subquery, true, false))
            }
            _ => return None,
        };
        subquery
    }

    fn convert_subquery(
        subquery: &rsonpath_syntax::JsonPathQuery,
        is_absolute: bool,
        is_existence_test: bool,
    ) -> FilterSubquery {
        let mut result = FilterSubquery {
            is_absolute,
            is_existence_test,
            segments: Vec::new(),
        };
        for segment in subquery.segments() {
            if segment.is_descendant() {
                panic!()
            }
            for selector in segment.selectors().iter() {
                match selector {
                    rsonpath_syntax::Selector::Name(name) => {
                        result.segments.push(Name(name.unquoted().to_string()));
                    }
                    rsonpath_syntax::Selector::Index(index) => {
                        result.segments.push(Index(match index {
                            rsonpath_syntax::Index::FromStart(index) => index.as_u64() as i64,
                            rsonpath_syntax::Index::FromEnd(index) => -(index.as_u64() as i64),
                        }));
                    }
                    _ => panic!(),
                }
            }
        }
        result
    }

    fn convert_singular_subquery(
        subquery: &rsonpath_syntax::SingularJsonPathQuery,
        is_absolute: bool,
        is_existence_test: bool
    ) -> FilterSubquery {
        let mut result = FilterSubquery {
            is_absolute,
            is_existence_test,
            segments: Vec::new(),
        };
        for segment in subquery.segments() {
            match segment {
                rsonpath_syntax::SingularSegment::Name(name) => {
                    result.segments.push(Name(name.unquoted().to_string()));
                }
                rsonpath_syntax::SingularSegment::Index(index) => {
                    result.segments.push(Index(match index {
                        rsonpath_syntax::Index::FromStart(index) => index.as_u64() as i64,
                        rsonpath_syntax::Index::FromEnd(index) => -(index.as_u64() as i64),
                    }));
                }
            }
        }
        result
    }
}

pub struct FilterGenerator {
    subquery_count: usize,
}

impl FilterGenerator {
    pub fn new() -> FilterGenerator {
        FilterGenerator { subquery_count: 0 }
    }

    pub fn generate_filter_procedures(
        &mut self,
        query_syntax: &rsonpath_syntax::JsonPathQuery,
    ) -> HashMap<FilterId, FilterProcedure> {
        let mut filter_procedures = HashMap::new();
        for (filter_expression, filter_id) in FilterUtils::get_all_filters(query_syntax) {
            filter_procedures.insert(
                filter_id.clone(),
                self.generate_filter_procedure(filter_expression, filter_id),
            );
        }
        filter_procedures
    }

    fn generate_filter_procedure(
        &mut self,
        filter_expression: &rsonpath_syntax::LogicalExpr,
        id: FilterId,
    ) -> FilterProcedure {
        self.subquery_count = 0;
        let expression = self.generate_filter_expr(filter_expression);
        FilterProcedure {
            name: format!("Filter_{}_{}", id.segment_index, id.selector_index),
            filter_id: id,
            arity: self.subquery_count,
            expression,
        }
    }

    fn generate_filter_expr(
        &mut self,
        filter_expr: &rsonpath_syntax::LogicalExpr,
    ) -> FilterExpression {
        match filter_expr {
            rsonpath_syntax::LogicalExpr::Or(lhs, rhs) => Or {
                lhs: Box::new(self.generate_filter_expr(lhs)),
                rhs: Box::new(self.generate_filter_expr(rhs)),
            },
            rsonpath_syntax::LogicalExpr::And(lhs, rhs) => And {
                lhs: Box::new(self.generate_filter_expr(lhs)),
                rhs: Box::new(self.generate_filter_expr(rhs)),
            },
            rsonpath_syntax::LogicalExpr::Not(expr) => Not {
                expr: Box::new(self.generate_filter_expr(expr)),
            },
            rsonpath_syntax::LogicalExpr::Comparison(comparison_expr) => Comparison {
                lhs: self.generate_comparable(comparison_expr.lhs()),
                rhs: self.generate_comparable(comparison_expr.rhs()),
                op: self.generate_comparison_op(comparison_expr.op()),
            },
            rsonpath_syntax::LogicalExpr::Test { .. } => {
                let param = ExistenceTest {
                    param_id: self.subquery_count,
                };
                self.subquery_count += 1;
                param
            }
        }
    }

    fn generate_comparable(&mut self, comparable: &rsonpath_syntax::Comparable) -> Comparable {
        match comparable {
            rsonpath_syntax::Comparable::Literal(literal) => Literal {
                value: self.generate_literal(literal),
            },
            AbsoluteSingularQuery { .. } | RelativeSingularQuery { .. } => {
                let param = Param {
                    id: self.subquery_count,
                };
                self.subquery_count += 1;
                param
            }
        }
    }

    fn generate_literal(&self, literal_syntax: &rsonpath_syntax::Literal) -> LiteralValue {
        match literal_syntax {
            rsonpath_syntax::Literal::String(json_str) => {
                LiteralValue::String(json_str.unquoted().to_string())
            }
            rsonpath_syntax::Literal::Number(json_num) => match json_num {
                rsonpath_syntax::num::JsonNumber::Int(json_int) => Int(json_int.as_i64()),
                rsonpath_syntax::num::JsonNumber::Float(json_float) => Float(json_float.as_f64()),
            },
            rsonpath_syntax::Literal::Bool(bool_value) => Bool(*bool_value),
            rsonpath_syntax::Literal::Null => Null,
        }
    }

    fn generate_comparison_op(
        &self,
        comparison_op_syntax: rsonpath_syntax::ComparisonOp,
    ) -> ComparisonOp {
        match comparison_op_syntax {
            rsonpath_syntax::ComparisonOp::EqualTo => ComparisonOp::EqualTo,
            rsonpath_syntax::ComparisonOp::NotEqualTo => ComparisonOp::NotEqualTo,
            rsonpath_syntax::ComparisonOp::LesserOrEqualTo => ComparisonOp::LessOrEqualTo,
            rsonpath_syntax::ComparisonOp::GreaterOrEqualTo => ComparisonOp::GreaterOrEqualTo,
            rsonpath_syntax::ComparisonOp::LessThan => ComparisonOp::LessThan,
            rsonpath_syntax::ComparisonOp::GreaterThan => ComparisonOp::GreaterThan,
        }
    }
}
