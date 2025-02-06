{% match expression %}
    {% when FilterExpression::Or with {lhs, rhs} %}
        {% let lhs_template = FilterExpressionTemplate::new(lhs) %}
        {% let rhs_template = FilterExpressionTemplate::new(rhs) %}
        ({{ lhs_template.render().unwrap() }} || {{ rhs_template.render().unwrap() }})
    {% when FilterExpression::And with {lhs, rhs} %}
        {% let lhs_template = FilterExpressionTemplate::new(lhs) %}
        {% let rhs_template = FilterExpressionTemplate::new(rhs) %}
        ({{ lhs_template.render().unwrap() }} && {{ rhs_template.render().unwrap() }})
    {% when FilterExpression::Not with {expr} %}
        {% let template = FilterExpressionTemplate::new(expr) %}
        !({{ template.render().unwrap() }})
    {% when FilterExpression::Comparison with {lhs, rhs, op} %}
        ({% call compile_comparable(lhs) %}
        {% match op %}
            {% when crate::ir::ComparisonOp::EqualTo %} ==
            {% when crate::ir::ComparisonOp::NotEqualTo %} !=
            {% when crate::ir::ComparisonOp::LessOrEqualTo %} <=
            {% when crate::ir::ComparisonOp::GreaterOrEqualTo %} >=
            {% when crate::ir::ComparisonOp::LessThan %} <
            {% when crate::ir::ComparisonOp::GreaterThan %} >
        {% endmatch %}
        {% call compile_comparable(rhs) %})
    {% when FilterExpression::ExistenceTest with {param_id} %} (params[{{param_id}}].type != subquery_result::NOTHING)
{% endmatch %}

{% macro compile_comparable(comparable) %}
    {% match comparable %}
        {% when Comparable::Param with {id} %} params[{{id}}]
        {% when Comparable::Literal with {value} %}
            {% match value %}
                {% when LiteralValue::String with (str) %} "{{str}}"
                {% when LiteralValue::Int with (x) %} {{x}}ll
                {% when LiteralValue::Float with (x) %} {{x}}
                {% when LiteralValue::Bool with (x) %} {{x}}
                {% when LiteralValue::Null %} null
            {% endmatch %}
    {% endmatch %}
{% endmacro %}

