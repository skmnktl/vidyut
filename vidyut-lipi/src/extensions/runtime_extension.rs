/// Runtime extension for transliteration schemes.
///
/// This module allows extending any base transliteration scheme with
/// additional character mappings at runtime, without modifying the core library.

use crate::mapping::{Mapping, Span, SpanKind};
use crate::scheme::Scheme;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// A runtime extension that can be applied to any base scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExtension {
    /// Name of this extension
    pub name: String,
    /// Version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Character mappings from source to extended representation
    pub mappings: FxHashMap<String, String>,
    /// Optional metadata
    pub metadata: ExtensionMetadata,
    /// Fallback mappings when target doesn't support extensions
    pub fallback_mappings: Option<FxHashMap<String, String>>,
}

/// Metadata about the extension
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionMetadata {
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub unicode_range: Option<String>,
    pub source_description: Option<String>,
}

/// An extended scheme created by applying extensions to a base scheme
#[derive(Debug, Clone)]
pub struct ExtendedScheme {
    base_scheme: Scheme,
    extensions: Vec<RuntimeExtension>,
    // Cached merged mappings
    forward_mappings: FxHashMap<String, String>,
    reverse_mappings: FxHashMap<String, String>,
}

impl RuntimeExtension {
    /// Load extension from YAML file
    pub fn from_yaml(yaml_str: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml_str)
    }
    
    /// Create a simple extension with just mappings
    pub fn simple(name: &str, mappings: FxHashMap<String, String>) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: format!("Runtime extension: {}", name),
            mappings,
            metadata: ExtensionMetadata::default(),
            fallback_mappings: None,
        }
    }
}

impl ExtendedScheme {
    /// Create an extended scheme by applying extensions to a base scheme
    pub fn new(base_scheme: Scheme, extensions: Vec<RuntimeExtension>) -> Self {
        let mut forward_mappings = FxHashMap::default();
        let mut reverse_mappings = FxHashMap::default();
        
        // Apply extensions in order
        for ext in &extensions {
            for (from, to) in &ext.mappings {
                forward_mappings.insert(from.clone(), to.clone());
                reverse_mappings.insert(to.clone(), from.clone());
            }
        }
        
        Self {
            base_scheme,
            extensions,
            forward_mappings,
            reverse_mappings,
        }
    }
    
    /// Get the extended mapping for a character/string
    pub fn map_forward(&self, input: &str) -> Option<&str> {
        self.forward_mappings.get(input).map(|s| s.as_str())
    }
    
    /// Get the reverse mapping for an extended representation
    pub fn map_reverse(&self, input: &str) -> Option<&str> {
        self.reverse_mappings.get(input).map(|s| s.as_str())
    }
    
    /// Check if a character is handled by extensions
    pub fn is_extended_char(&self, ch: char) -> bool {
        let ch_str = ch.to_string();
        self.forward_mappings.contains_key(&ch_str)
    }
    
    /// Apply this extended scheme to enhance a base mapping
    pub fn enhance_mapping(&self, base_mapping: &mut Mapping) {
        // Add extension mappings to the base mapping
        for (from, to) in &self.forward_mappings {
            // Determine the appropriate SpanKind based on the mapping
            let kind = if to.contains('{') && to.contains('}') {
                SpanKind::Accent
            } else {
                SpanKind::Other
            };
            
            base_mapping.all.insert(
                from.clone(),
                Span::new(from.clone(), to.clone(), kind)
            );
        }
    }
}

/// Discover unmapped characters in text compared to a base scheme
pub fn discover_unmapped_characters(text: &str, base_scheme: Scheme) -> Vec<char> {
    let mut unmapped = Vec::new();
    let dummy_mapping = Mapping::new(base_scheme, base_scheme);
    
    for ch in text.chars() {
        let ch_str = ch.to_string();
        if dummy_mapping.get(&ch_str).is_none() && !ch.is_whitespace() {
            unmapped.push(ch);
        }
    }
    
    unmapped.sort_unstable();
    unmapped.dedup();
    unmapped
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_extension() {
        let mut mappings = FxHashMap::default();
        mappings.insert("\u{E311}".to_string(), "{U1}".to_string());
        mappings.insert("\u{E312}".to_string(), "{U2}".to_string());
        
        let ext = RuntimeExtension::simple("test_pua", mappings);
        assert_eq!(ext.name, "test_pua");
        assert_eq!(ext.mappings.get("\u{E311}"), Some(&"{U1}".to_string()));
    }
    
    #[test]
    fn test_extended_scheme() {
        let mut mappings = FxHashMap::default();
        mappings.insert("X".to_string(), "{X}".to_string());
        
        let ext = RuntimeExtension::simple("custom", mappings);
        let extended = ExtendedScheme::new(Scheme::Devanagari, vec![ext]);
        
        assert_eq!(extended.map_forward("X"), Some("{X}"));
        assert_eq!(extended.map_reverse("{X}"), Some("X"));
        assert!(extended.is_extended_char('X'));
        assert!(!extended.is_extended_char('A'));
    }
}