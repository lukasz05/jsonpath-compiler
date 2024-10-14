#ifndef HELPERS_H
#define HELPERS_H

#include <iostream>
#include <vector>
#include <string>
#include <simdjson.h>

using namespace std;
using namespace simdjson;

enum singular_selector_type
{
    NAME,
    INDEX
};

enum comparison_op
{
    EQUAL_TO,
    NOT_EQUAL_TO,
    LESS_OR_EQUAL_TO,
    GREATER_OR_EQUAL_TO,
    LESS_THAN,
    GREATER_THAN
};

enum comparable_type
{
    STRING,
    INT,
    FLOAT,
    BOOL,
    _NULL,
    NOTHING,
    OBJECT,
    ARRAY
};

struct singular_selector_value
{
    string name;
    int64_t index;

    singular_selector_value() {}
    singular_selector_value(string value) : name(value) {}
    singular_selector_value(int64_t value) : index(value) {}
};

struct comparable_value
{
    string string_value;
    int64_t int_value;
    double float_value;
    bool bool_value;
    string obj_raw_json_value;
    string arr_raw_json_value;

    comparable_value() {}
    comparable_value(string value) : string_value(value) {}
    comparable_value(int64_t value) : int_value(value) {}
    comparable_value(double value) : float_value(value) {}
    comparable_value(bool value) : bool_value(value) {}
};

struct singular_selector
{
    singular_selector_type type;
    singular_selector_value value;

    singular_selector(singular_selector_type type, singular_selector_value value)
        : type(type), value(value)
    {
    }
};

struct comparable
{
    comparable_type type;
    comparable_value value;
};

string get_jsonpointer_encoded_string(string_view s);

comparable evaluate_singular_query(const vector<singular_selector> &selectors, string base_pointer, const padded_string &json);
bool compare(const comparable &a, const comparable &b, const comparison_op &op);

#endif