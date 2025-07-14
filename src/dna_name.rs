// DNA Name logic for Rust port of test_blendfile_dna.py
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnaName {
    pub name_full: String,
}

impl DnaName {
    pub fn new(name: &str) -> Self {
        Self { name_full: name.to_string() }
    }
    pub fn name_only(&self) -> String {
        let mut s = self.name_full.as_str();
        // Remove pointer and method pointer syntax
        if s.starts_with("(*") {
            s = &s[2..];
            if let Some(end) = s.find(")()") {
                s = &s[..end];
            }
        } else if s.starts_with('*') {
            s = &s[1..];
        }
        // Remove array brackets
        if let Some(idx) = s.find('[') {
            s = &s[..idx];
        }
        s.to_string()
    }
    pub fn is_pointer(&self) -> bool {
        self.name_full.starts_with('*') || self.name_full.starts_with("(*")
    }
    pub fn is_method_pointer(&self) -> bool {
        self.name_full.starts_with("(*")
    }
    pub fn array_size(&self) -> usize {
        let mut size = 1;
        let mut s = self.name_full.as_str();
        while let Some(start) = s.find('[') {
            if let Some(end) = s[start+1..].find(']') {
                let num = &s[start+1..start+1+end];
                if let Ok(n) = num.parse::<usize>() {
                    size *= n;
                }
                s = &s[start+1+end+1..];
            } else {
                break;
            }
        }
        size
    }
}
