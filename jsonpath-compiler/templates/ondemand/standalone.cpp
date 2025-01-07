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

{% if mmap %}
string_view map_input(const char* filename);
{% else %}
string read_input(const char* filename);
{% endif %}

{% for procedure in procedures %}
void {{procedure.name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t>> &all_results);
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
    selectors_0(root_node, nullptr, all_results);
    cout << "[\n";
    bool first = true;
    for (const auto &[buf_ptr, start, end] : all_results)
    {
        if (!first)
            cout << ",";
        cout << "  " << buf_ptr->substr(start, end - start);
        first = false;
    }
    set<string*> deleted_bufs;
    for (auto [buf_ptr, _start, _end] : all_results) {
        if (deleted_bufs.contains(buf_ptr))
            continue;
        deleted_bufs.insert(buf_ptr);
        delete buf_ptr;
    }
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