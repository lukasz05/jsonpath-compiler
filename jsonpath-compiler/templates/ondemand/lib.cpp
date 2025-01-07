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

{% for procedure in procedures %}
void {{procedure.name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t>> &all_results);
{% endfor %}

{% if bindings %}
extern "C" void free_result_buffer(char* result_buf) {
    delete result_buf;
}
{% endif %}

{% for query_name in query_names %}
string {{query_name}}(const char* padded_input, size_t length)
{
    ondemand::parser parser;
    ondemand::document doc = parser.iterate(padded_input, length - 64, length);
    ondemand::value root_node = doc.get_value().value();
    vector<tuple<string *, size_t, size_t>> all_results;
    {{query_name}}_selectors_0(root_node, nullptr, all_results);
    string result;
    bool first = true;
    result += "[\n";
    for (const auto &[buf_ptr, start, end] : all_results)
    {
        if (!first)
            result += ",";
        result += " ";
        result += buf_ptr->substr(start, end - start);
        first = false;
    }
    set<string*> deleted_bufs;
    for (auto [buf_ptr, _start, _end] : all_results) {
        if (deleted_bufs.contains(buf_ptr))
            continue;
        deleted_bufs.insert(buf_ptr);
        delete buf_ptr;
    }
    result += "]\n";
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

void traverse_and_save_selected_nodes(ondemand::value &node, string* result_buf)
{
    if (result_buf != nullptr)
        *result_buf += node.raw_json().value();
}

{% for procedure in procedures %}
    {{ procedure.render().unwrap() }}
{% endfor %}

{% if !bindings %}
#endif
{% endif %}