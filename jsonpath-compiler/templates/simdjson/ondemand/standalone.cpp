{%- import "macros.cpp" as common -%}

{%- if logging -%}
    #define SIMDJSON_VERBOSE_LOGGING 1
{%- endif -%}

#include <iostream>
#include <fstream>
#include <vector>
#include <queue>
#include <set>
#include <unordered_set>
#include <unordered_map>
#include <string>
#include <algorithm>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include "simdjson.h"

using namespace std;
using namespace simdjson;

{%- if Self::are_any_filters(self) -%}
    constexpr uint8_t _SEGMENT_COUNT = {{segments_count}};

    {%- call common::generate_filter_aux_structures(Self::max_subqueries_in_filter_count(self)) -%}

    {%- call common::generate_filter_aux_procedures_declarations("") -%}

    {%- call common::generate_filter_procedures_declarations(filter_procedures, "") -%}

    {%- call common::generate_filter_aux_structures_instances(filter_subqueries, "") -%}
{%- endif -%}

{%- if mmap -%}
    string_view map_and_pad_input(const char* filename, size_t &capacity);
{%- else -%}
    string read_and_pad_input(const char *filename);
{%- endif -%}

{%- call common::generate_procedures_declarations(procedures) -%}

int main(int argc, char **argv)
{
    ondemand::parser parser;
    {%- if mmap -%}
        size_t capacity;
        const auto padded_input = map_and_pad_input(argv[1], capacity);
        ondemand::document doc = parser.iterate(padded_input, capacity);
    {%- else -%}
        const auto padded_input = read_and_pad_input(argv[1]);
        ondemand::document doc = parser.iterate(padded_input);
    {%- endif -%}
    ondemand::value root_node = doc.get_value().value();
    {%- if Self::are_any_filters(self) -%}
        vector<tuple<string *, size_t, size_t, selection_condition*>> all_results;
        unordered_set<int> filter_instances_ids;
        selection_condition *segment_conditions[_SEGMENT_COUNT] = {};
        current_node_data current_node {false, false, 0, 0, {}};
        _selectors_0(root_node, nullptr, all_results, segment_conditions, filter_instances_ids, current_node);
    {%- else -%}
        vector<tuple<string *, size_t, size_t>> all_results;
        _selectors_0(root_node, nullptr, all_results);
    {%- endif -%}
    cout << "[\n";
    bool first = true;
    unordered_set<string*> bufs_to_free;
    {%- if Self::are_any_filters(self) -%}
        for (const auto &[buf_ptr, start, end, selection_condition] : all_results)
    {%- else -%}
        for (const auto &[buf_ptr, start, end] : all_results)
    {%- endif -%}
    {
        {%- if Self::are_any_filters(self) -%}
            bool condition_value;
            _try_evaluate_selection_condition(selection_condition, condition_value);
            if (condition_value)
            {
        {%- endif -%}
        if (!first)
            cout << ",";
        cout << "  " << string_view(buf_ptr->data() + start, end - start);
        first = false;
        {%- if Self::are_any_filters(self) -%}
            }
        {%- endif -%}
        bufs_to_free.insert(buf_ptr);
    }
    cout << "]\n";
    for (auto buf_ptr : bufs_to_free)
        delete buf_ptr;
    {%- if Self::are_any_filters(self) -%}
        for (auto filter_instance : all_filter_instances)
            delete filter_instance;
        for (auto selection_condition : selection_conditions_to_delete)
            delete selection_condition;
    {%- endif -%}
    return 0;
}

{%- if mmap -%}
    string_view map_and_pad_input(const char* filename, size_t &capacity)
    {
        const int fd = open(filename, O_RDONLY);
        if (fd == -1) exit(1);
        struct stat sb{};
        if (fstat(fd, &sb) == -1) exit(1);
        capacity = sb.st_size + SIMDJSON_PADDING;
        const auto addr = static_cast<const char*>(mmap(nullptr, capacity, PROT_READ, MAP_PRIVATE, fd, 0u));
        if (addr == MAP_FAILED) exit(1);
        return {addr};
    }
{%- else -%}
    string read_and_pad_input(const char *filename)
    {
        ostringstream buf;
        ifstream input(filename);
        buf << input.rdbuf();
        string input_str = buf.str();
        input_str.reserve(input_str.size() + SIMDJSON_PADDING);
        return input_str;
    }
{%- endif -%}

{%- if Self::are_any_filters(self) -%}
    {%- call common::generate_filter_aux_procedures_definitions("", filter_procedures) -%}
{%- endif -%}

{%- call common::generate_traverse_and_save_selected_nodes_procedure(Self::are_any_filters(self), eager_filter_evaluation, "") -%}

{%- for procedure in procedures -%}
    {{ procedure.render().unwrap() }}
{%- endfor -%}

{%- for filter_procedure in filter_procedures -%}
    {{ filter_procedure.render().unwrap() }}
{%- endfor -%}