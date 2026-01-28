//! Cast controller bridge for mofa-cast
//!
//! Connects to dora as `mofa-cast-controller` dynamic node.
//! Sends script segments to TTS nodes and receives audio.

use crate::bridge::{BridgeState, DoraBridge};
use crate::data::{AudioData, DoraData, EventMetadata, LogEntry, LogLevel};
use crate::error::{BridgeError, BridgeResult};
use crate::shared_state::SharedDoraState;
use arrow::array::Array;
use crossbeam_channel::{bounded, Receiver, Sender};
use dora_node_api::{DoraNode, Event, IntoArrow, dora_core::config::DataId, dora_core::config::NodeId};
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;
use log::{debug, error, info, trace, warn};

/// Cast controller bridge - manages batch TTS workflow
///
/// Status updates (connected/disconnected/error) are communicated via SharedDoraState.
/// Audio data and logs are pushed directly to SharedDoraState for UI consumption.
pub struct CastControllerBridge {
    /// Node ID (e.g., "mofa-cast-controller")
    node_id: String,
    /// Current state
    state: Arc<RwLock<BridgeState>>,
    /// Shared state for direct UI communication
    shared_state: Option<Arc<SharedDoraState>>,
    /// Text segment sender from widget
    text_sender: Sender<String>,
    /// Text segment receiver for dora
    text_receiver: Receiver<String>,
    /// Stop signal
    stop_sender: Option<Sender<()>>,
    /// Worker thread handle
    worker_handle: Option<thread::JoinHandle<()>>,
    /// Current speaker (for audio metadata)
    current_speaker: Arc<RwLock<Option<String>>>,
}

impl CastControllerBridge {
    /// Create a new cast controller bridge
    pub fn new(node_id: &str) -> Self {
        Self::with_shared_state(node_id, None)
    }

    /// Create a new cast controller bridge with shared state
    pub fn with_shared_state(node_id: &str, shared_state: Option<Arc<SharedDoraState>>) -> Self {
        let (text_tx, text_rx) = bounded(100);

        Self {
            node_id: node_id.to_string(),
            state: Arc::new(RwLock::new(BridgeState::Disconnected)),
            shared_state,
            text_sender: text_tx,
            text_receiver: text_rx,
            stop_sender: None,
            worker_handle: None,
            current_speaker: Arc::new(RwLock::new(None)),
        }
    }

    /// Send a text segment to dora (widget calls this)
    pub fn send_text(&self, text: impl Into<String>) -> BridgeResult<()> {
        self.text_sender
            .send(text.into())
            .map_err(|_| BridgeError::ChannelSendError)
    }

    /// Run the dora event loop in background thread
    fn run_event_loop(
        node_id: String,
        state: Arc<RwLock<BridgeState>>,
        shared_state: Option<Arc<SharedDoraState>>,
        text_receiver: Receiver<String>,
        stop_receiver: Option<Receiver<()>>,
        current_speaker: Arc<RwLock<Option<String>>>,
    ) {
        info!("Starting cast controller bridge event loop for {}", node_id);

        // Initialize dora node
        let (mut node, mut events) = match DoraNode::init_from_node_id(NodeId::from(node_id.clone())) {
            Ok(n) => {
                info!("Dora node initialized successfully for {}", node_id);
                n
            }
            Err(e) => {
                error!("Failed to init dora node {}: {}", node_id, e);
                *state.write() = BridgeState::Error;
                if let Some(ref ss) = shared_state {
                    let mut status = ss.status.read();
                    status.last_error = Some(format!("Init failed: {}", e));
                    ss.status.set(status);
                }
                return;
            }
        };

        info!("Setting state to Connected for {}", node_id);
        *state.write() = BridgeState::Connected;
        if let Some(ref ss) = shared_state {
            let mut status = ss.status.read();
            status.active_bridges.push(node_id.clone());
            ss.status.set(status);
        }

        // Event loop
        info!("Cast controller bridge event loop starting");
        loop {
            // Check for stop signal
            if let Some(ref rx) = stop_receiver {
                if rx.try_recv().is_ok() {
                    info!("Cast controller bridge received stop signal");
                    break;
                }
            }

            // Check for text segments to send
            let mut received_count = 0;
            while let Ok(text) = text_receiver.try_recv() {
                received_count += 1;

                // Extract speaker from text (for audio metadata)
                let speaker = Self::extract_speaker_from_text(&text);
                if let Some(ref spk) = speaker {
                    *current_speaker.write() = Some(spk.clone());
                    debug!("Updated current speaker: {}", spk);
                }

                info!("Sending text segment to dora ({} chars)", text.len());
                if let Err(e) = Self::send_text_to_dora(&mut node, &text) {
                    error!("Failed to send text: {}", e);
                } else {
                    info!("Text segment sent successfully");
                }

                // Small delay to avoid overwhelming the dataflow
                // (gives TTS time to process each segment)
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            if received_count > 0 {
                info!("Processed {} text segments from UI", received_count);
            }

            // Receive dora events with timeout (100ms to avoid excessive timeout errors)
            match events.recv_timeout(std::time::Duration::from_millis(100)) {
                Some(event) => {
                    Self::handle_dora_event(event, shared_state.as_ref(), current_speaker.clone());
                }
                None => {
                    // Timeout is normal, loop back to check for pending sends
                    trace!("No events from dora node (timeout)");
                }
            }
        }

        info!("Cast controller bridge event loop ended for {}", node_id);
        *state.write() = BridgeState::Disconnected;
        if let Some(ref ss) = shared_state {
            let mut status = ss.status.read();
            status.active_bridges.retain(|id| id != &node_id);
            ss.status.set(status);
        }
    }

    /// Send text segment to dora node
    fn send_text_to_dora(node: &mut DoraNode, text: &str) -> Result<(), String> {
        debug!("Sending text to dora: {} chars", text.len());

        // Send text data
        let data = text.to_string().into_arrow();
        let output_id: DataId = "text".to_string().into();
        node.send_output(output_id, Default::default(), data)
            .map_err(|e| format!("Failed to send text: {}", e))
    }

    /// Handle incoming event from dora
    fn handle_dora_event(event: Event, shared_state: Option<&Arc<SharedDoraState>>, current_speaker: Arc<RwLock<Option<String>>>) {
        match event {
            Event::Input { id, data, metadata } => {
                let input_id = id.as_str();
                info!("Received input: id={}", input_id);

                // Extract metadata
                let mut event_meta = EventMetadata::default();
                for (key, value) in metadata.parameters.iter() {
                    let string_value = match value {
                        dora_node_api::Parameter::String(s) => s.clone(),
                        dora_node_api::Parameter::Integer(i) => i.to_string(),
                        dora_node_api::Parameter::Float(f) => f.to_string(),
                        dora_node_api::Parameter::Bool(b) => b.to_string(),
                        _ => "".to_string(),
                    };
                    event_meta.values.insert(key.clone(), string_value);
                }

                // Parse input data
                match input_id {
                    input_id if input_id.starts_with("audio") || input_id == "audio" => {
                        info!("Received audio input from {}", input_id);
                        // Push audio data to shared state
                        if let Some(ss) = shared_state {
                            // Get current speaker for audio metadata
                            let speaker = current_speaker.read().clone();
                            if let Some(audio) = Self::extract_audio(&data, &event_meta, speaker) {
                                info!("Extracted audio: {} samples, {}Hz, {} channels",
                                      audio.samples.len(), audio.sample_rate, audio.channels);
                                ss.audio.push(audio);
                            } else {
                                warn!("Failed to extract audio from arrow data");
                            }
                        }
                    }
                    input_id if input_id.starts_with("segment_complete") || input_id == "segment_complete" => {
                        info!("Segment complete signal received from {}", input_id);
                        // Log segment completion - the UI polls for audio data separately
                        if let Some(ss) = shared_state {
                            ss.logs.push(LogEntry::new(
                                LogLevel::Info,
                                format!("Segment complete: {}", input_id),
                                "cast_controller",
                            ));
                        }
                    }
                    "log" | "log_tts" => {
                        info!("Received log input from {}", input_id);
                        if let Some(ss) = shared_state {
                            match Self::extract_text(&data) {
                                Some(log_text) => {
                                    info!("Log received: {}", log_text);
                                    ss.logs.push(LogEntry::new(
                                        LogLevel::Info,
                                        log_text,
                                        input_id,
                                    ));
                                }
                                None => {
                                    warn!("Failed to extract log text, data_type: {:?}", data.0.data_type());
                                    // Try to print raw data for debugging
                                    if data.0.len() > 0 {
                                        warn!("Log data length: {}", data.0.len());
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        debug!("Unhandled input ID: {}", input_id);
                    }
                }
            }
            Event::Stop { .. } => {
                info!("Dora node stopped");
                // Status update is handled by run_event_loop
            }
            _ => {
                // Ignore all other events (including Event::Error, Event::InputClosed)
                // Match PromptInputBridge behavior - these errors are typically transient
            }
        }
    }

    /// Extract audio data from dora arrow data
    fn extract_audio(data: &dora_node_api::ArrowData, metadata: &EventMetadata, speaker: Option<String>) -> Option<AudioData> {
        use arrow::array::{Float32Array, Float64Array, Int16Array, Array};
        use arrow::datatypes::DataType;

        let array = &data.0;
        if array.is_empty() {
            return None;
        }

        // Try to get sample rate from metadata
        let sample_rate = metadata.get("sample_rate")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(24000);

        let (samples, effective_sample_rate) = match array.data_type() {
            // Direct float arrays (PrimeSpeech style)
            DataType::Float32 => {
                let float_array = array.as_any().downcast_ref::<Float32Array>()?;
                let samples: Vec<f32> = float_array.values().to_vec();
                (samples, sample_rate)
            }
            DataType::Float64 => {
                let float_array = array.as_any().downcast_ref::<Float64Array>()?;
                let samples: Vec<f32> = float_array.values().iter().map(|&v| v as f32).collect();
                (samples, sample_rate)
            }
            DataType::Int16 => {
                let int_array = array.as_any().downcast_ref::<Int16Array>()?;
                let samples: Vec<f32> = int_array.values().iter().map(|&v| v as f32 / 32768.0).collect();
                (samples, sample_rate)
            }
            // Nested array structure (Kokoro TTS style: pa.array([numpy_array]))
            DataType::List(_) | DataType::LargeList(_) | DataType::FixedSizeList(_, _) => {
                // Kokoro TTS sends: pa.array([audio_array]) where audio_array is numpy float32 array
                // This creates a ListArray with one element containing the actual samples
                let list_array = arrow::array::as_list_array(array);
                if list_array.len() > 0 {
                    let first_value = list_array.value(0);
                    match first_value.data_type() {
                        DataType::Float32 => {
                            let float_array = first_value.as_any().downcast_ref::<Float32Array>()?;
                            let samples: Vec<f32> = float_array.values().to_vec();
                            (samples, sample_rate)
                        }
                        DataType::Float64 => {
                            let float_array = first_value.as_any().downcast_ref::<Float64Array>()?;
                            let samples: Vec<f32> = float_array.values().iter().map(|&v| v as f32).collect();
                            (samples, sample_rate)
                        }
                        _ => {
                            debug!("Unsupported nested array type: {:?}", first_value.data_type());
                            return None;
                        }
                    }
                } else {
                    debug!("Empty list array");
                    return None;
                }
            }
            _ => {
                debug!("Unsupported audio data type: {:?}", array.data_type());
                return None;
            }
        };

        Some(AudioData {
            samples,
            sample_rate: effective_sample_rate,
            channels: 1,
            participant_id: speaker,
            question_id: None,
        })
    }

    /// Extract speaker name from text segment
    fn extract_speaker_from_text(text: &str) -> Option<String> {
        // Try JSON format first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
            if let Some(speaker) = json.get("speaker").and_then(|v| v.as_str()) {
                return Some(speaker.to_string());
            }
        }

        // Fallback: extract first line (simple format: "Speaker\ntext")
        if let Some(first_line) = text.lines().next() {
            let trimmed = first_line.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }

        None
    }

    /// Extract text data from arrow array
    fn extract_text(data: &dora_node_api::ArrowData) -> Option<String> {
        match data.0.data_type() {
            arrow::datatypes::DataType::Utf8 => {
                let array = data.0.as_any().downcast_ref::<arrow::array::StringArray>()?;
                if array.len() > 0 {
                    return Some(array.value(0).to_string());
                }
            }
            arrow::datatypes::DataType::LargeUtf8 => {
                let array = data.0.as_any().downcast_ref::<arrow::array::LargeStringArray>()?;
                if array.len() > 0 {
                    return Some(array.value(0).to_string());
                }
            }
            _ => {
                debug!("Unsupported text data type: {:?}", data.0.data_type());
            }
        }
        None
    }
}

impl Drop for CastControllerBridge {
    fn drop(&mut self) {
        // Send stop signal
        if let Some(stop_tx) = self.stop_sender.take() {
            let _ = stop_tx.send(());
        }

        // Wait for worker thread to finish
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl DoraBridge for CastControllerBridge {
    fn node_id(&self) -> &str {
        &self.node_id
    }

    fn state(&self) -> BridgeState {
        *self.state.read()
    }

    fn connect(&mut self) -> BridgeResult<()> {
        if self.is_connected() {
            return Err(BridgeError::AlreadyConnected);
        }

        *self.state.write() = BridgeState::Connecting;

        let (stop_tx, stop_rx) = bounded(1);
        self.stop_sender = Some(stop_tx);

        let node_id = self.node_id.clone();
        let state = Arc::clone(&self.state);
        let shared_state = self.shared_state.clone();
        let text_receiver = self.text_receiver.clone();
        let current_speaker = Arc::clone(&self.current_speaker);

        let handle = thread::spawn(move || {
            Self::run_event_loop(node_id, state, shared_state, text_receiver, Some(stop_rx), current_speaker);
        });

        self.worker_handle = Some(handle);

        // Wait briefly for connection (same as PromptInputBridge)
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Check if connection actually succeeded
        let final_state = self.state.read();
        info!("Connection attempt result: state={:?}", *final_state);
        if !matches!(*final_state, BridgeState::Connected) {
            error!("Worker thread failed to connect: state={:?}", *final_state);
            return Err(BridgeError::ConnectionFailed(format!("Connection failed: {:?}", *final_state)));
        }

        Ok(())
    }

    fn disconnect(&mut self) -> BridgeResult<()> {
        if let Some(stop_tx) = self.stop_sender.take() {
            let _ = stop_tx.send(());
        }

        if let Some(handle) = self.worker_handle.take() {
            // Wait with timeout to avoid blocking indefinitely
            let timeout = std::time::Duration::from_secs(2);
            let start = std::time::Instant::now();

            loop {
                if start.elapsed() > timeout {
                    warn!("Cast controller bridge disconnect timeout after {:?}", timeout);
                    break;
                }

                if handle.is_finished() {
                    let _ = handle.join();
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        *self.state.write() = BridgeState::Disconnected;
        Ok(())
    }

    fn send(&self, output_id: &str, data: DoraData) -> BridgeResult<()> {
        if !self.is_connected() {
            return Err(BridgeError::NotConnected);
        }

        match (output_id, data) {
            ("text", DoraData::Text(text)) => {
                info!("Sending text via bridge: {} chars", text.len());
                if let Err(_) = self.text_sender.send(text) {
                    error!("Failed to send text to worker thread (channel closed?)");
                    return Err(BridgeError::ChannelSendError);
                }
                info!("Text sent to worker thread successfully");
            }
            _ => {
                warn!("Unknown output: {}", output_id);
            }
        }

        Ok(())
    }

    fn expected_inputs(&self) -> Vec<String> {
        vec![
            "audio".to_string(),
            "segment_complete".to_string(),
            "log".to_string(),
            "log_tts".to_string(),
        ]
    }

    fn expected_outputs(&self) -> Vec<String> {
        vec!["text".to_string()]
    }
}
