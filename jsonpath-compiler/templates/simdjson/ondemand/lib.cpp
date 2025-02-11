{%- import "macros.cpp" as common -%}

{% if !bindings %}
#ifndef {{filename.replace(".", "_")|upper}}
#define {{filename.replace(".", "_")|upper}}
{% endif %}

{% if logging %}
#define SIMDJSON_VERBOSE_LOGGING 1
{% endif %}

#include <vector>
#include <queue>
#include <set>
#include <string>
#include <algorithm>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include "simdjson.h"

using namespace std;
using namespace simdjson;

{% if Self::are_any_filters(self) %}
{% call common::generate_filter_aux_structures(Self::max_subqueries_in_filter_count(self)) %}

{% for query_name in Self::query_names(self) %}
{% if Self::are_any_filters_in_query(self, query_name) %}
constexpr uint8_t {{query_name}}_SEGMENT_COUNT = {{Self::query_segments_count(self, query_name)}};
{% call common::generate_filter_aux_procedures_declarations(query_name.to_string()) %}
{% call common::generate_filter_procedures_declarations(Self::query_filter_procedures(self, query_name), query_name.to_string()) %}
{% endif %}

{% if Self::are_any_filters_in_query(self, query_name) %}
{% call common::generate_filter_aux_structures_instances(Self::query_filter_subqueries(self, query_name), query_name.to_string()) %}
{% endif %}
{% endfor %}
{% endif %}

{% call common::generate_procedures_declarations(Self::all_procedures(self)) %}

{% if bindings %}
extern "C" void free_result_buffer(char* result_buf) {
    delete result_buf;
}
{% endif %}

{% for query_name in Self::query_names(self) %}
string {{query_name}}(const char* padded_input, size_t length)
{
    ondemand::parser parser;
    ondemand::document doc = parser.iterate(padded_input, length - 64, length);
    ondemand::value root_node = doc.get_value().value();
    {% if Self::are_any_filters_in_query(self, query_name) %}
    vector<tuple<string *, size_t, size_t, selection_condition*>> all_results;
    vector<filter_instance*> filter_instances;
    selection_condition *segment_conditions[MAX_SUBQUERIES_IN_FILTER] = {};
    {{query_name}}_selectors_0(root_node, nullptr, all_results, segment_conditions, filter_instances, {false, false, 0, 0, {}});
    {% else %}
    vector<tuple<string *, size_t, size_t>> all_results;
    {{query_name}}_selectors_0(root_node, nullptr, all_results);
    {% endif %}
    string result;
    bool first = true;
    result += "[\n";
    string *prev_ptr = nullptr;
    {% if Self::are_any_filters_in_query(self, query_name) %}
    for (const auto &[buf_ptr, start, end, selection_condition] : all_results)
    {% else %}
    for (const auto &[buf_ptr, start, end] : all_results)
    {% endif %}
    {
        {% if Self::are_any_filters_in_query(self, query_name) %}
        if ({{query_name}}_evaluate_selection_condition(selection_condition))
        {
        {% endif %}
        if (!first)
            result += ",";
        result += " ";
        result += string_view(buf_ptr->data() + start, end - start);
        first = false;
        {% if Self::are_any_filters_in_query(self, query_name) %}
        }
        {% endif %}
        if (prev_ptr != nullptr && buf_ptr != prev_ptr)
            delete prev_ptr;
        prev_ptr = buf_ptr;
    }
    if (prev_ptr != nullptr)
        delete prev_ptr;
    result += "]\n";
    {% if Self::are_any_filters_in_query(self, query_name) %}
    for (auto filter_instance : all_filter_instances)
        delete filter_instance;
    for (auto selection_condition : all_selection_conditions)
        delete selection_condition;
    all_filter_instances.clear();
    all_selection_conditions.clear();
    {% endif %}
    return result;
}

{% if bindings %}
extern "C" const char* {{query_name}}_binding(const char* padded_input, size_t input_length, size_t* result_length)
{
    string res_str = {{query_name}}(padded_input, input_length);
    char* res = new char[res_str.length() + 1];
    res_str.copy(res, res_str.length() + 1);
    *result_length = res_str.length();
    return res;
}
{% endif %}

{% endfor %}


{% for (query_name, filter_procedures) in filter_procedures %}
{% call common::generate_filter_aux_procedures_definitions(query_name, filter_procedures) %}
{% endfor %}

{% call common::generate_traverse_and_save_selected_nodes_procedure(Self::are_any_filters(self)) %}

{% for procedure in Self::all_procedures(self) %}
    {{ procedure.render().unwrap() }}
{% endfor %}

{% for filter_procedure in Self::all_filters_procedures(self) %}
    {{ filter_procedure.render().unwrap() }}
{% endfor %}

{% if !bindings %}
#endif
{% endif %}