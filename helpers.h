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

enum comparison_op {
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

union singular_selector_value
{
    string name;
    int64_t index;

    singular_selector_value() {}
    singular_selector_value(string value) : name(value) {}
    singular_selector_value(int64_t value) : index(value) {}
    singular_selector_value(const singular_selector_value& selector) {}
    ~singular_selector_value() {}
};

union comparable_value
{
    string string_value;
    int64_t int_value;
    double float_value;
    bool bool_value;
    dom::object obj_value;
    dom::array arr_value;

    comparable_value() {}
    comparable_value(string value) : string_value(value) {}
    comparable_value(int64_t value) : int_value(value) {}
    comparable_value(double value) : float_value(value) {}
    comparable_value(bool value) : bool_value(value) {}
    comparable_value(dom::object value) : obj_value(value) {}
    comparable_value(dom::array value) : arr_value(value) {}
    comparable_value(const comparable_value& comparable) {}
    ~comparable_value() {}
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

bool is_equal(const comparable &a, const comparable &b);
bool is_less_than(const comparable &a, const comparable &b);
bool is_number(const comparable &c);
bool is_equal_obj(const dom::object &a, const dom::object &b);
bool is_equal_arr(const dom::array &a, const dom::array &b);
bool is_equal_elem(const dom::element &a, const dom::element &b);
bool is_equal_num(const dom::element &a, const dom::element &b);
int compare_numbers(const comparable &a, const comparable &b);

#endif