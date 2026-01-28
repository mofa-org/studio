//! Dora Integration for mofa-cast
//!
//! Manages the lifecycle of Dora dataflow for batch TTS synthesis.

use crossbeam_channel::{Receiver, Sender, bounded};
use mofa_dora_bridge::{
    controller::DataflowController,
    data::AudioData,
    dispatcher::DynamicNodeDispatcher,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use parking_lot::RwLock;
use std::collections::HashMap;
use crate::dora_process_manager::DoraProcessManager;

// ============================================================================
// VOICE CONFIGURATION
// ============================================================================

/// Voice configuration for a speaker
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceConfig {
    /// Speaker name (e.g., "host", "guest1", "guest2")
    pub speaker: String,
    /// Voice name (e.g., "Luo Xiang", "Yang Mi", "Ma Yun")
    pub voice_name: String,
    /// Speed factor (0.5 - 2.0, 1.0 = normal)
    pub speed: f32,
}

impl VoiceConfig {
    /// Create a new voice configuration
    pub fn new(speaker: impl Into<String>, voice_name: impl Into<String>, speed: f32) -> Self {
        Self {
            speaker: speaker.into(),
            voice_name: voice_name.into(),
            speed: speed.clamp(0.5, 2.0),  // Clamp to valid range
        }
    }

    /// Get default voice configurations for common speakers
    /// Smart mapping based on speaker name patterns
    pub fn get_defaults(speakers: &[String]) -> Vec<Self> {
        speakers.iter().map(|speaker| {
            // Normalize speaker name
            let normalized = speaker.to_lowercase();

            // Smart voice assignment based on speaker role
            let voice_name = if normalized.contains("host") || normalized.contains("主持") {
                "Luo Xiang"  // 主持人 - 深沉男声
            } else if normalized.contains("guest1") || normalized.contains("嘉宾1") {
                "Ma Yun"     // 嘉宾1 - 激昂男声
            } else if normalized.contains("guest2") || normalized.contains("嘉宾2") {
                "Ma Baoguo"  // 嘉宾2 - 特色声音
            } else if normalized.contains("guest") || normalized.contains("嘉宾") {
                "Ma Yun"     // 默认嘉宾
            } else {
                "Luo Xiang"  // 默认
            };

            VoiceConfig::new(speaker, voice_name, 1.0)
        }).collect()
    }
}

/// Voice mapping configuration for all speakers
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceMapping {
    /// Map of speaker name to voice configuration
    pub voices: Vec<VoiceConfig>,
}

impl VoiceMapping {
    /// Create a new voice mapping
    pub fn new() -> Self {
        Self {
            voices: Vec::new(),
        }
    }

    /// Get voice configuration for a speaker
    pub fn get_voice_for_speaker(&self, speaker: &str) -> Option<&VoiceConfig> {
        self.voices.iter().find(|v| v.speaker == speaker)
    }

    /// Add or update voice configuration for a speaker
    pub fn set_voice(&mut self, config: VoiceConfig) {
        // Remove existing config for this speaker if any
        self.voices.retain(|v| v.speaker != config.speaker);
        // Add new config
        self.voices.push(config);
    }

    /// Get default voice mapping from speakers
    pub fn from_speakers(speakers: &[String]) -> Self {
        Self {
            voices: VoiceConfig::get_defaults(speakers),
        }
    }
}

impl Default for VoiceMapping {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DATA MODELS

/// State shared between UI and Dora integration
#[derive(Debug, Default)]
pub struct DoraState {
    /// Whether the dataflow is running
    pub dataflow_running: bool,
    /// Current dataflow ID
    pub dataflow_id: Option<String>,
    /// Connection status
    pub controller_connected: bool,
    /// Last synthesized audio data
    pub pending_audio: Vec<AudioData>,
    /// Pending script segments to send (for sequential sending)
    pub pending_segments: Vec<ScriptSegment>,
    /// Current segment index being processed
    pub current_segment_index: usize,
    /// Total segments to process
    pub total_segments: usize,
    /// Voice mapping configuration (which voice each speaker uses)
    pub voice_mapping: VoiceMapping,
}

/// Commands sent from UI to Dora integration
#[derive(Debug, Clone)]
pub enum DoraCommand {
    /// Start the TTS dataflow
    StartDataflow {
        dataflow_path: PathBuf,
        env_vars: HashMap<String, String>,
    },
    /// Stop the dataflow
    StopDataflow,
    /// Send text segments to TTS dataflow
    SendScriptSegments {
        segments: Vec<ScriptSegment>,
    },
    /// Set voice mapping for speakers
    SetVoiceMapping {
        voice_mapping: VoiceMapping,
    },
}

/// Script segment for TTS synthesis
#[derive(Debug, Clone)]
pub struct ScriptSegment {
    /// Speaker name
    pub speaker: String,
    /// Text content to synthesize
    pub text: String,
    /// Segment index in the script
    pub segment_index: usize,
    /// Voice name to use for this segment
    pub voice_name: String,
    /// Speed factor for this segment
    pub speed: f32,
}

/// Events sent from Dora integration to UI
#[derive(Debug, Clone)]
pub enum DoraEvent {
    /// Dataflow started
    DataflowStarted { dataflow_id: String },
    /// Dataflow stopped
    DataflowStopped,
    /// Audio segment completed
    AudioSegment { data: AudioData },
    /// Synthesis progress update
    Progress {
        current: usize,
        total: usize,
        speaker: String,
    },
    /// Error occurred
    Error { message: String },
    /// Log message
    Log { message: String },
}

/// Dora integration manager for mofa-cast
pub struct DoraIntegration {
    /// Shared state
    state: Arc<RwLock<DoraState>>,
    /// Command sender (UI → Dora thread)
    command_tx: Sender<DoraCommand>,
    /// Event receiver (Dora thread → UI)
    event_rx: Receiver<DoraEvent>,
    /// Worker thread handle
    worker_handle: Option<thread::JoinHandle<()>>,
    /// Stop signal
    stop_tx: Option<Sender<()>>,
    /// Dora process manager
    process_manager: Arc<std::sync::Mutex<DoraProcessManager>>,
}

impl DoraIntegration {
    /// Create a new Dora integration
    pub fn new() -> Self {
        let (command_tx, command_rx) = bounded(100);
        let (event_tx, event_rx) = bounded(100);
        let (stop_tx, stop_rx) = bounded(1);

        let state = Arc::new(RwLock::new(DoraState::default()));

        // Create process manager
        let process_manager = Arc::new(std::sync::Mutex::new(DoraProcessManager::new()));
        let process_manager_clone = Arc::clone(&process_manager);

        // Spawn worker thread
        let state_clone = state.clone();
        let handle = thread::spawn(move || {
            Self::worker_thread(state_clone, command_rx, event_tx, stop_rx, process_manager_clone);
        });

        Self {
            state,
            command_tx,
            event_rx,
            worker_handle: Some(handle),
            stop_tx: Some(stop_tx),
            process_manager,
        }
    }

    /// Get shared state
    pub fn state(&self) -> Arc<RwLock<DoraState>> {
        self.state.clone()
    }

    /// Send command to Dora thread
    pub fn send_command(&self, command: DoraCommand) -> bool {
        ::log::info!("📤 Sending command to Dora worker: {:?}", std::mem::discriminant(&command));
        let result = self.command_tx.send(command).is_ok();
        if result {
            ::log::info!("✓ Command sent successfully");
        } else {
            ::log::error!("❌ Failed to send command (channel closed?)");
        }
        result
    }

    /// Poll for events from Dora thread
    pub fn poll_events(&self) -> Vec<DoraEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }
        events
    }

    /// Worker thread that processes Dora commands
    fn worker_thread(
        state: Arc<RwLock<DoraState>>,
        command_rx: Receiver<DoraCommand>,
        event_tx: Sender<DoraEvent>,
        stop_rx: Receiver<()>,
        process_manager: Arc<std::sync::Mutex<DoraProcessManager>>,
    ) {
        // Start dora processes automatically
        ::log::info!("Starting dora daemon and coordinator...");
        if let Err(e) = process_manager.lock().unwrap().start() {
            ::log::error!("Failed to start dora processes: {}", e);
            let _ = event_tx.send(DoraEvent::Error {
                message: format!("Failed to start dora: {}", e),
            });
            return;
        }

        let mut dispatcher: Option<DynamicNodeDispatcher> = None;

        loop {
            // Check for stop signal
            if stop_rx.try_recv().is_ok() {
                ::log::info!("Dora integration worker: received stop signal");
                break;
            }

            // Process commands
            while let Ok(command) = command_rx.try_recv() {
                match command {
                    DoraCommand::StartDataflow { dataflow_path, env_vars } => {
                        ::log::info!("Starting Dora dataflow: {:?}", dataflow_path);

                        // Ensure dora processes are running
                        if !process_manager.lock().unwrap().is_running() {
                            ::log::warn!("Dora processes not running, restarting...");
                            if let Err(e) = process_manager.lock().unwrap().start() {
                                ::log::error!("Failed to restart dora processes: {}", e);
                                let _ = event_tx.send(DoraEvent::Error {
                                    message: format!("Failed to restart dora: {}", e),
                                });
                                continue;
                            }
                        }

                        // Set environment variables
                        for (key, value) in &env_vars {
                            ::log::info!("Setting env var: {}={}", key, value);
                            std::env::set_var(key, value);
                        }

                        // Create dataflow controller
                        match DataflowController::new(&dataflow_path) {
                            Ok(mut controller) => {
                                // Pass env vars to controller
                                controller.set_envs(env_vars);

                                // Create dispatcher
                                let mut disp = DynamicNodeDispatcher::new(controller);

                                // Start dataflow
                                match disp.start() {
                                    Ok(dataflow_id) => {
                                        state.write().dataflow_running = true;
                                        state.write().dataflow_id = Some(dataflow_id.clone());

                                        let _ = event_tx.send(DoraEvent::DataflowStarted {
                                            dataflow_id,
                                        });

                                        dispatcher = Some(disp);
                                        ::log::info!("Dataflow started successfully");
                                    }
                                    Err(e) => {
                                        ::log::error!("Failed to start dataflow: {:?}", e);
                                        let _ = event_tx.send(DoraEvent::Error {
                                            message: format!("Failed to start dataflow: {}", e),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                ::log::error!("Failed to create controller: {:?}", e);
                                let _ = event_tx.send(DoraEvent::Error {
                                    message: format!("Failed to create controller: {}", e),
                                });
                            }
                        }
                    }

                    DoraCommand::StopDataflow => {
                        ::log::info!("Stopping Dora dataflow");

                        if let Some(mut disp) = dispatcher.take() {
                            match disp.stop() {
                                Ok(_) => {
                                    ::log::info!("Dataflow stopped successfully");
                                }
                                Err(e) => {
                                    ::log::error!("Failed to stop dataflow: {:?}", e);
                                    let _ = event_tx.send(DoraEvent::Error {
                                        message: format!("Failed to stop dataflow: {}", e),
                                    });
                                }
                            }
                        }

                        state.write().dataflow_running = false;
                        state.write().dataflow_id = None;

                        let _ = event_tx.send(DoraEvent::DataflowStopped);
                    }

                    DoraCommand::SetVoiceMapping { voice_mapping } => {
                        ::log::info!("Setting voice mapping for {} speakers", voice_mapping.voices.len());

                        // Update voice mapping in state
                        state.write().voice_mapping = voice_mapping;

                        // Log the mapping
                        for voice_config in &state.read().voice_mapping.voices {
                            ::log::info!("  Speaker '{}' → Voice '{}' (speed: {:.1})",
                                       voice_config.speaker, voice_config.voice_name, voice_config.speed);
                        }

                        let _ = event_tx.send(DoraEvent::Log {
                            message: format!("Voice mapping updated for {} speakers", state.read().voice_mapping.voices.len()),
                        });
                    }

                    DoraCommand::SendScriptSegments { mut segments } => {
                        ::log::info!("Queueing {} script segments for sequential processing", segments.len());

                        if let Some(ref disp) = dispatcher {
                            // Store all segments in state
                            let total = segments.len();
                            state.write().pending_segments = segments.clone();
                            state.write().total_segments = total;
                            state.write().current_segment_index = 0;

                            // Send only the FIRST segment
                            if let Some(first_segment) = segments.first() {
                                if let Some(bridge) = disp.get_bridge("mofa-cast-controller") {
                                    // For single-voice mode: send "speaker\ntext" format
                                    // For multi-voice mode: send JSON with voice routing info
                                    // Auto-detect based on whether all segments use the same voice
                                    let all_same_voice = segments.iter().all(|s| s.voice_name == segments[0].voice_name);

                                    let text = if all_same_voice {
                                        // Single voice mode - use simple format
                                        ::log::info!("Single-voice mode: {}", first_segment.voice_name);
                                        format!("{}\n{}", first_segment.speaker, first_segment.text)
                                    } else {
                                        // Multi-voice mode - use JSON format
                                        let data = serde_json::json!({
                                            "speaker": first_segment.speaker,
                                            "text": first_segment.text,
                                            "voice_name": first_segment.voice_name,
                                            "speed": first_segment.speed
                                        });
                                        data.to_string()
                                    };

                                    ::log::info!("Sending FIRST segment 1/{}: {} chars",
                                        total, text.len());

                                    if let Err(e) = bridge.send("text", mofa_dora_bridge::DoraData::Text(text.clone())) {
                                        ::log::error!("Failed to send first segment: {}", e);
                                        let _ = event_tx.send(DoraEvent::Error {
                                            message: format!("Failed to send first segment: {}", e),
                                        });
                                        // Clear pending segments on error
                                        state.write().pending_segments.clear();
                                    } else {
                                        ::log::info!("✓ First segment sent, waiting for segment_complete before sending next");
                                        let _ = event_tx.send(DoraEvent::Progress {
                                            current: first_segment.segment_index,
                                            total,
                                            speaker: first_segment.speaker.clone(),
                                        });
                                    }
                                } else {
                                    ::log::error!("mofa-cast-controller bridge not available");
                                    let _ = event_tx.send(DoraEvent::Error {
                                        message: "mofa-cast-controller bridge not available".to_string(),
                                    });
                                }
                            }

                            let _ = event_tx.send(DoraEvent::Log {
                                message: format!("Queued {} segments, will send sequentially", total),
                            });
                        } else {
                            ::log::error!("Dispatcher not available, dataflow not running?");
                            let _ = event_tx.send(DoraEvent::Error {
                                message: "Dataflow not running".to_string(),
                            });
                        }
                    }
                }
            }

            // Poll for events via SharedDoraState
            if let Some(ref disp) = dispatcher {
                let shared_state = disp.shared_state();

                // Check for audio data FIRST - this triggers next segment send
                let audio_chunks = shared_state.audio.drain();
                let audio_count = audio_chunks.len();
                for audio_data in audio_chunks {
                    ::log::info!("Received audio segment: {} samples, {}Hz",
                               audio_data.samples.len(), audio_data.sample_rate);
                    let _ = event_tx.send(DoraEvent::AudioSegment { data: audio_data });
                }

                // If we received audio, send the next segment (serial processing)
                if audio_count > 0 {
                    let current_idx = state.read().current_segment_index;
                    let total = state.read().total_segments;

                    // Increment index
                    state.write().current_segment_index = current_idx + 1;

                    // Check if there are more segments to send
                    let pending = state.read().pending_segments.clone();
                    if current_idx + 1 < pending.len() {
                        let next_segment = &pending[current_idx + 1];
                        if let Some(bridge) = disp.get_bridge("mofa-cast-controller") {
                            // Auto-detect single vs multi-voice mode
                            let all_same_voice = pending.iter().all(|s| s.voice_name == pending[0].voice_name);

                            let text = if all_same_voice {
                                // Single voice mode - use simple format
                                format!("{}\n{}", next_segment.speaker, next_segment.text)
                            } else {
                                // Multi-voice mode - use JSON format
                                let data = serde_json::json!({
                                    "speaker": next_segment.speaker,
                                    "text": next_segment.text,
                                    "voice_name": next_segment.voice_name,
                                    "speed": next_segment.speed
                                });
                                data.to_string()
                            };

                            ::log::info!("🚀 Sending NEXT segment {}/{}: {} chars (after audio received)",
                                       current_idx + 2, total, text.len());

                            if let Err(e) = bridge.send("text", mofa_dora_bridge::DoraData::Text(text.clone())) {
                                ::log::error!("Failed to send segment {}: {}", current_idx + 2, e);
                                // Clear remaining segments on error
                                state.write().pending_segments.clear();
                            } else {
                                ::log::info!("✓ Segment {}/{} sent", current_idx + 2, total);
                                let _ = event_tx.send(DoraEvent::Progress {
                                    current: next_segment.segment_index,
                                    total,
                                    speaker: next_segment.speaker.clone(),
                                });
                            }
                        }
                    } else {
                        ::log::info!("✅ All {} segments sent! Waiting for remaining audio...", total);
                        // Clear pending segments
                        state.write().pending_segments.clear();
                    }
                }

                // Check for bridge status changes
                if let Some(status) = shared_state.status.read_if_dirty() {
                    if let Some(error) = status.last_error {
                        ::log::error!("Bridge error: {}", error);
                        let _ = event_tx.send(DoraEvent::Error { message: error });
                    }
                }
            }

            // Small sleep to prevent busy waiting
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Cleanup
        if let Some(mut disp) = dispatcher {
            let _ = disp.stop();
        }

        // Stop dora processes
        ::log::info!("Stopping dora daemon and coordinator...");
        process_manager.lock().unwrap().stop();

        ::log::info!("Dora integration worker: exiting");
    }

    /// Start the TTS dataflow
    pub fn start_dataflow(&self, path: PathBuf) -> bool {
        self.start_dataflow_with_env(path, HashMap::new())
    }

    /// Start the TTS dataflow with environment variables
    pub fn start_dataflow_with_env(&self, path: PathBuf, env_vars: HashMap<String, String>) -> bool {
        self.send_command(DoraCommand::StartDataflow {
            dataflow_path: path,
            env_vars,
        })
    }

    /// Stop the TTS dataflow
    pub fn stop_dataflow(&self) -> bool {
        self.send_command(DoraCommand::StopDataflow)
    }

    /// Send script segments for synthesis
    pub fn send_script_segments(&self, segments: Vec<ScriptSegment>) -> bool {
        self.send_command(DoraCommand::SendScriptSegments { segments })
    }

    /// Set voice mapping for speakers
    pub fn set_voice_mapping(&self, voice_mapping: VoiceMapping) -> bool {
        self.send_command(DoraCommand::SetVoiceMapping { voice_mapping })
    }
}

impl Drop for DoraIntegration {
    fn drop(&mut self) {
        // Stop worker thread
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }

        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }

        // Stop dora processes as final cleanup
        ::log::info!("Drop: Stopping dora processes...");
        self.process_manager.lock().unwrap().stop();
    }
}
