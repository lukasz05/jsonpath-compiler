#include "helpers.h"

#include <map>

bool is_number(const comparable &c);
bool is_less_than(const comparable &a, const comparable &b);
int compare_numbers(const comparable &a, const comparable &b);
bool are_equal(const comparable &a, const comparable &b);
bool are_equal_obj(const string &raw_json_a, const string &raw_json_b);
bool are_equal_obj(const dom::object &a, const dom::object &b);
bool are_equal_arr(const string &raw_json_a, const string &raw_json_b);
bool are_equal_arr(const dom::array &a, const dom::array &b);
bool are_equal_elem(const dom::element &a, const dom::element &b);
bool are_equal_num(const dom::element &a, const dom::element &b);

string get_jsonpointer_encoded_string(string_view s)
{
    string res = "";
    for (char c : s)
    {
        if (c == '~')
            res += "~0";
        else if (c == '/')
            res += "~1";
        else
            res += c;
    }
    return res;
}

comparable evaluate_singular_query(const vector<singular_selector> &selectors, string base_pointer, const padded_string &json)
{
    simdjson::error_code error;
    ondemand::parser parser;
    auto root_node = parser.iterate(json);
    ondemand::value current_node;
    root_node.at_pointer(base_pointer).get(current_node);
    for (auto selector : selectors)
    {
        switch (selector.type)
        {
        case NAME:
            if (current_node.find_field(selector.value.name).get(current_node))
                return {NOTHING, {}};
            break;

        case INDEX:
            int64_t index = selector.value.index;
            if (index < 0)
            {
                size_t element_count = current_node.count_elements();
                index += element_count;
            }
            if (index < 0 || current_node.at(index).get(current_node))
                return {NOTHING, {}};
            break;
        }
    }

    string_view string_view_value;
    if (!current_node.get_string().get(string_view_value))
        return {STRING, {string(string_view_value)}};

    int64_t int_value;
    if (!current_node.get_int64().get(int_value))
        return {INT, {int_value}};

    double float_value;
    if (!current_node.get_double().get(float_value))
        return {FLOAT, {float_value}};

    bool bool_value;
    if (!current_node.get_bool().get(bool_value))
        return {BOOL, {bool_value}};

    bool is_null;
    if (!current_node.is_null().get(is_null) && is_null)
        return {_NULL, {}};

    ondemand::array array_value;
    if (!current_node.get_array().get(array_value))
    {
        string_view raw_json_value_view;
        error = array_value.raw_json().get(raw_json_value_view);
        if (error)
        {
            cerr << error << "\n";
            exit(EXIT_FAILURE);
        }
        comparable_value comp_value;
        comp_value.arr_raw_json_value = string(raw_json_value_view);
        return {ARRAY, comp_value};
    }

    ondemand::object object_value;
    if (!current_node.get_object().get(object_value))
    {
        string_view raw_json_value_view;
        error = object_value.raw_json().get(raw_json_value_view);
        if (error)
        {
            cerr << error << "\n";
            exit(EXIT_FAILURE);
        }
        comparable_value comp_value;
        comp_value.obj_raw_json_value = string(raw_json_value_view);
        return {OBJECT, comp_value};
    }

    return {};
}

bool compare(const comparable &a, const comparable &b, const comparison_op &op)
{
    switch (op)
    {
    case EQUAL_TO:
        return are_equal(a, b);

    case LESS_THAN:
        return is_less_than(a, b);

    case NOT_EQUAL_TO:
        return !compare(a, b, EQUAL_TO);

    case LESS_OR_EQUAL_TO:
        return compare(a, b, LESS_THAN) || compare(a, b, EQUAL_TO);

    case GREATER_THAN:
        return compare(b, a, LESS_THAN);

    case GREATER_OR_EQUAL_TO:
        return compare(b, a, LESS_OR_EQUAL_TO);
    }
}

bool is_number(const comparable &c)
{
    return c.type == INT || c.type == FLOAT;
}

bool is_less_than(const comparable &a, const comparable &b)
{
    if (is_number(a) && is_number(b))
        return compare_numbers(a, b) == -1;

    if (a.type == STRING && b.type == STRING)
        return a.value.string_value < b.value.string_value;

    return false;
}

int compare_numbers(const comparable &a, const comparable &b)
{
    if (a.type == INT && b.type == INT)
        return (a.value.int_value > b.value.int_value) - (a.value.int_value < b.value.int_value);
    if (a.type == INT && b.type == FLOAT)
        return (a.value.int_value > b.value.float_value) - (a.value.int_value < b.value.float_value);
    if (a.type == FLOAT && b.type == INT)
        return (a.value.float_value > b.value.int_value) - (a.value.float_value < b.value.int_value);

    return (a.value.float_value > b.value.float_value) - (a.value.float_value < b.value.float_value);
}

bool are_equal(const comparable &a, const comparable &b)
{
    if (a.type == _NULL && b.type == _NULL)
        return true;

    if (a.type == NOTHING && b.type == NOTHING)
        return true;

    if (a.type == BOOL && b.type == BOOL)
        return a.value.bool_value == b.value.bool_value;

    if (a.type == STRING && b.type == STRING)
        return a.value.string_value == b.value.string_value;

    if (is_number(a) && is_number(b))
        return compare_numbers(a, b) == 0;

    if (a.type == OBJECT && b.type == OBJECT)
        return are_equal_obj(a.value.obj_raw_json_value, b.value.obj_raw_json_value);

    if (a.type == ARRAY && b.type == ARRAY)
        return are_equal_arr(a.value.arr_raw_json_value, b.value.arr_raw_json_value);

    return false;
}

bool are_equal_obj(const string &raw_json_a, const string &raw_json_b)
{
    dom::parser dom_parser_a, dom_parser_b;
    dom::object obj_a = dom_parser_a.parse(padded_string(raw_json_a)).get_object();
    dom::object obj_b = dom_parser_b.parse(padded_string(raw_json_b)).get_object();
    return are_equal_obj(obj_a, obj_b);
}

bool are_equal_obj(const dom::object &a, const dom::object &b)
{
    if (a.size() != b.size())
        return false;

    map<string_view, dom::element> kvs_a, kvs_b;

    for (dom::key_value_pair kv : a)
        kvs_a[kv.key] = kv.value;
    if (kvs_a.size() != a.size())
        return false;

    for (dom::key_value_pair kv : b)
        kvs_b[kv.key] = kv.value;
    if (kvs_b.size() != b.size())
        return false;

    if (kvs_a.size() != kvs_b.size())
        return false;

    for (auto kv : kvs_a)
        if (!are_equal_elem(kv.second, kvs_b[kv.first]))
            return false;

    return true;
}

bool are_equal_arr(const string &raw_json_a, const string &raw_json_b)
{
    dom::parser dom_parser_a, dom_parser_b;
    dom::array arr_b = dom_parser_a.parse(padded_string(raw_json_b)).get_array();
    dom::array arr_a = dom_parser_b.parse(padded_string(raw_json_a)).get_array();
    return are_equal_arr(arr_a, arr_b);
}

bool are_equal_arr(const dom::array &a, const dom::array &b)
{
    if (a.size() != b.size())
        return false;

    dom::element a_elems[a.size()];
    size_t i = 0;
    for (dom::element elem : a)
        a_elems[i++] = elem;
    i = 0;
    for (dom::element elem : b)
        if (!are_equal_elem(a_elems[i++], elem))
            return false;

    return true;
}

bool are_equal_elem(const dom::element &a, const dom::element &b)
{
    if (a.is_null() && b.is_null())
        return true;

    if (a.is_bool() && b.is_bool())
    {
        bool a_bool = a.get_bool();
        bool b_bool = b.get_bool();
        return a_bool == b_bool;
    }

    if (a.is_string() && b.is_string())
    {
        string_view a_str = a.get_string();
        string_view b_str = b.get_string();
        return a_str == b_str;
    }

    if (a.is_number() && b.is_number())
        return are_equal_num(a, b);

    if (a.is_object() && b.is_object())
        return are_equal_obj(a.get_object(), b.get_object());

    if (a.is_array() && b.is_array())
        return are_equal_arr(a.get_array(), b.get_array());

    return false;
}

bool are_equal_num(const dom::element &a, const dom::element &b)
{
    if (a.is_int64() && b.is_int64())
    {
        int64_t a_val = a.get_int64();
        int64_t b_val = b.get_int64();
        return a_val == b_val;
    }
    return false;
}