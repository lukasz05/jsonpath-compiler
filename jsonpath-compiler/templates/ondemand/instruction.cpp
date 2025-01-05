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
            {{query_name}}_{{name|lower}}({{current_node}}, results_in_progress, all_results);
        {% else %}
            {{name|lower}}({{current_node}}, results_in_progress, all_results);
        {% endif %}
    {% when Instruction::SaveCurrentNodeDuringTraversal with { instruction } %}
        string* buf_ptr = new string();
        all_results.push_back(buf_ptr);
        results_in_progress.push_back(buf_ptr);
        {% let template = InstructionTemplate::new(instruction, current_node, query_name) %}
        {{ template.render().unwrap() }}
        results_in_progress.pop_back();
    {% when Instruction::Continue %}
        continue;
    {% when Instruction::TraverseCurrentNodeSubtree %}
        traverse_and_save_selected_nodes({{current_node}}, results_in_progress);
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
            add_to_all_bufs(results_in_progress, string_view("{"));
            bool first = true;
            for (ondemand::field field : object)
            {
                string_view key = field.unescaped_key();
                for (const auto &buf_ptr : results_in_progress)
                {
                    if (!first) *buf_ptr += ", ";
                    *buf_ptr += "\"";
                    *buf_ptr += key;
                    *buf_ptr += "\":";
                }
                first = false;
                {% call compile_instructions(instructions, "field.value()") %}
            }
            add_to_all_bufs(results_in_progress, string_view("}"));
        }
    {% endif %}
{% endmacro %}

{% macro compile_array_iteration(loop_instruction) %}
    {% if let ForEachElement { instructions } = loop_instruction %}
        ondemand::array array;
        if (!node.get_array().get(array))
        {
            add_to_all_bufs(results_in_progress, string_view("["));
            bool first = true;
            size_t index = 0;
            {% if self::is_array_length_needed(instructions) %}
            size_t array_length = array.count_elements();
            {% endif %}
            for (ondemand::value element : array)
            {
                if (!first)
                {
                    index++;
                    add_to_all_bufs(results_in_progress, string_view(", "));
                }
                first = false;
                {% call compile_instructions(instructions, "element") %}
            }
            add_to_all_bufs(results_in_progress, string_view("]"));
        }
    {% endif %}
{% endmacro %}