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
    {% when Instruction::ExecuteProcedureOnChild with { name } %}
        {% if !query_name.is_empty() %}
            {{query_name}}_{{name|lower}}({{current_node}}, result_buf, all_results);
        {% else %}
            {{name|lower}}({{current_node}}, result_buf, all_results);
        {% endif %}
    {% when Instruction::SaveCurrentNodeDuringTraversal with { instruction } %}
        if (result_buf == nullptr)
            result_buf = new string();
        size_t result_i = all_results.size();
        all_results.emplace_back(result_buf, result_buf->size(), 0);
        {% let template = InstructionTemplate::new(instruction, current_node, query_name) %}
        {{ template.render().unwrap() }}
        get<2>(all_results[result_i]) = result_buf->size();
    {% when Instruction::Continue %}
        continue;
    {% when Instruction::TraverseCurrentNodeSubtree %}
        traverse_and_save_selected_nodes({{current_node}}, result_buf);
{% endmatch %}

{% macro compile_instructions(instructions, current_node) %}
    {% for instruction in instructions %}
    {% let template = InstructionTemplate::new(instruction, current_node, query_name) %}
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
            {% if crate::compiler::simdjson::is_array_length_needed(instructions) %}
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