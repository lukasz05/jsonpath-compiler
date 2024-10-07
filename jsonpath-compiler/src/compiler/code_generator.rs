pub struct CodeGenerator {
    indent_length: usize,
    code: String,
    current_indent_level: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            indent_length: 4,
            code: String::new(),
            current_indent_level: 0,
        }
    }

    pub fn get_code(&self) -> String {
        self.code.clone()
    }

    pub fn indent(&mut self) -> &Self {
        let indent_str = " ".repeat(self.indent_length * self.current_indent_level);
        self.code.push_str(&indent_str);
        self
    }

    pub fn write(&mut self, code: &str) -> &Self {
        self.code.push_str(code);
        self
    }

    pub fn write_line(&mut self, line: &str) -> &Self {
        self.indent();
        self.code.push_str(line);
        self.code.push('\n');
        self
    }

    pub fn write_lines(&mut self, lines: &[&str]) -> &Self {
        for line in lines {
            self.write_line(line);
        }
        self
    }

    pub fn write_extra_indented_line(&mut self, line: &str) -> &Self {
        self.current_indent_level += 1;
        self.write_line(line);
        self.current_indent_level -= 1;
        self
    }

    pub fn start_block(&mut self) -> &Self {
        self.write_line("{");
        self.current_indent_level += 1;
        self
    }

    pub fn end_block(&mut self) -> &Self {
        self.current_indent_level -= 1;
        self.write_line("}");
        self
    }
}
