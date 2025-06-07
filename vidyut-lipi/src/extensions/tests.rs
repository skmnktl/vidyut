#[cfg(test)]
mod tests {
    use crate::extensions::runtime_extension::*;
    use rustc_hash::FxHashMap;

    #[test]
    fn test_runtime_extension_creation() {
        let mut mappings = FxHashMap::default();
        mappings.insert("X".to_string(), "{X}".to_string());
        
        let ext = RuntimeExtension::simple("test", mappings);
        assert_eq!(ext.name, "test");
        assert_eq!(ext.mappings.get("X"), Some(&"{X}".to_string()));
    }

    #[test]
    fn test_extension_metadata() {
        let metadata = ExtensionMetadata {
            category: Some("test".to_string()),
            subcategory: None,
            unicode_range: Some("U+E300-E3FF".to_string()),
            source_description: Some("Test extension".to_string()),
        };
        
        assert_eq!(metadata.category, Some("test".to_string()));
        assert_eq!(metadata.unicode_range, Some("U+E300-E3FF".to_string()));
    }

    #[test]
    fn test_discover_unmapped_characters() {
        use crate::scheme::Scheme;
        
        let text = "agniX"; // Contains unmapped character X
        let unmapped = discover_unmapped_characters(text, Scheme::HarvardKyoto);
        
        // X should be unmapped in HarvardKyoto
        assert!(unmapped.contains(&'X'));
    }
}