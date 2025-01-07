void {{name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t>> &all_results)
{
{% for instruction in instructions %}
    {{ instruction.render().unwrap() }}
{% endfor %}
{% if !Self::are_object_members_iterated(self) %}
    {{ EMPTY_OBJECT_ITERATION.render().unwrap() }}
{% endif %}
{% if !Self::are_array_elements_iterated(self) %}
    {{ EMPTY_ARRAY_ITERATION.render().unwrap() }}
{% endif %}
    if (node.is_scalar())
        traverse_and_save_selected_nodes(node, result_buf);
}