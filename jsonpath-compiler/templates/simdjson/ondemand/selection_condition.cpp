{%- match condition -%}
    {%- when SelectionCondition::Filter with {id} -%}
        selection_condition::new_filter(f_instance_{{id.segment_index}}_{{id.selector_index}})
    {%- when SelectionCondition::RuntimeSegmentCondition with {segment_index} -%}
        segment_conditions[{{segment_index}}]
    {%- when SelectionCondition::And with {lhs, rhs} -%}
        {%- let lhs_template = SelectionConditionTemplate::new(lhs) -%}
        {%- let rhs_template = SelectionConditionTemplate::new(rhs) -%}
        selection_condition::new_and({{ lhs_template.render().unwrap() }}, {{ rhs_template.render().unwrap() }})
    {%- when SelectionCondition::Or with {lhs, rhs} -%}
        {%- let lhs_template = SelectionConditionTemplate::new(lhs) -%}
        {%- let rhs_template = SelectionConditionTemplate::new(rhs) -%}
        selection_condition::new_or({{ lhs_template.render().unwrap() }}, {{ rhs_template.render().unwrap() }})
{%- endmatch -%}
