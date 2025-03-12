{% if are_any_filters %}
void {{query_name}}_{{name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t, selection_condition*>> &all_results, selection_condition *segment_conditions[], vector<filter_instance*> &filter_instances, current_node_data &current_node)
{% else %}
void {{query_name}}_{{name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t>> &all_results)
{% endif %}
{
bool _is_scalar = node.is_scalar();
{% if are_any_filters %}
bool is_member = current_node.is_member;
bool is_element = current_node.is_element;
{% endif %}
{% for instruction in instructions %}
    {{ instruction.render().unwrap() }}
{% endfor %}
{% if !Self::are_object_members_iterated(self) %}
    {{ EMPTY_OBJECT_ITERATION.render().unwrap() }}
{% endif %}
{% if !Self::are_array_elements_iterated(self) %}
    {{ EMPTY_ARRAY_ITERATION.render().unwrap() }}
{% endif %}
    if (_is_scalar)
        {% if are_any_filters %}
        {
        current_node.is_member = false;
        current_node.is_element = false;
        traverse_and_save_selected_nodes(node, result_buf, filter_instances, current_node);
        }
    for (int i = 0; i < filter_instances.size(); i++)
        filter_instances[i]->restore_current_subqueries_segments();
        {% else %}
        traverse_and_save_selected_nodes(node, result_buf);
        {% endif %}
}