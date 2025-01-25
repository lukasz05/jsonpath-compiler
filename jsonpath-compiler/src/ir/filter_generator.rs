use itertools::chain;
use rsonpath_syntax::{JsonPathQuery, LogicalExpr, SingularJsonPathQuery, SingularSegment};
use rsonpath_syntax::Comparable::{AbsoluteSingularQuery, RelativeSingularQuery};
use rsonpath_syntax::Selector::{Index, Name};
use rsonpath_syntax::TestExpr::{Absolute, Relative};

use crate::ir::{Comparable, ComparisonOp, FilterExpression, FilterId, FilterProcedure, FilterSubqueryPath, LiteralValue, SegmentIndex};
use crate::ir::Comparable::{Literal, Param};
use crate::ir::FilterExpression::{And, BoolParam, Comparison, Not, Or};
use crate::ir::LiteralValue::{Bool, Float, Int, Null};

pub struct FilterUtils {}

impl FilterUtils {
    pub fn get_all_filters(
        query_syntax: &JsonPathQuery
    ) -> Vec<(&LogicalExpr, FilterId)> {
        let mut filters = Vec::new();
        for i in 0..query_syntax.segments().len() {
            filters.append(&mut Self::get_all_filters_in_segment(query_syntax, i));
        }
        filters
    }

    pub fn get_all_filters_in_segment(
        query_syntax: &JsonPathQuery,
        segment_index: SegmentIndex,
    ) -> Vec<(&LogicalExpr, FilterId)> {
        let mut filters = Vec::new();
        let selectors = query_syntax.segments()[segment_index].selectors();
        for (selector_index, selector) in selectors.iter().enumerate() {
            if let rsonpath_syntax::Selector::Filter(filter_expression) = selector {
                filters.push((filter_expression, FilterId { segment_index, selector_index }));
            }
        }
        filters
    }
}

pub struct FilterSubqueryFinder {
    subquery_count: usize,
}


impl FilterSubqueryFinder {
    pub fn new() -> FilterSubqueryFinder {
        FilterSubqueryFinder { subquery_count: 0 }
    }

    pub fn get_all_subqueries_paths(
        &mut self,
        query_syntax: &JsonPathQuery,
    ) -> Vec<FilterSubqueryPath> {
        let mut result = Vec::new();
        for (filter_expr, filter_id) in FilterUtils::get_all_filters(query_syntax) {
            result.append(&mut self.get_subqueries_paths_in_filter(filter_expr, &filter_id));
        }
        result
    }

    pub fn get_subqueries_paths_in_filter(
        &mut self,
        filter_expr: &LogicalExpr,
        filter_id: &FilterId,
    ) -> Vec<FilterSubqueryPath> {
        self.subquery_count = 0;
        self.get_all_subqueries_paths_in_expr(filter_expr, filter_id)
    }

    fn get_all_subqueries_paths_in_expr(
        &mut self,
        filter_expr: &LogicalExpr,
        filter_id: &FilterId,
    ) -> Vec<FilterSubqueryPath> {
        match filter_expr {
            LogicalExpr::Or(lhs, rhs) => {
                chain![
                    self.get_all_subqueries_paths_in_expr(lhs, filter_id),
                    self.get_all_subqueries_paths_in_expr(rhs, filter_id)
                ].collect()
            }
            LogicalExpr::And(lhs, rhs) => {
                chain![
                    self.get_all_subqueries_paths_in_expr(lhs, filter_id),
                    self.get_all_subqueries_paths_in_expr(rhs, filter_id)
                ].collect()
            }
            LogicalExpr::Not(expr) => {
                self.get_all_subqueries_paths_in_expr(expr, filter_id)
            }
            LogicalExpr::Comparison(comparison_expr) => {
                vec![
                    self.get_subquery_from_comparable(comparison_expr.lhs(), filter_id),
                    self.get_subquery_from_comparable(comparison_expr.rhs(), filter_id),
                ].into_iter()
                    .filter(|subquery| subquery.is_some())
                    .map(|subquery| subquery.unwrap())
                    .collect()
            }
            LogicalExpr::Test(test_expr) => {
                self.subquery_count += 1;
                let (relative, path) = match test_expr {
                    Relative(subquery) => {
                        (true, Self::get_subquery_path(subquery))
                    }
                    Absolute(subquery) => {
                        (false, Self::get_subquery_path(subquery))
                    }
                };
                vec![
                    FilterSubqueryPath::new(
                        filter_id.clone(),
                        self.subquery_count,
                        path,
                        relative,
                    )
                ]
            }
        }
    }

    fn get_subquery_from_comparable(
        &mut self,
        comparison_expr: &rsonpath_syntax::Comparable,
        filter_id: &FilterId,
    ) -> Option<FilterSubqueryPath> {
        let (relative, path) = match comparison_expr {
            RelativeSingularQuery(subquery) => {
                (true, Self::get_singular_subquery_path(subquery))
            }
            AbsoluteSingularQuery(subquery) => {
                (false, Self::get_singular_subquery_path(subquery))
            }
            _ => return None
        };
        self.subquery_count += 1;
        Some(FilterSubqueryPath::new(filter_id.clone(), self.subquery_count, path, relative))
    }

    fn get_subquery_path(subquery: &JsonPathQuery) -> String {
        let mut path = String::new();
        for segment in subquery.segments() {
            if segment.is_descendant() {
                panic!()
            }
            for selector in segment.selectors().iter() {
                match selector {
                    Name(name) => {
                        if !path.is_empty() {
                            path.push('.');
                        }
                        path.push_str(name.unquoted())
                    }
                    Index(index) => {
                        path.push_str(&format!("[{}]", index))
                    }
                    _ => panic!()
                }
            }
        }
        path
    }

    fn get_singular_subquery_path(subquery: &SingularJsonPathQuery) -> String {
        let mut path = String::new();
        for segment in subquery.segments() {
            match segment {
                SingularSegment::Name(name) => {
                    if !path.is_empty() {
                        path.push('.');
                    }
                    path.push_str(name.unquoted())
                }
                SingularSegment::Index(index) => {
                    path.push_str(&format!("[{}]", index))
                }
            }
        }
        path
    }
}

pub struct FilterGenerator {
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
        query_syntax: &JsonPathQuery,
    ) -> Vec<FilterProcedure> {
        let mut filter_procedures = Vec::new();
        for (filter_expression, id) in FilterUtils::get_all_filters(query_syntax) {
            filter_procedures.push(self.generate_filter_procedure(filter_expression, id));
        }
        filter_procedures
    }

    fn generate_filter_procedure(
        &mut self,
        filter_expression: &LogicalExpr,
        id: FilterId,
    ) -> FilterProcedure {
        self.subquery_count = 0;
        let expression = self.generate_filter_expr(filter_expression);
        FilterProcedure {
            name: format!("Filter_{}_{}", id.segment_index, id.selector_index),
            arity: self.subquery_count,
            expression,
        }
    }

    fn generate_filter_expr(&mut self, filter_expr: &LogicalExpr) -> FilterExpression {
        match filter_expr {
            LogicalExpr::Or(lhs, rhs) => {
                Or {
                    lhs: Box::new(self.generate_filter_expr(lhs)),
                    rhs: Box::new(self.generate_filter_expr(rhs)),
                }
            }
            LogicalExpr::And(lhs, rhs) => {
                And {
                    lhs: Box::new(self.generate_filter_expr(lhs)),
                    rhs: Box::new(self.generate_filter_expr(rhs)),
                }
            }
            LogicalExpr::Not(expr) => {
                Not { expr: Box::new(self.generate_filter_expr(expr)) }
            }
            LogicalExpr::Comparison(comparison_expr) => {
                Comparison {
                    lhs: self.generate_comparable(comparison_expr.lhs()),
                    rhs: self.generate_comparable(comparison_expr.rhs()),
                    op: self.generate_comparison_op(comparison_expr.op()),
                }
            }
            LogicalExpr::Test { .. } => {
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
            AbsoluteSingularQuery { .. } | RelativeSingularQuery { .. } => {
                self.subquery_count += 1;
                Param { id: self.subquery_count }
            }
        }
    }

    fn generate_literal(&self, literal_syntax: &rsonpath_syntax::Literal) -> LiteralValue {
        match literal_syntax {
            rsonpath_syntax::Literal::String(json_str) => {
                LiteralValue::String(json_str.unquoted().to_string())
            }
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