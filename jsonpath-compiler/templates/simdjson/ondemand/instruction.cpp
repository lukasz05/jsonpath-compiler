{%- import "macros.cpp" as common -%}

{%- match instruction -%}
    {%- when Instruction::ForEachElement with { instructions } -%}
        {%- call compile_array_iteration(instruction) -%}
    {%- when Instruction::ForEachMember with { instructions } -%}
        {%- call compile_object_iteration(instruction) -%}
    {%- when Instruction::IfCurrentIndexEquals with { index, instructions } -%}
        if (index == {{index}})
        {
            {%- call compile_instructions(instructions, current_node) -%}
        }
    {%- when Instruction::IfCurrentIndexFromEndEquals with { index, instructions } -%}
        if (array_length - index == {{index}})
        {
            {%- call compile_instructions(instructions, current_node) -%}
        }
    {%- when Instruction::IfCurrentMemberNameEquals with { name, instructions } -%}
        if (key == "{{ rsonpath_syntax::str::escape(name, rsonpath_syntax::str::EscapeMode::DoubleQuoted) }}")
        {
            {%- call compile_instructions(instructions, current_node) -%}
        }
    {%- when Instruction::IfActiveFilterInstance with { instructions } -%}
        {%- if are_any_filters -%}
            if (!filter_instances_ids.empty())
            {
                {%- call compile_instructions(instructions, current_node) -%}
            }
        {%- endif -%}
    {%- when Instruction::ExecuteProcedureOnChild with { conditions, segments, name } -%}
        {%- if are_any_filters -%}
            selection_condition* new_segment_conditions[{{query_name}}_SEGMENT_COUNT] = {};
            {%- if eager_filter_evaluation -%}
                bool all_segment_conditions_always_false = true;
            {%- endif -%}
            {%- for (i, condition) in conditions.iter().enumerate() -%}
                {%- if let Some(condition) = condition -%}
                    {%- let template = SelectionConditionTemplate::new(condition) -%}
                    new_segment_conditions[{{i}}] = {{ template.render().unwrap() }};
                    {%- if i > 0 -%}
                        if (segment_conditions[{{i-1}}] != nullptr)
                            new_segment_conditions[{{i}}] = selection_condition::new_and(segment_conditions[{{i-1}}], new_segment_conditions[{{i}}]);
                    {%- endif -%}
                {%- else -%}
                    if (segment_conditions[{{i}}] != nullptr)
                        new_segment_conditions[{{i}}] = segment_conditions[{{i}}];
                    {%- if i > 0 -%}
                        else if (segment_conditions[{{i-1}}] != nullptr)
                            new_segment_conditions[{{i}}] = segment_conditions[{{i-1}}];
                    {%- endif -%}
                {%- endif -%}
                {%- if eager_filter_evaluation && segments.contains(i) -%}
                    if ({{query_name}}_try_evaluate_selection_condition(new_segment_conditions[{{i}}], segment_condition_value))
                    {
                        if (segment_condition_value)
                        {
                            new_segment_conditions[{{i}}] = nullptr;
                            all_segment_conditions_always_false = false;
                        }
                        else
                            new_segment_conditions[{{i}}] = &always_false_condition;
                    }
                    else
                        all_segment_conditions_always_false = false;
                {%- endif -%}
            {%- endfor -%}
            {%- if eager_filter_evaluation -%}
                if (!all_segment_conditions_always_false)
                {
            {%- endif -%}
            {%- if current_node == "field.value()" -%}
                current_node.is_member = true;
                current_node.is_element = false;
                current_node.key = key;
            {%- else if current_node == "element" -%}
                current_node.is_member = false;
                current_node.is_element = true;
                current_node.array_length = array_length;
                current_node.index = index;
            {%- else -%}
                current_node.is_member = false
                current_node.is_element = false;
            {%- endif -%}
            {{query_name}}_{{name|lower}}({{current_node}}, result_buf, all_results, new_segment_conditions, filter_instances_ids, current_node);
            {%- if eager_filter_evaluation -%}
                }
            {%- endif -%}
        {%- else -%}
            {{query_name}}_{{name|lower}}({{current_node}}, result_buf, all_results);
        {%- endif -%}
    {%- when Instruction::SaveCurrentNodeDuringTraversal with { condition, instruction } -%}
        if (result_buf == nullptr)
            result_buf = new string();
        size_t result_i = all_results.size();
        size_t buf_start_pos = result_buf->size();
        {%- if are_any_filters -%}
            {%- if let Some(condition) = condition -%}
                {%- let template = SelectionConditionTemplate::new(condition) -%}
                {%- if eager_filter_evaluation -%}
                    auto condition = {{ template.render().unwrap() }};
                    bool condition_value;
                    if ({{query_name}}_try_evaluate_selection_condition(condition, condition_value))
                    {
                        if (condition_value)
                        {
                            {% if eager_filter_evaluation %}
                                result_in_progress_conditions.push_back(&always_true_condition);
                            {% endif %}
                            all_results.emplace_back(result_buf, result_buf->size(), 0, nullptr);
                        }
                        else
                            result_in_progress_conditions.push_back(&always_false_condition);
                    }
                    else
                    {
                        {% if eager_filter_evaluation %}
                            result_in_progress_conditions.push_back(condition);
                        {% endif %}
                        all_results.emplace_back(result_buf, result_buf->size(), 0, condition);
                    }
                {%- else -%}
                    all_results.emplace_back(result_buf, result_buf->size(), 0, {{ template.render().unwrap() }});
                {%- endif -%}
            {%- else -%}
                {% if eager_filter_evaluation %}
                    result_in_progress_conditions.push_back(&always_true_condition);
                {% endif %}
                all_results.emplace_back(result_buf, result_buf->size(), 0, nullptr);
            {%- endif -%}
        {%- else -%}
            all_results.emplace_back(result_buf, result_buf->size(), 0);
        {%- endif -%}
        {%- let template = InstructionTemplate::new(instruction, current_node, query_name, filter_subqueries.to_owned(), are_any_filters.clone(), eager_filter_evaluation.clone()) -%}
        {{ template.render().unwrap() }}
        {% if are_any_filters && eager_filter_evaluation %}
            if (!{{query_name}}_try_evaluate_selection_condition(condition, condition_value) || condition_value) {
                get<0>(all_results[result_i]) = new string(*result_buf, buf_start_pos);
                get<1>(all_results[result_i]) = 0;
                get<2>(all_results[result_i]) = result_buf->size() - buf_start_pos;
            }
            result_in_progress_conditions.pop_back();
            if (result_in_progress_conditions.empty())
                result_buf->clear();
        {% else %}
            get<2>(all_results[result_i]) = result_buf->size();
        {% endif %}
    {%- when Instruction::Continue -%}
        continue;
    {%- when Instruction::TraverseCurrentNodeSubtree -%}
        {%- if are_any_filters -%}
            {%- if current_node == "field.value()" -%}
                current_node.is_member = true;
                current_node.is_element = false;
                current_node.key = key;
            {%- else if current_node == "element" -%}
                current_node.is_member = false;
                current_node.is_element = true;
                current_node.array_length = array_length;
                current_node.index = index;
            {%- else -%}
                current_node.is_member = false
                current_node.is_element = false;
            {%- endif -%}
                {{query_name}}_traverse_and_save_selected_nodes({{current_node}}, result_buf, filter_instances_ids, current_node);
        {%- else -%}
            {{query_name}}_traverse_and_save_selected_nodes({{current_node}}, result_buf);
        {%- endif -%}
    {%- when Instruction::StartFilterExecution with { filter_id} -%}
        {%- call common::compile_start_filter_execution(filter_id, query_name) -%}
    {%- when Instruction::EndFiltersExecution -%}
        for (int filter_id = first_added_filter_id; filter_id < first_added_filter_id + added_filter_instances; filter_id++)
        {
            {%- if eager_filter_evaluation -%}
                if (filter_instances_ids.erase(filter_id) == 1)
                {
                    auto f_instance = all_filter_instances[filter_id];
                    auto filter_function = {{query_name}}_get_filter_function(f_instance->filter_segment_index, f_instance->filter_selector_index);
                    bool value = filter_function(f_instance->subqueries_results);
                    filters_results.try_emplace(f_instance->id, value);
                }
            {%- else -%}
                filter_instances_ids.erase(filter_id);
            {%- endif -%}
        }
        added_filter_instances = 0;
    {%- when Instruction::UpdateSubqueriesState -%}
        {%- call common::compile_update_subqueries_state(query_name, eager_filter_evaluation) -%}
{%- endmatch -%}

{%- macro compile_instructions(instructions, current_node) -%}
    {%- for instruction in instructions -%}
        {%- let template = InstructionTemplate::new(instruction, current_node, query_name, filter_subqueries.to_owned(), are_any_filters.clone(), eager_filter_evaluation.clone()) -%}
        {{ template.render().unwrap() }}
    {%- endfor -%}
{%- endmacro -%}

{%- macro compile_object_iteration(loop_instruction) -%}
    {%- if let ForEachMember { instructions } = loop_instruction -%}
        ondemand::object object;

        if (!node.get_object().get(object))
        {
            bool is_result_saving_in_progress = result_buf != nullptr;
            {% if are_any_filters && eager_filter_evaluation %}
                is_result_saving_in_progress &= {{query_name}}_check_result_in_progress_conditions();
            {% endif %}
            if (is_result_saving_in_progress)
                *result_buf += "{";
            bool first = true;
            for (ondemand::field field : object)
            {
                {%- if are_any_filters -%}
                    added_filter_instances = 0;
                    first_added_filter_id = all_filter_instances.size();
                {%- endif -%}
                string_view key = field.unescaped_key();
                if (is_result_saving_in_progress)
                {
                    if (!first)
                        *result_buf += ", ";
                    *result_buf += "\"";
                    *result_buf += key;
                    *result_buf += "\":";
                }
                first = false;
                {%- call compile_instructions(instructions, "field.value()") -%}
            }
            if (is_result_saving_in_progress)
                *result_buf += "}";
        }
    {%- endif -%}
{%- endmacro -%}

{%- macro compile_array_iteration(loop_instruction) -%}
    {%- if let ForEachElement { instructions } = loop_instruction -%}
        ondemand::array array;
        if (!node.get_array().get(array))
        {
            bool is_result_saving_in_progress = result_buf != nullptr;
            {% if are_any_filters && eager_filter_evaluation %}
                is_result_saving_in_progress &= {{query_name}}_check_result_in_progress_conditions();
            {% endif %}
            if (is_result_saving_in_progress)
                *result_buf += "[";
            bool first = true;
            size_t index = 0;
            {%- if crate::targets::simdjson::is_array_length_needed(instructions) -%}
                size_t array_length = array.count_elements();
            {%- else if are_any_filters -%}
                size_t array_length = 0;
                for (int filter_instance_id : filter_instances_ids)
                {
                    if (all_filter_instances[filter_instance_id]->is_any_current_subquery_negative_index())
                    {
                        array_length = array.count_elements();
                        break;
                    }
                }
            {%- endif -%}
            for (ondemand::value element : array)
            {
                {%- if are_any_filters -%}
                    added_filter_instances = 0;
                    first_added_filter_id = all_filter_instances.size();
                {%- endif -%}
                if (!first)
                {
                    index++;
                    if (is_result_saving_in_progress)
                        *result_buf += ", ";
                }
                first = false;
                {%- call compile_instructions(instructions, "element") -%}
            }
            if (is_result_saving_in_progress)
                *result_buf += "]";
        }
    {%- endif -%}
{%- endmacro -%}
