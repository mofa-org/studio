# Dora MaaS Client

A high-performance Model-as-a-Service (MaaS) client node for Dora that provides seamless integration with cloud AI providers. Designed as a drop-in replacement for local LLM nodes like dora-qwen3, with streaming support and intelligent text segmentation for real-time applications.

## Architecture Overview

```
┌─────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│   Dora Events   │────▶│   MaaS Client    │────▶│  Cloud Provider  │
│  (text input)   │     │                  │     │  (OpenAI, etc)   │
└─────────────────┘     │  ┌────────────┐  │     └──────────────────┘
                        │  │ Segmenter  │  │              │
                        │  └────────────┘  │              │
                        │  ┌────────────┐  │              ▼
┌─────────────────┐     │  │  Session   │  │     ┌──────────────────┐
│  Text-to-Speech │◀────│  │  Manager   │  │◀────│   SSE Stream     │
│   (segments)    │     │  └────────────┘  │     └──────────────────┘
└─────────────────┘     └──────────────────┘
```

## Key Features

### 🚀 High-Performance Streaming
- **Low Latency**: ~0.5s to first token with SSE (Server-Sent Events) streaming
- **Real-time Processing**: Chunks are processed as they arrive from the API
- **Intelligent Segmentation**: Buffers streaming chunks into meaningful segments for TTS

### 🔄 Smart Text Segmentation
- **Punctuation-based**: Automatically segments at sentence boundaries (。！？.!?)
- **Multi-language**: Supports both Chinese and English punctuation
- **Word Count Fallback**: Segments after 10 words if no punctuation found
- **TTS Optimized**: Prevents overwhelming TTS with single-word chunks

### 🌐 Multi-Provider Support
- **OpenAI**: GPT-4, GPT-4o, GPT-3.5-turbo
- **Google Gemini**: Gemini Pro, Gemini Flash
- **Extensible**: Easy to add new providers with OpenAI-compatible APIs

### 💬 Session Management
- **Conversation History**: Maintains context across multiple exchanges
- **Configurable Memory**: Set maximum history exchanges to manage token usage
- **Multi-session**: Supports concurrent sessions with different contexts

### 🔌 Drop-in Replacement
- **Dora API Compatible**: Same inputs/outputs as dora-qwen3
- **Event-driven**: Pure async architecture without threading
- **Zero Migration**: Works with existing Dora dataflows

## Installation

### From Source
```bash
cd dora/node-hub/dora-maas-client
cargo build --release
```

### As Dora Node
```bash
cargo install dora-maas-client
```

## Configuration

Create a `maas_config.toml` file:

```toml
# Model selection
default_model = "gpt-4o"  # Options: gpt-4, gpt-4o, gpt-4o-mini, gpt-3.5

# System prompt for the assistant
system_prompt = """You are a helpful AI assistant. 
Provide clear, concise responses suitable for voice interaction."""

# Conversation history management
max_history_exchanges = 30  # Keep last 30 Q&A pairs
enable_streaming = true     # Enable SSE streaming for low latency
log_level = "INFO"         # DEBUG, INFO, WARN, ERROR

# OpenAI Provider
[[providers]]
id = "openai"
kind = "openai"
api_key = "sk-..."  # Or use "env:OPENAI_API_KEY"
api_url = "https://api.openai.com/v1"
proxy = false

# Google Gemini Provider
[[providers]]
id = "gemini"
kind = "gemini"
api_key = "AIza..."  # Or use "env:GEMINI_API_KEY"
api_url = "https://generativelanguage.googleapis.com/v1beta"
proxy = false

# Model routing
[[models]]
id = "gpt-4"
route = { provider = "openai", model = "gpt-4-turbo-preview" }

[[models]]
id = "gpt-4o"
route = { provider = "openai", model = "gpt-4o" }

[[models]]
id = "gpt-4o-mini"
route = { provider = "openai", model = "gpt-4o-mini" }

[[models]]
id = "gemini-pro"
route = { provider = "gemini", model = "gemini-1.5-pro-latest" }
```

## Usage in Dataflow

### Basic Integration
```yaml
nodes:
  - id: maas-client
    build: cargo build -p dora-maas-client --release
    path: ../../target/release/dora-maas-client
    inputs:
      text: asr/transcription
      control: controller/control
    outputs:
      - text
      - status
      - log
    env:
      CONFIG: maas_config.toml
```

### Voice Chat Application
```yaml
nodes:
  # Speech recognition
  - id: asr
    path: dora-asr
    inputs:
      audio: microphone/audio
    outputs:
      - transcription

  # MaaS client with streaming
  - id: maas-client
    path: dora-maas-client
    inputs:
      text: asr/transcription
    outputs:
      - text  # Segmented output for TTS
      - log

  # Text-to-speech
  - id: tts
    path: dora-primespeech
    inputs:
      text: maas-client/text
    outputs:
      - audio

  # Audio playback
  - id: speaker
    path: dora-speaker
    inputs:
      audio: tts/audio
```

## API Reference

### Inputs

| Input | Type | Description |
|-------|------|-------------|
| `text` | StringArray | Primary text input for chat completion |
| `text_to_audio` | StringArray | Alternative input (compatibility) |
| `control` | StringArray | Control commands |

### Outputs

| Output | Type | Description |
|-------|------|-------------|
| `text` | StringArray | Generated response (segmented if streaming) |
| `status` | StringArray | Node status ("ready", etc) |
| `log` | JSON String | Structured logs with timestamp |

### Control Commands

| Command | Description |
|---------|-------------|
| `reset` | Clear conversation history for session |
| `ready` | Request ready status |
| `exit` | Remove session and cleanup |

📖 **Complete API Specification**: See [API.md](API.md) for detailed input/output specifications, metadata fields, cancellation handling, configuration options, and integration examples.

## Streaming & Segmentation

When `enable_streaming = true`, the client:

1. **Receives SSE chunks** from the API as they're generated
2. **Buffers chunks** in the StreamSegmenter
3. **Emits segments** when encountering:
   - Sentence endings (。！？.!?)
   - Clause boundaries (；：;:)
   - Soft breaks after sufficient content (，,)
   - 10 words without punctuation

This ensures TTS receives meaningful phrases instead of individual words:

```
Without segmentation (old):
"你" -> "好" -> "，" -> "今" -> "天" -> "天" -> "气" -> ...

With segmentation (new):
"你好，" -> "今天天气很好。" -> "要出去走走吗？"
```

## Performance Metrics

| Metric | Non-streaming | Streaming |
|--------|--------------|-----------|
| First Token | 2-3s | ~0.5s |
| Complete Response | 2-3s | 2-3s |
| TTS Start | After full response | After first segment |
| User Experience | Wait for full answer | Hear response immediately |

## Development

### Project Structure
```
dora-maas-client/
├── src/
│   ├── main.rs        # Event loop and Dora integration
│   ├── client.rs      # Provider client implementations
│   ├── config.rs      # Configuration management
│   ├── streaming.rs   # SSE stream parsing
│   └── segmenter.rs   # Text segmentation logic
├── Cargo.toml
└── README.md
```

### Testing
```bash
# Run unit tests
cargo test

# Integration test with streaming
cd examples/test-maas-client
dora up
dora start streaming_dataflow.yml
```

### Adding New Providers

1. Implement the `ChatClient` trait:
```rust
#[async_trait::async_trait]
impl ChatClient for YourClient {
    async fn complete(&self, request: CreateChatCompletionRequest) 
        -> Result<CreateChatCompletionResponse> { /* ... */ }
    
    async fn complete_streaming(&self, request: CreateChatCompletionRequest,
        chunk_sender: mpsc::UnboundedSender<String>) 
        -> Result<String> { /* ... */ }
}
```

2. Add to `ProviderConfig` enum in `config.rs`
3. Update `create_clients()` to instantiate your client

## Troubleshooting

### Empty Responses
- Check API key is valid
- Verify model name in routing configuration
- Enable `LOG_LEVEL: DEBUG` for detailed diagnostics

### High Latency
- Ensure `enable_streaming = true` for real-time applications
- Check network connectivity to API endpoints
- Verify proxy settings if behind corporate firewall

### Segmentation Issues
- Adjust `max_words_without_punctuation` in `StreamSegmenter::new()`
- Check language-specific punctuation in `segmenter.rs`

## License

Apache 2.0 - See LICENSE file for details

## Contributing

Contributions welcome! Please ensure:
- Code follows Rust idioms and passes `cargo clippy`
- Tests pass with `cargo test`
- Documentation is updated for new features

## Example Applications

- `/examples/test-maas-client` - Basic streaming test
- `/examples/mac-aec-chat` - Voice chat with echo cancellation
- `/examples/voice-chatbot` - Full voice assistant implementation