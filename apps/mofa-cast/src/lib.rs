//! MoFA Cast - Transform your optimized scripts into multi-voice podcast audio
//!
//! This app provides:
//! - Multi-format script importing (plain text, JSON, Markdown)
//! - Automatic speaker detection and voice assignment
//! - Multi-voice batch TTS synthesis with PrimeSpeech
//! - Audio mixing and WAV export
//!
//! **Note**: Script optimization should be done externally using ChatGPT, Claude, or other AI tools.

pub mod screen;
pub mod transcript_parser;
pub mod tts_batch;
pub mod audio_mixer;
pub mod dora_integration;
pub mod dora_process_manager;
pub mod recent_files;
pub mod script_templates;

pub use screen::CastScreen;

// Re-export commonly used transcript types
pub use transcript_parser::{
    JsonParser, MarkdownParser, Message, Metadata, ParseError, ParserFactory,
    PlainTextParser, Speaker, Transcript, TranscriptFormat, TranscriptParser,
};

// Re-export TTS batch types
pub use tts_batch::{
    AudioSegment, BatchTtsSynthesizer, DoraKokoroTtsEngine, KokoroBackend, MockTtsEngine,
    ScriptSegmenter, TtsConfig, TtsEngine, TtsEngineWrapper, TtsError, TtsFactory, TtsProgress,
    TtsRequest, TtsResult,
};

// Re-export audio mixer types
pub use audio_mixer::{
    AudioMixer, AudioMetadata, AudioSegmentInfo, ExportFormat, MixerConfig, MixerError,
    MixerRequest, MixerResult, Mp3Bitrate,
};

// Re-export Dora integration types
pub use dora_integration::{
    DoraIntegration, DoraState, DoraCommand, DoraEvent, ScriptSegment,
    VoiceConfig, VoiceMapping,
};

use makepad_widgets::Cx;
use mofa_widgets::{MofaApp, AppInfo};

/// MoFA Cast app descriptor
pub struct MoFaCastApp;

impl MofaApp for MoFaCastApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "MoFA Cast",
            id: "mofa-cast",
            description: "Transform your optimized scripts into multi-voice podcast audio with local TTS",
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}

/// Register all MoFA Cast widgets with Makepad
/// (Kept for backwards compatibility - calls MoFaCastApp::live_design)
pub fn live_design(cx: &mut Cx) {
    MoFaCastApp::live_design(cx);
}
