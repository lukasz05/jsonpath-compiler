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
            {{query_name}}_{{name|lower}}({{current_node}}, all_results);
        {% else %}
            {{name|lower}}({{current_node}}, all_results);
        {% endif %}
    {% when Instruction::SaveCurrentNodeDuringTraversal with { instruction, condition} %}
        all_results.push_back(simdjson::to_string({{current_node}}));
        {% let template = InstructionTemplate::new(instruction, current_node, query_name) %}
        {{ template.render().unwrap() }}
    {% when Instruction::Continue %}
        continue;
    {% when Instruction::TraverseCurrentNodeSubtree %}
    {% when Instruction::RegisterSubqueryPath { subquery_path }%}
    {% when Instruction::TryUpdateSubqueries %}
{% endmatch %}

{% macro compile_instructions(instructions, current_node) %}
    {% for instruction in instructions %}
    {% let template = InstructionTemplate::new(instruction, current_node, query_name) %}
        {{ template.render().unwrap() }}
    {% endfor %}
{% endmacro %}

{% macro compile_object_iteration(loop_instruction) %}
    {% if let ForEachMember { instructions } = loop_instruction %}
        dom::object object;
        if (!node.get_object().get(object))
        {
            bool first = true;
            for (dom::key_value_pair field : object)
            {
                string_view key = field.key;
                {% call compile_instructions(instructions, "field.value") %}
            }
        }
    {% endif %}
{% endmacro %}

{% macro compile_array_iteration(loop_instruction) %}
    {% if let ForEachElement { instructions } = loop_instruction %}
        dom::array array;
        if (!node.get_array().get(array))
        {
            bool first = true;
            size_t index = 0;
            {% if crate::compiler::simdjson::is_array_length_needed(instructions) %}
            size_t array_length = array.size();
            {% endif %}
            for (dom::element element : array)
            {
                if (!first)
                    index++;
                first = false;
                {% call compile_instructions(instructions, "element") %}
            }
        }
    {% endif %}
{% endmacro %}