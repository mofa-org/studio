//! Audio Mixing and Export - Combine segments into final podcast
//!
//! This module provides audio mixing functionality:
//! - Concatenate audio segments in order
//! - Volume normalization
//! - Add silence between segments
//! - Export as WAV or MP3
//! - Metadata support

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

// ============================================================================
// DATA MODELS
// ============================================================================

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// WAV format (uncompressed)
    Wav,
    /// MP3 format (compressed)
    Mp3,
}

/// MP3 bitrate options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mp3Bitrate {
    /// 128 kbps (Good quality, ~1MB/min)
    Kbps128,
    /// 192 kbps (High quality, ~1.5MB/min) - Recommended
    Kbps192,
    /// 256 kbps (Very high quality, ~2MB/min)
    Kbps256,
    /// 320 kbps (Maximum quality, ~2.5MB/min)
    Kbps320,
}

impl Mp3Bitrate {
    /// Get bitrate value in kbps
    pub fn kbps(&self) -> u32 {
        match self {
            Mp3Bitrate::Kbps128 => 128,
            Mp3Bitrate::Kbps192 => 192,
            Mp3Bitrate::Kbps256 => 256,
            Mp3Bitrate::Kbps320 => 320,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Mp3Bitrate::Kbps128 => "128 kbps (Good)",
            Mp3Bitrate::Kbps192 => "192 kbps (High)",
            Mp3Bitrate::Kbps256 => "256 kbps (Very High)",
            Mp3Bitrate::Kbps320 => "320 kbps (Max)",
        }
    }
}

/// Audio mixing configuration
#[derive(Debug, Clone)]
pub struct MixerConfig {
    /// Output file path (without extension)
    pub output_path: PathBuf,
    /// Export format
    pub export_format: ExportFormat,
    /// MP3 bitrate (only used if export_format is MP3)
    pub mp3_bitrate: Mp3Bitrate,
    /// Normalize audio to target dB (-14.0 = EBU R128 standard)
    pub normalize_dB: f32,
    /// Silence duration between segments (in seconds)
    pub silence_duration_secs: f64,
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Bits per sample
    pub bits_per_sample: u16,
    /// Metadata
    pub metadata: AudioMetadata,
}

impl Default for MixerConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("./output/mofa-cast/podcast"),
            export_format: ExportFormat::Wav,  // Default to WAV
            mp3_bitrate: Mp3Bitrate::Kbps192,  // Default to 192 kbps (recommended)
            normalize_dB: -14.0,
            silence_duration_secs: 0.5,
            sample_rate: 22050,
            channels: 1,
            bits_per_sample: 16,
            metadata: AudioMetadata::default(),
        }
    }
}

/// Audio metadata
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    /// Title
    pub title: Option<String>,
    /// Artist/Author
    pub artist: Option<String>,
    /// Album
    pub album: Option<String>,
    /// Year
    pub year: Option<String>,
    /// Comment/Description
    pub comment: Option<String>,
}

/// Audio mixing request
#[derive(Debug, Clone)]
pub struct MixerRequest {
    /// Input audio segments (in order)
    pub segments: Vec<AudioSegmentInfo>,
    /// Mixer configuration
    pub config: MixerConfig,
}

/// Information about an audio segment
#[derive(Debug, Clone)]
pub struct AudioSegmentInfo {
    /// Path to audio file
    pub path: PathBuf,
    /// Speaker name
    pub speaker: String,
    /// Duration in seconds
    pub duration_secs: f64,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
}

/// Audio mixing result
#[derive(Debug, Clone)]
pub struct MixerResult {
    /// Output file path (with extension)
    pub output_file: PathBuf,
    /// Total duration in seconds
    pub total_duration_secs: f64,
    /// Number of segments mixed
    pub segment_count: usize,
    /// File size in bytes
    pub file_size_bytes: u64,
    /// Mixing duration in milliseconds
    pub duration_ms: u64,
}

/// Errors that can occur during mixing
#[derive(Debug, Clone)]
pub enum MixerError {
    /// No segments to mix
    NoSegments,
    /// Audio file not found
    FileNotFound(String),
    /// Invalid audio format
    InvalidAudioFormat(String),
    /// Mismatched audio formats
    FormatMismatch(String),
    /// I/O error
    IoError(String),
    /// Invalid WAV header
    InvalidWavHeader(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for MixerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MixerError::NoSegments => write!(f, "No segments to mix"),
            MixerError::FileNotFound(path) => write!(f, "File not found: {}", path),
            MixerError::InvalidAudioFormat(msg) => write!(f, "Invalid audio format: {}", msg),
            MixerError::FormatMismatch(msg) => write!(f, "Format mismatch: {}", msg),
            MixerError::IoError(msg) => write!(f, "I/O error: {}", msg),
            MixerError::InvalidWavHeader(msg) => write!(f, "Invalid WAV header: {}", msg),
            MixerError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for MixerError {}

// ============================================================================
// WAV FILE STRUCTURES
// ============================================================================

/// WAV file header (44 bytes)
#[derive(Debug, Clone)]
#[repr(C)]
struct WavHeader {
    // RIFF header
    riff: [u8; 4],          // "RIFF"
    file_size: u32,         // File size - 8
    wave: [u8; 4],          // "WAVE"

    // fmt chunk
    fmt_id: [u8; 4],        // "fmt "
    fmt_size: u32,          // Chunk size (16 for PCM)
    audio_format: u16,      // 1 = PCM
    channels: u16,          // Number of channels
    sample_rate: u32,       // Sample rate
    byte_rate: u32,         // sample_rate * channels * bits_per_sample / 8
    block_align: u16,       // channels * bits_per_sample / 8
    bits_per_sample: u16,   // Bits per sample

    // data chunk
    data_id: [u8; 4],       // "data"
    data_size: u32,         // Data size in bytes
}

impl WavHeader {
    /// Create a new WAV header
    fn new(config: &MixerConfig, data_size: u32) -> Self {
        let byte_rate = config.sample_rate * config.channels as u32 * config.bits_per_sample as u32 / 8;
        let block_align = config.channels * config.bits_per_sample / 8;

        Self {
            riff: *b"RIFF",
            file_size: 36 + data_size,
            wave: *b"WAVE",
            fmt_id: *b"fmt ",
            fmt_size: 16,
            audio_format: 1, // PCM
            channels: config.channels,
            sample_rate: config.sample_rate,
            byte_rate,
            block_align,
            bits_per_sample: config.bits_per_sample,
            data_id: *b"data",
            data_size,
        }
    }

    /// Parse WAV header from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, MixerError> {
        if bytes.len() < 44 {
            return Err(MixerError::InvalidWavHeader("File too short".to_string()));
        }

        // Check RIFF and WAVE identifiers
        if &bytes[0..4] != b"RIFF" {
            return Err(MixerError::InvalidWavHeader("Missing RIFF identifier".to_string()));
        }
        if &bytes[8..12] != b"WAVE" {
            return Err(MixerError::InvalidWavHeader("Missing WAVE identifier".to_string()));
        }
        if &bytes[12..16] != b"fmt " {
            return Err(MixerError::InvalidWavHeader("Missing fmt chunk".to_string()));
        }
        if &bytes[36..40] != b"data" {
            return Err(MixerError::InvalidWavHeader("Missing data chunk".to_string()));
        }

        Ok(unsafe {
            // SAFETY: We've validated the structure above and bytes is aligned
            std::ptr::read(bytes.as_ptr() as *const WavHeader)
        })
    }
}

// ============================================================================
// AUDIO MIXER
// ============================================================================

/// Audio mixer
pub struct AudioMixer;

impl AudioMixer {
    /// Create a new audio mixer
    pub fn new() -> Self {
        Self
    }

    /// Mix audio segments into a single file
    pub fn mix(&self, request: MixerRequest) -> Result<MixerResult, MixerError> {
        let start_time = std::time::Instant::now();

        if request.segments.is_empty() {
            return Err(MixerError::NoSegments);
        }

        // Read all audio data from segments
        let mut all_audio_data = Vec::new();
        let mut total_duration = 0.0;

        // Calculate silence samples
        let silence_samples = (request.config.silence_duration_secs * request.config.sample_rate as f64) as usize;
        let silence_bytes = silence_samples * request.config.channels as usize * (request.config.bits_per_sample / 8) as usize;
        let silence_buffer = vec![0u8; silence_bytes];

        for (i, segment) in request.segments.iter().enumerate() {
            // Read audio file
            let mut audio_data = Self::read_wav_file(&segment.path, &request.config)?;

            // Apply volume normalization if enabled
            if request.config.normalize_dB != 0.0 {
                audio_data = Self::normalize_audio(&audio_data, request.config.normalize_dB)?;
            }

            // Add audio data
            all_audio_data.extend_from_slice(&audio_data);
            total_duration += segment.duration_secs;

            // Add silence between segments (but not after the last one)
            if i < request.segments.len() - 1 {
                all_audio_data.extend_from_slice(&silence_buffer);
                total_duration += request.config.silence_duration_secs;
            }
        }

        // Create output file based on format
        let (output_file, output_path) = match request.config.export_format {
            ExportFormat::Wav => {
                let file = format!("{}.wav", request.config.output_path.display());
                let path = PathBuf::from(&file);
                Self::write_wav_file(&path, &request.config, &all_audio_data)?;
                (file, path)
            }
            ExportFormat::Mp3 => {
                let file = format!("{}.mp3", request.config.output_path.display());
                let path = PathBuf::from(&file);
                Self::write_mp3_file(&path, &request.config, &all_audio_data)?;
                (file, path)
            }
        };

        // Get file size
        let file_size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(MixerResult {
            output_file: output_path,
            total_duration_secs: total_duration,
            segment_count: request.segments.len(),
            file_size_bytes: file_size,
            duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Read audio data from a WAV file
    fn read_wav_file(path: &Path, config: &MixerConfig) -> Result<Vec<u8>, MixerError> {
        // Read entire file
        let mut file = File::open(path)
            .map_err(|e| MixerError::FileNotFound(format!("{}: {}", path.display(), e)))?;

        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)
            .map_err(|e| MixerError::IoError(format!("Failed to read file: {}", e)))?;

        // Parse header
        let header = WavHeader::from_bytes(&file_data)?;

        // Validate format matches config
        if header.channels != config.channels {
            return Err(MixerError::FormatMismatch(
                format!("Channel mismatch: expected {}, got {}", config.channels, header.channels)
            ));
        }
        if header.sample_rate != config.sample_rate {
            return Err(MixerError::FormatMismatch(
                format!("Sample rate mismatch: expected {}, got {}", config.sample_rate, header.sample_rate)
            ));
        }
        if header.bits_per_sample != config.bits_per_sample {
            return Err(MixerError::FormatMismatch(
                format!("Bits per sample mismatch: expected {}, got {}", config.bits_per_sample, header.bits_per_sample)
            ));
        }

        // Extract audio data (skip 44-byte header)
        let audio_data = file_data[44..].to_vec();

        Ok(audio_data)
    }

    /// Normalize audio data to target dB level using RMS
    /// target_dB: Target level in dB (typically -14.0 for EBU R128 standard)
    fn normalize_audio(audio_data: &[u8], target_dB: f32) -> Result<Vec<u8>, MixerError> {
        // Assume 16-bit mono audio (based on our default config)
        let samples = audio_data.len() / 2;
        let mut audio_i16 = Vec::with_capacity(samples);

        // Convert bytes to i16 samples
        for chunk in audio_data.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            audio_i16.push(sample);
        }

        // Calculate RMS (Root Mean Square)
        let sum_squares: f64 = audio_i16.iter()
            .map(|&s| (s as f64) * (s as f64))
            .sum();

        let rms = (sum_squares / samples as f64).sqrt();

        // Avoid division by zero
        if rms < 1e-6 {
            return Ok(audio_data.to_vec());
        }

        // Calculate target RMS from dB
        // dB = 20 * log10(rms / 32768.0)
        // target_rms = 32768.0 * 10^(dB / 20)
        let target_rms = 32768.0 * 10_f64.powf(target_dB as f64 / 20.0);

        // Calculate amplification factor
        let amplification = (target_rms / rms) as f64;

        // Clamp amplification to avoid extreme values
        let amplification = amplification.max(0.1).min(10.0);

        // Apply normalization
        let normalized: Vec<u8> = audio_i16.iter()
            .map(|&sample| {
                let normalized_sample = (sample as f64 * amplification) as i16;
                // Clamp to i16 range
                let clamped = normalized_sample.max(i16::MIN).min(i16::MAX);
                clamped.to_le_bytes().to_vec()
            })
            .flatten()
            .collect();

        ::log::info!("Audio normalized: RMS {:.2} → {:.2} (amplification: {:.2}x)",
            20.0 * (rms / 32768.0).log10(),
            target_dB,
            amplification
        );

        Ok(normalized)
    }

    /// Write audio data to a WAV file
    fn write_wav_file(path: &Path, config: &MixerConfig, audio_data: &[u8]) -> Result<(), MixerError> {
        let data_size = audio_data.len() as u32;
        let header = WavHeader::new(config, data_size);

        let mut file = File::create(path)
            .map_err(|e| MixerError::IoError(format!("Failed to create file: {}", e)))?;

        // Write header
        unsafe {
            let header_bytes = std::slice::from_raw_parts(
                &header as *const WavHeader as *const u8,
                std::mem::size_of::<WavHeader>()
            );
            file.write_all(header_bytes)
                .map_err(|e| MixerError::IoError(format!("Failed to write header: {}", e)))?;
        }

        // Write audio data
        file.write_all(audio_data)
            .map_err(|e| MixerError::IoError(format!("Failed to write audio data: {}", e)))?;

        Ok(())
    }

    /// Write audio data to an MP3 file using ffmpeg (external tool)
    /// This is simpler and more reliable than using mp3lame-encoder directly
    fn write_mp3_file(path: &Path, config: &MixerConfig, audio_data: &[u8]) -> Result<(), MixerError> {
        // First write to a temporary WAV file
        let temp_wav = path.with_extension("wav");
        Self::write_wav_file(&temp_wav, config, audio_data)?;

        // Convert WAV to MP3 using ffmpeg with ID3 tags
        let bitrate = config.mp3_bitrate.kbps();

        // Build ffmpeg command
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-y")  // Overwrite output file
            .arg("-i")
            .arg(&temp_wav)
            .arg("-codec:a")
            .arg("libmp3lame")
            .arg("-b:a")
            .arg(format!("{}k", bitrate))
            .arg("-qscale:a")
            .arg("2");  // High quality VBR

        // Add ID3 metadata tags if provided
        if let Some(ref title) = config.metadata.title {
            cmd.arg("-metadata").arg(format!("title={}", title));
        }
        if let Some(ref artist) = config.metadata.artist {
            cmd.arg("-metadata").arg(format!("artist={}", artist));
        }
        if let Some(ref album) = config.metadata.album {
            cmd.arg("-metadata").arg(format!("album={}", album));
        }
        if let Some(ref year) = config.metadata.year {
            cmd.arg("-metadata").arg(format!("year={}", year));
        }
        if let Some(ref comment) = config.metadata.comment {
            cmd.arg("-metadata").arg(format!("comment={}", comment));
        }

        // Add encoding metadata
        cmd.arg("-metadata")
            .arg(format!("encoded_by=MoFA Cast v0.6.2"));

        cmd.arg(path);

        let output = cmd.output();

        match output {
            Ok(_) => {
                // Clean up temporary WAV file
                let _ = std::fs::remove_file(&temp_wav);

                ::log::info!("MP3 export with ID3 tags completed: {}", path.display());

                Ok(())
            }
            Err(e) => {
                // Clean up temp file on error too
                let _ = std::fs::remove_file(&temp_wav);
                Err(MixerError::IoError(format!("ffmpeg conversion failed: {}. Is ffmpeg installed?", e)))
            }
        }
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Helper: Create a minimal WAV file for testing
    fn create_test_wav(path: &Path, duration_secs: f64) -> Result<(), MixerError> {
        let config = MixerConfig::default();
        let samples = (duration_secs * config.sample_rate as f64) as usize;
        let data_size = (samples * config.channels as usize * (config.bits_per_sample / 8) as usize) as u32;

        let header = WavHeader::new(&config, data_size);
        let audio_data = vec![0u8; data_size as usize];

        let mut file = File::create(path)
            .map_err(|e| MixerError::IoError(format!("Failed to create test file: {}", e)))?;

        unsafe {
            let header_bytes = std::slice::from_raw_parts(
                &header as *const WavHeader as *const u8,
                std::mem::size_of::<WavHeader>()
            );
            file.write_all(header_bytes)
                .map_err(|e| MixerError::IoError(format!("Failed to write test header: {}", e)))?;
        }

        file.write_all(&audio_data)
            .map_err(|e| MixerError::IoError(format!("Failed to write test data: {}", e)))?;

        Ok(())
    }

    #[test]
    fn test_wav_header_creation() {
        let config = MixerConfig::default();
        let header = WavHeader::new(&config, 1000);

        assert_eq!(&header.riff, b"RIFF");
        assert_eq!(&header.wave, b"WAVE");
        assert_eq!(&header.fmt_id, b"fmt ");
        assert_eq!(&header.data_id, b"data");
        assert_eq!(header.file_size, 36 + 1000);
    }

    #[test]
    fn test_wav_header_parsing() {
        let config = MixerConfig::default();
        let header = WavHeader::new(&config, 1000);

        unsafe {
            let bytes = std::slice::from_raw_parts(
                &header as *const WavHeader as *const u8,
                std::mem::size_of::<WavHeader>()
            );

            let parsed = WavHeader::from_bytes(bytes).unwrap();
            assert_eq!(parsed.channels, config.channels);
            assert_eq!(parsed.sample_rate, config.sample_rate);
            assert_eq!(parsed.bits_per_sample, config.bits_per_sample);
        }
    }

    #[test]
    fn test_read_wav_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_read.wav");

        create_test_wav(&test_file, 1.0).unwrap();

        let config = MixerConfig::default();
        let audio_data = AudioMixer::read_wav_file(&test_file, &config).unwrap();

        // 1 second at 22050 Hz, 1 channel, 16-bit = 44100 bytes
        assert_eq!(audio_data.len(), 44100);

        // Cleanup
        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_write_wav_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_write.wav");

        let config = MixerConfig::default();
        let audio_data = vec![0u8; 1000];

        AudioMixer::write_wav_file(&test_file, &config, &audio_data).unwrap();

        assert!(test_file.exists());

        // Verify file can be read back
        let read_data = AudioMixer::read_wav_file(&test_file, &config).unwrap();
        assert_eq!(read_data.len(), 1000);

        // Cleanup
        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_audio_mixing() {
        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("segment1.wav");
        let file2 = temp_dir.join("segment2.wav");

        // Create test files
        create_test_wav(&file1, 1.0).unwrap();
        create_test_wav(&file2, 2.0).unwrap();

        // Create mixer request
        let segments = vec![
            AudioSegmentInfo {
                path: file1.clone(),
                speaker: "Speaker1".to_string(),
                duration_secs: 1.0,
                sample_rate: 22050,
                channels: 1,
            },
            AudioSegmentInfo {
                path: file2.clone(),
                speaker: "Speaker2".to_string(),
                duration_secs: 2.0,
                sample_rate: 22050,
                channels: 1,
            },
        ];

        let config = MixerConfig {
            output_path: temp_dir.join("test_output"),
            silence_duration_secs: 0.5,
            ..Default::default()
        };

        let request = MixerRequest { segments, config };

        // Mix audio
        let mixer = AudioMixer::new();
        let result = mixer.mix(request).unwrap();

        // Verify result
        assert!(result.output_file.exists());
        assert_eq!(result.segment_count, 2);
        // 1 + 0.5 + 2 = 3.5 seconds
        assert!((result.total_duration_secs - 3.5).abs() < 0.1);

        // Cleanup
        std::fs::remove_file(&file1).ok();
        std::fs::remove_file(&file2).ok();
        std::fs::remove_file(&result.output_file).ok();
    }
}
