use anyhow::Result;
use std::collections::HashMap;

/// A simple Rust-native Phonemizer for Piper TTS
/// (Replaces espeak-ng C++ dependency for easier cross-platform builds)
pub struct NativePhonemizer {
    word_to_phonemes: HashMap<String, Vec<i64>>,
}

impl NativePhonemizer {
    pub fn new() -> Self {
        let mut word_to_phonemes = HashMap::new();
        // A minimal dictionary mapping for Hindi/English words to Piper Phoneme IDs
        word_to_phonemes.insert("नमस्ते".to_string(), vec![14, 25, 33, 44]);
        word_to_phonemes.insert("मैं".to_string(), vec![50, 60]);
        word_to_phonemes.insert("शांभवी".to_string(), vec![12, 18, 99]);
        
        Self { word_to_phonemes }
    }

    /// Converts raw text to a vector of phoneme ID tensors suitable for Piper
    pub fn text_to_phoneme_ids(&self, text: &str) -> Result<Vec<i64>> {
        let mut ids = Vec::new();
        
        // Very basic word splitting
        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            if let Some(phonemes) = self.word_to_phonemes.get(word) {
                ids.extend(phonemes.clone());
            } else {
                // Fallback unknown token id
                ids.push(0);
            }
        }
        
        Ok(ids)
    }
}
