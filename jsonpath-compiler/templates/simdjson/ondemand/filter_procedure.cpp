bool {{name|lower}}(
{% for i in 0..arity %}
    {% if i > 0 %} , {% endif %}
    subquery_result param{{i}}
{% endfor %}
)
{
    return {{expression.render().unwrap()}};
}