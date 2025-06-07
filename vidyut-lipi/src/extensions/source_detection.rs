/// Automatic source detection for Vedic text processing.
///
/// This module analyzes file paths and content to automatically identify:
/// - Which veda (Rigveda, Yajurveda, Samaveda, Atharvaveda)
/// - Which sakha (Shakala, Taittiriya, Kauthuma, Shaunaka)
/// - Which source encoding or provider
/// And selects the appropriate extension scheme configuration.

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Source detection configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceDetectionConfig {
    /// Detection rules for identifying sources
    pub rules: Vec<DetectionRule>,
}

/// A rule for detecting source type from path and content
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DetectionRule {
    /// Name of this detection rule
    pub name: String,
    
    /// Conditions that must be met
    pub conditions: Vec<DetectionCondition>,
    
    /// Target scheme to use if conditions match
    pub target_scheme: String,
    
    /// Confidence in this detection
    pub confidence: f32,
}

/// Condition for source detection
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DetectionCondition {
    /// Text content contains specific string
    #[serde(rename = "contains")]
    Contains { text: String },
    
    /// File path contains specific string
    #[serde(rename = "source_path_contains")]
    SourcePathContains { path_fragment: String },
    
    /// Content contains PUA characters
    #[serde(rename = "contains_pua")]
    ContainsPUA { enabled: bool },
    
    /// File extension matches
    #[serde(rename = "file_extension")]
    FileExtension { extension: String },
    
    /// Content matches regex pattern
    #[serde(rename = "pattern_match")]
    PatternMatch { pattern: String },
    
    /// File size is within range
    #[serde(rename = "file_size_range")]
    FileSizeRange { min_bytes: Option<usize>, max_bytes: Option<usize> },
}

/// Result of source detection
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Detected scheme name
    pub scheme_name: String,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    
    /// Which rule matched
    pub matched_rule: String,
    
    /// Additional metadata detected
    pub metadata: SourceMetadata,
}

/// Metadata about the detected source
#[derive(Debug, Clone, Default)]
pub struct SourceMetadata {
    /// Detected veda
    pub veda: Option<String>,
    
    /// Detected sakha
    pub sakha: Option<String>,
    
    /// Detected source
    pub source: Option<String>,
    
    /// Detected text type (samhita, brahmana, aranyaka, upanishad)
    pub text_type: Option<String>,
    
    /// Whether PUA characters were found
    pub has_pua: bool,
    
    /// Detected accent patterns
    pub accent_patterns: Vec<String>,
    
    /// Detected section markers
    pub section_markers: Vec<String>,
}

/// Source detection engine
pub struct SourceDetector {
    /// Detection configuration
    config: SourceDetectionConfig,
    
    /// Compiled regex patterns for efficiency
    compiled_patterns: FxHashMap<String, regex::Regex>,
    
    /// Cache of previous detections
    detection_cache: FxHashMap<String, DetectionResult>,
}

impl SourceDetector {
    /// Create new source detector with configuration
    pub fn new(config: SourceDetectionConfig) -> Result<Self, DetectionError> {
        let mut compiled_patterns = FxHashMap::default();
        
        // Pre-compile regex patterns
        for rule in &config.rules {
            for condition in &rule.conditions {
                if let DetectionCondition::PatternMatch { pattern } = condition {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| DetectionError::PatternCompilationFailed(e.to_string()))?;
                    compiled_patterns.insert(pattern.clone(), regex);
                }
            }
        }
        
        Ok(Self {
            config,
            compiled_patterns,
            detection_cache: FxHashMap::default(),
        })
    }
    
    /// Create detector with default udapaana configuration
    pub fn with_udapaana_config() -> Result<Self, DetectionError> {
        let config = Self::create_udapaana_detection_config();
        Self::new(config)
    }
    
    /// Detect source type from file path and content
    pub fn detect_source<P: AsRef<Path>>(
        &mut self, 
        file_path: P, 
        content: &str
    ) -> Result<DetectionResult, DetectionError> {
        let file_path = file_path.as_ref();
        let cache_key = format!("{}:{}", file_path.display(), content.len());
        
        // Check cache first
        if let Some(cached) = self.detection_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        let mut best_match: Option<DetectionResult> = None;
        let mut metadata = SourceMetadata::default();
        
        // Extract metadata from path and content
        self.extract_metadata(file_path, content, &mut metadata);
        
        // Try each detection rule
        for rule in &self.config.rules {
            if let Ok(confidence) = self.evaluate_rule(rule, file_path, content, &metadata) {
                if confidence > 0.0 {
                    let result = DetectionResult {
                        scheme_name: rule.target_scheme.clone(),
                        confidence: confidence * rule.confidence,
                        matched_rule: rule.name.clone(),
                        metadata: metadata.clone(),
                    };
                    
                    if best_match.is_none() || result.confidence > best_match.as_ref().unwrap().confidence {
                        best_match = Some(result);
                    }
                }
            }
        }
        
        let result = best_match.unwrap_or_else(|| DetectionResult {
            scheme_name: "Unknown".to_string(),
            confidence: 0.0,
            matched_rule: "none".to_string(),
            metadata,
        });
        
        // Cache the result
        self.detection_cache.insert(cache_key, result.clone());
        
        Ok(result)
    }
    
    /// Extract metadata from file path and content
    fn extract_metadata(&self, file_path: &Path, content: &str, metadata: &mut SourceMetadata) {
        let path_str = file_path.to_string_lossy().to_lowercase();
        
        // Detect veda from path
        if path_str.contains("rigveda") {
            metadata.veda = Some("rigveda".to_string());
        } else if path_str.contains("yajurveda") {
            metadata.veda = Some("yajurveda".to_string());
        } else if path_str.contains("samaveda") {
            metadata.veda = Some("samaveda".to_string());
        } else if path_str.contains("atharvaveda") {
            metadata.veda = Some("atharvaveda".to_string());
        }
        
        // Detect sakha from path
        if path_str.contains("shakala") {
            metadata.sakha = Some("shakala".to_string());
        } else if path_str.contains("taittiriya") {
            metadata.sakha = Some("taittiriya".to_string());
        } else if path_str.contains("kauthuma") {
            metadata.sakha = Some("kauthuma".to_string());
        } else if path_str.contains("shaunaka") {
            metadata.sakha = Some("shaunaka".to_string());
        }
        
        // Detect source from path
        if path_str.contains("vedanidhi") {
            metadata.source = Some("vedanidhi".to_string());
        } else if path_str.contains("vedavms") {
            metadata.source = Some("vedavms".to_string());
        }
        
        // Detect text type from path and content
        if path_str.contains("samhita") || content.contains("संहिता") {
            metadata.text_type = Some("samhita".to_string());
        } else if path_str.contains("brahmana") || content.contains("ब्राह्मण") {
            metadata.text_type = Some("brahmana".to_string());
        } else if path_str.contains("aranyaka") || content.contains("आरण्यक") {
            metadata.text_type = Some("aranyaka".to_string());
        } else if path_str.contains("upanishad") || content.contains("उपनिषत्") {
            metadata.text_type = Some("upanishad".to_string());
        }
        
        // Detect PUA characters
        metadata.has_pua = content.chars().any(|c| {
            let code = c as u32;
            (0xE000..=0xF8FF).contains(&code)
        });
        
        // Detect accent patterns
        let accent_patterns = ["q", "#", "$", "##", "~m", "~n", "~g", "(gm)", "(gg)"];
        for pattern in &accent_patterns {
            if content.contains(pattern) {
                metadata.accent_patterns.push(pattern.to_string());
            }
        }
        
        // Detect section markers
        let section_markers = ["काण्डम्", "अष्टकम्", "आर्चिकम्", "गानम्", "पदम्"];
        for marker in &section_markers {
            if content.contains(marker) {
                metadata.section_markers.push(marker.to_string());
            }
        }
    }
    
    /// Evaluate a detection rule against file path and content
    fn evaluate_rule(
        &self,
        rule: &DetectionRule,
        file_path: &Path,
        content: &str,
        metadata: &SourceMetadata
    ) -> Result<f32, DetectionError> {
        let total_conditions = rule.conditions.len() as f32;
        let mut met_conditions = 0.0;
        
        for condition in &rule.conditions {
            if self.evaluate_condition(condition, file_path, content, metadata)? {
                met_conditions += 1.0;
            }
        }
        
        // All conditions must be met for a rule to match
        if met_conditions == total_conditions {
            Ok(1.0)
        } else {
            Ok(0.0)
        }
    }
    
    /// Evaluate a single detection condition
    fn evaluate_condition(
        &self,
        condition: &DetectionCondition,
        file_path: &Path,
        content: &str,
        metadata: &SourceMetadata
    ) -> Result<bool, DetectionError> {
        match condition {
            DetectionCondition::Contains { text } => {
                Ok(content.contains(text))
            },
            
            DetectionCondition::SourcePathContains { path_fragment } => {
                Ok(file_path.to_string_lossy().contains(path_fragment))
            },
            
            DetectionCondition::ContainsPUA { enabled } => {
                Ok(*enabled == metadata.has_pua)
            },
            
            DetectionCondition::FileExtension { extension } => {
                Ok(file_path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == extension)
                    .unwrap_or(false))
            },
            
            DetectionCondition::PatternMatch { pattern } => {
                if let Some(regex) = self.compiled_patterns.get(pattern) {
                    Ok(regex.is_match(content))
                } else {
                    // Fallback to simple string matching
                    Ok(content.contains(pattern))
                }
            },
            
            DetectionCondition::FileSizeRange { min_bytes, max_bytes } => {
                let content_size = content.len();
                let min_ok = min_bytes.map(|min| content_size >= min).unwrap_or(true);
                let max_ok = max_bytes.map(|max| content_size <= max).unwrap_or(true);
                Ok(min_ok && max_ok)
            },
        }
    }
    
    /// Create default detection configuration for udapaana sources
    fn create_udapaana_detection_config() -> SourceDetectionConfig {
        SourceDetectionConfig {
            rules: vec![
                // Rigveda Shakala from vedanidhi
                DetectionRule {
                    name: "rigveda_shakala_vedanidhi".to_string(),
                    conditions: vec![
                        DetectionCondition::Contains { text: "शाकलसंहिता".to_string() },
                        DetectionCondition::Contains { text: "अष्टकम्".to_string() },
                        DetectionCondition::SourcePathContains { path_fragment: "rigveda/shakala/vedanidhi".to_string() },
                    ],
                    target_scheme: "RigvedaShakalaVedanidhi".to_string(),
                    confidence: 0.95,
                },
                
                // Yajurveda Taittiriya from vedanidhi
                DetectionRule {
                    name: "yajurveda_taittiriya_vedanidhi".to_string(),
                    conditions: vec![
                        DetectionCondition::Contains { text: "तैत्तिरीयसंहिता".to_string() },
                        DetectionCondition::Contains { text: "काण्डम्".to_string() },
                        DetectionCondition::SourcePathContains { path_fragment: "yajurveda/taittiriya/vedanidhi".to_string() },
                    ],
                    target_scheme: "YajurvedaTaittiriyaVedanidhi".to_string(),
                    confidence: 0.95,
                },
                
                // Yajurveda Taittiriya from vedavms
                DetectionRule {
                    name: "yajurveda_taittiriya_vedavms".to_string(),
                    conditions: vec![
                        DetectionCondition::PatternMatch { pattern: "TS [0-9]+".to_string() },
                        DetectionCondition::Contains { text: "Baraha".to_string() },
                        DetectionCondition::SourcePathContains { path_fragment: "yajurveda/taittiriya/vedavms".to_string() },
                    ],
                    target_scheme: "YajurvedaTaittiriyaVedavms".to_string(),
                    confidence: 0.95,
                },
                
                // Samaveda Kauthuma from vedanidhi
                DetectionRule {
                    name: "samaveda_kauthuma_vedanidhi".to_string(),
                    conditions: vec![
                        DetectionCondition::Contains { text: "आर्चिकम्".to_string() },
                        DetectionCondition::ContainsPUA { enabled: true },
                        DetectionCondition::SourcePathContains { path_fragment: "samaveda/kauthuma/vedanidhi".to_string() },
                    ],
                    target_scheme: "SamavedaKauthumaVedanidhi".to_string(),
                    confidence: 0.95,
                },
                
                // Atharvaveda Shaunaka from vedanidhi
                DetectionRule {
                    name: "atharvaveda_shaunaka_vedanidhi".to_string(),
                    conditions: vec![
                        DetectionCondition::Contains { text: "शौनकसंहिता".to_string() },
                        DetectionCondition::Contains { text: "काण्डम्".to_string() },
                        DetectionCondition::SourcePathContains { path_fragment: "atharvaveda/shaunaka/vedanidhi".to_string() },
                    ],
                    target_scheme: "AtharvavedaShaunakaVedanidhi".to_string(),
                    confidence: 0.95,
                },
                
                // Generic Baraha detection (fallback)
                DetectionRule {
                    name: "generic_baraha_vedic".to_string(),
                    conditions: vec![
                        DetectionCondition::PatternMatch { pattern: "[q#$]".to_string() },
                        DetectionCondition::FileExtension { extension: "json".to_string() },
                    ],
                    target_scheme: "BarahaSouthVedic".to_string(),
                    confidence: 0.6,
                },
            ],
        }
    }
    
    /// Get detection statistics
    pub fn get_cache_stats(&self) -> (usize, Vec<String>) {
        let cache_size = self.detection_cache.len();
        let schemes: FxHashSet<String> = self.detection_cache
            .values()
            .map(|result| result.scheme_name.clone())
            .collect();
        let unique_schemes: Vec<String> = schemes.into_iter().collect();
        
        (cache_size, unique_schemes)
    }
    
    /// Clear detection cache
    pub fn clear_cache(&mut self) {
        self.detection_cache.clear();
    }
}

/// Errors that can occur during source detection
#[derive(Debug, Clone)]
pub enum DetectionError {
    /// Pattern compilation failed
    PatternCompilationFailed(String),
    
    /// File reading error
    FileError(String),
    
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for DetectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DetectionError::PatternCompilationFailed(msg) => {
                write!(f, "Pattern compilation failed: {}", msg)
            },
            DetectionError::FileError(msg) => {
                write!(f, "File error: {}", msg)
            },
            DetectionError::ConfigError(msg) => {
                write!(f, "Configuration error: {}", msg)
            },
        }
    }
}

impl std::error::Error for DetectionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_rigveda_detection() {
        let mut detector = SourceDetector::with_udapaana_config().unwrap();
        
        let path = PathBuf::from("data/vedic_texts/rigveda/shakala/vedanidhi/samhita/source/01010102_शाकलसंहिता_-_(01010102)_अष्टकम्.json");
        let content = "शाकलसंहिता अष्टकम् agni#mILe# purohitam";
        
        let result = detector.detect_source(&path, content).unwrap();
        
        assert_eq!(result.scheme_name, "RigvedaShakalaVedanidhi");
        assert!(result.confidence > 0.9);
        assert_eq!(result.metadata.veda, Some("rigveda".to_string()));
        assert_eq!(result.metadata.sakha, Some("shakala".to_string()));
        assert_eq!(result.metadata.source, Some("vedanidhi".to_string()));
    }
    
    #[test]
    fn test_samaveda_pua_detection() {
        let mut detector = SourceDetector::with_udapaana_config().unwrap();
        
        let path = PathBuf::from("data/vedic_texts/samaveda/kauthuma/vedanidhi/samhita/source/030301_आर्चिकम्.json");
        let content = "आर्चिकम् अग्न\u{E311}ए या\u{E312}हि";
        
        let result = detector.detect_source(&path, content).unwrap();
        
        assert_eq!(result.scheme_name, "SamavedaKauthumaVedanidhi");
        assert!(result.confidence > 0.9);
        assert!(result.metadata.has_pua);
        assert_eq!(result.metadata.veda, Some("samaveda".to_string()));
    }
}