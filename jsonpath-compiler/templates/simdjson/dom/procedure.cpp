void {{name|lower}}(dom::element &node, vector<string> &all_results)
{
    {%- for instruction in instructions -%}
        {{ instruction.render().unwrap() }}
    {%- endfor -%}
    {%- if !Self::are_object_members_iterated(self) -%}
        {{ EMPTY_OBJECT_ITERATION.render().unwrap() }}
    {%- endif -%}
    {%- if !Self::are_array_elements_iterated(self) -%}
        {{ EMPTY_ARRAY_ITERATION.render().unwrap() }}
    {%- endif -%}
}