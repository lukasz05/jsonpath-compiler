use crate::ir::{LogicalExpression, Query, Segment, Selector};
use std::collections::HashMap;

pub struct SubqueryToCompile<'a> {
    pub function_name: String,
    pub subquery: &'a Query,
}

pub fn find_subqueries(query: &Query) -> HashMap<*const Query, SubqueryToCompile> {
    let mut subqueries = HashMap::new();
    find_subqueries_in_query(&mut subqueries, query);
    subqueries
}

fn find_subqueries_in_query<'a>(
    subqueries: &mut HashMap<*const Query, SubqueryToCompile<'a>>,
    query: &'a Query,
) {
    for segment in &query.segments {
        find_subqueries_in_segment(subqueries, segment);
    }
}

fn find_subqueries_in_segment<'a>(
    subqueries: &mut HashMap<*const Query, SubqueryToCompile<'a>>,
    segment: &'a Segment,
) {
    let selectors = match segment {
        Segment::ChildSegment { selectors } => selectors,
        Segment::DescendantSegment { selectors } => selectors,
    };
    for selector in selectors {
        find_subqueries_in_selector(subqueries, selector);
    }
}

fn find_subqueries_in_selector<'a>(
    subqueries: &mut HashMap<*const Query, SubqueryToCompile<'a>>,
    selector: &'a Selector,
) {
    if let Selector::Filter { logical_expression } = selector {
        find_subqueries_in_logical_expression(subqueries, logical_expression);
    }
}

fn find_subqueries_in_logical_expression<'a>(
    subqueries: &mut HashMap<*const Query, SubqueryToCompile<'a>>,
    logical_expression: &'a LogicalExpression,
) {
    match logical_expression {
        LogicalExpression::And { lhs, rhs } => {
            find_subqueries_in_logical_expression(subqueries, lhs);
            find_subqueries_in_logical_expression(subqueries, rhs);
        }
        LogicalExpression::Or { lhs, rhs } => {
            find_subqueries_in_logical_expression(subqueries, lhs);
            find_subqueries_in_logical_expression(subqueries, rhs);
        }
        LogicalExpression::Not { expr } => {
            find_subqueries_in_logical_expression(subqueries, expr);
        }
        LogicalExpression::ExistenceTest {
            subquery,
            absolute: _absolute,
        } => {
            let subquery_ref = subquery as *const Query;
            let function_name = get_new_subquery_function_name(subqueries);
            subqueries.insert(
                subquery_ref,
                SubqueryToCompile {
                    function_name,
                    subquery,
                },
            );
            find_subqueries_in_query(subqueries, subquery);
        }
        _ => {}
    }
}

fn get_new_subquery_function_name(
    existing_subqueries: &HashMap<*const Query, SubqueryToCompile>,
) -> String {
    return format!("subquery_test_{}", existing_subqueries.len() + 1);
}
