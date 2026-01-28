//! Transcript Parser - Parse various chat transcript formats
//!
//! This module provides parsers for different chat formats:
//! - Plain text (speaker: message)
//! - JSON (OpenAI chat format)
//! - Markdown (GitHub discussions)

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::transcript_parser::TranscriptFormat::*;

// ============================================================================
// DATA MODELS
// ============================================================================

/// Complete transcript with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub messages: Vec<Message>,
    pub metadata: Metadata,
}

/// Individual message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub speaker: String,
    pub text: String,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Metadata about the transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub participants: Vec<String>,
    pub format: TranscriptFormat,
}

/// Supported transcript formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranscriptFormat {
    PlainText,
    Json,
    Markdown,
    Unknown,
}

/// Speaker information for TTS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speaker {
    pub name: String,
    pub message_count: usize,
    pub total_characters: usize,
}

// ============================================================================
// TRAIT DEFINITIONS
// ============================================================================

/// Parser trait for different transcript formats
pub trait TranscriptParser {
    /// Parse content into a Transcript
    fn parse(&self, content: &str) -> Result<Transcript, ParseError>;

    /// Check if this parser can handle the content
    fn can_parse(&self, content: &str) -> bool;

    /// Get the format this parser handles
    fn format(&self) -> TranscriptFormat;
}

/// Errors that can occur during parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidFormat(String),
    NoMessagesFound,
    InvalidJson(String),
    InvalidTimestamp(String),
    Other(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::NoMessagesFound => write!(f, "No messages found in transcript"),
            ParseError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            ParseError::InvalidTimestamp(msg) => write!(f, "Invalid timestamp: {}", msg),
            ParseError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

// ============================================================================
// PLAIN TEXT PARSER
// ============================================================================

/// Parser for plain text format (speaker: message)
///
/// Example:
/// ```text
/// Alice: Hello, how are you?
/// Bob: I'm doing great!
/// Alice: That's wonderful to hear.
/// ```
pub struct PlainTextParser {
    /// Regex pattern for matching "speaker: message" format
    pattern: Regex,
}

impl PlainTextParser {
    pub fn new() -> Self {
        // Match "speaker: message" at the start of a line
        // Speaker can contain letters, numbers, spaces, and common punctuation
        let pattern = Regex::new(r"^([A-Za-z0-9\s_\.]+):\s*(.+)$").unwrap();
        Self { pattern }
    }

    /// Extract speakers from messages
    fn extract_speakers(&self, messages: &[Message]) -> Vec<String> {
        let mut speakers = HashSet::new();
        for msg in messages {
            speakers.insert(msg.speaker.clone());
        }
        let mut speaker_list: Vec<_> = speakers.into_iter().collect();
        speaker_list.sort();
        speaker_list
    }
}

impl Default for PlainTextParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptParser for PlainTextParser {
    fn parse(&self, content: &str) -> Result<Transcript, ParseError> {
        let mut messages = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(caps) = self.pattern.captures(line) {
                let speaker = caps.get(1).unwrap().as_str().trim().to_string();
                let text = caps.get(2).unwrap().as_str().trim().to_string();

                if !text.is_empty() {
                    messages.push(Message {
                        speaker,
                        text,
                        timestamp: None, // Plain text doesn't have timestamps
                    });
                }
            }
        }

        if messages.is_empty() {
            return Err(ParseError::NoMessagesFound);
        }

        let speakers = self.extract_speakers(&messages);

        Ok(Transcript {
            messages,
            metadata: Metadata {
                title: None,
                date: None,
                participants: speakers,
                format: TranscriptFormat::PlainText,
            },
        })
    }

    fn can_parse(&self, content: &str) -> bool {
        // Check if at least 30% of non-empty lines match the pattern
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() {
            return false;
        }

        let matching = lines.iter().filter(|l| self.pattern.is_match(l)).count();
        let ratio = matching as f64 / lines.len() as f64;

        ratio >= 0.3
    }

    fn format(&self) -> TranscriptFormat {
        TranscriptFormat::PlainText
    }
}

// ============================================================================
// JSON PARSER
// ============================================================================

/// Parser for JSON format (OpenAI chat format)
///
/// Example:
/// ```json
/// [
///   {"role": "user", "content": "Hello"},
///   {"role": "assistant", "content": "Hi there!"}
/// ]
/// ```
#[derive(Debug, Serialize, Deserialize)]
struct JsonMessage {
    role: String,
    content: String,
    #[serde(default)]
    timestamp: Option<String>,
}

pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }

    fn parse_timestamp(&self, ts: &str) -> Result<DateTime<Utc>, ParseError> {
        // Try ISO 8601 format first
        if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try common formats
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S",
            "%Y/%m/%d %H:%M:%S",
        ];

        for fmt in &formats {
            if let Ok(dt) = DateTime::parse_from_str(ts, fmt) {
                return Ok(dt.with_timezone(&Utc));
            }
        }

        Err(ParseError::InvalidTimestamp(ts.to_string()))
    }

    fn extract_speakers(&self, messages: &[Message]) -> Vec<String> {
        let mut speakers = HashSet::new();
        for msg in messages {
            speakers.insert(msg.speaker.clone());
        }
        let mut speaker_list: Vec<_> = speakers.into_iter().collect();
        speaker_list.sort();
        speaker_list
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptParser for JsonParser {
    fn parse(&self, content: &str) -> Result<Transcript, ParseError> {
        // Try to parse as JSON array
        let json_msgs: Vec<JsonMessage> =
            serde_json::from_str(content).map_err(|e| ParseError::InvalidJson(e.to_string()))?;

        if json_msgs.is_empty() {
            return Err(ParseError::NoMessagesFound);
        }

        let mut messages = Vec::new();

        for json_msg in json_msgs {
            let text = json_msg.content.trim().to_string();
            if text.is_empty() {
                continue;
            }

            let timestamp = if let Some(ts) = json_msg.timestamp {
                Some(self.parse_timestamp(&ts)?)
            } else {
                None
            };

            messages.push(Message {
                speaker: json_msg.role,
                text,
                timestamp,
            });
        }

        if messages.is_empty() {
            return Err(ParseError::NoMessagesFound);
        }

        let speakers = self.extract_speakers(&messages);

        Ok(Transcript {
            messages,
            metadata: Metadata {
                title: None,
                date: None,
                participants: speakers,
                format: TranscriptFormat::Json,
            },
        })
    }

    fn can_parse(&self, content: &str) -> bool {
        // Check if content starts with [ or {
        let trimmed = content.trim();
        trimmed.starts_with('[') || trimmed.starts_with('{')
    }

    fn format(&self) -> TranscriptFormat {
        TranscriptFormat::Json
    }
}

// ============================================================================
// MARKDOWN PARSER
// ============================================================================

/// Parser for Markdown format (GitHub discussions)
///
/// Example:
/// ```markdown
/// ### @alice
/// Hello, how are you?
///
/// ### @bob
/// I'm doing great!
/// ```
pub struct MarkdownParser {
    /// Regex pattern for matching "### @speaker" format
    pattern: Regex,
}

impl MarkdownParser {
    pub fn new() -> Self {
        // Match markdown headers with optional @mention
        // Pattern: "### @speaker" or "### speaker"
        // Use (?m) for multiline mode so ^ and $ match line boundaries
        let pattern = Regex::new(r"(?m)^#+\s*@?([A-Za-z0-9_\-]+)\s*$").unwrap();
        Self { pattern }
    }

    fn extract_speakers(&self, messages: &[Message]) -> Vec<String> {
        let mut speakers = HashSet::new();
        for msg in messages {
            speakers.insert(msg.speaker.clone());
        }
        let mut speaker_list: Vec<_> = speakers.into_iter().collect();
        speaker_list.sort();
        speaker_list
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptParser for MarkdownParser {
    fn parse(&self, content: &str) -> Result<Transcript, ParseError> {
        let mut messages = Vec::new();
        let mut current_speaker: Option<String> = None;
        let mut current_text = String::new();

        for line in content.lines() {
            let line = line.trim();

            // Check if this is a speaker header
            if let Some(caps) = self.pattern.captures(line) {
                // Save previous message if exists
                if let (Some(speaker), true) = (&current_speaker, !current_text.trim().is_empty()) {
                    messages.push(Message {
                        speaker: speaker.clone(),
                        text: current_text.trim().to_string(),
                        timestamp: None,
                    });
                }

                // Start new message
                current_speaker = Some(caps.get(1).unwrap().as_str().to_string());
                current_text = String::new();
            } else if let Some(_) = &current_speaker {
                // Accumulate text for current speaker
                if !line.is_empty() && !line.starts_with('#') {
                    if !current_text.is_empty() {
                        current_text.push(' ');
                    }
                    current_text.push_str(line);
                }
            }
        }

        // Don't forget the last message
        if let (Some(speaker), true) = (&current_speaker, !current_text.trim().is_empty()) {
            messages.push(Message {
                speaker: speaker.clone(),
                text: current_text.trim().to_string(),
                timestamp: None,
            });
        }

        if messages.is_empty() {
            return Err(ParseError::NoMessagesFound);
        }

        let speakers = self.extract_speakers(&messages);

        Ok(Transcript {
            messages,
            metadata: Metadata {
                title: None,
                date: None,
                participants: speakers,
                format: TranscriptFormat::Markdown,
            },
        })
    }

    fn can_parse(&self, content: &str) -> bool {
        // Check if content contains markdown headers with speakers
        let header_count = self.pattern.find_iter(content).count();

        // Need at least 2 different speakers
        if header_count < 2 {
            return false;
        }

        // Check if lines are present
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() < 4 {
            return false;
        }

        true
    }

    fn format(&self) -> TranscriptFormat {
        TranscriptFormat::Markdown
    }
}

// ============================================================================
// PARSER FACTORY
// ============================================================================

/// Factory for auto-detecting and parsing transcripts
pub struct ParserFactory {
    parsers: Vec<Box<dyn TranscriptParser>>,
}

impl ParserFactory {
    pub fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(JsonParser::new()),
                Box::new(MarkdownParser::new()),
                Box::new(PlainTextParser::new()),
            ],
        }
    }

    /// Auto-detect format and parse
    pub fn parse_auto(&self, content: &str) -> Result<Transcript, ParseError> {
        for parser in &self.parsers {
            if parser.can_parse(content) {
                return parser.parse(content);
            }
        }

        Err(ParseError::InvalidFormat(
            "Unable to auto-detect transcript format".to_string(),
        ))
    }

    /// Get detected format without parsing
    pub fn detect_format(&self, content: &str) -> TranscriptFormat {
        for parser in &self.parsers {
            if parser.can_parse(content) {
                return parser.format();
            }
        }
        TranscriptFormat::Unknown
    }

    /// Parse with specific format
    pub fn parse_with_format(
        &self,
        content: &str,
        format: TranscriptFormat,
    ) -> Result<Transcript, ParseError> {
        for parser in &self.parsers {
            if parser.format() == format {
                return parser.parse(content);
            }
        }
        Err(ParseError::InvalidFormat(format!(
            "No parser available for format: {:?}",
            format
        )))
    }
}

impl Default for ParserFactory {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TRANSCRIPT EXTENSIONS
// ============================================================================

impl Transcript {
    /// Get all unique speakers with statistics
    pub fn get_speakers(&self) -> Vec<Speaker> {
        let mut speaker_stats: std::collections::HashMap<String, Speaker> = std::collections::HashMap::new();

        for msg in &self.messages {
            let entry = speaker_stats
                .entry(msg.speaker.clone())
                .or_insert_with(|| Speaker {
                    name: msg.speaker.clone(),
                    message_count: 0,
                    total_characters: 0,
                });

            entry.message_count += 1;
            entry.total_characters += msg.text.len();
        }

        let mut speakers: Vec<_> = speaker_stats.into_values().collect();
        speakers.sort_by(|a, b| a.name.cmp(&b.name));
        speakers
    }

    /// Get total character count
    pub fn total_characters(&self) -> usize {
        self.messages.iter().map(|m| m.text.len()).sum()
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Filter messages by speaker
    pub fn filter_by_speaker(&self, speaker: &str) -> Vec<&Message> {
        self.messages
            .iter()
            .filter(|m| m.speaker == speaker)
            .collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_parser() {
        let parser = PlainTextParser::new();
        let content = r#"Alice: Hello, how are you?
Bob: I'm doing great!
Alice: That's wonderful to hear."#;

        let result = parser.parse(content);
        assert!(result.is_ok());

        let transcript = result.unwrap();
        assert_eq!(transcript.messages.len(), 3);
        assert_eq!(transcript.messages[0].speaker, "Alice");
        assert_eq!(transcript.messages[1].speaker, "Bob");
        assert_eq!(transcript.metadata.format, TranscriptFormat::PlainText);
    }

    #[test]
    fn test_json_parser() {
        let parser = JsonParser::new();
        let content = r#"[
            {"role": "user", "content": "Hello"},
            {"role": "assistant", "content": "Hi there!"}
        ]"#;

        let result = parser.parse(content);
        assert!(result.is_ok());

        let transcript = result.unwrap();
        assert_eq!(transcript.messages.len(), 2);
        assert_eq!(transcript.messages[0].speaker, "user");
        assert_eq!(transcript.metadata.format, TranscriptFormat::Json);
    }

    #[test]
    fn test_markdown_parser() {
        let parser = MarkdownParser::new();
        let content = r#"### @alice
Hello, how are you?

### @bob
I'm doing great!"#;

        let result = parser.parse(content);
        assert!(result.is_ok());

        let transcript = result.unwrap();
        assert_eq!(transcript.messages.len(), 2);
        assert_eq!(transcript.messages[0].speaker, "alice");
        assert_eq!(transcript.metadata.format, TranscriptFormat::Markdown);
    }

    #[test]
    fn test_auto_detection() {
        let factory = ParserFactory::new();

        // Plain text
        let plain = "Alice: Hello\nBob: Hi";
        assert_eq!(factory.detect_format(plain), TranscriptFormat::PlainText);

        // JSON
        let json = r#"[{"role": "user", "content": "Hi"}]"#;
        assert_eq!(factory.detect_format(json), TranscriptFormat::Json);

        // Markdown - check if parser can parse it
        let md_parser = MarkdownParser::new();
        let md = r#"### @alice
Hello, how are you?

### @bob
I'm doing great!"#;

        // First verify MarkdownParser can handle this
        assert!(md_parser.can_parse(md), "MarkdownParser should be able to parse this");

        // Then check factory detection
        assert_eq!(factory.detect_format(md), TranscriptFormat::Markdown);
    }

    #[test]
    fn test_get_speakers() {
        let parser = PlainTextParser::new();
        let content = r#"Alice: First message
Bob: Second message
Alice: Third message"#;

        let transcript = parser.parse(content).unwrap();
        let speakers = transcript.get_speakers();

        assert_eq!(speakers.len(), 2);
        assert_eq!(speakers[0].name, "Alice");
        assert_eq!(speakers[0].message_count, 2);
        assert_eq!(speakers[1].name, "Bob");
        assert_eq!(speakers[1].message_count, 1);
    }
}
