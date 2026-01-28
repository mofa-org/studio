//! TTS Batch Synthesis - Convert script segments to audio
//!
//! This module provides batch TTS synthesis functionality:
//! - Script segmentation by speaker
//! - Parallel TTS processing (interface ready for Dora)
//! - Audio file management
//! - Progress tracking
//! - Error handling and retry logic

use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinSet;

// ============================================================================
// DATA MODELS
// ============================================================================

/// Audio segment in a podcast script
#[derive(Debug, Clone, PartialEq)]
pub struct AudioSegment {
    /// Segment index in the full script
    pub index: usize,
    /// Speaker name
    pub speaker: String,
    /// Text content to synthesize
    pub text: String,
    /// Estimated duration in seconds (calculated from text length)
    pub estimated_duration_secs: f64,
    /// Path to generated audio file (after synthesis)
    pub audio_path: Option<PathBuf>,
}

/// TTS synthesis configuration
#[derive(Debug, Clone)]
pub struct TtsConfig {
    /// Base directory for audio output
    pub output_dir: PathBuf,
    /// Audio sample rate (Hz)
    pub sample_rate: u32,
    /// Audio channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Voice assignments per speaker (speaker_name -> voice_id)
    pub voice_assignments: HashMap<String, String>,
    /// Maximum concurrent TTS tasks
    pub max_concurrent_tasks: usize,
}

impl Default for TtsConfig {
    fn default() -> Self {
        let mut voice_assignments = HashMap::new();
        voice_assignments.insert("Host".to_string(), "default_voice".to_string());

        Self {
            output_dir: PathBuf::from("./output/audio"),
            sample_rate: 22050,
            channels: 1,
            voice_assignments,
            max_concurrent_tasks: 3,
        }
    }
}

/// TTS synthesis request
#[derive(Debug, Clone)]
pub struct TtsRequest {
    /// Audio segments to synthesize
    pub segments: Vec<AudioSegment>,
    /// TTS configuration
    pub config: TtsConfig,
}

/// TTS synthesis result
#[derive(Debug, Clone)]
pub struct TtsResult {
    /// Total segments processed
    pub total_segments: usize,
    /// Successfully synthesized segments
    pub successful_segments: usize,
    /// Failed segments
    pub failed_segments: usize,
    /// Total audio duration in seconds
    pub total_duration_secs: f64,
    /// Output directory containing all audio files
    pub output_dir: PathBuf,
    /// Synthesis duration in milliseconds
    pub duration_ms: u64,
}

/// Progress update during synthesis
#[derive(Debug, Clone)]
pub struct TtsProgress {
    /// Current segment index
    pub current_segment: usize,
    /// Total segments
    pub total_segments: usize,
    /// Current speaker
    pub speaker: String,
    /// Current segment text preview
    pub text_preview: String,
    /// Percentage complete (0-100)
    pub percentage: f64,
}

/// Progress callback type
pub type ProgressCallback = Arc<Mutex<Box<dyn Fn(TtsProgress) + Send + Sync>>>;

/// Errors that can occur during TTS synthesis
#[derive(Debug, Clone)]
pub enum TtsError {
    /// No segments to synthesize
    NoSegments,
    /// Output directory creation failed
    OutputDirectoryError(String),
    /// TTS engine error
    TtsEngineError(String),
    /// File write error
    FileWriteError(String),
    /// Invalid voice assignment
    InvalidVoice(String),
    /// Synthesis timeout
    Timeout,
    /// Other error
    Other(String),
}

impl std::fmt::Display for TtsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TtsError::NoSegments => write!(f, "No segments to synthesize"),
            TtsError::OutputDirectoryError(msg) => write!(f, "Output directory error: {}", msg),
            TtsError::TtsEngineError(msg) => write!(f, "TTS engine error: {}", msg),
            TtsError::FileWriteError(msg) => write!(f, "File write error: {}", msg),
            TtsError::InvalidVoice(voice) => write!(f, "Invalid voice: {}", voice),
            TtsError::Timeout => write!(f, "Synthesis timed out"),
            TtsError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for TtsError {}

// ============================================================================
// TRAIT DEFINITIONS
// ============================================================================

/// TTS engine trait for extensibility
pub trait TtsEngine: Send + Sync + Clone {
    /// Synthesize text to audio file
    fn synthesize(&self, text: &str, output_path: &Path, voice: &str) -> Result<(), TtsError>;

    /// Get engine name
    fn engine_name(&self) -> &str;
}

/// Enum to wrap different TTS engines
#[derive(Clone)]
pub enum TtsEngineWrapper {
    Mock(MockTtsEngine),
    Kokoro(DoraKokoroTtsEngine),
}

impl TtsEngine for TtsEngineWrapper {
    fn synthesize(&self, text: &str, output_path: &Path, voice: &str) -> Result<(), TtsError> {
        match self {
            TtsEngineWrapper::Mock(engine) => engine.synthesize(text, output_path, voice),
            TtsEngineWrapper::Kokoro(engine) => engine.synthesize(text, output_path, voice),
        }
    }

    fn engine_name(&self) -> &str {
        match self {
            TtsEngineWrapper::Mock(engine) => engine.engine_name(),
            TtsEngineWrapper::Kokoro(engine) => engine.engine_name(),
        }
    }
}

// ============================================================================
// SCRIPT SEGMENTATION
// ============================================================================

/// Script segmenter - splits refined script into audio segments
pub struct ScriptSegmenter {
    /// Regex to match speaker lines
    speaker_regex: Regex,
}

impl ScriptSegmenter {
    /// Create a new segmenter
    pub fn new() -> Result<Self, TtsError> {
        // Match speaker patterns: [Speaker]: text, @Speaker: text, or Speaker: text
        // Support English letters, numbers, @ symbol, underscores, hyphens, spaces, and Chinese characters
        let speaker_regex = Regex::new(r"^[@\[]?([A-Za-z0-9_\-\s\u4e00-\u9fff]+)\]?\s*:(.+)$")
            .map_err(|e| TtsError::Other(format!("Failed to create regex: {}", e)))?;

        Ok(Self { speaker_regex })
    }

    /// Parse refined script into audio segments
    ///
    /// Expected format:
    /// ```text
    /// [Speaker Name]: Dialogue text here.
    /// Another Speaker: More dialogue here.
    /// ```
    pub fn segment_script(&self, script: &str) -> Result<Vec<AudioSegment>, TtsError> {
        let mut segments = Vec::new();
        let lines: Vec<&str> = script.lines().collect();

        for (index, line) in lines.iter().enumerate() {
            let line = line.trim();

            // Skip empty lines and headers
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Try to match speaker pattern
            if let Some(captures) = self.speaker_regex.captures(line) {
                let speaker = captures.get(1).map(|m| m.as_str()).unwrap_or("Unknown").trim();
                let text = captures.get(2).map(|m| m.as_str()).unwrap_or("").trim();

                if !text.is_empty() {
                    let estimated_duration = self.estimate_duration(text);
                    segments.push(AudioSegment {
                        index,
                        speaker: speaker.to_string(),
                        text: text.to_string(),
                        estimated_duration_secs: estimated_duration,
                        audio_path: None,
                    });
                }
            }
        }

        if segments.is_empty() {
            return Err(TtsError::NoSegments);
        }

        Ok(segments)
    }

    /// Estimate audio duration from text length
    /// Average speaking rate: ~150 words per minute
    fn estimate_duration(&self, text: &str) -> f64 {
        let word_count = text.split_whitespace().count() as f64;
        let words_per_second = 150.0 / 60.0; // 150 words per minute
        word_count / words_per_second
    }
}

impl Default for ScriptSegmenter {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

// ============================================================================
// BATCH TTS SYNTHESIZER
// ============================================================================

/// Batch TTS synthesizer
pub struct BatchTtsSynthesizer<E: TtsEngine + 'static> {
    engine: E,
    segmenter: ScriptSegmenter,
}

impl<E: TtsEngine + 'static> BatchTtsSynthesizer<E> {
    /// Create a new batch synthesizer
    pub fn new(engine: E) -> Result<Self, TtsError> {
        Ok(Self {
            engine,
            segmenter: ScriptSegmenter::new()?,
        })
    }

    /// Synthesize all segments in a script
    pub async fn synthesize(
        &self,
        request: TtsRequest,
        progress: Option<ProgressCallback>,
    ) -> Result<TtsResult, TtsError> {
        let start_time = std::time::Instant::now();

        // Create output directory
        std::fs::create_dir_all(&request.config.output_dir)
            .map_err(|e| TtsError::OutputDirectoryError(format!("Failed to create output directory: {}", e)))?;

        // Create speaker directories
        for speaker in request.segments.iter().map(|s| &s.speaker) {
            let speaker_dir = request.config.output_dir.join(speaker);
            std::fs::create_dir_all(&speaker_dir)
                .map_err(|e| TtsError::OutputDirectoryError(format!("Failed to create speaker directory: {}", e)))?;
        }

        // Process segments in parallel
        let mut join_set = JoinSet::new();
        let segments = request.segments;
        let total_segments = segments.len();
        let config = Arc::new(request.config);
        let engine = self.engine.clone();
        let results = Arc::new(Mutex::new(Vec::new()));
        let progress_arc = progress.clone();

        // Launch synthesis tasks
        for segment in segments {
            let config_clone = config.clone();
            let engine_clone = engine.clone();
            let results_clone = results.clone();
            let progress_clone = progress_arc.clone();
            let total = total_segments;

            join_set.spawn(async move {
                let result = Self::synthesize_segment(
                    &engine_clone,
                    segment.clone(),
                    &config_clone,
                );

                // Store result
                if let Ok(ref seg) = result {
                    results_clone.lock().unwrap().push(seg.clone());
                }

                // Send progress update
                if let Some(progress_cb) = progress_clone {
                    let progress_cb = progress_cb.lock().unwrap();
                    let completed = results_clone.lock().unwrap().len();
                    progress_cb(TtsProgress {
                        current_segment: completed,
                        total_segments: total,
                        speaker: segment.speaker.clone(),
                        text_preview: Self::truncate_text(&segment.text, 50),
                        percentage: (completed as f64 / total as f64) * 100.0,
                    });
                }

                result
            });

            // Limit concurrent tasks
            while join_set.len() >= config.max_concurrent_tasks {
                if let Some(result) = join_set.join_next().await {
                    // Task completed, continue
                    let _ = result;
                }
            }
        }

        // Wait for remaining tasks
        while let Some(result) = join_set.join_next().await {
            let _ = result;
        }

        // Collect results
        let synthesized = results.lock().unwrap();
        let successful_segments = synthesized.len();
        let failed_segments = total_segments - successful_segments;
        let total_duration: f64 = synthesized.iter()
            .map(|s| s.estimated_duration_secs)
            .sum();

        Ok(TtsResult {
            total_segments,
            successful_segments,
            failed_segments,
            total_duration_secs: total_duration,
            output_dir: config.output_dir.clone(),
            duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Synthesize a single segment
    fn synthesize_segment(
        engine: &E,
        segment: AudioSegment,
        config: &TtsConfig,
    ) -> Result<AudioSegment, TtsError> {
        // Get voice for this speaker
        let voice = config.voice_assignments
            .get(&segment.speaker)
            .unwrap_or(&"default_voice".to_string())
            .clone();

        // Create output filename
        let filename = format!("segment_{:04}.wav", segment.index);
        let speaker_dir = config.output_dir.join(&segment.speaker);
        let output_path = speaker_dir.join(&filename);

        // Synthesize
        engine.synthesize(&segment.text, &output_path, &voice)?;

        Ok(AudioSegment {
            audio_path: Some(output_path),
            ..segment
        })
    }

    /// Truncate text to preview length
    fn truncate_text(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len])
        }
    }
}

// ============================================================================
// MOCK TTS ENGINE (for testing)
// ============================================================================

/// Mock TTS engine for testing without actual TTS
#[derive(Clone)]
pub struct MockTtsEngine;

impl TtsEngine for MockTtsEngine {
    fn synthesize(&self, text: &str, output_path: &Path, _voice: &str) -> Result<(), TtsError> {
        // Simulate TTS delay based on text length
        let delay_ms = (text.len() as u64).min(200) * 2;
        std::thread::sleep(Duration::from_millis(delay_ms));

        // Create a speaker directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TtsError::FileWriteError(format!("Failed to create directory: {}", e)))?;
        }

        // Calculate duration based on text length (rough estimate: 150 words/minute)
        let word_count = text.split_whitespace().count() as f64;
        let duration_secs = word_count / 150.0 * 60.0;
        let duration_secs = duration_secs.max(0.5); // Minimum 0.5 seconds

        // Generate simple test audio (sine wave)
        let sample_rate: u32 = 22050;
        let frequency = 440.0; // A4 note
        let num_samples = (duration_secs * sample_rate as f64) as usize;
        let num_channels: u16 = 1;
        let bits_per_sample: u16 = 16;

        // Generate PCM data (16-bit signed, mono)
        let mut audio_data = Vec::with_capacity(num_samples * num_channels as usize * (bits_per_sample / 8) as usize);

        for i in 0..num_samples {
            let t = i as f64 / sample_rate as f64;
            // Generate sine wave with amplitude scaled to 80% of max
            let amplitude = 32767.0 * 0.3;
            let sample = (amplitude * (2.0 * std::f64::consts::PI * frequency * t).sin()) as i16;

            // Add simple decay (fade out over duration)
            let decay = 1.0 - (i as f64 / num_samples as f64) * 0.5;
            let sample = (sample as f64 * decay) as i16;

            audio_data.push((sample & 0xFF) as u8);
            audio_data.push(((sample >> 8) & 0xFF) as u8);
        }

        let data_size = audio_data.len() as u32;

        // Write WAV file with header and audio data
        let mut wav_data = Vec::with_capacity(44 + audio_data.len());

        // RIFF header
        wav_data.extend_from_slice(b"RIFF");
        wav_data.extend_from_slice(&(36 + data_size).to_le_bytes());
        wav_data.extend_from_slice(b"WAVE");

        // fmt chunk
        wav_data.extend_from_slice(b"fmt ");
        wav_data.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
        wav_data.extend_from_slice(&1u16.to_le_bytes());  // Audio format (PCM)
        wav_data.extend_from_slice(&num_channels.to_le_bytes());
        wav_data.extend_from_slice(&sample_rate.to_le_bytes());
        let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
        wav_data.extend_from_slice(&byte_rate.to_le_bytes());
        let block_align = num_channels * (bits_per_sample / 8);
        wav_data.extend_from_slice(&(block_align as u16).to_le_bytes());
        wav_data.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        wav_data.extend_from_slice(b"data");
        wav_data.extend_from_slice(&data_size.to_le_bytes());

        // Audio data
        wav_data.extend_from_slice(&audio_data);

        std::fs::write(output_path, wav_data)
            .map_err(|e| TtsError::FileWriteError(format!("Failed to write WAV file: {}", e)))?;

        log::debug!("MockTtsEngine: synthesized {} chars -> {} ({:.2}s)",
            text.len(),
            output_path.display(),
            duration_secs
        );

        Ok(())
    }

    fn engine_name(&self) -> &str {
        "mock-tts"
    }
}

// ============================================================================
// DORA KOKORO TTS ENGINE
// ============================================================================

use std::process::Command;

/// Backend selection for Kokoro TTS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KokoroBackend {
    /// Auto-detect (MLX on macOS, CPU elsewhere)
    Auto,
    /// Force MLX backend (Apple Silicon only)
    Mlx,
    /// Force CPU backend (cross-platform)
    Cpu,
}

impl KokoroBackend {
    /// Convert to environment variable value
    fn as_env_value(&self) -> &'static str {
        match self {
            KokoroBackend::Auto => "auto",
            KokoroBackend::Mlx => "mlx",
            KokoroBackend::Cpu => "cpu",
        }
    }
}

/// Dora Kokoro TTS engine configuration
#[derive(Clone)]
pub struct DoraKokoroTtsEngine {
    /// Backend selection
    backend: KokoroBackend,
    /// Language code (en, zh, ja, ko)
    language: String,
    /// Voice name (e.g., "af_heart", "bf_alice")
    voice: String,
    /// Speed factor (0.5 - 2.0)
    speed: f32,
    /// Path to the batch synthesis Python script
    script_path: String,
}

impl DoraKokoroTtsEngine {
    /// Create a new Dora Kokoro TTS engine
    pub fn new() -> Self {
        Self {
            backend: KokoroBackend::Auto,
            language: "en".to_string(),
            voice: "af_heart".to_string(),
            speed: 1.0,
            // Path to the batch synthesis script (we'll create this)
            script_path: "node-hub/dora-kokoro-tts/batch_synthesize.py".to_string(),
        }
    }

    /// Set backend
    pub fn with_backend(mut self, backend: KokoroBackend) -> Self {
        self.backend = backend;
        self
    }

    /// Set language
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Set voice
    pub fn with_voice(mut self, voice: impl Into<String>) -> Self {
        self.voice = voice.into();
        self
    }

    /// Set speed (0.5 - 2.0)
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed.clamp(0.5, 2.0);
        self
    }

    /// Set custom script path
    pub fn with_script_path(mut self, path: impl Into<String>) -> Self {
        self.script_path = path.into();
        self
    }

    /// Get the Python script path (resolved relative to project root)
    fn resolve_script_path(&self) -> Result<PathBuf, TtsError> {
        // Start from current directory and find the script
        let cwd = std::env::current_dir()
            .map_err(|e| TtsError::TtsEngineError(format!("Failed to get current directory: {}", e)))?;

        // Try multiple possible paths
        let possible_paths = vec![
            cwd.join(&self.script_path),
            cwd.join("apps").join("mofa-cast").join(&self.script_path),
            cwd.join("..").join(&self.script_path),
        ];

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(TtsError::TtsEngineError(format!(
            "Kokoro batch script not found at: {} (searched in multiple locations)",
            self.script_path
        )))
    }
}

impl Default for DoraKokoroTtsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TtsEngine for DoraKokoroTtsEngine {
    fn synthesize(&self, text: &str, output_path: &Path, voice: &str) -> Result<(), TtsError> {
        // Validate input
        if text.trim().is_empty() {
            return Err(TtsError::TtsEngineError("Text is empty".to_string()));
        }

        // Create output directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TtsError::FileWriteError(format!("Failed to create directory: {}", e)))?;
        }

        // Resolve script path
        let script_path = self.resolve_script_path()?;

        // Determine which voice to use (parameter or default)
        let voice = if !voice.is_empty() { voice } else { &self.voice };

        log::info!(
            "Kokoro TTS: synthesizing {} chars with voice '{}', language '{}', backend '{:?}'",
            text.len(),
            voice,
            self.language,
            self.backend
        );

        // Call Python script with arguments
        let output = Command::new("python3")
            .arg(&script_path)
            .arg("--text")
            .arg(text)
            .arg("--output")
            .arg(output_path)
            .arg("--voice")
            .arg(voice)
            .arg("--language")
            .arg(&self.language)
            .arg("--speed")
            .arg(&self.speed.to_string())
            .arg("--backend")
            .arg(self.backend.as_env_value())
            .output()
            .map_err(|e| TtsError::TtsEngineError(format!("Failed to execute Python: {}", e)))?;

        // Check if script succeeded
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(TtsError::TtsEngineError(format!(
                "Kokoro TTS script failed:\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        // Verify output file was created
        if !output_path.exists() {
            return Err(TtsError::FileWriteError(format!(
                "Kokoro TTS: Output file not created: {}",
                output_path.display()
            )));
        }

        log::info!("Kokoro TTS: saved audio to {}", output_path.display());

        Ok(())
    }

    fn engine_name(&self) -> &str {
        "dora-kokoro-tts"
    }
}

// ============================================================================
// FACTORY
// ============================================================================

/// TTS engine factory
pub struct TtsFactory;

impl TtsFactory {
    /// Create a mock TTS engine for testing
    pub fn create_mock_engine() -> MockTtsEngine {
        MockTtsEngine
    }

    /// Create a Dora Kokoro TTS engine for local synthesis
    ///
    /// # Example
    /// ```rust
    /// let engine = TtsFactory::create_dora_kokoro_engine()
    ///     .with_backend(KokoroBackend::Auto)
    ///     .with_language("en")
    ///     .with_voice("af_heart")
    ///     .with_speed(1.0);
    /// ```
    pub fn create_dora_kokoro_engine() -> DoraKokoroTtsEngine {
        DoraKokoroTtsEngine::new()
    }

    // TODO: Add more local TTS engines in future:
    // - pub fn create_primespeech_engine(...) -> PrimeSpeechEngine
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_segmentation() {
        let segmenter = ScriptSegmenter::new().unwrap();

        let script = r#"[Host]: Welcome to our podcast today.
[Guest]: Thank you for having me.
[Host]: Let's get started."#;

        let segments = segmenter.segment_script(script).unwrap();

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].speaker, "Host");
        assert_eq!(segments[1].speaker, "Guest");
        assert_eq!(segments[2].speaker, "Host");
    }

    #[test]
    fn test_estimated_duration() {
        let segmenter = ScriptSegmenter::new().unwrap();

        let text = "This is a test sentence with ten words here.";
        let duration = segmenter.estimate_duration(text);

        // 10 words / (150 words / 60 seconds) = 4 seconds
        assert!((duration - 4.0).abs() < 0.5);
    }

    #[test]
    fn test_mock_tts_engine() {
        let engine = MockTtsEngine;
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_tts.wav");

        let result = engine.synthesize("Hello world", &output_path, "default");

        assert!(result.is_ok());
        assert!(output_path.exists());

        // Cleanup
        std::fs::remove_file(&output_path).ok();
    }

    #[tokio::test]
    async fn test_batch_synthesis() {
        let engine = TtsFactory::create_mock_engine();
        let synthesizer = BatchTtsSynthesizer::new(engine).unwrap();

        let script = r#"[Host]: Welcome to the show.
[Guest]: Great to be here.
[Host]: Let's begin."#;

        let segments = ScriptSegmenter::new().unwrap().segment_script(script).unwrap();

        let temp_dir = std::env::temp_dir();
        let config = TtsConfig {
            output_dir: temp_dir.join("test_tts_batch"),
            ..Default::default()
        };

        let request = TtsRequest { segments, config };

        let result = synthesizer.synthesize(request, None).await.unwrap();

        assert_eq!(result.total_segments, 3);
        assert_eq!(result.successful_segments, 3);
        assert_eq!(result.failed_segments, 0);

        // Cleanup
        std::fs::remove_dir_all(temp_dir.join("test_tts_batch")).ok();
    }
}
