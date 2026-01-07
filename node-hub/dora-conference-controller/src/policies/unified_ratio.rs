use super::Policy;
use std::collections::HashMap;

/// Policy pattern configuration that determines behavior mode from syntax
#[derive(Debug, Clone)]
pub enum PolicyPattern {
    /// Ratio-based or priority-based policy
    RatioPriority {
        participants: Vec<String>,
        weights: Vec<Weight>,
    },
    /// Sequential policy
    Sequential {
        participants: Vec<String>,
        loop_forever: bool,
    },
}

/// Weight types
#[derive(Debug, Clone)]
pub enum Weight {
    /// Priority marker
    Priority,
    /// Ratio value
    Ratio(f64),
}

/// Parser for policy configuration
pub struct PatternParser;

impl PatternParser {
    /// Parse pattern string
    pub fn parse(pattern: &str) -> Result<PolicyPattern, String> {
        let trimmed = pattern.trim();
        if trimmed.contains('→') {
            return Self::parse_sequential(trimmed);
        }
        Self::parse_ratio_priority(trimmed)
    }

    /// Parse sequential pattern
    fn parse_sequential(pattern: &str) -> Result<PolicyPattern, String> {
        let cleaned = pattern.trim_matches(|c| c == '[' || c == ']').trim();
        let participants: Vec<String> = cleaned.split('→').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if participants.is_empty() {
            return Err("No valid participants found".to_string());
        }
        Ok(PolicyPattern::Sequential { participants, loop_forever: true })
    }

    /// Parse ratio/priority pattern
    fn parse_ratio_priority(pattern: &str) -> Result<PolicyPattern, String> {
        let cleaned = pattern.trim_matches(|c| c == '[' || c == ']').trim();
        if cleaned.is_empty() {
            return Err("Pattern cannot be empty".to_string());
        }

        // Parse comma-separated entries, handling parentheses
        let mut participants = Vec::new();
        let mut weights = Vec::new();
        let mut i = 0;
        let chars: Vec<char> = cleaned.chars().collect();

        while i < chars.len() {
            // Skip whitespace and commas
            while i < chars.len() && (chars[i].is_whitespace() || chars[i] == ',') {
                i += 1;
            }
            if i >= chars.len() { break; }

            // Check if entry starts with parenthesis
            if chars[i] == '(' {
                // Find closing parenthesis
                let start = i;
                let mut depth = 1;
                i += 1;
                while i < chars.len() && depth > 0 {
                    if chars[i] == '(' { depth += 1; }
                    else if chars[i] == ')' { depth -= 1; }
                    i += 1;
                }
                if depth > 0 {
                    return Err("Unmatched parenthesis".to_string());
                }
                let entry: String = chars[start..i].iter().collect();
                // Parse the parenthesized entry
                if let Some((name, weight)) = Self::parse_parenthesized_entry(&entry)? {
                    participants.push(name);
                    weights.push(weight);
                }
            } else {
                // Simple entry without parentheses, read until comma or end
                let start = i;
                while i < chars.len() && chars[i] != ',' && chars[i] != ')' {
                    i += 1;
                }
                let entry: String = chars[start..i].iter().collect();
                let entry = entry.trim();
                if !entry.is_empty() {
                    participants.push(entry.to_string());
                    weights.push(Weight::Ratio(1.0));
                }
            }
        }

        if participants.is_empty() {
            return Err("No participants found".to_string());
        }
        Ok(PolicyPattern::RatioPriority { participants, weights })
    }

    /// Parse a parenthesized entry like "(Name, weight)"
    fn parse_parenthesized_entry(entry: &str) -> Result<Option<(String, Weight)>, String> {
        let trimmed = entry.trim();
        if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
            return Ok(None);
        }
        let inner = trimmed[1..trimmed.len()-1].trim();
        let parts: Vec<&str> = inner.splitn(2, ',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid format in '{}': expected (Name, weight)", entry));
        }
        let name = parts[0].to_string();
        let weight_str = parts[1];
        let weight = if weight_str == "*" {
            Weight::Priority
        } else if let Ok(ratio) = weight_str.parse::<f64>() {
            if ratio <= 0.0 { return Err("Ratio must be positive".to_string()); }
            Weight::Ratio(ratio)
        } else {
            return Err("Invalid weight".to_string());
        };
        Ok(Some((name, weight)))
    }

    /// Validate configuration
    pub fn validate(pattern: &PolicyPattern) -> Result<(), String> {
        match pattern {
            PolicyPattern::RatioPriority { participants, weights } => {
                if participants.len() != weights.len() {
                    return Err("Mismatched lengths".to_string());
                }
                if participants.is_empty() {
                    return Err("Need participants".to_string());
                }
                for weight in weights {
                    if let Weight::Ratio(ratio) = weight {
                        if *ratio <= 0.0 { return Err("Positive ratios only".to_string()); }
                    }
                }
                Ok(())
            }
            PolicyPattern::Sequential { participants, .. } => {
                if participants.is_empty() { return Err("Need participants".to_string()); }
                Ok(())
            }
        }
    }
}

/// Policy implementation
pub struct UnifiedRatioPolicy {
    pattern: PolicyPattern,
    position: usize,
    word_counts: HashMap<String, usize>,
    last_speaker: Option<String>,
    sequential_cycle: usize,
    ratio_priority_cycle: usize,  // Track cycles for ratio/priority mode
    round_speakers: Vec<String>,  // Track speakers in current round
}

impl UnifiedRatioPolicy {
    pub fn new() -> Self {
        Self {
            pattern: PolicyPattern::RatioPriority {
                participants: Vec::new(),
                weights: Vec::new(),
            },
            position: 0,
            word_counts: HashMap::new(),
            last_speaker: None,
            sequential_cycle: 0,
            ratio_priority_cycle: 0,
            round_speakers: Vec::new(),
        }
    }

    /// Configure policy from pattern string
    pub fn configure(&mut self, pattern_str: &str) -> Result<(), String> {
        let pattern = PatternParser::parse(pattern_str)?;
        PatternParser::validate(&pattern)?;
        self.position = 0;
        self.word_counts.clear();
        self.last_speaker = None;
        self.sequential_cycle = 0;
        self.ratio_priority_cycle = 0;
        match &pattern {
            PolicyPattern::RatioPriority { participants, .. } | PolicyPattern::Sequential { participants, .. } => {
                for participant in participants {
                    self.word_counts.insert(participant.clone(), 0);
                }
            }
        }
        self.pattern = pattern;
        Ok(())
    }

    /// Get participants
    pub fn get_participants(&self) -> Vec<String> {
        match &self.pattern {
            PolicyPattern::RatioPriority { participants, .. } | PolicyPattern::Sequential { participants, .. } => participants.clone(),
        }
    }

    /// Reset word counts
    pub fn reset_counts(&mut self) {
        for count in self.word_counts.values_mut() { *count = 0; }
        self.position = 0;
        self.last_speaker = None;
        self.sequential_cycle = 0;
        self.ratio_priority_cycle = 0;
    }

    /// Get statistics
    pub fn get_stats(&self) -> serde_json::Value {
        let mut stats = serde_json::Map::new();
        match &self.pattern {
            PolicyPattern::RatioPriority { participants, weights } => {
                stats.insert("mode".to_string(), serde_json::Value::String("ratio_priority".to_string()));
                stats.insert("participants".to_string(), serde_json::Value::Array(participants.iter().map(|p| serde_json::Value::String(p.clone())).collect()));
                let weight_objects: Vec<_> = participants.iter().zip(weights).map(|(p,w)| {
                    let mut obj = serde_json::Map::new();
                    obj.insert("name".to_string(), serde_json::Value::String(p.clone()));
                    match w { Weight::Priority => obj.insert("weight".to_string(), serde_json::Value::String("*".to_string())), Weight::Ratio(r) => obj.insert("weight".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(*r).unwrap())) };
                    serde_json::Value::Object(obj)
                }).collect();
                stats.insert("weights".to_string(), serde_json::Value::Array(weight_objects));
            }
            PolicyPattern::Sequential { participants, loop_forever } => {
                stats.insert("mode".to_string(), serde_json::Value::String("sequential".to_string()));
                stats.insert("sequence".to_string(), serde_json::Value::Array(participants.iter().map(|p| serde_json::Value::String(p.clone())).collect()));
                stats.insert("loop_forever".to_string(), serde_json::Value::Bool(*loop_forever));
            }
        }
        let word_count_obj: serde_json::Map<String, serde_json::Value> = self.word_counts.iter().map(|(k,v)| (k.clone(), serde_json::Value::Number(serde_json::Number::from(*v)))).collect();
        stats.insert("word_counts".to_string(), serde_json::Value::Object(word_count_obj));
        // Compute next speaker from sequence and position
        if let PolicyPattern::Sequential { participants, .. } = &self.pattern {
            if !participants.is_empty() {
                let next_speaker = &participants[self.position % participants.len()];
                stats.insert("next_speaker".to_string(), serde_json::Value::String(next_speaker.clone()));
            }
        }
        stats.insert("cycle".to_string(), serde_json::Value::Number(self.get_current_cycle().into()));
        if let Some(last) = &self.last_speaker { stats.insert("current_speaker".to_string(), serde_json::Value::String(last.clone())); }
        serde_json::Value::Object(stats)
    }
}

impl Policy for UnifiedRatioPolicy {
    fn update_word_count(&mut self, speaker: &str, word_count: usize) {
        if let Some(count) = self.word_counts.get_mut(speaker) { *count += word_count; }
        // Update last_speaker when we receive input - this is who just spoke
        self.last_speaker = Some(speaker.to_string());

        // Track speaker in current round (avoid duplicates)
        if !self.round_speakers.contains(&speaker.to_string()) {
            self.round_speakers.push(speaker.to_string());
        }
    }

    fn determine_next_speaker(&mut self) -> Option<String> {
        match &self.pattern {
            PolicyPattern::RatioPriority { participants, weights } => {
                let p = participants.clone();
                let w = weights.clone();
                self.determine_ratio_or_priority_speaker(&p, &w)
            },
            PolicyPattern::Sequential { participants, loop_forever } => {
                let p = participants.clone();
                self.determine_sequential_speaker(&p, *loop_forever)
            },
        }
    }

    fn all_participants_completed(&self) -> bool {
        let participants = self.get_participants();
        // All participants have completed if each has spoken at least once in current round
        participants.iter().all(|p| self.round_speakers.contains(p))
    }

    fn increment_cycle(&mut self) {
        match &self.pattern {
            PolicyPattern::RatioPriority { .. } => {
                self.ratio_priority_cycle += 1;
            },
            PolicyPattern::Sequential { .. } => {
                // Sequential cycle is already incremented in determine_sequential_speaker
            },
        }
    }

    fn get_current_cycle(&self) -> usize {
        match &self.pattern {
            PolicyPattern::RatioPriority { .. } => self.ratio_priority_cycle,
            PolicyPattern::Sequential { .. } => self.sequential_cycle,
        }
    }

    fn reset_round_tracking(&mut self) {
        self.round_speakers.clear();
    }
}

impl UnifiedRatioPolicy {
    /// Ratio/priority selection
    fn determine_ratio_or_priority_speaker(&mut self, participants: &[String], weights: &[Weight]) -> Option<String> {
        let mut priority_candidates = Vec::new();
        let mut non_priority_participants = Vec::new();
        let mut non_priority_weights = Vec::new();
        for (i, (participant, weight)) in participants.iter().zip(weights).enumerate() {
            match weight {
                Weight::Priority => { if self.last_speaker.as_deref() != Some(participant) { priority_candidates.push((i, participant.clone())); } }
                Weight::Ratio(_) => { non_priority_participants.push(participant.clone()); non_priority_weights.push(weights[i].clone()); }
            }
        }

        // Cold start detection: if no one has spoken yet, start with first non-priority participant
        let total_words: usize = self.word_counts.values().sum();
        let is_cold_start = total_words == 0 && self.last_speaker.is_none();

        // On cold start, skip priority participants - let a non-priority participant speak first
        if !priority_candidates.is_empty() && !is_cold_start {
            let (_idx, speaker) = &priority_candidates[self.position % priority_candidates.len()];
            self.position = (self.position + 1) % priority_candidates.len();
            self.last_speaker = Some(speaker.clone());
            return Some(speaker.clone());
        }
        if non_priority_participants.is_empty() { return None; }
        let ratios: Vec<f64> = non_priority_weights.iter().map(|w| if let Weight::Ratio(r) = w { *r } else { 0.0 }).collect();
        let total_ratio: f64 = ratios.iter().sum();
        if total_ratio <= 0.0 { return None; }
        let mut actual_counts = Vec::new();
        for participant in &non_priority_participants { actual_counts.push(*self.word_counts.get(participant).unwrap_or(&0)); }
        let total_actual: usize = actual_counts.iter().sum();
        let mut best_speaker_idx = 0;
        let mut best_score = f64::MIN;
        for i in 0..non_priority_participants.len() {
            if non_priority_participants.len() > 1 && self.last_speaker.as_deref() == Some(&non_priority_participants[i]) { continue; }
            let ideal_words = if total_actual == 0 { ratios[i] / total_ratio } else { (ratios[i] / total_ratio) * (total_actual as f64) };
            let actual_words = actual_counts[i] as f64;
            let ratio_difference = (ideal_words - actual_words) / ratios[i].max(1.0);
            if ratio_difference > best_score { best_score = ratio_difference; best_speaker_idx = i; }
        }
        let speaker = non_priority_participants[best_speaker_idx].clone();
        self.last_speaker = Some(speaker.clone());
        Some(speaker)
    }

    /// Sequential selection
    fn determine_sequential_speaker(&mut self, participants: &[String], loop_forever: bool) -> Option<String> {
        if participants.is_empty() { return None; }
        let speaker = participants[self.position].clone();
        self.position += 1;
        if self.position >= participants.len() {
            if loop_forever { self.position = 0; self.sequential_cycle += 1; }
            else { self.position = participants.len() - 1; }
        }
        self.last_speaker = Some(speaker.clone());
        Some(speaker)
    }
}

impl Default for UnifiedRatioPolicy {
    fn default() -> Self { Self::new() }
}
