/*!
Extended SLP1 format for canonical vedic accent representation.

This module defines the Extended SLP1 format that serves as the canonical
representation for all vedic texts, regardless of their source encoding.
*/

use std::fmt;

/// Extended SLP1 text with vedic accent information.
///
/// This is the canonical format used internally by udapaana for storing
/// and processing vedic texts. All source encodings are converted to this
/// format to enable consistent processing across different digitization schemes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtendedSlp1 {
    text: String,
    metadata: ExtendedSlp1Metadata,
}

/// Metadata associated with Extended SLP1 text.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtendedSlp1Metadata {
    /// Density of accent markings (for indexing and search)
    pub accent_density: f32,
    
    /// Word count
    pub word_count: usize,
    
    /// Source schema ID that generated this text
    pub source_schema: Option<String>,
    
    /// Statistical accent distribution
    pub accent_stats: AccentStats,
}

/// Statistical distribution of accent types in the text.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct AccentStats {
    pub udatta_count: usize,
    pub anudatta_count: usize,
    pub svarita_count: usize,
    pub pragrhya_count: usize,
    pub custom_count: usize,
}

impl ExtendedSlp1 {
    /// Create new Extended SLP1 text.
    pub fn new(text: String) -> Self {
        let metadata = Self::calculate_metadata(&text);
        Self { text, metadata }
    }
    
    /// Create Extended SLP1 with explicit metadata.
    pub fn with_metadata(text: String, metadata: ExtendedSlp1Metadata) -> Self {
        Self { text, metadata }
    }
    
    /// Get the text content.
    pub fn text(&self) -> &str {
        &self.text
    }
    
    /// Get the metadata.
    pub fn metadata(&self) -> &ExtendedSlp1Metadata {
        &self.metadata
    }
    
    /// Calculate metadata from text content.
    fn calculate_metadata(text: &str) -> ExtendedSlp1Metadata {
        let mut stats = AccentStats::default();
        let mut total_chars = 0;
        let mut accent_chars = 0;
        
        for ch in text.chars() {
            total_chars += 1;
            match ch {
                '\'' => { stats.udatta_count += 1; accent_chars += 1; },
                '`' => { stats.anudatta_count += 1; accent_chars += 1; },
                '^' => { stats.svarita_count += 1; accent_chars += 1; },
                '~' => { stats.pragrhya_count += 1; accent_chars += 1; },
                _ if Self::is_custom_accent(ch) => { 
                    stats.custom_count += 1; 
                    accent_chars += 1; 
                },
                _ => {},
            }
        }
        
        let accent_density = if total_chars > 0 {
            accent_chars as f32 / total_chars as f32
        } else {
            0.0
        };
        
        let word_count = text.split_whitespace().count();
        
        ExtendedSlp1Metadata {
            accent_density,
            word_count,
            source_schema: None,
            accent_stats: stats,
        }
    }
    
    /// Check if character is a custom accent marker.
    fn is_custom_accent(ch: char) -> bool {
        // Custom markers: = + { } [ ] # and numbered variants
        matches!(ch, '=' | '+' | '{' | '}' | '[' | ']' | '#') ||
        (ch.is_ascii_digit() && ch != '0') // _1, _2, :1, :2, etc.
    }
    
    /// Validate that text conforms to Extended SLP1 format.
    pub fn validate(&self) -> Result<(), String> {
        // Basic validation - ensure only valid Extended SLP1 characters
        let valid_base = "aiueoAIUEOkKgGNcCjJYwWqQRtTdDnpPbBmyrlvSzsh";
        let valid_accents = "`'^~";
        let valid_punct = "|/";
        let valid_editorial = "{}[]+=";
        let valid_numbers = "0123456789";
        let valid_space = " \t\n\r";
        let valid_other = "_:#-";
        
        let valid_chars: String = format!("{}{}{}{}{}{}{}", 
                                        valid_base, valid_accents, valid_punct, 
                                        valid_editorial, valid_numbers, valid_space, valid_other);
        
        for ch in self.text.chars() {
            if !valid_chars.contains(ch) {
                return Err(format!("Invalid character in Extended SLP1: '{}'", ch));
            }
        }
        
        // TODO: Add more sophisticated validation:
        // - Accent markers should follow syllables
        // - Balanced editorial brackets
        // - Valid numbered variants
        
        Ok(())
    }
    
    /// Extract accent pattern for analysis.
    pub fn accent_pattern(&self) -> String {
        self.text.chars()
            .filter(|&ch| matches!(ch, '\'' | '`' | '^' | '~') || Self::is_custom_accent(ch))
            .collect()
    }
    
    /// Count syllables in the text.
    pub fn syllable_count(&self) -> usize {
        // Count vowels as proxy for syllables
        self.text.chars()
            .filter(|&ch| matches!(ch, 'a' | 'i' | 'u' | 'e' | 'o' | 'A' | 'I' | 'U' | 'E' | 'O'))
            .count()
    }
    
    /// Extract words (splitting on whitespace and punctuation).
    pub fn words(&self) -> Vec<&str> {
        self.text.split_whitespace()
            .flat_map(|word| word.split(&['|', '/', '{', '}', '[', ']'][..]))
            .filter(|word| !word.is_empty())
            .collect()
    }
    
    /// Get mutable reference to metadata
    pub fn metadata_mut(&mut self) -> &mut ExtendedSlp1Metadata {
        &mut self.metadata
    }
    
    /// Get immutable reference to metadata
    pub fn metadata(&self) -> &ExtendedSlp1Metadata {
        &self.metadata
    }
}

impl fmt::Display for ExtendedSlp1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl From<String> for ExtendedSlp1 {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for ExtendedSlp1 {
    fn from(text: &str) -> Self {
        Self::new(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let text = "agni'm meqe^ puro'hitam`";
        let extended = ExtendedSlp1::new(text.to_string());
        assert_eq!(extended.text(), text);
    }

    #[test]
    fn test_metadata_calculation() {
        let text = "agni'm meqe^ puro'hitam`|";
        let extended = ExtendedSlp1::new(text.to_string());
        let meta = extended.metadata();
        
        assert_eq!(meta.accent_stats.udatta_count, 2);
        assert_eq!(meta.accent_stats.anudatta_count, 1);
        assert_eq!(meta.accent_stats.svarita_count, 1);
        assert!(meta.accent_density > 0.0);
    }

    #[test]
    fn test_validation() {
        let valid = ExtendedSlp1::new("agni'm meqe^".to_string());
        assert!(valid.validate().is_ok());
        
        let invalid = ExtendedSlp1::new("agni@invalid".to_string());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_accent_pattern() {
        let text = "agni'm meqe^ puro'hitam`";
        let extended = ExtendedSlp1::new(text.to_string());
        assert_eq!(extended.accent_pattern(), "'`^'`");
    }
}