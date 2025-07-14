// Rust port of test_blendfile_dna_io.py
// Tests for UTF-8 string writing/trimming for blendfile fields

#[cfg(test)]
mod tests {
    use blend_file_reader::dna_io::BigEndianTypes;

    #[test]
    fn test_trim_utf8() {
        let mut buf = Vec::new();
        let s = "බියර්";
        let max_len = 15;
        // Compute expected trimmed string at valid UTF-8 boundary
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
        let mut expect_bytes = trimmed.as_bytes().to_vec();
        expect_bytes.push(0);
        BigEndianTypes::write_string(&mut buf, s, max_len);
        assert_eq!(buf, expect_bytes);
    }

    #[test]
    fn test_utf8() {
        let mut buf = Vec::new();
        let s = "බියර්";
        BigEndianTypes::write_string(&mut buf, s, 16);
        let mut expect_bytes = s.as_bytes().to_vec();
        expect_bytes.push(0);
        assert_eq!(buf, expect_bytes);
    }
}
