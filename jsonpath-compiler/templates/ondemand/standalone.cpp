{% import "macros.cpp" as macros %}
{% if logging %}
#define SIMDJSON_VERBOSE_LOGGING 1
{% endif %}

#include <iostream>
#include <fstream>
#include <vector>
#include <queue>
#include <string>
#include <algorithm>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <simdjson.h>

using namespace std;
using namespace simdjson;

{% if mmap %}
string_view map_input(const char* filename);
{% else %}
string read_input(const char* filename);
{% endif %}

{% for procedure in procedures %}
void {{procedure.name|lower}}(ondemand::value &node, vector<string*> &results_in_progress, vector<string*> &all_results);
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
    ondemand::value root_node = parser.iterate(json).get_value().value();
    vector<string*> results_in_progress;
    vector<string*> all_results;
    selectors_0(root_node, results_in_progress, all_results);
    cout << "[\n";
    bool first = true;
    for (const auto &buf_ptr : all_results)
    {
        if (!first) cout << ",";
        cout << "  " << *buf_ptr;
        first = false;
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