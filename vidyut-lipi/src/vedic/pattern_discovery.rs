/*!
Pattern discovery engine for unknown vedic text encodings.

This module analyzes vedic text corpora to automatically detect accent notation
patterns and other encoding conventions without prior knowledge of the schema.
*/

use std::collections::HashMap;
use std::path::Path;
use crate::vedic::{VedicResult, VedicError};

/// Pattern discovery engine for vedic texts.
pub struct PatternDiscovery {
    config: DiscoveryConfig,
}

/// Configuration for pattern discovery.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Minimum frequency threshold for pattern detection
    pub min_frequency: usize,
    
    /// Confidence threshold for automatic acceptance
    pub confidence_threshold: f64,
    
    /// Enable interactive refinement
    pub interactive_mode: bool,
    
    /// Maximum pattern length to consider
    pub max_pattern_length: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            min_frequency: 5,
            confidence_threshold: 0.8,
            interactive_mode: true,
            max_pattern_length: 3,
        }
    }
}

/// A discovered pattern in the text.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredPattern {
    /// The pattern string (e.g., "1", "\\", "\ue041")
    pub pattern: String,
    
    /// How the pattern is positioned relative to syllables
    pub position: PatternPosition,
    
    /// What linguistic scope this pattern applies to
    pub scope: PatternScope,
    
    /// Statistical confidence in this pattern
    pub confidence: PatternConfidence,
    
    /// Suggested mapping to Extended SLP1
    pub suggested_mapping: String,
    
    /// Example occurrences in the text
    pub examples: Vec<String>,
}

/// Position of pattern relative to syllables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternPosition {
    /// Pattern appears before the syllable
    Pre,
    /// Pattern appears after the syllable
    Post,
    /// Pattern appears above/below (combining characters)
    Combining,
    /// Pattern replaces part of the syllable
    Replacing,
    /// Pattern appears independently
    Independent,
}

/// Linguistic scope of the pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternScope {
    /// Applies to individual syllables
    Syllable,
    /// Applies to entire words
    Word,
    /// Applies to phrases or lines
    Phrase,
    /// Punctuation or structural markers
    Structural,
}

/// Confidence metrics for a discovered pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct PatternConfidence {
    /// How frequently the pattern appears
    pub frequency_score: f64,
    
    /// How consistently the pattern is used
    pub consistency_score: f64,
    
    /// How well position matches known vedic patterns
    pub position_score: f64,
    
    /// Overall linguistic plausibility
    pub linguistic_score: f64,
    
    /// Whether human has validated this pattern
    pub human_validated: bool,
    
    /// Combined confidence score (0.0 to 1.0)
    pub overall_confidence: f64,
}

impl PatternDiscovery {
    /// Create a new pattern discovery engine.
    pub fn new() -> Self {
        Self::with_config(DiscoveryConfig::default())
    }
    
    /// Create pattern discovery with custom configuration.
    pub fn with_config(config: DiscoveryConfig) -> Self {
        Self { config }
    }
    
    /// Analyze a text file to discover accent patterns.
    pub fn analyze_file<P: AsRef<Path>>(&self, path: P) -> VedicResult<Vec<DiscoveredPattern>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| VedicError::PatternDiscoveryFailed(e.to_string()))?;
        self.analyze_text(&content)
    }
    
    /// Analyze text content to discover accent patterns.
    pub fn analyze_text(&self, text: &str) -> VedicResult<Vec<DiscoveredPattern>> {
        let mut patterns = Vec::new();
        
        // 1. Detect numeric post-syllable patterns
        patterns.extend(self.detect_numeric_patterns(text)?);
        
        // 2. Detect diacritical mark patterns
        patterns.extend(self.detect_diacritical_patterns(text)?);
        
        // 3. Detect custom unicode patterns (Private Use Area)
        patterns.extend(self.detect_custom_unicode_patterns(text)?);
        
        // 4. Detect punctuation patterns
        patterns.extend(self.detect_punctuation_patterns(text)?);
        
        // 5. Calculate confidence scores
        for pattern in &mut patterns {
            pattern.confidence = self.calculate_confidence(pattern, text);
        }
        
        // 6. Filter by confidence threshold
        patterns.retain(|p| p.confidence.overall_confidence >= self.config.confidence_threshold);
        
        // 7. Interactive refinement if enabled
        if self.config.interactive_mode {
            patterns = self.interactive_refinement(patterns, text)?;
        }
        
        Ok(patterns)
    }
    
    /// Detect numeric patterns after syllables (e.g., "1", "2", "3").
    fn detect_numeric_patterns(&self, text: &str) -> VedicResult<Vec<DiscoveredPattern>> {
        let mut patterns = Vec::new();
        let mut numeric_counts = HashMap::new();
        let mut examples = HashMap::new();
        
        // Look for syllable + number patterns
        let syllable_regex = regex::Regex::new(r"[aiueoAIUEO][kKgGNcCjJYwWqQRtTdDnpPbBmyrlvSzsh]*([0-9]+)")
            .map_err(|e| VedicError::PatternDiscoveryFailed(e.to_string()))?;
        
        for capture in syllable_regex.captures_iter(text) {
            if let Some(number) = capture.get(1) {
                let num_str = number.as_str();
                *numeric_counts.entry(num_str.to_string()).or_insert(0) += 1;
                examples.entry(num_str.to_string())
                    .or_insert_with(Vec::new)
                    .push(capture.get(0).unwrap().as_str().to_string());
            }
        }
        
        for (number, count) in numeric_counts {
            if count >= self.config.min_frequency {
                let suggested_mapping = match number.as_str() {
                    "1" => "'",     // udatta
                    "2" => "`",     // anudatta  
                    "3" => "^",     // svarita
                    _ => &format!("_{}", number), // custom variant
                };
                
                patterns.push(DiscoveredPattern {
                    pattern: number.clone(),
                    position: PatternPosition::Post,
                    scope: PatternScope::Syllable,
                    confidence: PatternConfidence::default(),
                    suggested_mapping: suggested_mapping.to_string(),
                    examples: examples.get(&number).cloned().unwrap_or_default(),
                });
            }
        }
        
        Ok(patterns)
    }
    
    /// Detect diacritical mark patterns.
    fn detect_diacritical_patterns(&self, text: &str) -> VedicResult<Vec<DiscoveredPattern>> {
        let mut patterns = Vec::new();
        let mut diacritic_counts = HashMap::new();
        let mut examples = HashMap::new();
        
        // Look for common diacritical marks
        for ch in text.chars() {
            if Self::is_diacritical_mark(ch) {
                let ch_str = ch.to_string();
                *diacritic_counts.entry(ch_str.clone()).or_insert(0) += 1;
                examples.entry(ch_str)
                    .or_insert_with(Vec::new)
                    .push(format!("...{}...", ch)); // TODO: Get better context
            }
        }
        
        for (diacritic, count) in diacritic_counts {
            if count >= self.config.min_frequency {
                let suggested_mapping = match diacritic.chars().next().unwrap() {
                    'á' | '\u{0301}' => "'",  // acute accent -> udatta
                    'à' | '\u{0300}' => "`",  // grave accent -> anudatta
                    'â' | '\u{0302}' => "^",  // circumflex -> svarita
                    _ => "_1", // custom
                };
                
                patterns.push(DiscoveredPattern {
                    pattern: diacritic.clone(),
                    position: PatternPosition::Combining,
                    scope: PatternScope::Syllable,
                    confidence: PatternConfidence::default(),
                    suggested_mapping: suggested_mapping.to_string(),
                    examples: examples.get(&diacritic).cloned().unwrap_or_default(),
                });
            }
        }
        
        Ok(patterns)
    }
    
    /// Detect custom unicode patterns (Private Use Area \ue000-\uf8ff).
    fn detect_custom_unicode_patterns(&self, text: &str) -> VedicResult<Vec<DiscoveredPattern>> {
        let mut patterns = Vec::new();
        let mut unicode_counts = HashMap::new();
        let mut examples = HashMap::new();
        
        for ch in text.chars() {
            if Self::is_private_use_area(ch) {
                let ch_str = ch.to_string();
                *unicode_counts.entry(ch_str.clone()).or_insert(0) += 1;
                examples.entry(ch_str)
                    .or_insert_with(Vec::new)
                    .push(format!("...{}...", ch)); // TODO: Get better context
            }
        }
        
        for (unicode_char, count) in unicode_counts {
            if count >= self.config.min_frequency {
                patterns.push(DiscoveredPattern {
                    pattern: unicode_char.clone(),
                    position: PatternPosition::Combining, // Usually combining
                    scope: PatternScope::Syllable,
                    confidence: PatternConfidence::default(),
                    suggested_mapping: "_1".to_string(), // Custom mapping needed
                    examples: examples.get(&unicode_char).cloned().unwrap_or_default(),
                });
            }
        }
        
        Ok(patterns)
    }
    
    /// Detect punctuation patterns.
    fn detect_punctuation_patterns(&self, text: &str) -> VedicResult<Vec<DiscoveredPattern>> {
        let mut patterns = Vec::new();
        let mut punct_counts = HashMap::new();
        let mut examples = HashMap::new();
        
        // Look for repeated punctuation patterns
        let punct_regex = regex::Regex::new(r"([|=\-_]{1,3})")
            .map_err(|e| VedicError::PatternDiscoveryFailed(e.to_string()))?;
        
        for capture in punct_regex.captures_iter(text) {
            if let Some(punct) = capture.get(1) {
                let punct_str = punct.as_str();
                *punct_counts.entry(punct_str.to_string()).or_insert(0) += 1;
                examples.entry(punct_str.to_string())
                    .or_insert_with(Vec::new)
                    .push(punct_str.to_string());
            }
        }
        
        for (punct, count) in punct_counts {
            if count >= self.config.min_frequency {
                let suggested_mapping = match punct.as_str() {
                    "|" => "|",      // danda
                    "||" => "||",    // dvidanda
                    "===" => "|||",  // section break
                    _ => &punct,     // preserve as-is
                };
                
                patterns.push(DiscoveredPattern {
                    pattern: punct.clone(),
                    position: PatternPosition::Independent,
                    scope: PatternScope::Structural,
                    confidence: PatternConfidence::default(),
                    suggested_mapping: suggested_mapping.to_string(),
                    examples: examples.get(&punct).cloned().unwrap_or_default(),
                });
            }
        }
        
        Ok(patterns)
    }
    
    /// Calculate confidence score for a pattern.
    fn calculate_confidence(&self, pattern: &DiscoveredPattern, text: &str) -> PatternConfidence {
        let frequency_score = (pattern.examples.len() as f64).min(100.0) / 100.0;
        
        // TODO: Implement more sophisticated confidence calculation
        let consistency_score = 0.8; // Placeholder
        let position_score = match pattern.position {
            PatternPosition::Post => 0.9, // Common for vedic accents
            PatternPosition::Combining => 0.8,
            PatternPosition::Pre => 0.6,
            _ => 0.5,
        };
        let linguistic_score = 0.7; // Placeholder
        
        let overall_confidence = (frequency_score + consistency_score + position_score + linguistic_score) / 4.0;
        
        PatternConfidence {
            frequency_score,
            consistency_score,
            position_score,
            linguistic_score,
            human_validated: false,
            overall_confidence,
        }
    }
    
    /// Interactive refinement of discovered patterns.
    fn interactive_refinement(&self, patterns: Vec<DiscoveredPattern>, _text: &str) -> VedicResult<Vec<DiscoveredPattern>> {
        // TODO: Implement interactive CLI for pattern review and refinement
        // For now, just return patterns as-is
        Ok(patterns)
    }
    
    /// Check if character is a diacritical mark.
    fn is_diacritical_mark(ch: char) -> bool {
        // Unicode combining diacritical marks and common accented characters
        (ch >= '\u{0300}' && ch <= '\u{036F}') || // Combining Diacritical Marks
        matches!(ch, 'á' | 'à' | 'â' | 'ã' | 'ā' | 'ă' | 'ȧ' | 'ạ' | 'ả' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' |
                 'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ẻ' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' |
                 'í' | 'ì' | 'î' | 'ï' | 'ī' | 'ĭ' | 'į' | 'ỉ' | 'ị' |
                 'ó' | 'ò' | 'ô' | 'õ' | 'ō' | 'ŏ' | 'ő' | 'ọ' | 'ỏ' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' |
                 'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'ủ' | 'ụ' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự')
    }
    
    /// Check if character is in Unicode Private Use Area.
    fn is_private_use_area(ch: char) -> bool {
        (ch >= '\u{E000}' && ch <= '\u{F8FF}') || // Private Use Area
        (ch >= '\u{F0000}' && ch <= '\u{FFFFD}') || // Supplementary Private Use Area-A
        (ch >= '\u{100000}' && ch <= '\u{10FFFD}') // Supplementary Private Use Area-B
    }
}

impl Default for PatternDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PatternConfidence {
    fn default() -> Self {
        Self {
            frequency_score: 0.0,
            consistency_score: 0.0,
            position_score: 0.0,
            linguistic_score: 0.0,
            human_validated: false,
            overall_confidence: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_pattern_detection() {
        let discovery = PatternDiscovery::new();
        let text = "agni1m indra2sya varu3Na1sya";
        let patterns = discovery.analyze_text(text).unwrap();
        
        // Should detect patterns "1", "2", "3"
        assert!(patterns.iter().any(|p| p.pattern == "1"));
        assert!(patterns.iter().any(|p| p.pattern == "2"));
        assert!(patterns.iter().any(|p| p.pattern == "3"));
    }

    #[test]
    fn test_diacritical_pattern_detection() {
        let discovery = PatternDiscovery::new();
        let text = "agnīm índrasya váruṇasya";
        let patterns = discovery.analyze_text(text).unwrap();
        
        // Should detect diacritical marks
        assert!(patterns.iter().any(|p| p.position == PatternPosition::Combining));
    }

    #[test]
    fn test_punctuation_pattern_detection() {
        let discovery = PatternDiscovery::new();
        let text = "agnim indra| varuNasya ||";
        let patterns = discovery.analyze_text(text).unwrap();
        
        // Should detect punctuation
        assert!(patterns.iter().any(|p| p.scope == PatternScope::Structural));
    }
}