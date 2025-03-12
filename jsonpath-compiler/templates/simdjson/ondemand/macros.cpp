{% macro generate_filter_aux_structures(max_subqueries_in_filter_count) %}
constexpr uint8_t MAX_SUBQUERIES_IN_FILTER = {{max_subqueries_in_filter_count}};

struct selection_condition;
struct filter;
struct filter_instance;
struct subquery_path_segment;
struct subquery_result;
struct current_node_data;

vector<selection_condition*> all_selection_conditions;
vector<filter_instance*> all_filter_instances;

vector<subquery_result *> reached_subqueries_results;

enum subquery_result_type
{
 NOTHING, STRING, INT, FLOAT, BOOL, __NULL, COMPLEX
};

struct subquery_result {
    string_view str_value;
    int64_t int_value;
    double float_value;
    bool bool_value;
    subquery_result_type type;

    partial_ordering operator<=>(const subquery_result& other) const
    {
        if (type == NOTHING && other.type == NOTHING) return partial_ordering::equivalent;
        if (type == __NULL && other.type == __NULL) return partial_ordering::equivalent;
        if (other.type == STRING) return *this <=> other.str_value;
        if (other.type == INT) return *this <=> other.int_value;
        if (other.type == FLOAT) return *this <=> other.float_value;
        if (other.type == BOOL) return *this <=> other.bool_value;

        return partial_ordering::unordered;
    }
    partial_ordering operator<=>(const string_view& other) const
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
    bool operator==(const string_view& other) const
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
        auto ptr = new selection_condition {AND, lhs, rhs, nullptr};
        all_selection_conditions.push_back(ptr);
        return ptr;
    }

    static selection_condition* new_or(const selection_condition *lhs, const selection_condition *rhs) {
        auto ptr = new selection_condition {OR, lhs, rhs, nullptr};
        all_selection_conditions.push_back(ptr);
        return ptr;
    }

    static selection_condition* new_filter(const filter_instance *filter) {
        auto ptr = new selection_condition {FILTER, nullptr, nullptr, filter};
        all_selection_conditions.push_back(ptr);
        return ptr;
    }
};

struct subquery_path_segment {
    const bool is_name;
    const char* name;
    const int64_t index;
    const subquery_path_segment *next;
};

struct filter_instance {
    bool is_active;
    uint8_t filter_segment_index;
    uint8_t filter_selector_index;
    array<subquery_path_segment *, MAX_SUBQUERIES_IN_FILTER> current_subqueries_segments;
    vector<array<subquery_path_segment *, MAX_SUBQUERIES_IN_FILTER>> current_subqueries_segments_backups;
    array<subquery_result, MAX_SUBQUERIES_IN_FILTER> subqueries_results;

    filter_instance(uint8_t segment_index, uint8_t selector_index)
        : filter_segment_index(segment_index), filter_selector_index(selector_index)
    {
    }

    void save_current_subqueries_segments()
    {
        current_subqueries_segments_backups.push_back(current_subqueries_segments);
    }

    void restore_current_subqueries_segments()
    {
        current_subqueries_segments = current_subqueries_segments_backups.back();
        current_subqueries_segments_backups.pop_back();
    }

    bool is_any_current_subquery_negative_index()
    {
        for (auto subquery_segment : current_subqueries_segments)
            if (subquery_segment != nullptr && !subquery_segment->is_name && subquery_segment->index < 0)
                return true;
        return false;
    }
};


struct current_node_data {
    bool is_member;
    bool is_element;
    uint64_t array_length;
    uint64_t index;
    string_view key;
};

typedef bool (*filter_function_ptr)(array<subquery_result, MAX_SUBQUERIES_IN_FILTER> subquery_result);
{% endmacro %}

{% macro generate_filter_aux_structures_instances(filter_subqueries, query_name) %}
{% for (filter_id, subqueries) in filter_subqueries %}
    {% for (subquery_index, subquery) in subqueries.iter().enumerate() %}
        {% for (segment_index, segment) in subquery.segments.iter().enumerate().rev() %}
            subquery_path_segment {{query_name}}_filter_{{filter_id.segment_index}}_{{filter_id.selector_index}}_subquery_{{subquery_index}}_segment_{{segment_index}} {
            {% match segment %}
                {% when FilterSubquerySegment::Name(name) %} true, "{{name}}", 0,
                {% when FilterSubquerySegment::Index(index) %} false, nullptr, {{index}},
            {% endmatch %}
            {% if segment_index == subquery.segments.len() - 1 %}
                nullptr
            {% else %}
                &{{query_name}}_filter_{{filter_id.segment_index}}_{{filter_id.selector_index}}_subquery_{{subquery_index}}_segment_{{segment_index+1}}
            {% endif %}
            };
        {% endfor %}
    {% endfor %}
{% endfor %}
{% endmacro %}

{% macro generate_filter_aux_procedures_declarations(query_name) %}
filter_function_ptr {{query_name}}_get_filter_function(uint8_t filter_segment_index, uint8_t filter_selector_index);
bool {{query_name}}_evaluate_selection_condition(const selection_condition *condition);
{% endmacro %}

{% macro generate_filter_aux_procedures_definitions(query_name, filter_procedures) %}
filter_function_ptr {{query_name}}_get_filter_function(uint8_t filter_segment_index, uint8_t filter_selector_index) {
    {% for filter_procedure in filter_procedures %}
        if (filter_segment_index == {{filter_procedure.filter_id.segment_index}} && filter_selector_index == {{filter_procedure.filter_id.selector_index}})
            return &{{query_name}}_{{filter_procedure.name|lower}};
    {% endfor %}
    return nullptr;
}

bool {{query_name}}_evaluate_selection_condition(const selection_condition *condition) {
    if (condition == nullptr) return true;
    switch (condition->type) {
        case selection_condition::AND:
            if (condition->lhs == nullptr)
                return {{query_name}}_evaluate_selection_condition(condition->rhs);
            if (condition->rhs == nullptr)
                return {{query_name}}_evaluate_selection_condition(condition->lhs);
            return {{query_name}}_evaluate_selection_condition(condition->lhs) && {{query_name}}_evaluate_selection_condition(condition->rhs);
        case selection_condition::OR:
            if (condition->lhs == nullptr || condition->rhs == nullptr)
                return true;
            return {{query_name}}_evaluate_selection_condition(condition->lhs) || {{query_name}}_evaluate_selection_condition(condition->rhs);
        case selection_condition::FILTER:
            auto filter_instance = condition->filter;
            auto filter_function = {{query_name}}_get_filter_function(filter_instance->filter_segment_index, filter_instance->filter_selector_index);
            return filter_function(filter_instance->subqueries_results);
    }
}
{% endmacro %}

{% macro generate_filter_procedures_declarations(filter_procedures, query_name) %}
{% for filter_procedure in filter_procedures %}
    bool {{query_name}}_{{filter_procedure.name|lower}}(array<subquery_result, MAX_SUBQUERIES_IN_FILTER> params);
{% endfor %}
{% endmacro %}

{% macro generate_traverse_and_save_selected_nodes_procedure(are_any_filters) %}

void traverse_and_save_selected_nodes(ondemand::value &node, string* result_buf)
{
    if (result_buf != nullptr)
        *result_buf += node.raw_json().value();
}

{% if are_any_filters %}
void traverse_and_save_selected_nodes(ondemand::value &node, string *result_buf,
                                      vector<filter_instance*> &filter_instances,
                                      current_node_data &current_node)
{
    bool is_member = current_node.is_member;
    bool is_element = current_node.is_element;
    bool _is_scalar = node.is_scalar();
    {% call compile_update_subqueries_state() %}

    if (_is_scalar) {
        if (result_buf != nullptr)
            *result_buf += node.raw_json().value();

        if (!reached_subqueries_results.empty()) {
            string_view str_value;
            int64_t int_value;
            double float_value;
            bool bool_value;
            subquery_result_type type;

            if (node.is_null())
                type = __NULL;
            else if (!node.get_string().get(str_value))
                type = STRING;
            else if (!node.get_int64().get(int_value))
                type = INT;
            else if (!node.get_double().get(float_value))
                type = FLOAT;
            else if (!node.get_bool().get(bool_value))
                type = BOOL;

            while (!reached_subqueries_results.empty())
            {
                auto subquery_result = reached_subqueries_results.back();
                reached_subqueries_results.pop_back();
                subquery_result->type = type;
                switch (type)
                {
                    case STRING:
                        subquery_result->str_value = str_value;
                    break;

                    case INT:
                        subquery_result->int_value = int_value;
                    break;

                    case FLOAT:
                        subquery_result->float_value = float_value;
                    break;

                    case BOOL:
                        subquery_result->bool_value = bool_value;
                    break;

                    default:
                        break;
                }
            }
        }
        if (is_member || is_element) {
            for (int i = 0; i < filter_instances.size(); i++)
                filter_instances[i]->restore_current_subqueries_segments();
        }
        return;
    }

    if (result_buf == nullptr && filter_instances.empty())
        return;

    if (result_buf != nullptr && filter_instances.empty()) {
        *result_buf += node.raw_json().value();
        return;
    }

    ondemand::object object;
    if (!node.get_object().get(object)) {
        if (result_buf != nullptr)
            *result_buf += "{";
        bool first = true;
        for (ondemand::field field : object) {
            string_view key = field.unescaped_key();
            if (result_buf != nullptr)
            {
                if (!first)
                    *result_buf += ", ";
                *result_buf += "\"";
                *result_buf += key;
                *result_buf += "\":";
            }
            first = false;
            current_node.is_member = true;
            current_node.is_element = false;
            current_node.key = key;
            traverse_and_save_selected_nodes(field.value(), result_buf, filter_instances, current_node);
        }
        if (result_buf != nullptr)
            *result_buf += "}";
    }

    ondemand::array array;
    if (!node.get_array().get(array)) {
        if (result_buf != nullptr)
            *result_buf += "[";
        bool first = true;
        size_t index = 0;


        size_t array_length = 0;
        for (int i = 0; i < filter_instances.size(); i++)
        {
            if (filter_instances[i]->is_any_current_subquery_negative_index())
            {
                array_length = array.count_elements();
                break;
            }
        }

        for (ondemand::value element : array) {
            if (!first)
            {
                index++;
                if (result_buf != nullptr)
                    *result_buf += ", ";
            }
            first = false;
            current_node.is_member = false;
            current_node.is_element = false;
            current_node.array_length = array_length;
            current_node.index = index;
            traverse_and_save_selected_nodes(element, result_buf, filter_instances, current_node);
        }
        if (result_buf != nullptr)
            *result_buf += "]";
    }

    if (is_member || is_element) {
        for (int i = 0; i < filter_instances.size(); i++)
            filter_instances[i]->restore_current_subqueries_segments();
    }
}
{% endif %}
{% endmacro %}

{% macro compile_start_filter_execution(filter_id, query_name) %}
auto* f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}} = new filter_instance({{filter_id.segment_index}}, {{filter_id.selector_index}});
{% for (subquery_index, subquery) in filter_subqueries.unwrap().get(filter_id).unwrap().into_iter().enumerate() %}
        f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->current_subqueries_segments[{{subquery_index}}] =
    {% if subquery.segments.is_empty() %}
            nullptr;
            reached_subqueries_results.push_back(
                &f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->subqueries_results[{{subquery_index}}]);
    {% else %}
            &{{query_name}}_filter_{{filter_id.segment_index}}_{{filter_id.selector_index}}_subquery_{{subquery_index}}_segment_0;
    {% endif %}
    f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->subqueries_results[{{subquery_index}}] = {};
{% endfor %}
filter_instances.push_back(f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}});
all_filter_instances.push_back(f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}});
{% endmacro %}

{% macro compile_update_subqueries_state() %}
if (current_node.is_member || current_node.is_element) {
    for (auto f_instance : filter_instances) {
        f_instance->save_current_subqueries_segments();
        if (!f_instance->is_active) {
            f_instance->is_active = true;
            continue;;
        }
        for (size_t i = 0; i < MAX_SUBQUERIES_IN_FILTER; i++) {
            auto subquery_segment = f_instance->current_subqueries_segments[i];
            if (subquery_segment == nullptr)
                continue;
            if (current_node.is_member && !subquery_segment->is_name) {
                f_instance->current_subqueries_segments[i] = nullptr;
                continue;
            }
            if (current_node.is_element && subquery_segment->is_name) {
                f_instance->current_subqueries_segments[i] = nullptr;
                continue;
            }
            if (current_node.is_member && current_node.key.compare(subquery_segment->name) != 0) {
                f_instance->current_subqueries_segments[i] = nullptr;
                continue;
            }
            if (current_node.is_element && current_node.index != subquery_segment->index
                && (subquery_segment->index >= 0 || subquery_segment->index + current_node.array_length != current_node.index)) {
                f_instance->current_subqueries_segments[i] = nullptr;
                continue;
            }
            if (subquery_segment->next == nullptr && f_instance->subqueries_results[i].type == NOTHING) {
                if (_is_scalar)
                    reached_subqueries_results.push_back(&f_instance->subqueries_results[i]);
                else
                    f_instance->subqueries_results[i].type = COMPLEX;
            }
            f_instance->current_subqueries_segments[i] = const_cast<subquery_path_segment *>(subquery_segment->next);
        }
    }
}
{% endmacro %}

{% macro generate_procedures_declarations(procedures) %}
{% for procedure in procedures %}
{% if procedure.are_any_filters %}
void {{procedure.query_name}}_{{procedure.name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t, selection_condition*>> &all_results, selection_condition *segment_conditions[], vector<filter_instance*> &filter_instances, current_node_data &current_node);
{% else %}
void {{procedure.query_name}}_{{procedure.name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t>> &all_results);
{% endif %}
{% endfor %}
{% endmacro %}