use std::iter;

use crate::ir::{ComparisonOp, Index, Instruction, Literal, Name, Query, Segment, Slice};
use crate::ir::Instruction::{And, Compare, ExistenceTest, FilterIteration, Not, Or,
                             PopAndPushAllChildren, PopAndPushChildByName, PopAndPushElementAtIndex,
                             PushCurrentFilterNode, PushLiteral, PushRootNode, SelectAllChildren,
                             SelectChildByName, SelectElementAtIndex, SelectNodeConditionally,
                             SelectSlice, WhileStackNotEmpty};
use crate::ir::Literal::{Bool, Float, Int, Null, String};

pub fn generate(query_syntax: &rsonpath_syntax::JsonPathQuery) -> Query {
    Query {
        segments: query_syntax.segments().into_iter()
            .map(|segment_syntax| generate_segment(segment_syntax))
            .collect()
    }
}

fn generate_segment(segment_syntax: &rsonpath_syntax::Segment) -> Segment {
    match segment_syntax {
        rsonpath_syntax::Segment::Child(selectors) => generate_child_segment(selectors),
        rsonpath_syntax::Segment::Descendant(selectors) =>
            generate_descendant_segment(selectors),
    }
}

fn generate_child_segment(selectors: &rsonpath_syntax::Selectors) -> Segment {
    Segment { instructions: generate_selectors(selectors) }
}

fn generate_descendant_segment(selectors: &rsonpath_syntax::Selectors) -> Segment {
    Segment {
        instructions: vec![
            WhileStackNotEmpty {
                instructions: generate_selectors(selectors).into_iter()
                    .chain(vec![PopAndPushAllChildren])
                    .collect()
            },
        ]
    }
}

fn generate_selectors(selectors: &rsonpath_syntax::Selectors) -> Vec<Instruction> {
    let mut instructions: Vec<Instruction> = Vec::new();
    for selector in selectors.iter() {
        instructions.extend(generate_selector(selector));
    }
    instructions
}

fn generate_selector(selector_syntax: &rsonpath_syntax::Selector) -> Vec<Instruction> {
    match selector_syntax {
        rsonpath_syntax::Selector::Name(name_syntax) => {
            let name = Name(name_syntax.unquoted().to_owned());
            vec![SelectChildByName { name }]
        }
        rsonpath_syntax::Selector::Wildcard => vec![SelectAllChildren],
        rsonpath_syntax::Selector::Index(index_syntax) => {
            let index = generate_index(index_syntax);
            vec![SelectElementAtIndex { index }]
        }
        rsonpath_syntax::Selector::Slice(slice_syntax) => {
            let slice = generate_slice(slice_syntax);
            vec![SelectSlice { slice }]
        }
        rsonpath_syntax::Selector::Filter(logical_expr_syntax) => {
            vec![
                FilterIteration {
                    instructions: iter::once(PushCurrentFilterNode)
                        .chain(generate_logical_expr(logical_expr_syntax))
                        .chain(vec![SelectNodeConditionally])
                        .collect()
                },
            ]
        }
    }
}

fn generate_logical_expr(logical_expr_syntax: &rsonpath_syntax::LogicalExpr) -> Vec<Instruction> {
    match logical_expr_syntax {
        rsonpath_syntax::LogicalExpr::Or(lhs, rhs) => {
            generate_logical_expr(lhs).into_iter()
                .chain(generate_logical_expr(rhs))
                .chain(iter::once(Or))
                .collect()
        }
        rsonpath_syntax::LogicalExpr::And(lhs, rhs) => {
            generate_logical_expr(lhs).into_iter()
                .chain(generate_logical_expr(rhs))
                .chain(iter::once(And))
                .collect()
        }
        rsonpath_syntax::LogicalExpr::Not(expr) => {
            generate_logical_expr(expr).into_iter()
                .chain(iter::once(Not))
                .collect()
        }
        rsonpath_syntax::LogicalExpr::Comparison(comparison_expr) => {
            generate_comparison(comparison_expr)
        }
        rsonpath_syntax::LogicalExpr::Test(test_expr) => {
            generate_existence_test(test_expr)
        }
    }
}

fn generate_comparison(comparison_expr_syntax: &rsonpath_syntax::ComparisonExpr) -> Vec<Instruction> {
    generate_comparable(comparison_expr_syntax.lhs()).into_iter()
        .chain(generate_comparable(comparison_expr_syntax.rhs()))
        .chain(iter::once(Compare { op: generate_comparison_op(&comparison_expr_syntax.op()) }))
        .collect()
}

fn generate_comparable(comparable_syntax: &rsonpath_syntax::Comparable) -> Vec<Instruction> {
    match comparable_syntax {
        rsonpath_syntax::Comparable::Literal(literal_syntax) => {
            vec![PushLiteral { literal: generate_literal(literal_syntax) }]
        }
        rsonpath_syntax::Comparable::AbsoluteSingularQuery(singular_query_syntax) => {
            iter::once(PushRootNode)
                .chain(generate_singular_query(singular_query_syntax))
                .collect()
        }
        rsonpath_syntax::Comparable::RelativeSingularQuery(singular_query_syntax) => {
            iter::once(PushCurrentFilterNode)
                .chain(generate_singular_query(singular_query_syntax))
                .collect()
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

fn generate_existence_test(test_expr: &rsonpath_syntax::TestExpr) -> Vec<Instruction> {
    match test_expr {
        rsonpath_syntax::TestExpr::Absolute(query_syntax) => {
            vec![ExistenceTest { absolute: true, subquery: generate(query_syntax) }]
        }
        rsonpath_syntax::TestExpr::Relative(query_syntax) => {
            vec![ExistenceTest { absolute: false, subquery: generate(query_syntax) }]
        }
    }
}


fn generate_singular_query(singular_query_syntax: &rsonpath_syntax::SingularJsonPathQuery) -> Vec<Instruction> {
    singular_query_syntax.segments().map(|singular_segment_syntax| {
        match singular_segment_syntax {
            rsonpath_syntax::SingularSegment::Name(name_syntax) => {
                let name = Name(name_syntax.unquoted().to_owned());
                PopAndPushChildByName { name }
            }
            rsonpath_syntax::SingularSegment::Index(index_syntax) => {
                let index = generate_index(index_syntax);
                PopAndPushElementAtIndex { index }
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
        step: step_syntax_as_i64(&slice_syntax.step()),
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
        rsonpath_syntax::Literal::Null => Null,
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
