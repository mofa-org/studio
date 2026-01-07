use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::env;

use dora_node_api::{
    DoraNode, Event, Parameter,
    arrow::array::{Array, AsArray, StringArray},
    dora_core::config::DataId,
};
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const NODE_NAME: &str = "dora-conference-bridge";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl LogLevel {
    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warn" | "warning" => Some(LogLevel::Warn),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            _ => None,
        }
    }

    fn allows(self, other: LogLevel) -> bool {
        other as i32 <= self as i32
    }
}

fn get_friendly_node_name(node_id: &str) -> String {
    // Check if we're in study mode by looking for environment variable
    let study_mode = std::env::var("DORA_STUDY_MODE")
        .unwrap_or_default()
        .to_ascii_lowercase() == "true";

    // Convert technical node IDs to user-friendly names
    if study_mode {
        match node_id {
            "bridge-to-tutor" => "Bridge to Tutor".to_string(),
            "bridge-to-student1" => "Bridge to Student1".to_string(),
            "bridge-to-student2" => "Bridge to Student2".to_string(),
            _ => {
                // Fallback: make technical ID more readable
                node_id.replace("-", " ")
                    .replace("bridge", "Bridge")
                    .replace("student", "Student")
                    .replace("tutor", "Tutor")
            }
        }
    } else {
        match node_id {
            "bridge-to-judge" => "Bridge to Judge".to_string(),
            "bridge-to-llm1" => "Bridge to LLM1".to_string(),
            "bridge-to-llm2" => "Bridge to LLM2".to_string(),
            _ => {
                // Fallback: make technical ID more readable
                node_id.replace("-", " ")
                    .replace("bridge", "Bridge")
                    .replace("llm", "LLM")
                    .replace("judge", "Judge")
            }
        }
    }
}

fn send_log(node: &mut DoraNode, level: LogLevel, config_level: LogLevel, message: &str) {
    if !config_level.allows(level) {
        return;
    }

    // Priority: Env var → Built-in node ID → Default constant
    let node_identifier = std::env::var("DORA_NODE_NAME")
        .or_else(|_| std::env::var("DORA_NODE_ID"))
        .unwrap_or_else(|_| get_friendly_node_name(&node.id().to_string()));

    let level_str = match level {
        LogLevel::Error => "ERROR",
        LogLevel::Warn => "WARNING",
        LogLevel::Info => "INFO",
        LogLevel::Debug => "DEBUG",
    };

    let log_data = serde_json::json!({
        "node": node_identifier,
        "level": level_str,
        "message": message,
        "timestamp": chrono::Utc::now().timestamp()
    });

    if let Err(err) = node.send_output(
        DataId::from("log".to_string()),
        Default::default(),
        StringArray::from(vec![log_data.to_string().as_str()]),
    ) {
        eprintln!("[Conference Bridge] Failed to send log output: {:?}", err);
    }
}


#[derive(Debug, Clone, PartialEq)]
enum SignalType {
    ResetSignal,      // session_status: "reset" - control signal, drop silently
    CancelledSignal,  // session_status: "cancelled" - control signal, drop silently
    TechnicalError,   // session_status: "error" - actual error, may need notification
    ContentError,     // Text-based errors like "Error:" - forward template if configured
    NormalContent,    // Regular content - forward as-is
}

/// Classify the type of signal from metadata and text content
fn classify_signal(metadata: &BTreeMap<String, Parameter>, text: &str) -> SignalType {
    // First check session_status in metadata
    if let Some(Parameter::String(status)) = metadata.get("session_status") {
        match status.as_str() {
            "reset" => return SignalType::ResetSignal,
            "cancelled" => return SignalType::CancelledSignal,
            "error" => return SignalType::TechnicalError,
            _ => {} // Fall through to text-based detection
        }
    }

    // Check for text-based error patterns
    if text.starts_with("Error:") || text.starts_with("error:") {
        SignalType::ContentError
    } else {
        // Default to normal content
        SignalType::NormalContent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundledMessage {
    participant: String,
    content: String,
    complete: bool,
}

#[derive(Debug, Clone)]
enum MessageState {
    Streaming {
        chunks: Vec<String>,
        metadata: BTreeMap<String, Parameter>,
    },
    Complete {
        content: String,
    },
}

impl MessageState {
    fn new_streaming() -> Self {
        MessageState::Streaming {
            chunks: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    fn add_chunk(&mut self, chunk: String, metadata: BTreeMap<String, Parameter>) {
        match self {
            MessageState::Streaming { chunks, metadata: meta } => {
                chunks.push(chunk);
                meta.extend(metadata);
            }
            _ => {}
        }
    }

    fn complete(&mut self) -> String {
        match self {
            MessageState::Streaming { chunks, .. } => {
                let content = chunks.join("");
                *self = MessageState::Complete {
                    content: content.clone(),
                };
                content
            }
            MessageState::Complete { content } => content.clone(),
        }
    }

    fn get_content(&self) -> String {
        match self {
            MessageState::Streaming { chunks, .. } => chunks.join(""),
            MessageState::Complete { content, .. } => content.clone(),
        }
    }

    fn is_complete(&self) -> bool {
        matches!(self, MessageState::Complete { .. })
    }

}

#[derive(Debug)]
struct InputPort {
    port_name: String,
    is_streaming: bool,  // Explicitly configured
    message_state: Option<MessageState>,
    ready: bool,
    draining: bool,
    was_already_ready: bool, // Track if we already logged this as ready
    signal_type: Option<SignalType>,  // Type of signal detected, if any
    should_forward: bool,  // Whether this input should be forwarded (false for control signals)
}

impl InputPort {
    fn new(port_name: String, is_streaming: bool) -> Self {
        Self {
            port_name,
            is_streaming,
            message_state: None,
            ready: false,
            draining: false,
            was_already_ready: false,
            signal_type: None,
            should_forward: true,  // Default to forwarding
        }
    }

    fn is_message_complete(&self, metadata: &BTreeMap<String, Parameter>) -> bool {
        // Check session_status
        if let Some(Parameter::String(status)) = metadata.get("session_status") {
            // ended = normal completion
            // error = error occurred
            // cancelled = user cancelled streaming (history preserved in LLM)
            // reset = user reset (streaming cancelled + history cleared)
            if status == "ended" || status == "error" || status == "cancelled" || status == "reset" {
                return true;
            }
        }

        // Check is_complete
        if let Some(Parameter::Bool(true)) = metadata.get("is_complete") {
            return true;
        }

        false
    }

    fn is_drop_status(metadata: &BTreeMap<String, Parameter>) -> bool {
        if let Some(Parameter::String(status)) = metadata.get("session_status") {
            // error, cancelled, and reset should all be dropped during forwarding
            // (they represent incomplete/interrupted responses)
            return status == "error" || status == "cancelled" || status == "reset";
        }
        false
    }

    /// Check if text content indicates an error (fallback when metadata isn't set)
    fn is_error_text(text: &str) -> bool {
        // Check for common error prefixes from maas-client
        text.starts_with("Error:") || text.starts_with("error:")
    }

    fn handle_input(&mut self, text: String, metadata: BTreeMap<String, Parameter>) -> bool {
        // Classify the signal type from metadata and text
        let signal_type = classify_signal(&metadata, &text);
        self.signal_type = Some(signal_type.clone());

        // For control signals (reset/cancelled), discard completely - don't accumulate or mark ready
        // These are notifications, not content to be forwarded
        if matches!(signal_type, SignalType::ResetSignal | SignalType::CancelledSignal) {
            // Clear any accumulated state - this input port is now empty
            self.message_state = None;
            self.ready = false;
            self.should_forward = false;
            return false;  // Not ready, nothing accumulated
        }

        // Determine if this should be forwarded based on signal type
        self.should_forward = match signal_type {
            SignalType::ResetSignal | SignalType::CancelledSignal => {
                // Already handled above, but keep for completeness
                false
            }
            SignalType::TechnicalError | SignalType::ContentError => {
                true   // Forward template message for errors
            }
            SignalType::NormalContent => {
                true   // Forward normal content as-is
            }
        };

        // Check if this is the start of a new message (session_status: "started")
        let is_new_start = metadata.get("session_status")
            .and_then(|p| match p {
                Parameter::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map_or(false, |status| status == "started");

        // Reset message state if this is a new start
        if is_new_start {
            self.message_state = Some(MessageState::new_streaming());
            self.ready = false;
            self.was_already_ready = false;
        }

        if self.is_streaming {
            // Streaming input - accumulate chunks
            if self.message_state.is_none() {
                self.message_state = Some(MessageState::new_streaming());
            }

            // Only add chunk if it's not an empty ending signal
            if let Some(state) = &mut self.message_state {
                let is_ending_signal = text.trim().is_empty() &&
                    metadata.get("session_status")
                        .and_then(|p| match p {
                            Parameter::String(s) => Some(s.as_str()),
                            _ => None,
                        })
                        .map_or(false, |status| status == "ended" || status == "error" || status == "cancelled" || status == "reset");

                if !is_ending_signal {
                    state.add_chunk(text, metadata.clone());
                }
            }

            // Check if complete (includes error, cancelled, reset status)
            if self.is_message_complete(&metadata) {
                if let Some(state) = &mut self.message_state {
                    state.complete();
                }
                self.ready = true;
                return true;  // Ready to forward (will be filtered in forward_bundle)
            }
        } else {
            // Non-streaming input - complete immediately
            self.message_state = Some(MessageState::Complete {
                content: text,
            });
            self.ready = true;
            return true;  // Ready to forward (will be filtered in forward_bundle)
        }

        false  // Not ready yet
    }

    fn get_bundled_message(&self) -> Option<BundledMessage> {
        self.message_state.as_ref().map(|state| BundledMessage {
            participant: self.port_name.clone(),
            content: state.get_content(),
            complete: state.is_complete(),
        })
    }

    fn reset(&mut self) {
        self.message_state = None;
        self.ready = false;
        self.draining = false;
        self.was_already_ready = false;
        self.signal_type = None;
        self.should_forward = true;
    }

    fn reset_with_drain(&mut self, drain: bool) {
        self.message_state = None;
        self.ready = false;
        self.draining = drain;
        self.was_already_ready = false;
        self.signal_type = None;
        self.should_forward = true;
    }

    fn is_streaming_active(&self) -> bool {
        matches!(self.message_state, Some(MessageState::Streaming { .. }))
    }

}

fn metadata_indicates_completion(metadata: &BTreeMap<String, Parameter>) -> bool {
    match metadata.get("session_status") {
        Some(Parameter::String(status)) if status == "ended" || status == "error" || status == "cancelled" || status == "reset" => return true,
        _ => {}
    }

    matches!(metadata.get("is_complete"), Some(Parameter::Bool(true)))
}

struct ConferenceBridge {
    inputs: HashMap<String, InputPort>,
    streaming_ports: HashSet<String>,
    expected_ports: HashSet<String>,
    log_level: LogLevel,
    arrival_queue: VecDeque<String>,
    current_question_id: u32,
    controller_question_id: Option<u32>,  // Track controller's question_id
    has_controller_input: bool,           // Flag if controller provided question_id
    last_status: String,  // Track last status to avoid duplicate logs
    resume_mode: bool,     // Track if bridge is in resume mode
    error_message_template: Option<String>,  // Template for error messages, {participant} will be replaced
  }

impl ConferenceBridge {
    fn new(
        streaming_ports: HashSet<String>,
        expected_ports: HashSet<String>,
        log_level: LogLevel,
        _increment_question_id: bool,  // Parameter kept for compatibility but ignored
        error_message_template: Option<String>,
    ) -> Self {
        let mut bridge = Self {
            inputs: HashMap::new(),
            streaming_ports,
            expected_ports,
            log_level,
            arrival_queue: VecDeque::new(),
            current_question_id: 0,
            controller_question_id: None,
            has_controller_input: false,
            last_status: String::new(),
            resume_mode: false,  // Start in paused mode
            error_message_template,
        };

        let preset_ports: Vec<String> = bridge
            .expected_ports
            .iter()
            .filter(|name| !name.trim().is_empty())
            .cloned()
            .collect();
        for port in preset_ports {
            bridge.register_input(port);
        }

        bridge
    }

    fn register_input(&mut self, port_name: String) {
        if !self.inputs.contains_key(&port_name) {
            let is_streaming = self.streaming_ports.contains(&port_name);
            self.inputs.insert(port_name.clone(), InputPort::new(port_name, is_streaming));
        }
    }

    /// Handle control input from conference controller
    fn handle_control_input(&mut self, node: &mut DoraNode,
                           control_text: &str, metadata: &dora_node_api::Metadata) -> Result<()> {

        if control_text == "resume" {
            // Extract question_id from controller's resume command
            if let Some(dora_node_api::Parameter::String(qid_str)) = metadata.parameters.get("question_id") {
                if let Ok(qid) = qid_str.parse::<u32>() {
                    self.controller_question_id = Some(qid);
                    self.has_controller_input = true;
                    self.resume_mode = true;

                    send_log(node, LogLevel::Info, self.log_level,
                        &format!("▶️ Bridge using controller question_id: {}", qid));
                } else {
                    send_log(node, LogLevel::Warn, self.log_level,
                        &format!("⚠️ Invalid question_id format: {}", qid_str));
                }
            } else {
                send_log(node, LogLevel::Warn, self.log_level,
                    "⚠️ Resume command without question_id - using default behavior");
                self.has_controller_input = false;
            }
        } else if control_text == "reset" {
            // Reset doesn't affect question_id - controller will provide new one in next resume
            self.controller_question_id = None;
            self.has_controller_input = false;
            self.resume_mode = false;
        }

        Ok(())
    }

    /// Send status output, but only if it has changed from last time (deduplication)
    fn send_status(&mut self, node: &mut DoraNode, status: &str) -> Result<()> {
        // Only send status if it has changed from last time
        if self.last_status != status {
            node.send_output(
                DataId::from("status".to_string()),
                Default::default(),
                StringArray::from(vec![status]),
            )
            .context("Failed to send status output")?;
            self.last_status = status.to_string();
        }
        Ok(())
    }

    fn handle_input(&mut self, port_name: &str, text: String, metadata: BTreeMap<String, Parameter>) -> bool {
        // Register input if not known
        self.register_input(port_name.to_string());

        // Track arrival order in FIFO queue (only add if not already present)
        if !self.arrival_queue.iter().any(|p| p == port_name) {
            self.arrival_queue.push_back(port_name.to_string());
        }

        // Extract question_id from metadata (use first arrival's question_id)
        if self.current_question_id == 0 {
            if let Some(Parameter::String(qid_str)) = metadata.get("question_id") {
                if let Ok(qid) = qid_str.parse::<u32>() {
                    self.current_question_id = qid;
                }
            }
        }

        // Handle the input
        if let Some(input) = self.inputs.get_mut(port_name) {
            input.handle_input(text, metadata)
        } else {
            false
        }
    }


    fn get_ready_inputs(&self) -> HashSet<String> {
        self.inputs.iter()
            .filter(|(_, input)| input.ready)
            .map(|(name, _)| name.clone())
            .collect()
    }

    fn handle_drain(
        &mut self,
        port_name: &str,
        is_starting: bool,
        is_complete: bool,
        has_session_status: bool,
    ) -> bool {
        if let Some(input) = self.inputs.get_mut(port_name) {
            if input.draining {
                if is_complete {
                    input.draining = false;
                    return true;
                }

                if is_starting || !has_session_status {
                    input.draining = false;
                    return false;
                }

                return true;
            }
        }

        false
    }

    fn finalize_cycle(&mut self, node: &mut DoraNode, status: &str) -> Result<()> {

        for (_, input) in self.inputs.iter_mut() {
            input.reset();
        }

        self.arrival_queue.clear();
        self.current_question_id = 0;

        self.send_status(node, status)?;
        Ok(())
    }

    fn reset_state(&mut self, node: &mut DoraNode) -> Result<()> {
        // Force clear all inputs EXCEPT human input - don't drain, just reset immediately
        // Any in-flight streaming chunks will be dropped
        // Preserve human input (from ASR) so it can be forwarded after reset
        for (port_name, input) in self.inputs.iter_mut() {
            // Check if this is human input by looking for "human" in port name or checking source
            // Human input typically comes from ASR (asr/transcription source)
            let is_human_input = port_name.to_lowercase().contains("human");

            if is_human_input {
                send_log(
                    node,
                    LogLevel::Info,
                    self.log_level,
                    &format!("🔄 PRESERVING human input during reset: {}", port_name),
                );
                // Don't reset human input - keep it for forwarding
                continue;
            }

            if input.is_streaming_active() {
                send_log(
                    node,
                    LogLevel::Info,
                    self.log_level,
                    &format!("🔄 Force clearing active streaming input: {}", port_name),
                );
            }
            input.reset();  // Force clear, don't drain
        }

        // Don't clear arrival_queue completely - remove non-human entries but keep human
        self.arrival_queue.retain(|port_name| port_name.to_lowercase().contains("human"));

        self.current_question_id = 0;
        self.resume_mode = false;  // Reset to pause mode

        send_log(
            node,
            LogLevel::Info,
            self.log_level,
            "✅ Bridge reset complete - all inputs cleared (human input preserved), ready for new conversation",
        );
        self.send_status(node, "reset")?;
        Ok(())
    }

    fn forward_bundle(&mut self, node: &mut DoraNode) -> Result<()> {
        send_log(
            node,
            LogLevel::Debug,
            self.log_level,
            &format!("🚀 FORWARDING BUNDLE - queue: {:?}, {} ready inputs", self.arrival_queue, self.get_ready_inputs().len()),
        );

        // Step 1: Collect messages in FIFO order and concatenate
        let mut concatenated_content = String::new();
        let mut forwarded_count = 0;

        // Iterate in FIFO queue order (not arbitrary HashMap order)
        for port_name in &self.arrival_queue {
            if let Some(input) = self.inputs.get(port_name) {
                if !input.ready {
                    continue; // Skip if not ready (cold start case)
                }

                // Handle signals based on type - either drop silently or forward template message
                if !input.should_forward {
                    send_log(
                        node,
                        LogLevel::Debug,
                        self.log_level,
                        &format!("🚫 Dropping {:?} signal from {}", input.signal_type, port_name),
                    );
                    continue;  // Skip control signals (reset, cancelled)
                }

                // Handle error signals that should be forwarded with template message
                if let Some(signal_type) = &input.signal_type {
                    if matches!(signal_type, SignalType::TechnicalError | SignalType::ContentError) {
                        // If we have an error message template, create and forward the error message
                        if let Some(template) = &self.error_message_template {
                            // Convert port_name to friendly participant name using study mode detection
                            let study_mode = std::env::var("DORA_STUDY_MODE")
                                .unwrap_or_default()
                                .to_ascii_lowercase() == "true";

                            let participant_name = if study_mode {
                                port_name
                                    .replace("student1", "Student1")
                                    .replace("student2", "Student2")
                                    .replace("tutor", "Tutor")
                            } else {
                                port_name
                                    .replace("llm1", "LLM1")
                                    .replace("llm2", "LLM2")
                                    .replace("judge", "Judge")
                            };

                            let error_message = template.replace("{participant}", &participant_name);
                            send_log(
                                node,
                                LogLevel::Warn,
                                self.log_level,
                                &format!("📢 {} had an error - sending notification: {}", port_name, error_message),
                            );

                            // Add error message to concatenated content
                            if !concatenated_content.is_empty() {
                                concatenated_content.push('\n');
                            }
                            concatenated_content.push_str(&error_message);
                            forwarded_count += 1;
                        } else {
                            // No template - just skip
                            send_log(
                                node,
                                LogLevel::Warn,
                                self.log_level,
                                &format!("❌ Dropping error input from {} - no error message template", port_name),
                            );
                        }
                        continue;
                    }

                    // Skip NormalContent - it will be handled by regular content processing below
                    if matches!(signal_type, SignalType::NormalContent) {
                        // Continue to regular content processing
                    }
                }

                if let Some(message) = input.get_bundled_message() {
                    // Skip empty messages (completion signals with no content)
                    if message.content.trim().is_empty() {
                        continue;
                    }

                    // Add content with newline separator
                    if !concatenated_content.is_empty() {
                        concatenated_content.push('\n');
                    }
                    concatenated_content.push_str(&message.content);
                    forwarded_count += 1;

                    send_log(
                        node,
                        LogLevel::Debug,
                        self.log_level,
                        &format!("📦 Adding {} to bundle: {} chars", message.participant, message.content.len()),
                    );
                }
            }
        }

        if forwarded_count == 0 {
            send_log(node, LogLevel::Debug, self.log_level, "No messages ready to forward");
            return Ok(());
        }

        // Clear the arrival queue and reset input states after forwarding
        self.arrival_queue.clear();
        for input in self.inputs.values_mut() {
            input.reset();
        }

        // Use controller's question_id if provided, otherwise generate default
        let output_question_id = if let Some(controller_qid) = self.controller_question_id {
            controller_qid  // Use controller's question_id
        } else {
            1  // Simple fallback for standalone usage
        };

        let mut output_metadata = BTreeMap::new();
        output_metadata.insert(
            "question_id".to_string(),
            Parameter::String(output_question_id.to_string()),
        );

        send_log(node, LogLevel::Debug, self.log_level,
            &format!("📤 Forwarding with question_id: {} ({})",
                output_question_id,
                if self.has_controller_input { "controller" } else { "fallback" }));

        send_log(node, LogLevel::Debug, self.log_level,
            &format!("📤 Sending {} chars from {} inputs", concatenated_content.len(), forwarded_count));

        // Step 3: Send concatenated output with metadata
        node.send_output(
            DataId::from("text".to_string()),
            output_metadata,
            StringArray::from(vec![concatenated_content.as_str()]),
        )
        .context("Failed to send bundled text output")?;

        send_log(
            node,
            LogLevel::Debug,
            self.log_level,
            &format!("✅ SENT: Successfully sent {} chars to text output", concatenated_content.len()),
        );

        // Step 4: Update state
        // No longer auto-incrementing question_id - controller manages it
        if self.has_controller_input {
            // Update stored question_id to match controller's latest
            if let Some(controller_qid) = self.controller_question_id {
                self.current_question_id = controller_qid;
            }
        }

        self.finalize_cycle(node, "forwarded")
    }
}

fn main() -> Result<()> {
    // Load configuration from environment
    let streaming_ports = env::var("STREAMING_PORTS").ok()
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect::<HashSet<String>>();

    let log_level = env::var("LOG_LEVEL").ok()
        .and_then(|s| LogLevel::parse(&s))
        .unwrap_or(LogLevel::Info);

    // INC_QUESTION_ID is no longer used - controller manages question_id
    let increment_question_id = env::var("INC_QUESTION_ID").ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false); // Kept for compatibility but ignored
    let (mut node, mut events) =
        DoraNode::init_from_env().context("Failed to initialize Dora node from environment")?;

    let mut expected_ports: HashSet<String> = node
        .node_config()
        .inputs
        .keys()
        .map(|data_id| data_id.to_string())
        .filter(|name| name != "control")
        .collect();
    if expected_ports.is_empty() {
        expected_ports = streaming_ports.clone();
    }

    // Read error message template - {participant} will be replaced with the participant name
    // Example: "{participant} is experiencing technical difficulties. We will proceed without their response."
    let error_message_template = env::var("ERROR_MESSAGE_TEMPLATE").ok();

    let mut bridge = ConferenceBridge::new(
        streaming_ports.clone(),
        expected_ports,
        log_level,
        increment_question_id,
        error_message_template,
    );

    send_log(
        &mut node,
        LogLevel::Info,
        log_level,
        "Conference bridge initialized - forwarding controlled by controller",
    );

    send_log(
        &mut node,
        LogLevel::Info,
        log_level,
        &format!("Increment question_id: {}", increment_question_id),
    );

    if !streaming_ports.is_empty() {
        send_log(
            &mut node,
            LogLevel::Info,
            log_level,
            &format!("Streaming ports: {:?}", streaming_ports),
        );
    } else {
        send_log(
            &mut node,
            LogLevel::Info,
            log_level,
            "No streaming ports - all inputs treated as non-streaming",
        );
    }

    bridge.send_status(&mut node, "waiting")?;

    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, data, metadata } => {
                let port_name = id.as_str().to_string();

                if port_name == "control" {
                    let control_array = data.as_string::<i32>();
                    let control_payload = control_array
                        .iter()
                        .filter_map(|value| value.map(str::to_string))
                        .collect::<Vec<String>>()
                        .join(" ");

                    let trimmed = control_payload.trim();
                    let mut command: Option<String> = None;

                    if trimmed.is_empty() {
                        send_log(&mut node, LogLevel::Warn, log_level, "Empty control message");
                    } else if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
                        if let Some(cmd) = value.get("command").and_then(|v| v.as_str()) {
                            command = Some(cmd.to_ascii_lowercase());
                        }
                    } else {
                        command = Some(trimmed.to_ascii_lowercase());
                    }

                    match command.as_deref() {
                        Some("reset") => {
                            // Handle controller's reset command
                            if let Err(e) = bridge.handle_control_input(&mut node, "reset", &metadata) {
                                send_log(&mut node, LogLevel::Error, log_level,
                                    &format!("❌ Error handling control input: {}", e));
                            }
                            bridge.reset_state(&mut node)?;
                            send_log(&mut node, LogLevel::Info, log_level, "🔄 Reset command received");
                        }
                        Some("resume") => {
                            // Handle controller's resume command with question_id
                            if let Err(e) = bridge.handle_control_input(&mut node, "resume", &metadata) {
                                send_log(&mut node, LogLevel::Error, log_level,
                                    &format!("❌ Error handling control input: {}", e));
                            }

                            let any_streaming = bridge.inputs.values().any(|input| input.is_streaming_active());
                            let ready_inputs = bridge.get_ready_inputs();

                            // Forward if there are ready inputs AND no ongoing streaming
                            if !ready_inputs.is_empty() && !any_streaming {
                                send_log(&mut node, LogLevel::Info, log_level,
                                    &format!("🚀 Forwarding {} ready inputs", ready_inputs.len()));
                                match bridge.forward_bundle(&mut node) {
                                    Ok(_) => {
                                        bridge.resume_mode = false;
                                        send_log(&mut node, LogLevel::Debug, log_level, "✅ Forward complete");
                                    }
                                    Err(e) => {
                                        send_log(&mut node, LogLevel::Error, log_level,
                                            &format!("❌ Forward failed: {}", e));
                                    }
                                }
                            } else {
                                send_log(&mut node, LogLevel::Debug, log_level,
                                    &format!("⏳ Waiting for inputs (ready={}, streaming={})", ready_inputs.len(), any_streaming));
                            }
                        }
                        Some(other) => {
                            send_log(&mut node, LogLevel::Warn, log_level, &format!("Unknown command: {}", other));
                        }
                        None => {}
                    }
                    continue;
                }

                let parameters = metadata.parameters;

                let text_array = data.as_string::<i32>();
                let text = text_array
                    .iter()
                    .filter_map(|value| value.map(str::to_string))
                    .collect::<Vec<String>>()
                    .join(" ");

                bridge.register_input(port_name.clone());
                let completion_signal = metadata_indicates_completion(&parameters);

                let session_status_value = parameters
                    .get("session_status")
                    .and_then(|param| match param {
                        Parameter::String(status) => Some(status.as_str()),
                        _ => None,
                    });
                let is_starting = session_status_value
                    .map(|status| status.eq_ignore_ascii_case("started"))
                    .unwrap_or(false);
                let has_session_status = session_status_value.is_some();

                if bridge.handle_drain(&port_name, is_starting, completion_signal, has_session_status) {
                    continue;
                }

                if text.trim().is_empty() && !completion_signal {
                    continue;
                }

                // CRITICAL: Check if this is a reset signal from participant output
                // session_status: "reset" from LLM output = LAST message from old debate
                // When we see this, discard ALL accumulated inputs (they're all from old debate)
                let is_reset_signal = session_status_value == Some("reset");

                if is_reset_signal {
                    send_log(&mut node, LogLevel::Info, log_level,
                        &format!("🔄 RESET SIGNAL from {} - discarding ALL queued inputs", port_name));
                    bridge.reset_state(&mut node)?;
                    continue;  // Skip further processing, wait for new debate
                }

                let input_ready = bridge.handle_input(&port_name, text, parameters);

                if input_ready {
                    // Mark as ready (deduplication)
                    if let Some(input) = bridge.inputs.get_mut(&port_name) {
                        if !input.was_already_ready {
                            input.was_already_ready = true;
                            // Log state changes only for errors
                            if matches!(input.signal_type, Some(SignalType::TechnicalError) | Some(SignalType::ContentError)) {
                                send_log(&mut node, LogLevel::Warn, log_level,
                                    &format!("❌ Input {} completed with ERROR", port_name));
                            }
                        }
                    }
                }

                // If input completed and bridge is in resume mode, check if we can forward
                if bridge.resume_mode && input_ready {
                    let any_streaming = bridge.inputs.values().any(|input| input.is_streaming_active());
                    let ready_inputs = bridge.get_ready_inputs();

                    // Only forward if no other inputs are still streaming
                    if !any_streaming && !ready_inputs.is_empty() {
                        send_log(&mut node, LogLevel::Info, log_level,
                            &format!("🚀 Forwarding {} ready inputs", ready_inputs.len()));

                        match bridge.forward_bundle(&mut node) {
                            Ok(_) => {
                                bridge.resume_mode = false;
                                send_log(&mut node, LogLevel::Debug, log_level, "✅ Forward complete");
                            }
                            Err(e) => {
                                send_log(&mut node, LogLevel::Error, log_level,
                                    &format!("❌ Forward failed: {}", e));
                            }
                        }
                    }
                }

                // Update status
                if bridge.resume_mode {
                    bridge.send_status(&mut node, "resume")?;
                } else {
                    bridge.send_status(&mut node, "waiting")?;
                }
            }
            Event::Stop(_) => {
                send_log(&mut node, LogLevel::Info, log_level, "Received stop event, shutting down");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
