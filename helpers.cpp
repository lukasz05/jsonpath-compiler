#include "helpers.h"

#include <map>

string get_jsonpointer_encoded_string(string_view s)
{
    string res = "";
    for (char c : s)
    {
        if (c == '~') res += "~0";
        else if (c == '/') res += "~1";
        else res += c;
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
        dom::parser dom_parser;
        dom::array materialized_array_value = dom_parser.parse(padded_string(array_value.raw_json())).get_array();
        return {ARRAY, {materialized_array_value}};
    }

    ondemand::object object_value;
    if (!current_node.get_object().get(object_value))
    {
        dom::parser dom_parser;
        dom::object materialized_object_value = dom_parser.parse(padded_string(array_value.raw_json())).get_object();
        return {OBJECT, {materialized_object_value}};
    }

    // TODO: error
    return {};
}

bool compare(const comparable &a, const comparable &b, const comparison_op &op)
{
    switch (op)
    {
        case EQUAL_TO:
            return is_equal(a, b);

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

bool is_equal(const comparable &a, const comparable &b)
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
        return is_equal_obj(a.value.obj_value, b.value.obj_value);

    if (a.type == ARRAY && b.type == ARRAY)
        return is_equal_arr(a.value.arr_value, b.value.arr_value);

    return false;
}

bool is_less_than(const comparable &a, const comparable &b)
{
    if (is_number(a) && is_number(b))
        return compare_numbers(a, b) == -1;

    if (a.type == STRING && b.type == STRING)
        return a.value.string_value < a.value.string_value; // TODO: czy działa dobrze dla pustych?

    return false; 
}

bool is_number(const comparable &c)
{
    return c.type == INT || c.type == FLOAT;
}

int compare_numbers(const comparable &a, const comparable &b)
{
    // TODO
    return 0;
}

bool is_equal_obj(const dom::object &a, const dom::object &b)
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
        if (!is_equal_elem(kv.second, kvs_b[kv.first]))
            return false;
    
    return true;
}

bool is_equal_arr(const dom::array &a, const dom::array &b)
{
    if (a.size() != b.size())
        return false;

    return true;
}

bool is_equal_elem(const dom::element &a, const dom::element &b)
{
    if (a.is_null() && b.is_null())
        return true;

    if (a.is_bool() && b.is_null())
    {
        bool a_bool, b_bool;
        a.get_bool().get(a_bool);
        b.get_bool().get(b_bool);
        return a_bool == b_bool;
    }

    if (a.is_string() && b.is_string())
    {
        string_view a_str, b_str;
        a.get_string().get(a_str);
        b.get_string().get(b_str);
        return a_str == b_str;
    }

    if (a.is_number() && b.is_number())
        return is_equal_num(a, b);

    if (a.is_object() && b.is_object())
        return is_equal_obj(a.get_object(), b.get_object());

    if (a.is_array() && b.is_array())
        return is_equal_arr(a.get_array(), b.get_array());

    return false;
}

bool is_equal_num(const dom::element &a, const dom::element &b)
{
    // TODO
    return false;
}