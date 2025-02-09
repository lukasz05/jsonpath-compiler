{% macro compile_start_filter_execution(filter_id) %}
auto* f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}} = new filter_instance({{filter_id.segment_index}}, {{filter_id.selector_index}});
{% for (subquery_index, subquery) in filter_subqueries.unwrap().get(filter_id).unwrap().into_iter().enumerate() %}
        f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->current_subqueries_segments[{{subquery_index}}] =
    {% if subquery.segments.is_empty() %}
            nullptr;
            reached_subqueries_results.push_back(
                &f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->subqueries_results[{{subquery_index}}]);
    {% else %}
            &filter_{{filter_id.segment_index}}_{{filter_id.selector_index}}_subquery_{{subquery_index}}_segment_0;
    {% endif %}
    f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->subqueries_results[{{subquery_index}}] = {};
{% endfor %}
filter_instances.push_back(f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}});
all_filter_instances.push_back(f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}});
{% endmacro %}

{% macro compile_update_subqueries_state() %}
vector<array<subquery_path_segment*, MAX_SUBQUERIES_IN_FILTER>> current_subqueries_segments_copies;
current_subqueries_segments_copies.reserve(filter_instances.size());
if (current_node.is_member || current_node.is_element) {
    for (auto f_instance : filter_instances) {
        current_subqueries_segments_copies.push_back(f_instance->current_subqueries_segments);
        for (size_t i = 0; i < MAX_SUBQUERIES_IN_FILTER; i++) {
            auto subquery_segment = f_instance->current_subqueries_segments[i];
            if (subquery_segment == nullptr)
                continue;
            if (current_node.is_member && !subquery_segment->is_name)
                continue;
            if (current_node.is_element && subquery_segment->is_name)
                continue;
            if (current_node.is_member && current_node.key.compare(subquery_segment->name) != 0) {
                f_instance->current_subqueries_segments[i] = nullptr;
                continue;
            }
            if (current_node.is_element && current_node.index != subquery_segment->index
                && (subquery_segment->index >= 0 || subquery_segment->index + current_node.array_length != current_node.index)) {
                f_instance->current_subqueries_segments[i] = nullptr;
                continue;
            }
            if (subquery_segment->next == nullptr) {
                if (node.is_scalar())
                    reached_subqueries_results.push_back(&f_instance->subqueries_results[i]);
                else
                    f_instance->subqueries_results[i].type = COMPLEX;
            }
            f_instance->current_subqueries_segments[i] = const_cast<subquery_path_segment *>(subquery_segment->next);
        }
    }
}
{% endmacro %}