use dora_node_api::{self, DoraNode, Event, Parameter};
use dora_node_api::arrow::array::{StringArray, AsArray};
use dora_conference_controller::policies::{Policy, UnifiedRatioPolicy};
use dora_core::config::DataId;
use eyre::Result;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

fn send_log(node: &mut DoraNode, level: LogLevel, config_level: LogLevel, message: &str) {
    if !config_level.allows(level) {
        return;
    }

    let node_name = std::env::var("DORA_NODE_NAME")
        .unwrap_or_else(|_| "conference-controller".to_string());

    let log_data = serde_json::json!({
        "level": format!("{:?}", level).to_uppercase(),
        "message": message,
        "node": node_name,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    });

    match node.send_output(
        DataId::from("log".to_string()),
        Default::default(),
        StringArray::from(vec![log_data.to_string().as_str()]),
    ) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("[Controller] Failed to send log: {}", message);
        }
    }
}

#[derive(Debug, Clone)]
struct ParticipantInput {
    id: String,
    text: String,
    timestamp: i64,
    word_count: usize,
    is_complete: bool,
}

#[derive(Debug, Clone)]
struct StreamingAccumulator {
    accumulated_text: String,
    accumulated_words: usize,
}

#[derive(Debug)]
enum ControllerState {
    Waiting,
    Processing,
}

struct ConferenceController {
    state: ControllerState,
    policy: UnifiedRatioPolicy,
    participant_inputs: HashMap<String, ParticipantInput>,
    streaming_accumulators: HashMap<String, StreamingAccumulator>,
    pattern: String,
    log_level: LogLevel,
}

impl ConferenceController {
    fn new(pattern: String, node: &mut DoraNode, log_level: LogLevel) -> Result<Self> {
        let mut policy = UnifiedRatioPolicy::new();
        policy.configure(&pattern)
            .map_err(|e| eyre::eyre!("Failed to configure policy from pattern: {}", e))?;

        send_log(node, LogLevel::Info, log_level, &format!("✅ Policy configured with participants: {:?}", policy.get_participants()));
        let stats = policy.get_stats();
        send_log(node, LogLevel::Info, log_level, &format!("📊 Policy configuration:\n{}", serde_json::to_string_pretty(&stats).unwrap()));

        Ok(Self {
            state: ControllerState::Waiting,
            policy,
            participant_inputs: HashMap::new(),
            streaming_accumulators: HashMap::new(),
            pattern,
            log_level,
        })
    }

    /// Check if metadata indicates the message is complete
    fn is_message_complete(&self, metadata: &dora_node_api::Metadata) -> bool {
        if let Some(Parameter::String(status)) = metadata.parameters.get("session_status") {
            return status == "ended" || status == "complete";
        }

        // Default to complete if no metadata (non-streaming)
        true
    }

    /// Accumulate streaming chunk and return whether message is now complete
    fn accumulate_streaming_input(
        &mut self,
        participant_id: &str,
        text: String,
        metadata: &dora_node_api::Metadata,
    ) -> (String, usize, bool) {
        let is_complete = self.is_message_complete(metadata);
        let word_count = text.split_whitespace().count();

        if !is_complete || self.streaming_accumulators.contains_key(participant_id) {
            // Streaming in progress or we have previous chunks
            let accumulator = self.streaming_accumulators.entry(participant_id.to_string())
                .or_insert_with(|| StreamingAccumulator {
                    accumulated_text: String::new(),
                    accumulated_words: 0,
                });

            if !accumulator.accumulated_text.is_empty() {
                accumulator.accumulated_text.push(' ');
            }
            accumulator.accumulated_text.push_str(&text);
            accumulator.accumulated_words += word_count;

            if is_complete {
                let complete_text = accumulator.accumulated_text.clone();
                let complete_words = accumulator.accumulated_words;
                self.streaming_accumulators.remove(participant_id);
                (complete_text, complete_words, true)
            } else {
                (accumulator.accumulated_text.clone(), accumulator.accumulated_words, false)
            }
        } else {
            // Non-streaming or first complete message
            (text, word_count, true)
        }
    }

    fn handle_participant_input(
        &mut self,
        participant_id: &str,
        text: String,
        metadata: &dora_node_api::Metadata,
        node: &mut DoraNode,
    ) -> Result<()> {
        println!("[CONTROLLER-STDOUT] 🔍 handle_participant_input() called for: {}", participant_id);

        // Accumulate streaming chunks and check if message is complete
        let (complete_text, word_count, is_complete) =
            self.accumulate_streaming_input(participant_id, text, metadata);

        println!("[CONTROLLER-STDOUT] 📊 accumulate_streaming_input result: complete={}, words={}, text_len={}",
                is_complete, word_count, complete_text.len());

        // Store the input
        let input = ParticipantInput {
            id: participant_id.to_string(),
            text: complete_text.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            word_count,
            is_complete,
        };
        self.participant_inputs.insert(participant_id.to_string(), input);

        // Always accumulate word counts, but only process when complete
        self.policy.update_word_count(participant_id, word_count);

        if is_complete {
            println!("[CONTROLLER] INPUT COMPLETE: {} ({} words)", participant_id, word_count);
            send_log(node, LogLevel::Info, self.log_level, &format!("📥 CONTROLLER: Input COMPLETE from {}: {} words", participant_id, word_count));
            send_log(node, LogLevel::Info, self.log_level, &format!("📊 CONTROLLER: Current policy state: {} ready inputs, {} expected in sequence", self.participant_inputs.len(), self.policy.get_participants().len()));

            // Only process next speaker when message is complete
            self.process_next_speaker(node)?;
        } else {
            send_log(node, LogLevel::Debug, self.log_level, &format!("⏳ CONTROLLER: Input INCOMPLETE from {}: {} words (still accumulating)", participant_id, word_count));
        }

        Ok(())
    }

    fn process_next_speaker(&mut self, node: &mut DoraNode) -> Result<()> {
        println!("[CONTROLLER-STDOUT] 🔍 process_next_speaker() called");

        // Determine the next speaker based on policy
        match self.policy.determine_next_speaker() {
            Some(next_speaker) => {
                println!("[CONTROLLER-STDOUT] 🎯 Policy determined next speaker: {}", next_speaker);
                send_log(node, LogLevel::Info, self.log_level, &format!("🎯 Next speaker determined: {}", next_speaker));

                // Map speaker to control output
                let control_output = match next_speaker.as_str() {
                    "judge" => "control_judge",
                    "llm2" => "control_llm2",
                    "llm1" => "control_llm1",
                    _ => {
                        println!("[CONTROLLER-STDOUT] ⚠️ Unknown next speaker: {}", next_speaker);
                        send_log(node, LogLevel::Warn, self.log_level, &format!("⚠️ Unknown next speaker: {}", next_speaker));
                        return Ok(());
                    }
                };

                println!("[CONTROLLER-STDOUT] 🗺️ Mapped speaker '{}' to control output: '{}'", next_speaker, control_output);

                // Send resume command to the appropriate bridge
                println!("[CONTROLLER-STDOUT] 🚀 SENDING RESUME to {} for speaker: {}", control_output, next_speaker);
                send_log(node, LogLevel::Info, self.log_level, &format!("🚀 CONTROLLER: Sending resume command to bridge via {}", control_output));
                send_log(node, LogLevel::Debug, self.log_level, &format!("📊 CONTROLLER: Next speaker determined: {} (policy: {})", next_speaker, serde_json::to_string(&self.policy.get_stats())?));

                match node.send_output(
                    DataId::from(control_output.to_string()),
                    Default::default(),
                    StringArray::from(vec!["resume"]),
                ) {
                    Ok(_) => println!("[CONTROLLER-STDOUT] ✅ RESUME COMMAND SENT to {}", control_output),
                    Err(e) => println!("[CONTROLLER-STDOUT] ❌ FAILED to send resume to {}: {}", control_output, e),
                }
        } else {
            println!("[CONTROLLER-STDOUT] ⚠️ No next speaker determined by policy");
            send_log(node, LogLevel::Warn, self.log_level, "⚠️ No next speaker determined");
        }

        // Send policy statistics
        println!("[CONTROLLER-STDOUT] 📊 Sending policy statistics");
        match node.send_output(
            DataId::from("status".to_string()),
            Default::default(),
            StringArray::from(vec![serde_json::to_string(&self.policy.get_stats())?.as_str()]),
        ) {
            Ok(_) => println!("[CONTROLLER-STDOUT] ✅ Policy statistics sent successfully"),
            Err(e) => println!("[CONTROLLER-STDOUT] ❌ Failed to send policy statistics: {}", e),
        }

        Ok(())
    }

    fn reset(&mut self, node: &mut DoraNode) -> Result<()> {
        send_log(node, LogLevel::Info, self.log_level, "🔄 Resetting controller state");
        self.participant_inputs.clear();
        self.policy.reset_counts();
        self.state = ControllerState::Waiting;
        Ok(())
    }

    fn get_stats(&self) -> serde_json::Value {
        let mut stats = self.policy.get_stats();

        if let serde_json::Value::Object(ref mut map) = stats {
            map.insert("input_count".to_string(), serde_json::Value::Number(self.participant_inputs.len().into()));
            map.insert(
                "controller_state".to_string(),
                serde_json::Value::String(format!("{:?}", self.state))
            );
        }

        stats
    }
}

/// Parse command line arguments and YAML configuration
fn load_pattern_from_env() -> Result<String> {
    // First check DORA_POLICY_PATTERN environment variable
    if let Ok(pattern) = env::var("DORA_POLICY_PATTERN") {
        println!("📋 Using pattern from DORA_POLICY_PATTERN env var");
        return Ok(pattern);
    }

    // Then check PATTERN environment variable
    if let Ok(pattern) = env::var("PATTERN") {
        println!("📋 Using pattern from PATTERN env var");
        return Ok(pattern);
    }

    // Default pattern
    println!("⚠️ No pattern specified, using default: [Judge → Defense → Prosecution]");
    Ok("[Judge → Defense → Prosecution]".to_string())
}

fn main() -> Result<()> {
    println!("🚀 Loading pattern configuration...");
    let pattern = load_pattern_from_env()?;

    let (mut node, events) = DoraNode::init_from_env()?;

    // Set up logging
    let log_level = env::var("LOG_LEVEL").ok()
        .and_then(|s| LogLevel::parse(&s))
        .unwrap_or(LogLevel::Info);

    send_log(&mut node, LogLevel::Info, log_level, &format!("🚀 Starting Conference Controller with pattern: {}", pattern));
    let mut controller = ConferenceController::new(pattern, &mut node, log_level)?;

    // Block on the event stream to get synchronous iteration
    let mut events = dora_node_api::futures::executor::block_on_stream(events);

    send_log(&mut node, LogLevel::Info, log_level, "🔌 Conference Controller ready and listening for events");
    send_log(&mut node, LogLevel::Info, log_level, "📢 TEST LOG: This should appear in the viewer");
    send_log(&mut node, LogLevel::Info, log_level, "📋 CONFIRMED: Controller log output is connected");
    send_log(&mut node, LogLevel::Info, log_level, "Accepted inputs: Participant inputs (e.g., llm1, llm2, llm3)");
    send_log(&mut node, LogLevel::Info, log_level, "Accepted inputs: control: 'reset' command");
    send_log(&mut node, LogLevel::Info, log_level, "Outputs: control: 'resume' commands to conference bridge");
    send_log(&mut node, LogLevel::Info, log_level, "Outputs: status: JSON stats about controller state");

    // Direct println! statements that should always be visible
    println!("[CONTROLLER-STDOUT] 🚀 CONTROLLER STARTED - Should see this in terminal");
    println!("[CONTROLLER-STDOUT] 📡 Log level: {:?}", log_level);
    println!("[CONTROLLER-STDOUT] 🔍 Testing direct stdout output");

    loop {
        let event = events.next();
        match event {
            Some(Event::Input {
                id,
                metadata,
                data,
                ..
            }) => {
                if id.as_str() == "control" {
                    // Extract text from control input
                    let control_array = data.as_string::<i32>();
                    let control_text = control_array
                        .iter()
                        .filter_map(|s| s)
                        .collect::<Vec<_>>()
                        .join(" ");
                    let control_text = control_text.trim();

                    match control_text.to_lowercase().as_str() {
                        "reset" => {
                            controller.reset(&mut node)?;
                        }
                        "ready" => {
                            node.send_output(
                                DataId::from("status".to_string()),
                                Default::default(),
                                StringArray::from(vec!["ready"]),
                            )?;
                        }
                        "stats" => {
                            let stats = controller.get_stats();
                            node.send_output(
                                DataId::from("status".to_string()),
                                Default::default(),
                                StringArray::from(vec![serde_json::to_string(&stats)?.as_str()]),
                            )?;
                        }
                        _ => {
                            send_log(&mut node, LogLevel::Warn, log_level, &format!("Unknown control command: {}", control_text));
                        }
                    }
                } else {
                    // Participant input - extract text
                    println!("[CONTROLLER-STDOUT] 📨 Raw input received from id: {}", id);

                    let text_array = data.as_string::<i32>();
                    let text = text_array
                        .iter()
                        .filter_map(|s| s)
                        .collect::<Vec<_>>()
                        .join(" ");

                    println!("[CONTROLLER-STDOUT] 📝 Extracted text from {}: {} chars", id, text.len());
                    send_log(&mut node, LogLevel::Debug, log_level, &format!("📨 Processing input from {}", id));

                    println!("[CONTROLLER-STDOUT] 🔄 Calling handle_participant_input for {}", id);
                    if let Err(e) = controller.handle_participant_input(id.as_str(), text, &metadata, &mut node) {
                        println!("[CONTROLLER-STDOUT] ❌ handle_participant_input failed for {}: {}", id, e);
                        send_log(&mut node, LogLevel::Error, log_level, &format!("❌ Error handling input: {}", e));
                    } else {
                        println!("[CONTROLLER-STDOUT] ✅ handle_participant_input completed for {}", id);
                    }
                }
            }
            Some(Event::Stop(_cause)) => {
                send_log(&mut node, LogLevel::Info, log_level, "🛑 Received stop event, shutting down");
                break;
            }
            Some(Event::Error(e)) => {
                send_log(&mut node, LogLevel::Error, log_level, &format!("❌ Error: {}", e));
            }
            _ => {}
        }
    }

    Ok(())
}
