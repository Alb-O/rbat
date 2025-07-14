// Rust port of test_blendfile_dna.py
// Tests for DNA name parsing, struct/field logic

#[cfg(test)]
mod tests {
    use blend_file_reader::dna_name::DnaName;

    #[test]
    fn test_simple_name() {
        let n = DnaName::new("Suzanne");
        assert_eq!(n.name_full, "Suzanne");
        assert_eq!(n.name_only(), "Suzanne");
        assert!(!n.is_pointer());
        assert!(!n.is_method_pointer());
        assert_eq!(n.array_size(), 1);
    }

    #[test]
    fn test_pointer() {
        let n = DnaName::new("*marker");
        assert_eq!(n.name_full, "*marker");
        assert_eq!(n.name_only(), "marker");
        assert!(n.is_pointer());
        assert!(!n.is_method_pointer());
        assert_eq!(n.array_size(), 1);
    }

    #[test]
    fn test_method_pointer() {
        let n = DnaName::new("(*delta_cache)()");
        assert_eq!(n.name_full, "(*delta_cache)()");
        assert_eq!(n.name_only(), "delta_cache");
        assert!(n.is_pointer());
        assert!(n.is_method_pointer());
        assert_eq!(n.array_size(), 1);
    }

    #[test]
    fn test_simple_array() {
        let n = DnaName::new("flame_smoke_color[3]");
        assert_eq!(n.name_full, "flame_smoke_color[3]");
        assert_eq!(n.name_only(), "flame_smoke_color");
        assert!(!n.is_pointer());
        assert!(!n.is_method_pointer());
        assert_eq!(n.array_size(), 3);
    }

    #[test]
    fn test_nested_array() {
        let n = DnaName::new("pattern_corners[4][2]");
        assert_eq!(n.name_full, "pattern_corners[4][2]");
        assert_eq!(n.name_only(), "pattern_corners");
        assert!(!n.is_pointer());
        assert!(!n.is_method_pointer());
        assert_eq!(n.array_size(), 8);
    }

    #[test]
    fn test_pointer_array() {
        let n = DnaName::new("*mtex[18]");
        assert_eq!(n.name_full, "*mtex[18]");
        assert_eq!(n.name_only(), "mtex");
        assert!(n.is_pointer());
        assert!(!n.is_method_pointer());
        assert_eq!(n.array_size(), 18);
    }
}
