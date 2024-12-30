#ifndef {{filename.replace(".", "_")|upper}}
#define {{filename.replace(".", "_")|upper}}

{% if logging %}
#define SIMDJSON_VERBOSE_LOGGING 1
{% endif %}

#include <vector>
#include <queue>
#include <string>
#include <algorithm>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include "simdjson.h"

using namespace std;
using namespace simdjson;

{% for procedure in procedures %}
void {{procedure.name|lower}}(ondemand::value &node, vector<string*> &results_in_progress, vector<string*> &all_results);
{% endfor %}

{% for query_name in query_names %}
string {{query_name}}(const string_view input)
{
    const auto json = padded_string(input);
    ondemand::parser parser;
    ondemand::value root_node = parser.iterate(json).get_value().value();
    vector<string*> results_in_progress;
    vector<string*> all_results;
    {{query_name}}_selectors_0(root_node, results_in_progress, all_results);
    string result;
    bool first = true;
    for (const auto &buf_ptr : all_results)
    {
        if (!first) result += ",";
        result += "  ";
        result += *buf_ptr;
        first = false;
        delete buf_ptr;
    }
    result += "]\n";
    return result;
}
{% endfor %}

void add_to_all_bufs(const vector<string*> &bufs, const string_view str)
{
    for (const auto &buf_ptr : bufs) *buf_ptr += str;
}

void traverse_and_save_selected_nodes(ondemand::value &node, vector<string*> &results_in_progress)
{
    if (!results_in_progress.empty())
        add_to_all_bufs(results_in_progress, node.raw_json());
}

{% for procedure in procedures %}
    {{ procedure.render().unwrap() }}
{% endfor %}

#endif