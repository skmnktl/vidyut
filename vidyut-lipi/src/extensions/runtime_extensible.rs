/// Runtime extensible mapping system for vidyut-lipi.
///
/// This module implements a dynamic mapping system that can:
/// 1. Discover unknown patterns during transliteration
/// 2. Extend source and destination schemas on-the-fly
/// 3. Ensure round-trip preservation of all source information
/// 4. Auto-generate bidirectional mappings for new patterns

use crate::mapping::Mapping;
use crate::scheme::Scheme;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use regex;

/// Configuration for runtime mapping extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExtensionConfig {
    /// Base scheme to extend from
    pub base_scheme: String,
    /// Patterns to detect during transliteration
    pub detection_patterns: Vec<DetectionPattern>,
    /// How to handle unknown patterns
    pub unknown_pattern_strategy: UnknownPatternStrategy,
    /// Round-trip validation settings
    pub validation: RoundTripValidation,
}

/// Strategy for handling unknown patterns encountered during transliteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnknownPatternStrategy {
    /// Preserve unknown patterns as-is in target scheme
    Preserve,
    /// Create placeholder mappings for unknown patterns
    CreatePlaceholders,
    /// Fail transliteration if unknown patterns are found
    Fail,
    /// Log unknown patterns but continue transliteration
    LogAndContinue,
}

/// Configuration for round-trip validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundTripValidation {
    /// Whether to enable round-trip testing
    pub enabled: bool,
    /// Tolerance for minor differences (e.g., whitespace normalization)
    pub tolerance: ValidationTolerance,
    /// What to do if round-trip fails
    pub on_failure: ValidationFailureAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationTolerance {
    /// Exact match required
    Exact,
    /// Allow whitespace normalization
    WhitespaceNormalized,
    /// Allow Unicode normalization
    UnicodeNormalized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationFailureAction {
    /// Return error
    Error,
    /// Log warning and continue
    Warn,
    /// Silent continue
    Ignore,
}

/// Pattern detection rule for identifying unknown accents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPattern {
    /// Name/identifier for this pattern type
    pub name: String,
    /// Regex or string pattern to match
    pub pattern: String,
    /// Type of pattern (accent, marker, etc.)
    pub pattern_type: PatternType,
    /// How to handle this pattern in target scheme
    pub target_mapping: TargetMapping,
    /// Confidence level for this detection
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Accent marker (udatta, anudatta, etc.)
    Accent,
    /// Sectional marker (end of verse, etc.)
    Section,
    /// Nasal annotation
    Nasal,
    /// Musical notation
    Musical,
    /// Unknown/custom
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetMapping {
    /// Map to specific string in target scheme
    Fixed(String),
    /// Generate unique placeholder
    Placeholder,
    /// Use source pattern as-is
    Passthrough,
    /// Map to closest equivalent accent
    BestFit(String),
}

/// Runtime extensible mapping system
pub struct RuntimeExtensibleMapping {
    /// Base mapping
    base_mapping: Mapping,
    /// Extended mappings discovered at runtime
    runtime_extensions: FxHashMap<String, String>,
    /// Reverse mappings for round-trip validation
    reverse_extensions: FxHashMap<String, String>,
    /// Configuration
    config: RuntimeExtensionConfig,
    /// Discovered patterns
    discovered_patterns: Vec<DiscoveredPattern>,
    /// Pattern matcher for detection
    pattern_matcher: PatternMatcher,
}

/// A pattern discovered during runtime
#[derive(Debug, Clone)]
pub struct DiscoveredPattern {
    /// The source pattern
    pub source: String,
    /// Generated target mapping
    pub target: String,
    /// Type of pattern detected
    pub pattern_type: PatternType,
    /// Confidence in detection
    pub confidence: f32,
    /// How many times this pattern was seen
    pub frequency: usize,
    /// Contexts where this pattern appeared
    pub contexts: Vec<String>,
}

/// Pattern matching engine for runtime detection
pub struct PatternMatcher {
    /// Compiled detection rules
    detection_rules: Vec<CompiledDetectionRule>,
    /// Cache of recently seen patterns
    pattern_cache: FxHashMap<String, PatternType>,
    /// Unknown patterns encountered
    unknown_patterns: FxHashSet<String>,
}

#[derive(Debug, Clone)]
struct CompiledDetectionRule {
    pattern: regex::Regex,
    rule: DetectionPattern,
}

impl RuntimeExtensibleMapping {
    /// Create new runtime extensible mapping
    pub fn new(from: Scheme, to: Scheme, config: RuntimeExtensionConfig) -> Self {
        let base_mapping = Mapping::new(from, to);
        let pattern_matcher = PatternMatcher::new(&config.detection_patterns);
        
        Self {
            base_mapping,
            runtime_extensions: FxHashMap::default(),
            reverse_extensions: FxHashMap::default(),
            config,
            discovered_patterns: Vec::new(),
            pattern_matcher,
        }
    }

    /// Transliterate with runtime pattern discovery and extension
    pub fn transliterate_with_discovery(&mut self, input: &str) -> Result<String, ExtensionError> {
        // First pass: detect unknown patterns
        let discovered = self.pattern_matcher.discover_patterns(input)?;
        
        // Extend mapping with discovered patterns
        for pattern in discovered {
            self.add_discovered_pattern(pattern)?;
        }
        
        // Second pass: transliterate with extended mapping
        let result = self.transliterate_extended(input)?;
        
        // Validate round-trip if enabled
        if self.config.validation.enabled {
            self.validate_round_trip(input, &result)?;
        }
        
        Ok(result)
    }

    /// Add a discovered pattern to the mapping
    fn add_discovered_pattern(&mut self, pattern: DiscoveredPattern) -> Result<(), ExtensionError> {
        let target_mapping = self.generate_target_mapping(&pattern)?;
        
        // Add forward mapping
        self.runtime_extensions.insert(pattern.source.clone(), target_mapping.clone());
        
        // Add reverse mapping for round-trip validation
        self.reverse_extensions.insert(target_mapping, pattern.source.clone());
        
        // Store the discovered pattern
        self.discovered_patterns.push(pattern);
        
        Ok(())
    }

    /// Generate appropriate target mapping for a discovered pattern
    fn generate_target_mapping(&self, pattern: &DiscoveredPattern) -> Result<String, ExtensionError> {
        // Find matching detection rule
        for rule in &self.config.detection_patterns {
            if self.pattern_matches_rule(&pattern.source, rule) {
                return self.apply_target_mapping(&pattern.source, &rule.target_mapping);
            }
        }
        
        // No matching rule - use unknown pattern strategy
        match self.config.unknown_pattern_strategy {
            UnknownPatternStrategy::Preserve => Ok(pattern.source.clone()),
            UnknownPatternStrategy::CreatePlaceholders => {
                Ok(format!("{{UNK_{}}}", self.generate_placeholder_id(&pattern.source)))
            },
            UnknownPatternStrategy::Fail => {
                Err(ExtensionError::UnknownPattern(pattern.source.clone()))
            },
            UnknownPatternStrategy::LogAndContinue => {
                eprintln!("Unknown pattern encountered: {}", pattern.source);
                Ok(pattern.source.clone())
            },
        }
    }

    /// Check if a pattern matches a detection rule
    fn pattern_matches_rule(&self, pattern: &str, rule: &DetectionPattern) -> bool {
        // Simple string matching for now - could be enhanced with regex
        pattern.contains(&rule.pattern)
    }

    /// Apply target mapping strategy
    fn apply_target_mapping(&self, source: &str, mapping: &TargetMapping) -> Result<String, ExtensionError> {
        match mapping {
            TargetMapping::Fixed(target) => Ok(target.clone()),
            TargetMapping::Placeholder => {
                Ok(format!("{{PLH_{}}}", self.generate_placeholder_id(source)))
            },
            TargetMapping::Passthrough => Ok(source.to_string()),
            TargetMapping::BestFit(equivalent) => Ok(equivalent.clone()),
        }
    }

    /// Generate unique placeholder ID for unknown patterns
    fn generate_placeholder_id(&self, pattern: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        pattern.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Transliterate using extended mapping
    fn transliterate_extended(&self, input: &str) -> Result<String, ExtensionError> {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = input.chars().collect();
        
        while i < chars.len() {
            let mut matched = false;
            
            // Try to match runtime extensions first (longest match)
            for len in (1..=std::cmp::min(10, chars.len() - i)).rev() {
                let substr: String = chars[i..i + len].iter().collect();
                if let Some(target) = self.runtime_extensions.get(&substr) {
                    result.push_str(target);
                    i += len;
                    matched = true;
                    break;
                }
            }
            
            if !matched {
                // Fall back to base mapping
                let _char_str = chars[i].to_string();
                // This is simplified - real implementation would use base_mapping properly
                result.push(chars[i]);
                i += 1;
            }
        }
        
        Ok(result)
    }

    /// Validate round-trip conversion
    fn validate_round_trip(&self, original: &str, transliterated: &str) -> Result<(), ExtensionError> {
        // Create reverse mapping
        let reconstructed = self.reverse_transliterate(transliterated)?;
        
        let matches = match self.config.validation.tolerance {
            ValidationTolerance::Exact => original == reconstructed,
            ValidationTolerance::WhitespaceNormalized => {
                self.normalize_whitespace(original) == self.normalize_whitespace(&reconstructed)
            },
            ValidationTolerance::UnicodeNormalized => {
                crate::unicode_norm::to_nfc(original) == crate::unicode_norm::to_nfc(&reconstructed)
            },
        };
        
        if !matches {
            let error = ExtensionError::RoundTripValidationFailed {
                original: original.to_string(),
                reconstructed,
                transliterated: transliterated.to_string(),
            };
            
            match self.config.validation.on_failure {
                ValidationFailureAction::Error => return Err(error),
                ValidationFailureAction::Warn => {
                    eprintln!("Round-trip validation warning: {}", error);
                },
                ValidationFailureAction::Ignore => {},
            }
        }
        
        Ok(())
    }

    /// Reverse transliterate using reverse mappings
    fn reverse_transliterate(&self, input: &str) -> Result<String, ExtensionError> {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = input.chars().collect();
        
        while i < chars.len() {
            let mut matched = false;
            
            // Try to match reverse extensions
            for len in (1..=std::cmp::min(20, chars.len() - i)).rev() {
                let substr: String = chars[i..i + len].iter().collect();
                if let Some(source) = self.reverse_extensions.get(&substr) {
                    result.push_str(source);
                    i += len;
                    matched = true;
                    break;
                }
            }
            
            if !matched {
                // Fall back to character-by-character
                result.push(chars[i]);
                i += 1;
            }
        }
        
        Ok(result)
    }

    /// Normalize whitespace for comparison
    fn normalize_whitespace(&self, text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Get discovered patterns for analysis
    pub fn get_discovered_patterns(&self) -> &[DiscoveredPattern] {
        &self.discovered_patterns
    }

    /// Export extension mappings for reuse
    pub fn export_extensions(&self) -> FxHashMap<String, String> {
        self.runtime_extensions.clone()
    }
}

impl PatternMatcher {
    /// Create new pattern matcher
    pub fn new(detection_patterns: &[DetectionPattern]) -> Self {
        let detection_rules = detection_patterns
            .iter()
            .filter_map(|rule| {
                regex::Regex::new(&rule.pattern)
                    .ok()
                    .map(|pattern| CompiledDetectionRule {
                        pattern,
                        rule: rule.clone(),
                    })
            })
            .collect();
        
        Self {
            detection_rules,
            pattern_cache: FxHashMap::default(),
            unknown_patterns: FxHashSet::default(),
        }
    }

    /// Discover patterns in input text
    pub fn discover_patterns(&mut self, input: &str) -> Result<Vec<DiscoveredPattern>, ExtensionError> {
        let mut discovered = Vec::new();
        let mut pattern_frequencies = FxHashMap::default();
        
        // Sliding window pattern detection
        let chars: Vec<char> = input.chars().collect();
        for window_size in 1..=8 {
            for i in 0..chars.len().saturating_sub(window_size) {
                if i + window_size > chars.len() {
                    break;
                }
                let pattern: String = chars[i..i + window_size].iter().collect();
                
                // Check if this looks like an accent pattern
                if self.is_potential_pattern(&pattern) {
                    *pattern_frequencies.entry(pattern.clone()).or_insert(0) += 1;
                    
                    // Extract context
                    let context_start = i.saturating_sub(5);
                    let context_end = std::cmp::min(i + window_size + 5, chars.len());
                    let context: String = chars[context_start..context_end].iter().collect();
                    
                    // Create discovered pattern
                    if let Some(pattern_type) = self.classify_pattern(&pattern) {
                        discovered.push(DiscoveredPattern {
                            source: pattern.clone(),
                            target: String::new(), // Will be filled later
                            pattern_type,
                            confidence: self.calculate_confidence(&pattern),
                            frequency: *pattern_frequencies.get(&pattern).unwrap_or(&0),
                            contexts: vec![context],
                        });
                    }
                }
            }
        }
        
        // Deduplicate and consolidate patterns
        self.consolidate_patterns(discovered)
    }

    /// Check if a string looks like a potential accent pattern
    fn is_potential_pattern(&self, pattern: &str) -> bool {
        // Check for known accent indicators
        pattern.contains('#') ||
        pattern.contains('q') ||
        pattern.contains('$') ||
        pattern.contains('(') ||
        pattern.contains('~') ||
        pattern.chars().any(|c| c as u32 >= 0xE300 && c as u32 <= 0xE3FF) || // PUA range
        (pattern.len() <= 4 && pattern.chars().any(|c| !c.is_alphabetic() && !c.is_whitespace()))
    }

    /// Classify what type of pattern this might be
    fn classify_pattern(&self, pattern: &str) -> Option<PatternType> {
        // Check against known rules first
        for rule in &self.detection_rules {
            if rule.pattern.is_match(pattern) {
                return Some(rule.rule.pattern_type.clone());
            }
        }
        
        // Heuristic classification
        if pattern.contains('#') || pattern.contains('q') {
            Some(PatternType::Accent)
        } else if pattern.contains('(') && pattern.contains(')') {
            Some(PatternType::Nasal)
        } else if pattern.chars().any(|c| c as u32 >= 0xE300 && c as u32 <= 0xE3FF) {
            Some(PatternType::Musical)
        } else {
            Some(PatternType::Custom("unknown".to_string()))
        }
    }

    /// Calculate confidence for pattern detection
    fn calculate_confidence(&self, pattern: &str) -> f32 {
        let mut confidence: f32 = 0.5; // Base confidence
        
        // Increase confidence for known markers
        if pattern.contains('#') || pattern.contains('q') || pattern.contains('$') {
            confidence += 0.3;
        }
        
        // Increase confidence for PUA codes
        if pattern.chars().any(|c| c as u32 >= 0xE300 && c as u32 <= 0xE3FF) {
            confidence += 0.4;
        }
        
        // Decrease confidence for very long patterns
        if pattern.len() > 5 {
            confidence -= 0.2;
        }
        
        confidence.clamp(0.0, 1.0)
    }

    /// Consolidate discovered patterns to remove duplicates
    fn consolidate_patterns(&self, patterns: Vec<DiscoveredPattern>) -> Result<Vec<DiscoveredPattern>, ExtensionError> {
        let mut consolidated = FxHashMap::default();
        
        for pattern in patterns {
            let entry = consolidated.entry(pattern.source.clone())
                .or_insert_with(|| pattern.clone());
            
            // Merge frequency and contexts
            entry.frequency += pattern.frequency;
            entry.contexts.extend(pattern.contexts);
            
            // Take highest confidence
            if pattern.confidence > entry.confidence {
                entry.confidence = pattern.confidence;
            }
        }
        
        Ok(consolidated.into_values().collect())
    }
}

/// Errors that can occur during runtime extension
#[derive(Debug, Clone)]
pub enum ExtensionError {
    /// Unknown pattern encountered with no handling strategy
    UnknownPattern(String),
    
    /// Round-trip validation failed
    RoundTripValidationFailed {
        original: String,
        reconstructed: String,
        transliterated: String,
    },
    
    /// Pattern compilation failed
    PatternCompilationFailed(String),
    
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionError::UnknownPattern(pattern) => {
                write!(f, "Unknown pattern encountered: {}", pattern)
            },
            ExtensionError::RoundTripValidationFailed { original, reconstructed, transliterated } => {
                write!(f, "Round-trip validation failed: '{}' -> '{}' -> '{}' (expected '{}')", 
                       original, transliterated, reconstructed, original)
            },
            ExtensionError::PatternCompilationFailed(msg) => {
                write!(f, "Pattern compilation failed: {}", msg)
            },
            ExtensionError::ConfigError(msg) => {
                write!(f, "Configuration error: {}", msg)
            },
        }
    }
}

impl std::error::Error for ExtensionError {}