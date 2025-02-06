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
constexpr uint8_t SEGMENT_COUNT = 10; // TODO

struct selection_condition;
struct filter;
struct filter_instance;
struct subquery_path_segment;
struct subquery_result;
struct current_node_data;

struct subquery_result {
    string str_value;
    int64_t int_value;
    double float_value;
    bool bool_value;
    enum {STRING, INT, FLOAT, BOOL, __NULL, NOTHING} type;

    partial_ordering operator<=>(const subquery_result& other) const
    {
        if (type == NOTHING && other.type == NOTHING) return partial_ordering::equivalent;
        if (type == __NULL && other.type == __NULL) return partial_ordering::equivalent;
        if (other.type == STRING) return *this <=> other.str_value;
        if (other.type == INT) return *this <=> int_value;
        if (other.type == FLOAT) return *this <=> float_value;
        if (other.type == BOOL) return *this <=> bool_value;

        return partial_ordering::unordered;
    }
    partial_ordering operator<=>(const string& other) const
    {
        if (type != STRING)
            return partial_ordering::unordered;

        const int cmp = str_value.compare(other);
        if (cmp == 0) return partial_ordering::equivalent;
        if (cmp < 0) return partial_ordering::less;
        return partial_ordering::greater;
    }
    partial_ordering operator<=>(const int64_t& other) const
    {
        if (type == INT) {
            if (int_value == other) return partial_ordering::equivalent;
            if (int_value < other) return partial_ordering::less;
            return partial_ordering::greater;
        }

        if (type == FLOAT) {
            if (float_value == other) return partial_ordering::equivalent;
            if (float_value < other) return partial_ordering::less;
            return partial_ordering::greater;
        }

        return partial_ordering::unordered;
    }
    partial_ordering operator<=>(const double& other) const
    {
        if (type == INT) {
            if (int_value == other) return partial_ordering::equivalent;
            if (int_value < other) return partial_ordering::less;
            return partial_ordering::greater;
        }

        if (type == FLOAT) {
            if (float_value == other) return partial_ordering::equivalent;
            if (float_value < other) return partial_ordering::less;
            return partial_ordering::greater;
        }

        return partial_ordering::unordered;
    }
    partial_ordering operator<=>(const bool& other) const
    {
        if (type != BOOL) return partial_ordering::unordered;

        return bool_value == other ? partial_ordering::equivalent : partial_ordering::unordered;
    }
    bool operator==(const subquery_result& other) const
    {
        return *this <=> other == partial_ordering::equivalent;
    }
    bool operator==(const string& other) const
    {
        return *this <=> other == partial_ordering::equivalent;
    }
    bool operator==(const int64_t& other) const
    {
        return *this <=> other == partial_ordering::equivalent;
    }
    bool operator==(const double& other) const
    {
        return *this <=> other == partial_ordering::equivalent;
    }
    bool operator==(const bool& other) const
    {
        return *this <=> other == partial_ordering::equivalent;
    }
};

struct selection_condition {
    const enum {AND, OR, FILTER} type;
    const selection_condition *lhs;
    const selection_condition *rhs;
    const filter_instance *filter;

    static selection_condition* new_and(const selection_condition *lhs, const selection_condition *rhs) {
        return new selection_condition {AND, lhs, rhs, nullptr};
    }

    static selection_condition* new_or(const selection_condition *lhs, const selection_condition *rhs) {
        return new selection_condition {OR, lhs, rhs, nullptr};
    }

    static selection_condition* new_filter(const filter_instance *filter) {
        return new selection_condition {FILTER, nullptr, nullptr, filter};
    }
};

struct filter_instance {
    const uint8_t filter_segment_index;
    const uint8_t filter_selector_index;
    subquery_path_segment *current_subqueries_segments[MAX_SUBQUERIES_IN_FILTER];
    subquery_result subqueries_results[MAX_SUBQUERIES_IN_FILTER];
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
                {% when FilterSubquerySegment::Name(name) %} true, "{{name}}", 0,
                {% when FilterSubquerySegment::Index(index) %} false, nullptr, {{index}},
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

{% for filter_procedure in filter_procedures %}
    bool {{filter_procedure.name|lower}}(subquery_result params[]);
{% endfor %}


typedef bool(*filter_function_ptr)(subquery_result[]);

filter_function_ptr get_filter_function(uint8_t filter_segment_index, uint8_t filter_selector_index) {
    {% for filter_procedure in filter_procedures %}
        if (filter_segment_index == {{filter_procedure.filter_id.segment_index}} && filter_selector_index == {{filter_procedure.filter_id.selector_index}})
            return &{{filter_procedure.name|lower}};
    {% endfor %}
    return nullptr;
}

bool evaluate_selection_condition(const selection_condition *condition) {
    if (condition == nullptr) return true;
    switch (condition->type) {
        case selection_condition::AND:
            return evaluate_selection_condition(condition->lhs) && evaluate_selection_condition(condition->rhs);
        case selection_condition::OR:
            return evaluate_selection_condition(condition->lhs) || evaluate_selection_condition(condition->rhs);
        case selection_condition::FILTER:
            auto filter_instance = condition->filter;
            auto filter_function = get_filter_function(filter_instance->filter_segment_index, filter_instance->filter_selector_index);
            return filter_function((subquery_result *)filter_instance->subqueries_results);
    }
}

{% endif %}

{% if mmap %}
string_view map_input(const char* filename);
{% else %}
string read_input(const char* filename);
{% endif %}

{% for procedure in procedures %}
{% if Self::are_any_filters(self) %}
void {{procedure.name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t, selection_condition*>> &all_results, selection_condition *segment_conditions[], vector<filter_instance*> &filter_instances, current_node_data current_node);
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
    {% if Self::are_any_filters(self) %}
    vector<tuple<string *, size_t, size_t, selection_condition*>> all_results;
    vector<filter_instance*> filter_instances;
    selectors_0(root_node, nullptr, all_results, nullptr, filter_instances, {false, false, 0, 0, {}});
    {% else %}
    vector<tuple<string *, size_t, size_t>> all_results;
    selectors_0(root_node, nullptr, all_results);
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
        if (evaluate_selection_condition(selection_condition))
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

{% for filter_procedure in filter_procedures %}
    {{ filter_procedure.render().unwrap() }}
{% endfor %}