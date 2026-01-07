use bytes::Bytes;
use eyre::{Result, eyre};
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Reasons why a request was cancelled
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum CancellationReason {
    UserRequested { source: String },
    Timeout { duration_secs: u64 },
    NodeShutdown,
    Error { details: String },
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StreamChoice {
    pub index: i32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<DeltaToolCall>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeltaToolCall {
    pub index: i32,
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub function: Option<DeltaFunctionCall>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeltaFunctionCall {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

use outfox_openai::spec::{ChatCompletionMessageToolCall, ChatCompletionToolType, FunctionCall};
/// Parse Server-Sent Events (SSE) stream from OpenAI-compatible APIs.
///
/// This function handles the streaming response format used by OpenAI's chat
/// completion API when `stream: true` is set. It parses SSE events, extracts
/// content chunks, and calls the provided callback for each chunk.
///
/// # Arguments
/// * `client` - HTTP client for making the request
/// * `url` - API endpoint URL
/// * `api_key` - API authentication key
/// * `request_body` - JSON request body with streaming enabled
/// * `on_chunk` - Callback function called for each text chunk
///
/// # Returns
/// * `Result<String>` - The complete accumulated response text
///
/// # Example Event Format
/// ```text
/// data: {"choices":[{"delta":{"content":"Hello"}}]}
/// data: {"choices":[{"delta":{"content":" world"}}]}
/// data: [DONE]
/// ```
use std::collections::HashMap;

/// Accumulates tool call deltas during streaming
#[derive(Default)]
pub struct ToolCallAccumulator {
    tool_calls: HashMap<i32, ToolCallBuilder>,
}

#[derive(Default, Clone)]
struct ToolCallBuilder {
    id: Option<String>,
    r#type: Option<String>,
    function_name: Option<String>,
    function_arguments: String,
}

impl ToolCallAccumulator {
    pub fn add_delta(&mut self, delta: &DeltaToolCall) {
        let builder = self.tool_calls.entry(delta.index).or_default();

        if let Some(id) = &delta.id {
            builder.id = Some(id.clone());
        }

        if let Some(r#type) = &delta.r#type {
            builder.r#type = Some(r#type.clone());
        }

        if let Some(function) = &delta.function {
            if let Some(name) = &function.name {
                builder.function_name = Some(name.clone());
            }
            if let Some(args) = &function.arguments {
                builder.function_arguments.push_str(args);
            }
        }
    }

    pub fn build_tool_calls(self) -> Vec<ChatCompletionMessageToolCall> {
        let mut tool_calls = Vec::new();

        for (_, builder) in self.tool_calls {
            if let (Some(id), Some(_), Some(name)) =
                (builder.id, builder.r#type, builder.function_name)
            {
                tool_calls.push(ChatCompletionMessageToolCall {
                    id,
                    kind: ChatCompletionToolType::Function,
                    function: FunctionCall {
                        name,
                        arguments: builder.function_arguments,
                    },
                });
            }
        }

        tool_calls
    }

    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

pub async fn stream_completion<F>(
    client: &reqwest::Client,
    url: String,
    api_key: String,
    request_body: serde_json::Value,
    mut on_chunk: F,
) -> Result<(String, Option<Vec<ChatCompletionMessageToolCall>>)>
where
    F: FnMut(String) -> Result<()>,
{
    // Convert request body to Bytes so it can be cloned for EventSource
    let body_bytes: Bytes = serde_json::to_vec(&request_body)?.into();
    let request = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(body_bytes);

    let mut event_source = EventSource::new(request)?;
    let mut accumulated_content = String::new();
    let mut tool_accumulator = ToolCallAccumulator::default();

    // Track last few raw SSE events for debugging
    let mut last_raw_events: std::collections::VecDeque<String> = std::collections::VecDeque::with_capacity(5);
    let mut chunk_count: usize = 0;

    while let Some(event) = event_source.next().await {
        match event {
            Ok(Event::Open) => {}
            Ok(Event::Message(msg)) => {
                let data = msg.data;
                chunk_count += 1;

                // Keep track of last 5 raw events for debugging
                if last_raw_events.len() >= 5 {
                    last_raw_events.pop_front();
                }
                last_raw_events.push_back(data.clone());

                // Check for end of stream
                if data == "[DONE]" {
                    break;
                }

                // Parse the JSON chunk
                match serde_json::from_str::<StreamChunk>(&data) {
                    Ok(chunk) => {
                        if let Some(choice) = chunk.choices.first() {
                            // Handle content
                            if let Some(content) = &choice.delta.content {
                                if !content.is_empty() {
                                    accumulated_content.push_str(content);
                                    on_chunk(content.clone())?;
                                }
                            }

                            // Handle tool calls
                            if let Some(tool_calls) = &choice.delta.tool_calls {
                                for delta_call in tool_calls {
                                    tool_accumulator.add_delta(delta_call);
                                }
                            }
                        }
                    }
                    Err(_e) => {
                        // Silently skip unparseable chunks (common with some APIs)
                    }
                }
            }
            Err(e) => {
                eprintln!("[SSE] Error after {} chunks: {}", chunk_count, e);
                eprintln!("[SSE] Error details: {:?}", e);
                return Err(eyre!("SSE error: {}", e));
            }
        }
    }

    // Build tool calls if any were accumulated
    let tool_calls = if tool_accumulator.has_tool_calls() {
        Some(tool_accumulator.build_tool_calls())
    } else {
        None
    };

    Ok((accumulated_content, tool_calls))
}

/// Stream completion with cancellation support
pub async fn stream_completion_with_cancellation<F>(
    client: &reqwest::Client,
    url: String,
    api_key: String,
    request_body: serde_json::Value,
    cancellation_token: CancellationToken,
    timeout_duration: Duration,
    mut on_chunk: F,
) -> Result<(String, Option<Vec<ChatCompletionMessageToolCall>>)>
where
    F: FnMut(String) -> Result<()>,
{
    // Convert request body to Bytes so it can be cloned for EventSource
    let body_bytes: Bytes = serde_json::to_vec(&request_body)?.into();
    let request = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(body_bytes);

    let mut event_source = EventSource::new(request)?;
    let mut accumulated_content = String::new();
    let mut tool_accumulator = ToolCallAccumulator::default();

    // Track last few raw SSE events for debugging
    let mut last_raw_events: std::collections::VecDeque<String> = std::collections::VecDeque::with_capacity(5);
    let mut chunk_count: usize = 0;

    loop {
        let timeout_future = tokio::time::sleep(timeout_duration);

        tokio::select! {
            event = event_source.next() => {
                match event {
                    Some(Ok(Event::Open)) => {}
                    Some(Ok(Event::Message(msg))) => {
                        let data = msg.data;
                        chunk_count += 1;

                        // Keep track of last 5 raw events for debugging
                        if last_raw_events.len() >= 5 {
                            last_raw_events.pop_front();
                        }
                        last_raw_events.push_back(data.clone());

                        // Check for end of stream
                        if data == "[DONE]" {
                            break;
                        }

                        // Parse the JSON chunk
                        match serde_json::from_str::<StreamChunk>(&data) {
                            Ok(chunk) => {
                                if let Some(choice) = chunk.choices.first() {
                                    // Handle content
                                    if let Some(content) = &choice.delta.content {
                                        if !content.is_empty() {
                                            accumulated_content.push_str(content);
                                            on_chunk(content.clone())?;
                                        }
                                    }

                                    // Handle tool calls
                                    if let Some(tool_calls) = &choice.delta.tool_calls {
                                        for delta_call in tool_calls {
                                            tool_accumulator.add_delta(delta_call);
                                        }
                                    }
                                }
                            }
                            Err(_e) => {
                                // Silently skip unparseable chunks
                            }
                        }
                    }
                    Some(Err(e)) => {
                        if cancellation_token.is_cancelled() {
                            return Err(eyre!("Stream cancelled by user"));
                        }
                        eprintln!("[SSE] Error after {} chunks: {}", chunk_count, e);
                        return Err(eyre!("SSE error: {}", e));
                    }
                    None => {
                        break;
                    }
                }
            }
            _ = cancellation_token.cancelled() => {
                event_source.close();
                return Err(eyre!("Stream cancelled by user"));
            }
            _ = timeout_future => {
                event_source.close();
                return Err(eyre!("Stream timed out after {:?}", timeout_duration));
            }
        }
    }

    // Build tool calls if any were accumulated
    let tool_calls = if tool_accumulator.has_tool_calls() {
        Some(tool_accumulator.build_tool_calls())
    } else {
        None
    };

    Ok((accumulated_content, tool_calls))
}
