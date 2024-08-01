use crate::ir::{Comparable, ComparisonOp, Index, Literal, LogicalExpression, Name, Query, Segment, Selector, SingularSelector, Slice};
use crate::ir::Comparable::SingularQuery;
use crate::ir::Literal::{Bool, Float, Int, Null, String};
use crate::ir::LogicalExpression::{And, Comparison, ExistenceTest, Not, Or};
use crate::ir::Segment::{ChildSegment, DescendantSegment};
use crate::ir::Selector::{AllChildren, ChildByName, ElementAtIndex, Filter};

pub fn generate(query_syntax: &rsonpath_syntax::JsonPathQuery) -> Query {
    Query {
        segments: query_syntax.segments().into_iter()
            .map(|segment_syntax| generate_segment(segment_syntax))
            .collect()
    }
}

fn generate_segment(segment_syntax: &rsonpath_syntax::Segment) -> Segment {
    match segment_syntax {
        rsonpath_syntax::Segment::Child(selectors) => {
            ChildSegment { selectors: generate_selectors(selectors) }
        }
        rsonpath_syntax::Segment::Descendant(selectors) => {
            DescendantSegment { selectors: generate_selectors(selectors) }
        }
    }
}

fn generate_selectors(selectors_syntax: &rsonpath_syntax::Selectors) -> Vec<Selector> {
    let mut selectors: Vec<Selector> = Vec::new();
    for selector in selectors_syntax.iter() {
        match selector {
            rsonpath_syntax::Selector::Name(name_syntax) => {
                let name = Name(name_syntax.unquoted().to_owned());
                selectors.push(ChildByName { name })
            }
            rsonpath_syntax::Selector::Wildcard => selectors.push(AllChildren),
            rsonpath_syntax::Selector::Index(index_syntax) => {
                let index = generate_index(index_syntax);
                selectors.push(ElementAtIndex { index })
            }
            rsonpath_syntax::Selector::Slice(slice_syntax) => {
                let slice = generate_slice(slice_syntax);
                selectors.push(Selector::Slice { slice })
            }
            rsonpath_syntax::Selector::Filter(logical_expr_syntax) => {
                let logical_expression = generate_logical_expr(logical_expr_syntax);
                selectors.push(Filter { logical_expression })
            }
        }
    }
    return selectors;
}

fn generate_logical_expr(logical_expr_syntax: &rsonpath_syntax::LogicalExpr) -> LogicalExpression {
    match logical_expr_syntax {
        rsonpath_syntax::LogicalExpr::Or(lhs, rhs) => {
            Or {
                lhs: Box::new(generate_logical_expr(lhs)),
                rhs: Box::new(generate_logical_expr(rhs)),
            }
        }
        rsonpath_syntax::LogicalExpr::And(lhs, rhs) => {
            And {
                lhs: Box::new(generate_logical_expr(lhs)),
                rhs: Box::new(generate_logical_expr(rhs)),
            }
        }
        rsonpath_syntax::LogicalExpr::Not(expr) => {
            Not { expr: Box::new(generate_logical_expr(expr)) }
        }
        rsonpath_syntax::LogicalExpr::Comparison(comparison_expr) => {
            generate_comparison(comparison_expr)
        }
        rsonpath_syntax::LogicalExpr::Test(test_expr) => {
            generate_existence_test(test_expr)
        }
    }
}

fn generate_comparison(comparison_expr_syntax: &rsonpath_syntax::ComparisonExpr) -> LogicalExpression {
    Comparison {
        lhs: generate_comparable(comparison_expr_syntax.lhs()),
        rhs: generate_comparable(comparison_expr_syntax.rhs()),
        op: generate_comparison_op(&comparison_expr_syntax.op()),
    }
}

fn generate_comparable(comparable_syntax: &rsonpath_syntax::Comparable) -> Comparable {
    match comparable_syntax {
        rsonpath_syntax::Comparable::Literal(literal_syntax) => {
            Comparable::Literal { literal: generate_literal(literal_syntax) }
        }
        rsonpath_syntax::Comparable::AbsoluteSingularQuery(singular_query_syntax) => {
            SingularQuery {
                absolute: true,
                selectors: generate_singular_query(singular_query_syntax),
            }
        }
        rsonpath_syntax::Comparable::RelativeSingularQuery(singular_query_syntax) => {
            SingularQuery {
                absolute: false,
                selectors: generate_singular_query(singular_query_syntax),
            }
        }
    }
}

fn generate_comparison_op(comparison_op_syntax: &rsonpath_syntax::ComparisonOp) -> ComparisonOp {
    match comparison_op_syntax {
        rsonpath_syntax::ComparisonOp::EqualTo => ComparisonOp::EqualTo,
        rsonpath_syntax::ComparisonOp::NotEqualTo => ComparisonOp::NotEqualTo,
        rsonpath_syntax::ComparisonOp::LesserOrEqualTo => ComparisonOp::LesserOrEqualTo,
        rsonpath_syntax::ComparisonOp::GreaterOrEqualTo => ComparisonOp::GreaterOrEqualTo,
        rsonpath_syntax::ComparisonOp::LessThan => ComparisonOp::LessThan,
        rsonpath_syntax::ComparisonOp::GreaterThan => ComparisonOp::GreaterThan
    }
}

fn generate_existence_test(test_expr: &rsonpath_syntax::TestExpr) -> LogicalExpression {
    match test_expr {
        rsonpath_syntax::TestExpr::Absolute(query_syntax) => {
            ExistenceTest {
                absolute: true,
                subquery: generate(query_syntax),
            }
        }
        rsonpath_syntax::TestExpr::Relative(query_syntax) => {
            ExistenceTest {
                absolute: false,
                subquery: generate(query_syntax),
            }
        }
    }
}

fn generate_singular_query(singular_query_syntax: &rsonpath_syntax::SingularJsonPathQuery) -> Vec<SingularSelector> {
    singular_query_syntax.segments().map(|singular_segment_syntax| {
        match singular_segment_syntax {
            rsonpath_syntax::SingularSegment::Name(name_syntax) => {
                let name = Name(name_syntax.unquoted().to_owned());
                SingularSelector::ChildByName { name }
            }
            rsonpath_syntax::SingularSegment::Index(index_syntax) => {
                let index = generate_index(index_syntax);
                SingularSelector::ElementAtIndex { index }
            }
        }
    }).collect()
}

fn generate_index(index_syntax: &rsonpath_syntax::Index) -> Index {
    Index(index_syntax_as_i64(index_syntax))
}

fn generate_slice(slice_syntax: &rsonpath_syntax::Slice) -> Slice {
    Slice {
        start: index_syntax_as_i64(&slice_syntax.start()),
        end: slice_syntax.end().map(|index_syntax| index_syntax_as_i64(&index_syntax)),
        step: step_syntax_as_i64(&slice_syntax.step())
    }
}

fn generate_literal(literal_syntax: &rsonpath_syntax::Literal) -> Literal {
    match literal_syntax {
        rsonpath_syntax::Literal::String(str) => String(str.unquoted().to_owned()),
        rsonpath_syntax::Literal::Number(num) => match num {
            rsonpath_syntax::num::JsonNumber::Int(x) => Int(x.as_i64()),
            rsonpath_syntax::num::JsonNumber::Float(x) => Float(x.as_f64()),
        },
        rsonpath_syntax::Literal::Bool(bool) => Bool(bool.to_owned()),
        rsonpath_syntax::Literal::Null => Null
    }
}

fn index_syntax_as_i64(index_syntax: &rsonpath_syntax::Index) -> i64 {
    match index_syntax {
        rsonpath_syntax::Index::FromStart(num) => num.as_u64() as i64,
        rsonpath_syntax::Index::FromEnd(num) => -(num.as_u64() as i64)
    }
}

fn step_syntax_as_i64(step_syntax: &rsonpath_syntax::Step) -> i64 {
    match step_syntax {
        rsonpath_syntax::Step::Forward(num) => num.as_u64() as i64,
        rsonpath_syntax::Step::Backward(num) => -(num.as_u64() as i64)
    }
}
