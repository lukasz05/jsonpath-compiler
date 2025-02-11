bool {{query_name}}_{{name|lower}}(array<subquery_result, MAX_SUBQUERIES_IN_FILTER> params)
{
    return {{expression.render().unwrap()}};
}