use eyre::{Result, eyre};
use outfox_openai::spec::{
    ChatCompletionMessageToolCall, CreateChatCompletionRequest, CreateChatCompletionResponse,
};
use reqwest::Client as HttpClient;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use std::time::Duration;

use crate::config::{GeminiConfig, OpenaiConfig, get_env_or_value};

/// Trait for chat completion clients supporting multiple providers.
///
/// Implementations of this trait provide a unified interface for interacting
/// with different LLM providers (OpenAI, Gemini, etc.) while maintaining
/// consistent behavior across providers.
#[async_trait::async_trait]
pub trait ChatClient: Send + Sync {
    /// Send a non-streaming chat completion request.
    ///
    /// # Arguments
    /// * `request` - The chat completion request with messages and parameters
    ///
    /// # Returns
    /// * `Result<CreateChatCompletionResponse>` - The complete response or error
    async fn complete(
        &self,
        request: CreateChatCompletionRequest,
    ) -> Result<CreateChatCompletionResponse>;

    /// Send a streaming chat completion request.
    ///
    /// Chunks are sent through the provided channel as they arrive from the API.
    /// The method returns the complete accumulated response after streaming ends.
    ///
    /// FIX: Updated to return both text and tool calls from streaming responses.
    /// This enables proper handling of function calling in streaming mode.
    ///
    /// # Arguments
    /// * `request` - The chat completion request
    /// * `chunk_sender` - Channel to send text chunks as they arrive
    ///
    /// # Returns
    /// * `Result<(String, Option<Vec<ChatCompletionMessageToolCall>>)>` - The complete accumulated response text and any tool calls
    async fn complete_streaming(
        &self,
        request: CreateChatCompletionRequest,
        chunk_sender: mpsc::UnboundedSender<String>,
    ) -> Result<(String, Option<Vec<ChatCompletionMessageToolCall>>)>;

    /// Send a streaming chat completion request with cancellation support.
    async fn complete_streaming_with_cancellation(
        &self,
        request: CreateChatCompletionRequest,
        chunk_sender: mpsc::UnboundedSender<String>,
        cancellation_token: CancellationToken,
        timeout_duration: Duration,
    ) -> Result<(String, Option<Vec<ChatCompletionMessageToolCall>>)>;
}

#[derive(Debug)]
pub struct GeminiClient {
    id: String,
    api_key: String,
    api_url: String,
    client: HttpClient,
}

impl GeminiClient {
    pub fn new(config: &GeminiConfig) -> Self {
        Self::new_with_timeout(config, Duration::from_secs(30))
    }

    pub fn new_with_timeout(config: &GeminiConfig, timeout: Duration) -> Self {
        let client = if config.proxy {
            HttpClient::builder()
                .timeout(timeout)
                .connect_timeout(Duration::from_secs(10))
                .pool_idle_timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| HttpClient::new())
        } else {
            HttpClient::builder()
                .timeout(timeout)
                .connect_timeout(Duration::from_secs(10))
                .pool_idle_timeout(Duration::from_secs(30))
                .no_proxy()
                .build()
                .unwrap_or_else(|_| HttpClient::new())
        };

        Self {
            id: config.id.clone(),
            api_key: get_env_or_value(&config.api_key),
            api_url: get_env_or_value(&config.api_url),
            client,
        }
    }
}

#[async_trait::async_trait]
impl ChatClient for GeminiClient {
    async fn complete(
        &self,
        request: CreateChatCompletionRequest,
    ) -> Result<CreateChatCompletionResponse> {
        let response = self
            .client
            .post(&self.api_url)
            .header("X-goog-api-key", self.api_key.clone())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        eprintln!("[{}] Got response with status: {}", self.id, status);

        if !status.is_success() {
            let error_text = response.text().await?;
            eprintln!("[{}] Error response: {}", self.id, error_text);
            return Err(eyre!("API Error: {}", error_text));
        }
        let text_data = response.text().await?;
        // Debug log the response for troubleshooting
        if text_data.is_empty() {
            eprintln!("[{}] Empty response received!", self.id);
        } else if text_data.len() < 1000 {
            eprintln!("[{}] Response: {}", self.id, text_data);
        } else {
            eprintln!(
                "[{}] Response (first 1000 chars): {}",
                self.id,
                &text_data[..1000]
            );
        }
        let completion: CreateChatCompletionResponse =
            serde_json::from_str(&text_data).map_err(|e| {
                eyre!(
                    "Failed to parse API response: {}. Response: {}",
                    e,
                    text_data
                )
            })?;
        Ok(completion)
    }

    async fn complete_streaming(
        &self,
        _request: CreateChatCompletionRequest,
        _chunk_sender: mpsc::UnboundedSender<String>,
    ) -> Result<(String, Option<Vec<ChatCompletionMessageToolCall>>)> {
        Err(eyre!(
            "Streaming not implemented for provider '{}'",
            self.id
        ))
    }

    async fn complete_streaming_with_cancellation(
        &self,
        _request: CreateChatCompletionRequest,
        _chunk_sender: mpsc::UnboundedSender<String>,
        _cancellation_token: CancellationToken,
        _timeout_duration: Duration,
    ) -> Result<(String, Option<Vec<ChatCompletionMessageToolCall>>)> {
        Err(eyre!(
            "Streaming with cancellation not implemented for provider '{}'",
            self.id
        ))
    }
}

/// OpenAI API client implementation.
///
/// Supports both standard completion and SSE streaming for real-time responses.
/// Compatible with OpenAI and OpenAI-compatible endpoints.
#[derive(Debug)]
pub struct OpenaiClient {
    id: String,
    api_key: String,
    api_url: String,
    client: HttpClient,
}

impl OpenaiClient {
    pub fn new(config: &OpenaiConfig) -> Self {
        Self::new_with_timeout(config, Duration::from_secs(30))
    }

    pub fn new_with_timeout(config: &OpenaiConfig, timeout: Duration) -> Self {
        let client = if config.proxy {
            HttpClient::builder()
                .timeout(timeout)
                .connect_timeout(Duration::from_secs(10))
                .pool_idle_timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| HttpClient::new())
        } else {
            HttpClient::builder()
                .timeout(timeout)
                .connect_timeout(Duration::from_secs(10))
                .pool_idle_timeout(Duration::from_secs(30))
                .no_proxy()
                .build()
                .unwrap_or_else(|_| HttpClient::new())
        };

        Self {
            id: config.id.clone(),
            api_key: get_env_or_value(&config.api_key),
            api_url: get_env_or_value(&config.api_url),
            client,
        }
    }
}

#[async_trait::async_trait]
impl ChatClient for OpenaiClient {
    async fn complete(
        &self,
        request: CreateChatCompletionRequest,
    ) -> Result<CreateChatCompletionResponse> {
        eprintln!(
            "[{}] Sending request to: {}/chat/completions",
            self.id, self.api_url
        );
        eprintln!("[{}] Request model: {:?}", self.id, request.model);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        eprintln!("[{}] Response status: {}", self.id, status);

        if !status.is_success() {
            let error_text = response.text().await?;
            eprintln!("[{}] Error: {}", self.id, error_text);
            return Err(eyre!("API Error: {}", error_text));
        }

        let text_data = response.text().await?;
        eprintln!("[{}] Response length: {} chars", self.id, text_data.len());
        if text_data.len() < 500 {
            eprintln!("[{}] Response: {}", self.id, text_data);
        }

        let completion: CreateChatCompletionResponse =
            serde_json::from_str(&text_data).map_err(|e| {
                eprintln!("[{}] Parse error: {}", self.id, e);
                eprintln!("[{}] Raw text: {}", self.id, text_data);
                eyre::Report::from(e)
            })?;
        Ok(completion)
    }

    async fn complete_streaming(
        &self,
        mut request: CreateChatCompletionRequest,
        chunk_sender: mpsc::UnboundedSender<String>,
    ) -> Result<(String, Option<Vec<ChatCompletionMessageToolCall>>)> {

        // Force streaming mode
        request.stream = Some(true);

        // Convert request to JSON value to modify it
        let request_json = serde_json::to_value(&request)?;

        let url = format!("{}/chat/completions", self.api_url);

        // Use the streaming module
        use crate::streaming::stream_completion;

        let (accumulated, tool_calls) = stream_completion(
            &self.client,
            url,
            self.api_key.clone(),
            request_json,
            |chunk| {
                chunk_sender
                    .send(chunk)
                    .map_err(|e| eyre!("Failed to send chunk: {}", e))
            },
        )
        .await?;

        // Return both text and tool calls
        Ok((accumulated, tool_calls))
    }

    async fn complete_streaming_with_cancellation(
        &self,
        mut request: CreateChatCompletionRequest,
        chunk_sender: mpsc::UnboundedSender<String>,
        cancellation_token: CancellationToken,
        timeout_duration: Duration,
    ) -> Result<(String, Option<Vec<ChatCompletionMessageToolCall>>)> {

        // Force streaming mode
        request.stream = Some(true);

        // Convert request to JSON value to modify it
        let request_json = serde_json::to_value(&request)?;

        let url = format!("{}/chat/completions", self.api_url);

        // Use the streaming module with cancellation support
        use crate::streaming::stream_completion_with_cancellation;

        let (accumulated, tool_calls) = stream_completion_with_cancellation(
            &self.client,
            url,
            self.api_key.clone(),
            request_json,
            cancellation_token,
            timeout_duration,
            |chunk| {
                chunk_sender
                    .send(chunk)
                    .map_err(|e| eyre!("Failed to send chunk: {}", e))
            },
        )
        .await?;

        // Return both text and tool calls
        Ok((accumulated, tool_calls))
    }
}
