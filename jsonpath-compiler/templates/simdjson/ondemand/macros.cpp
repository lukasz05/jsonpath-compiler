{%- macro generate_filter_aux_structures(max_subqueries_in_filter_count) -%}
    constexpr uint8_t MAX_SUBQUERIES_IN_FILTER = {{max_subqueries_in_filter_count}};

    struct selection_condition;
    struct filter;
    struct filter_instance;
    struct subquery_path_segment;
    struct subquery_result;
    struct current_node_data;

    static vector<selection_condition*> selection_conditions_to_delete;
    static vector<filter_instance*> all_filter_instances;
    static unordered_map<int, bool> filters_results;

    static vector<subquery_result *> reached_subqueries_results;

    enum subquery_result_type
    {
     NOTHING, STRING, INT, FLOAT, BOOL, __NULL, COMPLEX
    };

    struct subquery_result {
        filter_instance* filter;
        string_view str_value;
        int64_t int_value;
        double float_value;
        bool bool_value;
        subquery_result_type type;
        bool exists;

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
        enum {AND, OR, FILTER, ALWAYS_FALSE, ALWAYS_TRUE} type;
        selection_condition *lhs;
        selection_condition *rhs;
        filter_instance *filter;

        static selection_condition* new_and(selection_condition *lhs, selection_condition *rhs) {
            auto ptr = new selection_condition {AND, lhs, rhs, nullptr};
            selection_conditions_to_delete.push_back(ptr);
            return ptr;
        }

        static selection_condition* new_or(selection_condition *lhs, selection_condition *rhs) {
            auto ptr = new selection_condition {OR, lhs, rhs, nullptr};
            selection_conditions_to_delete.push_back(ptr);
            return ptr;
        }

        static selection_condition* new_filter(filter_instance *filter) {
            auto ptr = new selection_condition {FILTER, nullptr, nullptr, filter};
            selection_conditions_to_delete.push_back(ptr);
            return ptr;
        }
    };

    static selection_condition always_false_condition {.type = selection_condition::ALWAYS_FALSE};
    static bool segment_condition_value;

    struct subquery_path_segment {
        const bool is_name;
        const char* name;
        const int64_t index;
        const subquery_path_segment *next;
    };

    struct filter_instance {
        int id;
        bool is_active;
        uint8_t filter_segment_index;
        uint8_t filter_selector_index;
        uint8_t subquery_count;
        uint8_t reached_subquery_count;
        array<subquery_path_segment *, MAX_SUBQUERIES_IN_FILTER> current_subqueries_segments;
        vector<subquery_path_segment *> current_subqueries_segments_backups;
        array<subquery_result, MAX_SUBQUERIES_IN_FILTER> subqueries_results;
        array<bool, MAX_SUBQUERIES_IN_FILTER> is_subquery_existence_test;

        filter_instance(int id, uint8_t segment_index, uint8_t selector_index, uint8_t subquery_count)
            : id(id), is_active(false), filter_segment_index(segment_index), filter_selector_index(selector_index),
              subquery_count(subquery_count), reached_subquery_count(0), current_subqueries_segments{},
              subqueries_results{}, is_subquery_existence_test{}
        {
        }

        void save_current_subqueries_segments() {
            current_subqueries_segments_backups.insert(current_subqueries_segments_backups.end(), current_subqueries_segments.begin(), current_subqueries_segments.begin() + subquery_count);
        }

        void restore_current_subqueries_segments() {
            if (current_subqueries_segments_backups.empty())
                return;
            copy(current_subqueries_segments_backups.end() - subquery_count, current_subqueries_segments_backups.end(), current_subqueries_segments.begin());
            current_subqueries_segments_backups.resize(current_subqueries_segments_backups.size() - subquery_count);
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
{%- endmacro -%}

{%- macro generate_filter_aux_structures_instances(filter_subqueries, query_name) -%}
    {%- for (filter_id, subqueries) in filter_subqueries -%}
        {%- for (subquery_index, subquery) in subqueries.iter().enumerate() -%}
            {%- for (segment_index, segment) in subquery.segments.iter().enumerate().rev() -%}
                subquery_path_segment {{query_name}}_filter_{{filter_id.segment_index}}_{{filter_id.selector_index}}_subquery_{{subquery_index}}_segment_{{segment_index}} {
                {%- match segment -%}
                    {%- when FilterSubquerySegment::Name(name) -%} true, "{{name}}", 0,
                    {%- when FilterSubquerySegment::Index(index) -%} false, nullptr, {{index}},
                {%- endmatch -%}
                {%- if segment_index == subquery.segments.len() - 1 -%}
                    nullptr
                {%- else -%}
                    &{{query_name}}_filter_{{filter_id.segment_index}}_{{filter_id.selector_index}}_subquery_{{subquery_index}}_segment_{{segment_index+1}}
                {%- endif -%}
                };
            {%- endfor -%}
        {%- endfor -%}
    {%- endfor -%}
{%- endmacro -%}

{%- macro generate_filter_aux_procedures_declarations(query_name) -%}
    filter_function_ptr {{query_name}}_get_filter_function(uint8_t filter_segment_index, uint8_t filter_selector_index);
    bool {{query_name}}_try_evaluate_selection_condition(selection_condition *condition, bool &value);
{%- endmacro -%}

{%- macro generate_filter_aux_procedures_definitions(query_name, filter_procedures) -%}
    filter_function_ptr {{query_name}}_get_filter_function(uint8_t filter_segment_index, uint8_t filter_selector_index) {
        {%- for filter_procedure in filter_procedures -%}
            if (filter_segment_index == {{filter_procedure.filter_id.segment_index}} && filter_selector_index == {{filter_procedure.filter_id.selector_index}})
                return &{{query_name}}_{{filter_procedure.name|lower}};
        {%- endfor -%}
        return nullptr;
    }

    bool {{query_name}}_try_evaluate_selection_condition(selection_condition *condition, bool &value) {
        if (condition == nullptr)
        {
            value = true;
            return true;
        }
        switch (condition->type) {
            case selection_condition::ALWAYS_FALSE: {
                value = false;
                return true;
            }
            case selection_condition::ALWAYS_TRUE: {
                value = true;
                return true;
            }
            case selection_condition::AND: {
                if (condition->lhs == nullptr && condition->rhs == nullptr)
                {
                    value = true;
                    condition->type = selection_condition::ALWAYS_TRUE;
                    return true;
                }
                if (condition->lhs == nullptr) {
                    bool success = {{query_name}}_try_evaluate_selection_condition(condition->rhs, value);
                    if (success) {
                        condition->type = value
                            ? selection_condition::ALWAYS_TRUE
                            : selection_condition::ALWAYS_FALSE;
                    }
                    return success;
                }
                if (condition->rhs == nullptr) {
                    bool success = {{query_name}}_try_evaluate_selection_condition(condition->lhs, value);
                    if (success) {
                        condition->type = value
                            ? selection_condition::ALWAYS_TRUE
                            : selection_condition::ALWAYS_FALSE;
                    }
                    return success;
                }
                bool lhs_value;
                bool lhs_success = {{query_name}}_try_evaluate_selection_condition(condition->lhs, lhs_value);
                if (lhs_success && !lhs_value)
                {
                    value = false;
                    condition->type = selection_condition::ALWAYS_FALSE;
                    return true;
                }
                bool rhs_value;
                bool rhs_success = {{query_name}}_try_evaluate_selection_condition(condition->rhs, rhs_value);
                if (rhs_success && !rhs_value)
                {
                    value = false;
                    condition->type = selection_condition::ALWAYS_FALSE;
                    return true;
                }
                if (rhs_success && lhs_success)
                {
                    value = true;
                    condition->type = selection_condition::ALWAYS_TRUE;
                    return true;
                }
                return false;
            }
            case selection_condition::OR: {
                if (condition->lhs == nullptr || condition->rhs == nullptr)
                {
                    value = true;
                    condition->type = selection_condition::ALWAYS_TRUE;
                    return true;
                }
                bool lhs_value;
                bool lhs_success = {{query_name}}_try_evaluate_selection_condition(condition->lhs, lhs_value);
                if (lhs_success && lhs_value)
                {
                    value = true;
                    condition->type = selection_condition::ALWAYS_TRUE;
                    return true;
                }
                bool rhs_value;
                bool rhs_success = {{query_name}}_try_evaluate_selection_condition(condition->rhs, rhs_value);
                if (rhs_success && rhs_value)
                {
                    value = true;
                    condition->type = selection_condition::ALWAYS_TRUE;
                    return true;
                }
                if (rhs_success & lhs_success)
                {
                    value = false;
                    condition->type = selection_condition::ALWAYS_FALSE;
                    return true;
                }
                return false;
            }
            case selection_condition::FILTER: {
                {%- if eager_filter_evaluation -%}
                    int filter_id = condition->filter->id;
                    if (filters_results.contains(filter_id))
                    {
                        value = filters_results.at(filter_id);
                        condition->type = value
                            ? selection_condition::ALWAYS_TRUE
                            : selection_condition::ALWAYS_FALSE;
                        return true;
                    }
                    return false;
                {%- else -%}
                    auto filter_instance = condition->filter;
                    auto filter_function = {{query_name}}_get_filter_function(filter_instance->filter_segment_index, filter_instance->filter_selector_index);
                    value = filter_function(filter_instance->subqueries_results);
                    condition->type = value
                        ? selection_condition::ALWAYS_TRUE
                        : selection_condition::ALWAYS_FALSE;
                    return true;
                {%- endif -%}
            }
            default:
                return false;
        }
    }
{%- endmacro -%}

{%- macro generate_filter_procedures_declarations(filter_procedures, query_name) -%}
    {%- for filter_procedure in filter_procedures -%}
        bool {{query_name}}_{{filter_procedure.name|lower}}(array<subquery_result, MAX_SUBQUERIES_IN_FILTER> params);
    {%- endfor -%}
{%- endmacro -%}

{%- macro generate_traverse_and_save_selected_nodes_procedure(are_any_filters, eager_filter_evaluation, query_name) -%}
    void {{query_name}}_traverse_and_save_selected_nodes(ondemand::value &node, string* result_buf)
    {
        if (result_buf != nullptr)
            *result_buf += node.raw_json().value();
    }

    {%- if are_any_filters -%}
        void {{query_name}}_traverse_and_save_selected_nodes(ondemand::value &node, string *result_buf,
                                              unordered_set<int> &filter_instances_ids,
                                              current_node_data &current_node)
        {
            bool is_member = current_node.is_member;
            bool is_element = current_node.is_element;
            bool _is_scalar = node.is_scalar();
            {%- call compile_update_subqueries_state(query_name, eager_filter_evaluation) -%}

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

                        auto filter_instance = subquery_result->filter;
                        if (++filter_instance->reached_subquery_count == filter_instance->subquery_count)
                        {
                            {%- if eager_filter_evaluation -%}
                                auto filter_function = {{query_name}}_get_filter_function(filter_instance->filter_segment_index, filter_instance->filter_selector_index);
                                bool value = filter_function(filter_instance->subqueries_results);
                                filters_results.try_emplace(filter_instance->id, value);
                            {%- endif -%}
                            filter_instances_ids.erase(filter_instance->id);
                        }
                    }
                }
                if (is_member || is_element) {
                    for (int filter_instance_id : filter_instances_ids) {
                        all_filter_instances[filter_instance_id]->restore_current_subqueries_segments();
                    }
                }
                return;
            }

            if (result_buf == nullptr && filter_instances_ids.empty())
                return;

            if (result_buf != nullptr && filter_instances_ids.empty()) {
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
                    {{query_name}}_traverse_and_save_selected_nodes(field.value(), result_buf, filter_instances_ids, current_node);
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
                for (int filter_instance_id : filter_instances_ids)
                {
                    if (all_filter_instances[filter_instance_id]->is_any_current_subquery_negative_index())
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
                    {{query_name}}_traverse_and_save_selected_nodes(element, result_buf, filter_instances_ids, current_node);
                }
                if (result_buf != nullptr)
                    *result_buf += "]";
            }

            if (is_member || is_element) {
                for (int filter_instance_id : filter_instances_ids) {
                    all_filter_instances[filter_instance_id]->restore_current_subqueries_segments();
                }
            }
        }
    {%- endif -%}
{%- endmacro -%}

{%- macro compile_start_filter_execution(filter_id, query_name) -%}
    auto* f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}} = new filter_instance(all_filter_instances.size(), {{filter_id.segment_index}}, {{filter_id.selector_index}}, {{filter_subqueries.unwrap().get(filter_id).unwrap().len()}});
    {%- for (subquery_index, subquery) in filter_subqueries.unwrap().get(filter_id).unwrap().into_iter().enumerate() -%}
            f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->subqueries_results[{{subquery_index}}] = {.filter = f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}};
            f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->current_subqueries_segments[{{subquery_index}}] =
        {%- if subquery.segments.is_empty() -%}
            nullptr;
            {%- if subquery.is_existence_test -%}
                f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->subqueries_results[{{subquery_index}}].exists = true;
            {%- else -%}
                reached_subqueries_results.push_back(
                    &f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->subqueries_results[{{subquery_index}}]);
            {%- endif -%}
        {%- else -%}
            &{{query_name}}_filter_{{filter_id.segment_index}}_{{filter_id.selector_index}}_subquery_{{subquery_index}}_segment_0;
        {%- endif -%}
        f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->is_subquery_existence_test[{{subquery_index}}] = {{subquery.is_existence_test}};
    {%- endfor -%}
    filter_instances_ids.insert(f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}}->id);
    all_filter_instances.push_back(f_instance_{{filter_id.segment_index}}_{{filter_id.selector_index}});
    added_filter_instances++;
{%- endmacro -%}

{%- macro compile_update_subqueries_state(query_name, eager_filter_evaluation) -%}
    if (current_node.is_member || current_node.is_element) {
        for (auto filter_instance_id : filter_instances_ids) {
            filter_instance* f_instance = all_filter_instances[filter_instance_id];
            if (!f_instance->is_active) {
                f_instance->is_active = true;
                continue;
            }
            f_instance->save_current_subqueries_segments();
            for (size_t i = 0; i < f_instance->subquery_count; i++) {
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
                if (subquery_segment->next == nullptr && !f_instance->subqueries_results[i].exists) {
                    f_instance->subqueries_results[i].exists = true;
                    f_instance->current_subqueries_segments[i] = nullptr;
                    if (f_instance->is_subquery_existence_test[i]) {
                        f_instance->reached_subquery_count++;
                        {%- if eager_filter_evaluation -%}
                        if (f_instance->reached_subquery_count == f_instance->subquery_count)
                        {
                            auto filter_function = {{query_name}}_get_filter_function(f_instance->filter_segment_index, f_instance->filter_selector_index);
                            bool value = filter_function(f_instance->subqueries_results);
                            filters_results.try_emplace(f_instance->id, value);
                        }
                        {%- endif -%}
                        continue;
                    }
                    if (_is_scalar)
                        reached_subqueries_results.push_back(&f_instance->subqueries_results[i]);
                    else
                        f_instance->subqueries_results[i].type = COMPLEX;
                    continue;
                }
                f_instance->current_subqueries_segments[i] = const_cast<subquery_path_segment *>(subquery_segment->next);
            }
        }
        erase_if(filter_instances_ids, [](const int id) {
            auto f_instance = all_filter_instances[id];
            return f_instance->reached_subquery_count == f_instance->subquery_count;
        });
    }
{%- endmacro -%}

{%- macro generate_procedures_declarations(procedures) -%}
    {%- for procedure in procedures -%}
        {%- if procedure.are_any_filters -%}
            void {{procedure.query_name}}_{{procedure.name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t, selection_condition*>> &all_results, selection_condition *segment_conditions[], unordered_set<int> &filter_instances_ids, current_node_data &current_node);
        {%- else -%}
            void {{procedure.query_name}}_{{procedure.name|lower}}(ondemand::value &node, string *result_buf, vector<tuple<string *, size_t, size_t>> &all_results);
        {%- endif -%}
    {%- endfor -%}
{%- endmacro -%}