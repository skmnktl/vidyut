/*!
Vedic text schema definition and management.

This module provides the schema system for defining how different vedic text
encodings map to Extended SLP1 format.
*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::vedic::{VedicResult, VedicError, ExtendedSlp1};
use crate::vedic::pattern_discovery::{PatternScope, PatternPosition};
use crate::{Mapping, Scheme};

/// Vedic text schema that defines encoding mappings.
#[derive(Debug, Clone)]
pub struct VedicSchema {
    pub id: String,
    pub config: SchemaConfig,
    pub validation_status: ValidationStatus,
    forward_mapping: HashMap<String, String>,
    reverse_mapping: HashMap<String, String>,
}

/// Schema configuration loaded from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    pub metadata: SchemaMetadata,
    pub source_encoding: SourceEncoding,
    pub accent_patterns: HashMap<String, AccentPattern>,
    pub punctuation_patterns: HashMap<String, PunctuationPattern>,
    pub editorial_patterns: HashMap<String, EditorialPattern>,
    pub transformations: TransformationRules,
    pub validation: ValidationConfig,
    #[serde(default)]
    pub extensions: HashMap<String, ExtensionConfig>,
}

/// Schema metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub extends: Vec<String>,
    pub description: String,
    pub author: String,
    pub created: String,
}

/// Source encoding specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEncoding {
    pub script: String,
    pub base_charset: String,
    pub case_sensitive: bool,
}

/// Accent pattern definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccentPattern {
    pub pattern: String,
    pub position: String, // "pre", "post", "combining"
    pub scope: String,    // "syllable", "word", "phrase"
    #[serde(default)]
    pub variant_of: Option<String>,
    #[serde(default)]
    pub maps_to: Option<String>,
}

/// Punctuation pattern definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunctuationPattern {
    pub pattern: String,
    pub maps_to: String,
}

/// Editorial pattern definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorialPattern {
    pub pattern: String,
    pub position: String,
    pub maps_to: String,
    #[serde(default)]
    pub regex: bool,
}

/// Transformation rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRules {
    #[serde(default)]
    pub remove_spaces_before_punct: bool,
    #[serde(default)]
    pub normalize_case: Option<String>,
    #[serde(default)]
    pub preserve_line_breaks: bool,
}

/// Validation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    #[serde(default = "default_true")]
    pub roundtrip_required: bool,
    #[serde(default = "default_corpus_sample_size")]
    pub corpus_sample_size: usize,
    #[serde(default = "default_manual_review_threshold")]
    pub manual_review_threshold: f64,
}

/// Extension configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    #[serde(default)]
    pub import: Vec<String>,
    #[serde(default)]
    pub override_patterns: Vec<String>,
}

/// Schema validation status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    Draft,
    Validated,
    Approved,
    Failed(String),
}

impl std::fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationStatus::Draft => write!(f, "draft"),
            ValidationStatus::Validated => write!(f, "validated"),
            ValidationStatus::Approved => write!(f, "approved"),
            ValidationStatus::Failed(msg) => write!(f, "failed: {}", msg),
        }
    }
}

impl std::str::FromStr for ValidationStatus {
    type Err = VedicError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(ValidationStatus::Draft),
            "validated" => Ok(ValidationStatus::Validated),
            "approved" => Ok(ValidationStatus::Approved),
            s if s.starts_with("failed:") => {
                Ok(ValidationStatus::Failed(s.strip_prefix("failed: ").unwrap_or("").to_string()))
            },
            _ => Err(VedicError::ConfigError(format!("Invalid validation status: {}", s))),
        }
    }
}

/// Schema manager for vedic text processing.
pub struct VedicSchemaManager {
    schemas: HashMap<String, VedicSchema>,
    base_schemas: HashMap<String, SchemaConfig>,
}

impl VedicSchema {
    /// Create new schema from configuration.
    pub fn from_config(id: String, config: SchemaConfig, status: ValidationStatus) -> VedicResult<Self> {
        let mut schema = Self {
            id,
            config,
            validation_status: status,
            forward_mapping: HashMap::new(),
            reverse_mapping: HashMap::new(),
        };
        
        schema.build_mappings()?;
        Ok(schema)
    }
    
    /// Build forward and reverse mappings from configuration.
    fn build_mappings(&mut self) -> VedicResult<()> {
        // Build accent pattern mappings
        for (name, pattern) in &self.config.accent_patterns {
            let target = pattern.maps_to.as_deref().unwrap_or_else(|| {
                // Default mappings based on pattern name
                match name.as_str() {
                    "udatta" => "'",
                    "anudatta" => "`", 
                    "svarita" => "^",
                    "pragrhya" => "~",
                    _ => "_1", // Custom variant
                }
            });
            
            self.forward_mapping.insert(pattern.pattern.clone(), target.to_string());
            self.reverse_mapping.insert(target.to_string(), pattern.pattern.clone());
        }
        
        // Build punctuation mappings
        for (_, pattern) in &self.config.punctuation_patterns {
            self.forward_mapping.insert(pattern.pattern.clone(), pattern.maps_to.clone());
            self.reverse_mapping.insert(pattern.maps_to.clone(), pattern.pattern.clone());
        }
        
        // Build editorial mappings
        for (_, pattern) in &self.config.editorial_patterns {
            if !pattern.regex {
                self.forward_mapping.insert(pattern.pattern.clone(), pattern.maps_to.clone());
                self.reverse_mapping.insert(pattern.maps_to.clone(), pattern.pattern.clone());
            }
        }
        
        Ok(())
    }
    
    /// Convert text to Extended SLP1 format.
    pub fn to_extended_slp1(&self, text: &str) -> VedicResult<ExtendedSlp1> {
        let mut result = text.to_string();
        
        // Apply transformations in order
        if self.config.transformations.normalize_case.as_deref() == Some("lower") {
            result = result.to_lowercase();
        }
        
        // Apply forward mappings
        for (source, target) in &self.forward_mapping {
            result = result.replace(source, target);
        }
        
        // Apply transformations
        if self.config.transformations.remove_spaces_before_punct {
            result = regex::Regex::new(r"\s+([|/])")
                .map_err(|e| VedicError::ConfigError(e.to_string()))?
                .replace_all(&result, "$1")
                .to_string();
        }
        
        let mut extended = ExtendedSlp1::new(result);
        extended.metadata_mut().source_schema = Some(self.id.clone());
        
        Ok(extended)
    }
    
    /// Convert from Extended SLP1 back to source format.
    pub fn from_extended_slp1(&self, extended: &ExtendedSlp1) -> VedicResult<String> {
        let mut result = extended.text().to_string();
        
        // Apply reverse mappings
        for (target, source) in &self.reverse_mapping {
            result = result.replace(target, source);
        }
        
        Ok(result)
    }
    
    /// Validate roundtrip conversion.
    pub fn validate_roundtrip(&self, text: &str) -> VedicResult<()> {
        let extended = self.to_extended_slp1(text)?;
        let reconstructed = self.from_extended_slp1(&extended)?;
        
        if text != reconstructed {
            return Err(VedicError::BijectivityError {
                original: text.to_string(),
                reconstructed,
                extended_form: extended.text().to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Get schema configuration as TOML string.
    pub fn to_toml(&self) -> VedicResult<String> {
        toml::to_string(&self.config)
            .map_err(|e| VedicError::ConfigError(e.to_string()))
    }
    
    /// Load schema from TOML string.
    pub fn from_toml(id: String, toml_str: &str) -> VedicResult<Self> {
        let config: SchemaConfig = toml::from_str(toml_str)
            .map_err(|e| VedicError::ConfigError(e.to_string()))?;
        
        Self::from_config(id, config, ValidationStatus::Draft)
    }
}

impl VedicSchemaManager {
    /// Create new schema manager.
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            base_schemas: HashMap::new(),
        }
    }
    
    /// Load base schemas for inheritance.
    pub fn load_base_schemas(&mut self, base_dir: &std::path::Path) -> VedicResult<()> {
        // TODO: Load base schemas from filesystem
        // For now, create some built-in base schemas
        self.load_builtin_base_schemas()
    }
    
    /// Load built-in base schemas.
    fn load_builtin_base_schemas(&mut self) -> VedicResult<()> {
        // Vedic core base schema
        let vedic_core = SchemaConfig {
            metadata: SchemaMetadata {
                name: "vedic-core".to_string(),
                version: "1.0.0".to_string(),
                extends: Vec::new(),
                description: "Common vedic accent and punctuation patterns".to_string(),
                author: "vidyut-lipi".to_string(),
                created: "2024-01-01".to_string(),
            },
            source_encoding: SourceEncoding {
                script: "generic".to_string(),
                base_charset: "unicode".to_string(),
                case_sensitive: true,
            },
            accent_patterns: [
                ("udatta".to_string(), AccentPattern {
                    pattern: "'".to_string(),
                    position: "post".to_string(),
                    scope: "syllable".to_string(),
                    variant_of: None,
                    maps_to: Some("'".to_string()),
                }),
                ("anudatta".to_string(), AccentPattern {
                    pattern: "`".to_string(),
                    position: "post".to_string(),
                    scope: "syllable".to_string(),
                    variant_of: None,
                    maps_to: Some("`".to_string()),
                }),
                ("svarita".to_string(), AccentPattern {
                    pattern: "^".to_string(),
                    position: "post".to_string(),
                    scope: "syllable".to_string(),
                    variant_of: None,
                    maps_to: Some("^".to_string()),
                }),
            ].into_iter().collect(),
            punctuation_patterns: [
                ("danda".to_string(), PunctuationPattern {
                    pattern: "|".to_string(),
                    maps_to: "|".to_string(),
                }),
                ("dvidanda".to_string(), PunctuationPattern {
                    pattern: "||".to_string(),
                    maps_to: "||".to_string(),
                }),
            ].into_iter().collect(),
            editorial_patterns: HashMap::new(),
            transformations: TransformationRules {
                remove_spaces_before_punct: false,
                normalize_case: None,
                preserve_line_breaks: true,
            },
            validation: ValidationConfig {
                roundtrip_required: true,
                corpus_sample_size: 1000,
                manual_review_threshold: 0.95,
            },
            extensions: HashMap::new(),
        };
        
        self.base_schemas.insert("vedic-core".to_string(), vedic_core);
        Ok(())
    }
    
    /// Create new schema with discovered patterns.
    pub fn create_schema(&mut self, id: &str, patterns: Vec<crate::vedic::DiscoveredPattern>) -> VedicResult<String> {
        // Convert discovered patterns to schema config
        let mut accent_patterns = HashMap::new();
        let mut punctuation_patterns = HashMap::new();
        
        for pattern in patterns {
            match pattern.scope {
                PatternScope::Syllable => {
                    accent_patterns.insert(
                        format!("pattern_{}", accent_patterns.len()),
                        AccentPattern {
                            pattern: pattern.pattern,
                            position: match pattern.position {
                                PatternPosition::Pre => "pre".to_string(),
                                PatternPosition::Post => "post".to_string(),
                                PatternPosition::Combining => "combining".to_string(),
                                _ => "post".to_string(),
                            },
                            scope: "syllable".to_string(),
                            variant_of: None,
                            maps_to: Some(pattern.suggested_mapping),
                        }
                    );
                },
                PatternScope::Structural => {
                    punctuation_patterns.insert(
                        format!("punct_{}", punctuation_patterns.len()),
                        PunctuationPattern {
                            pattern: pattern.pattern,
                            maps_to: pattern.suggested_mapping,
                        }
                    );
                },
                _ => {}, // TODO: Handle other scopes
            }
        }
        
        let config = SchemaConfig {
            metadata: SchemaMetadata {
                name: id.to_string(),
                version: "1.0.0".to_string(),
                extends: vec!["vedic-core".to_string()],
                description: format!("Auto-generated schema for {}", id),
                author: "pattern-discovery".to_string(),
                created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            },
            source_encoding: SourceEncoding {
                script: "unknown".to_string(),
                base_charset: "unicode".to_string(),
                case_sensitive: true,
            },
            accent_patterns,
            punctuation_patterns,
            editorial_patterns: HashMap::new(),
            transformations: TransformationRules {
                remove_spaces_before_punct: true,
                normalize_case: Some("lower".to_string()),
                preserve_line_breaks: true,
            },
            validation: ValidationConfig {
                roundtrip_required: true,
                corpus_sample_size: 1000,
                manual_review_threshold: 0.95,
            },
            extensions: HashMap::new(),
        };
        
        let schema = VedicSchema::from_config(id.to_string(), config, ValidationStatus::Draft)?;
        self.schemas.insert(id.to_string(), schema);
        
        Ok(id.to_string())
    }
    
    /// Get schema by ID.
    pub fn get_schema(&self, id: &str) -> VedicResult<&VedicSchema> {
        self.schemas.get(id)
            .ok_or_else(|| VedicError::SchemaNotFound(id.to_string()))
    }
    
    /// Save schema to manager.
    pub fn save_schema(&mut self, schema: VedicSchema) -> VedicResult<()> {
        let id = schema.id.clone();
        self.schemas.insert(id, schema);
        Ok(())
    }
    
    /// List all schema IDs.
    pub fn list_schemas(&self) -> Vec<&str> {
        self.schemas.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for VedicSchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for serde defaults
fn default_true() -> bool { true }
fn default_corpus_sample_size() -> usize { 1000 }
fn default_manual_review_threshold() -> f64 { 0.95 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let config = SchemaConfig {
            metadata: SchemaMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                extends: Vec::new(),
                description: "Test schema".to_string(),
                author: "test".to_string(),
                created: "2024-01-01".to_string(),
            },
            source_encoding: SourceEncoding {
                script: "test".to_string(),
                base_charset: "ascii".to_string(),
                case_sensitive: true,
            },
            accent_patterns: HashMap::new(),
            punctuation_patterns: HashMap::new(),
            editorial_patterns: HashMap::new(),
            transformations: TransformationRules {
                remove_spaces_before_punct: false,
                normalize_case: None,
                preserve_line_breaks: true,
            },
            validation: ValidationConfig {
                roundtrip_required: true,
                corpus_sample_size: 100,
                manual_review_threshold: 0.9,
            },
            extensions: HashMap::new(),
        };
        
        let schema = VedicSchema::from_config("test".to_string(), config, ValidationStatus::Draft);
        assert!(schema.is_ok());
    }

    #[test]
    fn test_roundtrip_validation() {
        let mut manager = VedicSchemaManager::new();
        manager.load_builtin_base_schemas().unwrap();
        
        // Create simple test schema
        let patterns = vec![];
        let schema_id = manager.create_schema("test", patterns).unwrap();
        let schema = manager.get_schema(&schema_id).unwrap();
        
        let text = "agnim";
        assert!(schema.validate_roundtrip(text).is_ok());
    }
}