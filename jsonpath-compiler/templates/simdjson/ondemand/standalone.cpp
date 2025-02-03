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

constexpr uint8_t MAX_SUBQUERIES_IN_FILTER = 5; // TODO

struct selection_condition;
struct filter;
struct filter_instance;
struct subquery_path_data;
struct subquery_path_segment;
struct current_node_data;

struct selection_condition {
    enum {AND, OR, FILTER} type;
    selection_condition *lhs;
    selection_condition *rhs;
    filter_instance *filter;
};

struct filter_instance {
    const uint8_t filter_segment_index;
    const uint8_t filter_selector_index;
    subquery_path_segment *current_subqueries_segments[MAX_SUBQUERIES_IN_FILTER];
};

struct subquery_path_segment {
    const bool is_name;
    const char* name;
    const int64_t index;
    const subquery_path_segment *next;
};

struct current_node_data {
    bool is_member;
    bool is_element;
    uint64_t array_length;
    uint64_t index;
    string_view key;
};


{% for (filter_id, subqueries) in filter_subqueries %}
    {% for (subquery_index, subquery) in subqueries.iter().enumerate() %}
        {% for (segment_index, segment) in subquery.segments.iter().enumerate().rev() %}
            subquery_path_segment filter_{{filter_id.segment_index}}_{{filter_id.selector_index}}_subquery_{{subquery_index}}_segment_{{segment_index}} {
            {% match segment %}
                {% when FilterSubquerySegment::Name(name) %}
                true, "{{name}}", 0,
                {% when FilterSubquerySegment::Index(index) %}
                false, nullptr, {{index}},
            {% endmatch %}
            {% if segment_index == subquery.segments.len() - 1 %}
                nullptr
            {% else %}
                &filter_{{filter_id.segment_index}}_{{filter_id.selector_index}}_subquery_{{subquery_index}}_segment_{{segment_index+1}}
            {% endif %}
            };
        {% endfor %}
    {% endfor %}
{% endfor %}

{% endif %}

{% if mmap %}
string_view map_input(const char* filename);
{% else %}
string read_input(const char* filename);
{% endif %}

{% for procedure in procedures %}
{% if Self::are_any_filters(self) %}
void {{procedure.name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t>> &all_results, selection_condition segment_conditions[], vector<filter_instance*> &filter_instances, current_node_data current_node);
{% else %}
void {{procedure.name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t>> &all_results);
{% endif %}
{% endfor %}

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
    vector<tuple<string *, size_t, size_t>> all_results;
    {% if Self::are_any_filters(self) %}
    vector<filter_instance*> filter_instances;

    selectors_0(root_node, nullptr, all_results, nullptr, filter_instances, {false, false, 0, 0, {}});
    {% else %}
    selectors_0(root_node, nullptr, all_results);
    {% endif %}
    cout << "[\n";
    bool first = true;
    string *prev_ptr = nullptr;
    for (const auto &[buf_ptr, start, end] : all_results)
    {
        if (!first)
            cout << ",";
        cout << "  " << string_view(buf_ptr->data() + start, end - start);
        first = false;
        if (prev_ptr != nullptr && buf_ptr != prev_ptr)
            delete prev_ptr;
        prev_ptr = buf_ptr;
    }
    if (prev_ptr != nullptr)
        delete prev_ptr;
    cout << "]\n";
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

void traverse_and_save_selected_nodes(ondemand::value &node, string* result_buf)
{
    if (result_buf != nullptr)
        *result_buf += node.raw_json().value();
}

{% for procedure in procedures %}
    {{ procedure.render().unwrap() }}
{% endfor %}