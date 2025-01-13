extern "C" {
    fn free_result_buffer(result_buf: *mut u8);
{% for query_name in query_names %}
    fn {{query_name}}_binding(padded_input: *const u8, input_length: usize, result_length: *mut usize) -> *mut u8;
{% endfor %}
}

{% for query_name in query_names %}
pub fn {{query_name}}(padded_input: &[u8]) -> String {
    let input_ptr = padded_input.as_ptr();
    let mut result_length: usize = 0;
    unsafe {
        let result_ptr: *mut u8 = {{query_name}}_binding(input_ptr, padded_input.len(), &mut result_length);
        let result_slice = std::slice::from_raw_parts(result_ptr, result_length);
        let result_str = String::from_utf8_unchecked(result_slice.to_vec());
        free_result_buffer(result_ptr);
        result_str
    }
}
{% endfor %}