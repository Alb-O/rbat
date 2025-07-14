// DNA IO logic for UTF-8 string writing/trimming
pub struct BigEndianTypes;

impl BigEndianTypes {
    pub fn write_string(buf: &mut Vec<u8>, s: &str, max_len: usize) {
        let mut end = 0;
        let mut total = 0;
        for (i, c) in s.char_indices() {
            let char_len = c.len_utf8();
            if total + char_len > max_len - 1 {
                break;
            }
            end = i + char_len;
            total += char_len;
        }
        let trimmed = &s[..end];
        buf.extend_from_slice(trimmed.as_bytes());
        buf.push(0); // Null terminator
    }
}
