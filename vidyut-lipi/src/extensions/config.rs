/// Configuration system for runtime extensible mappings.
///
/// This module provides YAML-based configuration for defining:
/// - Detection patterns for unknown accents
/// - Mapping strategies for discovered patterns  
/// - Round-trip validation settings
/// - Source and destination scheme extensions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use super::runtime_extensible::{
    RuntimeExtensionConfig, DetectionPattern, PatternType, TargetMapping,
    UnknownPatternStrategy, RoundTripValidation, ValidationTolerance, ValidationFailureAction
};

/// Complete configuration for vidyut-lipi extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VidyutExtensionConfig {
    /// Version of the configuration format
    pub version: String,
    
    /// Available scheme extensions
    pub scheme_extensions: HashMap<String, SchemeExtensionConfig>,
    
    /// Global detection patterns
    pub global_patterns: Vec<DetectionPattern>,
    
    /// Default settings
    pub defaults: DefaultConfig,
}

/// Configuration for extending a specific scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeExtensionConfig {
    /// Base scheme to extend
    pub base_scheme: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Source-specific patterns (e.g., for specific udapaana sources)
    pub sources: HashMap<String, SourceConfig>,
    
    /// Global patterns for this scheme
    pub patterns: Vec<DetectionPattern>,
    
    /// How to handle unknown patterns
    pub unknown_strategy: UnknownPatternStrategy,
    
    /// Validation settings
    pub validation: RoundTripValidation,
}

/// Configuration for a specific source (e.g., vedanidhi, vedavms)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Source identifier
    pub source_id: String,
    
    /// Source-specific detection patterns
    pub patterns: Vec<DetectionPattern>,
    
    /// Known accent mappings for this source
    pub known_mappings: HashMap<String, String>,
    
    /// Custom processing rules
    pub processing_rules: Vec<ProcessingRule>,
}

/// Processing rule for handling specific patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingRule {
    /// Name of the rule
    pub name: String,
    
    /// Condition for applying this rule
    pub condition: RuleCondition,
    
    /// Action to take when condition matches
    pub action: RuleAction,
    
    /// Priority of this rule (higher = applied first)
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Pattern matches regex
    PatternMatches(String),
    
    /// Pattern appears in specific context
    InContext { pattern: String, before: Option<String>, after: Option<String> },
    
    /// Pattern frequency exceeds threshold
    FrequencyAbove(usize),
    
    /// Custom condition
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Map to specific target
    MapTo(String),
    
    /// Generate placeholder with prefix
    Placeholder(String),
    
    /// Apply transformation
    Transform(TransformAction),
    
    /// Skip this pattern
    Skip,
    
    /// Flag for manual review
    FlagForReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformAction {
    /// Convert to uppercase
    ToUpper,
    
    /// Convert to lowercase
    ToLower,
    
    /// Add prefix
    AddPrefix(String),
    
    /// Add suffix
    AddSuffix(String),
    
    /// Apply regex replacement
    RegexReplace { pattern: String, replacement: String },
    
    /// Custom transformation
    Custom(String),
}

/// Default configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    /// Default unknown pattern strategy
    pub unknown_strategy: UnknownPatternStrategy,
    
    /// Default validation settings
    pub validation: RoundTripValidation,
    
    /// Default confidence threshold for pattern detection
    pub confidence_threshold: f32,
    
    /// Maximum pattern length to consider
    pub max_pattern_length: usize,
}

impl VidyutExtensionConfig {
    /// Load configuration from YAML file
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml_str(&content)
    }
    
    /// Load configuration from YAML string
    pub fn from_yaml_str(yaml: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(yaml).map_err(ConfigError::YamlError)
    }
    
    /// Save configuration to YAML file
    pub fn to_yaml_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let yaml = self.to_yaml_str()?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
    
    /// Convert configuration to YAML string
    pub fn to_yaml_str(&self) -> Result<String, ConfigError> {
        serde_yaml::to_string(self).map_err(ConfigError::YamlError)
    }
    
    /// Get runtime configuration for a specific scheme and source
    pub fn get_runtime_config(&self, scheme: &str, source: Option<&str>) -> Result<RuntimeExtensionConfig, ConfigError> {
        let scheme_config = self.scheme_extensions.get(scheme)
            .ok_or_else(|| ConfigError::SchemeNotFound(scheme.to_string()))?;
        
        let mut patterns = scheme_config.patterns.clone();
        patterns.extend(self.global_patterns.clone());
        
        // Add source-specific patterns if specified
        if let Some(source_id) = source {
            if let Some(source_config) = scheme_config.sources.get(source_id) {
                patterns.extend(source_config.patterns.clone());
            }
        }
        
        Ok(RuntimeExtensionConfig {
            base_scheme: scheme_config.base_scheme.clone(),
            detection_patterns: patterns,
            unknown_pattern_strategy: scheme_config.unknown_strategy.clone(),
            validation: scheme_config.validation.clone(),
        })
    }
    
    /// Create a default configuration for common Vedic schemes
    pub fn create_vedic_default() -> Self {
        Self {
            version: "1.0".to_string(),
            scheme_extensions: Self::create_vedic_schemes(),
            global_patterns: Self::create_global_patterns(),
            defaults: DefaultConfig {
                unknown_strategy: UnknownPatternStrategy::CreatePlaceholders,
                validation: RoundTripValidation {
                    enabled: true,
                    tolerance: ValidationTolerance::UnicodeNormalized,
                    on_failure: ValidationFailureAction::Warn,
                },
                confidence_threshold: 0.7,
                max_pattern_length: 8,
            },
        }
    }
    
    fn create_vedic_schemes() -> HashMap<String, SchemeExtensionConfig> {
        let mut schemes = HashMap::new();
        
        // Baraha South with Vedic extensions
        schemes.insert("BarahaSouthVedic".to_string(), SchemeExtensionConfig {
            base_scheme: "BarahaSouth".to_string(),
            description: "Baraha South with Vedic accent support".to_string(),
            sources: Self::create_baraha_sources(),
            patterns: Self::create_baraha_patterns(),
            unknown_strategy: UnknownPatternStrategy::CreatePlaceholders,
            validation: RoundTripValidation {
                enabled: true,
                tolerance: ValidationTolerance::Exact,
                on_failure: ValidationFailureAction::Error,
            },
        });
        
        // Devanagari with Samaveda PUA support
        schemes.insert("DevanagariSamaveda".to_string(), SchemeExtensionConfig {
            base_scheme: "Devanagari".to_string(),
            description: "Devanagari with Samaveda PUA codes".to_string(),
            sources: Self::create_samaveda_sources(),
            patterns: Self::create_samaveda_patterns(),
            unknown_strategy: UnknownPatternStrategy::CreatePlaceholders,
            validation: RoundTripValidation {
                enabled: true,
                tolerance: ValidationTolerance::UnicodeNormalized,
                on_failure: ValidationFailureAction::Warn,
            },
        });
        
        schemes
    }
    
    fn create_baraha_sources() -> HashMap<String, SourceConfig> {
        let mut sources = HashMap::new();
        
        sources.insert("vedanidhi".to_string(), SourceConfig {
            source_id: "vedanidhi".to_string(),
            patterns: vec![
                DetectionPattern {
                    name: "anudatta_q".to_string(),
                    pattern: "q".to_string(),
                    pattern_type: PatternType::Accent,
                    target_mapping: TargetMapping::Fixed("{A}".to_string()),
                    confidence: 0.9,
                },
                DetectionPattern {
                    name: "svarita_hash".to_string(),
                    pattern: "#".to_string(),
                    pattern_type: PatternType::Accent,
                    target_mapping: TargetMapping::Fixed("{S}".to_string()),
                    confidence: 0.9,
                },
            ],
            known_mappings: HashMap::from([
                ("q".to_string(), "{A}".to_string()),
                ("#".to_string(), "{S}".to_string()),
                ("$".to_string(), "{DS}".to_string()),
            ]),
            processing_rules: vec![],
        });
        
        sources.insert("vedavms".to_string(), SourceConfig {
            source_id: "vedavms".to_string(),
            patterns: vec![
                DetectionPattern {
                    name: "nasal_annotations".to_string(),
                    pattern: r"\([a-z]+\)".to_string(),
                    pattern_type: PatternType::Nasal,
                    target_mapping: TargetMapping::Placeholder,
                    confidence: 0.8,
                },
            ],
            known_mappings: HashMap::new(),
            processing_rules: vec![],
        });
        
        sources
    }
    
    fn create_samaveda_sources() -> HashMap<String, SourceConfig> {
        let mut sources = HashMap::new();
        
        sources.insert("vedanidhi_samaveda".to_string(), SourceConfig {
            source_id: "vedanidhi_samaveda".to_string(),
            patterns: vec![
                DetectionPattern {
                    name: "pua_musical_accents".to_string(),
                    pattern: r"[\uE300-\uE3FF]".to_string(),
                    pattern_type: PatternType::Musical,
                    target_mapping: TargetMapping::Placeholder,
                    confidence: 0.95,
                },
            ],
            known_mappings: HashMap::from([
                ("\u{E311}".to_string(), "{U1}".to_string()),
                ("\u{E312}".to_string(), "{U2}".to_string()),
                ("\u{E322}".to_string(), "{A}".to_string()),
                ("\u{E324}".to_string(), "{S}".to_string()),
            ]),
            processing_rules: vec![],
        });
        
        sources
    }
    
    fn create_baraha_patterns() -> Vec<DetectionPattern> {
        vec![
            DetectionPattern {
                name: "vedic_accents".to_string(),
                pattern: r"[q#$]".to_string(),
                pattern_type: PatternType::Accent,
                target_mapping: TargetMapping::Placeholder,
                confidence: 0.8,
            },
            DetectionPattern {
                name: "nasal_groups".to_string(),
                pattern: r"~[a-zA-Z]".to_string(),
                pattern_type: PatternType::Nasal,
                target_mapping: TargetMapping::Placeholder,
                confidence: 0.7,
            },
        ]
    }
    
    fn create_samaveda_patterns() -> Vec<DetectionPattern> {
        vec![
            DetectionPattern {
                name: "samaveda_pua".to_string(),
                pattern: r"[\uE300-\uE3FF]".to_string(),
                pattern_type: PatternType::Musical,
                target_mapping: TargetMapping::Placeholder,
                confidence: 0.9,
            },
        ]
    }
    
    fn create_global_patterns() -> Vec<DetectionPattern> {
        vec![
            DetectionPattern {
                name: "unknown_symbols".to_string(),
                pattern: r"[^\p{L}\p{N}\p{M}\p{P}\p{S}\p{Z}]".to_string(),
                pattern_type: PatternType::Custom("unknown".to_string()),
                target_mapping: TargetMapping::Placeholder,
                confidence: 0.5,
            },
        ]
    }
}

/// Errors that can occur during configuration processing
#[derive(Debug)]
pub enum ConfigError {
    /// IO error reading/writing files
    IoError(std::io::Error),
    
    /// YAML parsing error
    YamlError(serde_yaml::Error),
    
    /// Scheme not found
    SchemeNotFound(String),
    
    /// Invalid configuration
    InvalidConfig(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(err) => write!(f, "IO error: {}", err),
            ConfigError::YamlError(err) => write!(f, "YAML error: {}", err),
            ConfigError::SchemeNotFound(scheme) => write!(f, "Scheme not found: {}", scheme),
            ConfigError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}