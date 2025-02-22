{%- import "macros.cpp" as common -%}

{% match instruction %}
    {% when Instruction::ForEachElement with { instructions } %}
        {% call compile_array_iteration(instruction) %}
    {% when Instruction::ForEachMember with { instructions } %}
        {% call compile_object_iteration(instruction) %}
    {% when Instruction::IfCurrentIndexEquals with { index, instructions } %}
        if (index == {{index}})
        {
            {% call compile_instructions(instructions, current_node) %}
        }
    {% when Instruction::IfCurrentIndexFromEndEquals with { index, instructions } %}
        if (array_length - index == {{index}})
        {
            {% call compile_instructions(instructions, current_node) %}
        }
    {% when Instruction::IfCurrentMemberNameEquals with { name, instructions } %}
        if (key == "{{ rsonpath_syntax::str::escape(name, rsonpath_syntax::str::EscapeMode::DoubleQuoted) }}")
        {
            {% call compile_instructions(instructions, current_node) %}
        }
    {% when Instruction::ExecuteProcedureOnChild with { conditions, name } %}
        {% if are_any_filters %}
            selection_condition* new_segment_conditions[{{query_name}}_SEGMENT_COUNT] = {};
            {% for (i, condition) in conditions.iter().enumerate() %}
                {% if let Some(condition) = condition %}
                    {% let template = SelectionConditionTemplate::new(condition) %}
                    new_segment_conditions[{{i}}] = segment_conditions[{{i}}] == nullptr
                        ? {{ template.render().unwrap() }}
                        : selection_condition::new_and(segment_conditions[{{i}}], {{ template.render().unwrap() }});
                {% else %}
                    new_segment_conditions[{{i}}] = segment_conditions[{{i}}];
                {% endif %}
            {% endfor %}
            {{query_name}}_{{name|lower}}({{current_node}}, result_buf, all_results,
            new_segment_conditions, filter_instances,
            {% if current_node == "field.value()" %}
            { true, false, 0, 0, key});
            {% else if current_node == "element" %}
            { false, true, array_length, index, {}});
            {% else %}
            { false, false, 0, 0, {}});
            {% endif %}
        {% else %}
            {{query_name}}_{{name|lower}}({{current_node}}, result_buf, all_results);
        {% endif %}
    {% when Instruction::SaveCurrentNodeDuringTraversal with { condition, instruction } %}
        if (result_buf == nullptr)
            result_buf = new string();
        size_t result_i = all_results.size();
        {% if are_any_filters %}
            {% if let Some(condition) = condition %}
            {% let template = SelectionConditionTemplate::new(condition) %}
            all_results.emplace_back(result_buf, result_buf->size(), 0, {{ template.render().unwrap() }});
            {% else %}
            all_results.emplace_back(result_buf, result_buf->size(), 0, nullptr);
            {% endif %}
        {% else %}
        all_results.emplace_back(result_buf, result_buf->size(), 0);
        {% endif %}
        {% let template = InstructionTemplate::new(instruction, current_node, query_name, filter_subqueries.to_owned(), are_any_filters.clone()) %}
        {{ template.render().unwrap() }}
        get<2>(all_results[result_i]) = result_buf->size();
    {% when Instruction::Continue %}
        continue;
    {% when Instruction::TraverseCurrentNodeSubtree %}
        {% if are_any_filters %}
        traverse_and_save_selected_nodes({{current_node}}, result_buf, filter_instances, reached_subqueries_results,
        {% if current_node == "field.value()" %}
        { true, false, 0, 0, key});
        {% else if current_node == "element" %}
        { false, true, array_length, index, {}});
        {% else %}
        { false, false, 0, 0, {}});
        {% endif %}
        {% else %}
        traverse_and_save_selected_nodes({{current_node}}, result_buf);
        {% endif %}
    {% when Instruction::StartFilterExecution with { filter_id} %}
        {% call common::compile_start_filter_execution(filter_id, query_name) %}
    {% when Instruction::EndFilterExecution %}
        filter_instances.pop_back();
    {% when Instruction::UpdateSubqueriesState %}
        vector<subquery_result*> reached_subqueries_results;
        {% call common::compile_update_subqueries_state() %}
{% endmatch %}

{% macro compile_instructions(instructions, current_node) %}
    {% for instruction in instructions %}
    {% let template = InstructionTemplate::new(instruction, current_node, query_name, filter_subqueries.to_owned(), are_any_filters.clone()) %}
        {{ template.render().unwrap() }}
    {% endfor %}
{% endmacro %}

{% macro compile_object_iteration(loop_instruction) %}
    {% if let ForEachMember { instructions } = loop_instruction %}
        ondemand::object object;
        if (!node.get_object().get(object))
        {
            if (result_buf != nullptr)
                *result_buf += "{";
            bool first = true;
            for (ondemand::field field : object)
            {
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
                {% call compile_instructions(instructions, "field.value()") %}
            }
            if (result_buf != nullptr)
                *result_buf += "}";
        }
    {% endif %}
{% endmacro %}

{% macro compile_array_iteration(loop_instruction) %}
    {% if let ForEachElement { instructions } = loop_instruction %}
        ondemand::array array;
        if (!node.get_array().get(array))
        {
            if (result_buf != nullptr)
                *result_buf += "[";
            bool first = true;
            size_t index = 0;
            {% if are_any_filters || crate::targets::simdjson::is_array_length_needed(instructions) %}
            size_t array_length = array.count_elements();
            {% endif %}
            for (ondemand::value element : array)
            {
                if (!first)
                {
                    index++;
                    if (result_buf != nullptr)
                        *result_buf += ", ";
                }
                first = false;
                {% call compile_instructions(instructions, "element") %}
            }
            if (result_buf != nullptr)
                *result_buf += "]";
        }
    {% endif %}
{% endmacro %}
