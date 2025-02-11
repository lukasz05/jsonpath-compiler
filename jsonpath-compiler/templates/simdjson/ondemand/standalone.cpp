{%- import "macros.cpp" as common -%}

{% if logging %}
#define SIMDJSON_VERBOSE_LOGGING 1
{% endif %}

#include <iostream>
#include <fstream>
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

constexpr uint8_t _SEGMENT_COUNT = {{segments_count}};

{% call common::generate_filter_aux_structures(Self::max_subqueries_in_filter_count(self)) %}

{% call common::generate_filter_aux_procedures_declarations("") %}

{% call common::generate_filter_procedures_declarations(filter_procedures, "") %}

{% call common::generate_filter_aux_structures_instances(filter_subqueries, "") %}

{% endif %}

{% if mmap %}
string_view map_input(const char* filename);
{% else %}
string read_input(const char* filename);
{% endif %}

{% call common::generate_procedures_declarations(procedures) %}

int main(int argc, char **argv)
{
{% if mmap %}
    const auto input = map_input(argv[1]);
{% else %}
    const auto input = read_input(argv[1]);
{% endif %}
    const auto json = padded_string(input);
    ondemand::parser parser;
    ondemand::document doc = parser.iterate(json);
    ondemand::value root_node = doc.get_value().value();
    {% if Self::are_any_filters(self) %}
    vector<tuple<string *, size_t, size_t, selection_condition*>> all_results;
    vector<filter_instance*> filter_instances;
    selection_condition *segment_conditions[MAX_SUBQUERIES_IN_FILTER] = {};
    _selectors_0(root_node, nullptr, all_results, segment_conditions, filter_instances, {false, false, 0, 0, {}});
    {% else %}
    vector<tuple<string *, size_t, size_t>> all_results;
    _selectors_0(root_node, nullptr, all_results);
    {% endif %}
    cout << "[\n";
    bool first = true;
    string *prev_ptr = nullptr;
    {% if Self::are_any_filters(self) %}
    for (const auto &[buf_ptr, start, end, selection_condition] : all_results)
    {% else %}
    for (const auto &[buf_ptr, start, end] : all_results)
    {% endif %}
    {
        {% if Self::are_any_filters(self) %}
        if (_evaluate_selection_condition(selection_condition))
        {
        {% endif %}
        if (!first)
            cout << ",";
        cout << "  " << string_view(buf_ptr->data() + start, end - start);
        first = false;
        {% if Self::are_any_filters(self) %}
        }
        {% endif %}
        if (prev_ptr != nullptr && buf_ptr != prev_ptr)
            delete prev_ptr;
        prev_ptr = buf_ptr;
    }
    if (prev_ptr != nullptr)
        delete prev_ptr;
    cout << "]\n";
    {% if Self::are_any_filters(self) %}
    for (auto filter_instance : all_filter_instances)
        delete filter_instance;
    for (auto selection_condition : all_selection_conditions)
        delete selection_condition;
    {% endif %}
    return 0;
}

{% if mmap %}
string_view map_input(const char* filename)
{
    const int fd = open(filename, O_RDONLY);
    if (fd == -1) exit(1);
    struct stat sb{};
    if (fstat(fd, &sb) == -1) exit(1);
    const size_t length = sb.st_size;
    const auto addr = static_cast<const char*>(mmap(nullptr, length, PROT_READ, MAP_PRIVATE, fd, 0u));
    if (addr == MAP_FAILED) exit(1);
    return {addr};
}
{% else %}
string read_input(const char* filename)
{
    ostringstream buf;
    ifstream input (filename);
    buf << input.rdbuf();
    return buf.str();
}
{% endif %}

{% if Self::are_any_filters(self) %}
{% call common::generate_filter_aux_procedures_definitions("", filter_procedures) %}
{% endif %}

{% call common::generate_traverse_and_save_selected_nodes_procedure(Self::are_any_filters(self)) %}

{% for procedure in procedures %}
    {{ procedure.render().unwrap() }}
{% endfor %}

{% for filter_procedure in filter_procedures %}
    {{ filter_procedure.render().unwrap() }}
{% endfor %}