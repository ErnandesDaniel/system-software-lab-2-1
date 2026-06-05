use crate::codegen::nasm::AsmGenerator;

impl AsmGenerator {
    pub(crate) fn emit_string_data(&mut self, label: &str, s: &str) {
        let bytes = Self::escape_string(s);
        self.data_section.push_str(&format!("{label} db "));

        if bytes.is_empty() {
            self.data_section.push('0');
        } else {
            for (i, b) in bytes.iter().enumerate() {
                if i > 0 {
                    self.data_section.push_str(", ");
                }
                self.data_section.push_str(&format!("{b}"));
            }
        }
        self.data_section.push_str(", 0\n");
    }

    fn escape_string(s: &str) -> Vec<u8> {
        let mut result = Vec::new();
        for c in s.chars() {
            match c {
                '\n' => {
                    result.push(10);
                }
                '\r' => {
                    result.push(13);
                }
                '\t' => {
                    result.push(9);
                }
                '\\' => {
                    result.push(92);
                }
                '"' => {
                    result.push(34);
                }
                _ => {
                    let mut buf = [0u8; 4];
                    let encoded = c.encode_utf8(&mut buf);
                    result.extend_from_slice(encoded.as_bytes());
                }
            }
        }
        result
    }
}
