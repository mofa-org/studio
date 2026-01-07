# Dora MaaS Client API Specification

## Overview

The Dora MaaS (Model-as-a-Service) Client is a cloud AI integration node for Dora dataflows. It provides a drop-in replacement for local LLM nodes with support for multiple cloud providers, real-time streaming, intelligent text segmentation, session management, and request cancellation.

**Location**: `node-hub/dora-maas-client/src/main.rs`

**Language**: Rust (async/await with Tokio runtime)

## Input API

### Input Ports

The MaaS client accepts inputs on the following ports:

#### 1. `text` (Primary Input)

**Description**: Main input port for user messages and text to be processed by the cloud AI provider.

**Data Type**: `StringArray`

**Metadata Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `session_id` | string | No | Session identifier for conversation management. Defaults to `"default"` if not provided. |
| `role` | string | No | Role of the message sender. Use `"assistant"` to cache assistant responses without triggering API calls. |
| `tools` | string (JSON) | No | MCP tool definitions in JSON format. Used for tool pass-through when `enable_local_mcp=false`. |

**Example**:

```yaml
node:
  id: maas-client
  operator:
    rust: dora-maas-client
  inputs:
    text: asr/text
  env:
    MAAS_CONFIG_PATH: maas_config.toml
```

**Input Flow**:
1. Text received on `text` port
2. Session lookup/creation based on `session_id`
3. Message added to conversation history
4. API call to configured cloud provider
5. Response streamed or returned as complete

#### 2. `text_to_audio` (Alternate Input)

**Description**: Alias for `text` port - alternative input name for compatibility with different dataflow patterns.

**Data Type**: `StringArray`

**Metadata Fields**: Same as `text` port

**Example**:

```yaml
node:
  id: maas-client
  operator:
    rust: dora-maas-client
  inputs:
    text_to_audio: asr/text
```

#### 3. `tool_results` (MCP Tool Results)

**Description**: Receives tool execution results from client when tools are executed externally (enable_local_mcp=false).

**Data Type**: `StringArray` (JSON format)

**Metadata Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `session_id` | string | No | Session identifier for conversation management. |

**Data Format**:
- JSON array of tuples: `[(tool_call_id: String, result: String), ...]`

**Example**:

```python
tool_results = [
    ("call_123", "Tool executed successfully"),
    ("call_456", "Error: File not found")
]
node.send_output("tool_results", json.dumps(tool_results))
```

**Flow**:
1. Tool calls received from LLM
2. Tool calls forwarded to client via `tool_calls` output
3. Client executes tools and returns results on `tool_results` input
4. Results added to conversation
5. Final LLM response generated with tool results

#### 4. `control` (Control Commands)

**Description**: Control port for session management and operational commands.

**Data Type**: `StringArray`

**Supported Commands** (as plain text or JSON):

| Command | Format | Description |
|---------|--------|-------------|
| `reset` | Plain text: `"reset"` or JSON: `{"command": "reset"}` | Reset conversation history for the session (keeps system prompt) |
| `ready` | Plain text: `"ready"` or JSON: `{"command": "ready"}` | Send ready status to downstream nodes |
| `exit` | Plain text: `"exit"` or JSON: `{"command": "exit"}` | Remove/close session |
| `prompt` | JSON: `{"prompt": "user text"}` | Send text through LLM pipeline (equivalent to `text` port) |

**Example**:

```yaml
node:
  id: maas-client
  operator:
    rust: dora-maas-client
  inputs:
    text: asr/text
    control: controller/command
```

**Control Examples**:

```python
# Reset session (JSON)
node.send_output("control", '{"command": "reset"}')

# Send prompt via control (JSON)
node.send_output("control", '{"prompt": "Hello, how are you?"}')

# Plain text command
node.send_output("control", "ready")
```

## Output API

### Output Ports

The MaaS client sends outputs on the following ports:

#### 1. `text` (Primary Output)

**Description**: Main output port for generated text responses and streaming chunks.

**Data Type**: `StringArray`

**Metadata Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `session_id` | string | Session identifier (passed through from input) |
| `session_status` | string | Session state: `"started"`, `"ongoing"`, `"ended"`, or `"cancelled"` |
| `segment_index` | string | Segment index counter (for streaming mode) |

**Session Status Values**:

- `"started"`: First text chunk in streaming mode
- `"ongoing"`: Intermediate text chunk in streaming mode
- `"ended"`: Final marker (empty string) indicating session completion
- `"cancelled"`: Session was cancelled (see Cancellation section)

**Output Flow** (Streaming Mode):

```
Streaming Response:
1. chunk_1 → session_status="started", segment_index="0", data="First segment..."
2. chunk_2 → session_status="ongoing", segment_index="1", data="Second segment..."
3. (final) → session_status="ended", segment_index="N", data=""
```

**Output Flow** (Non-Streaming):

```
Single Response:
1. response → session_status="started", data="Complete response"
2. (final) → session_status="ended", data=""
```

**Cancellation Detection**:

When a request is cancelled, the output will have:
```python
{
    "type": "text",
    "data": "Error: Stream cancelled by user",
    "metadata": {
        "session_status": "cancelled",
        "session_id": "session_123"
    }
}
```

**Example Usage** (Downstream Node):

```python
# Conference bridge or TTS node
event = node.next()
if event["type"] == "text":
    session_status = event["metadata"].get("session_status", "unknown")
    if session_status == "cancelled":
        # Handle cancellation
        print("🚨 Request was cancelled")
    elif session_status == "ended":
        # Session complete
        print("Session ended")
```

#### 2. `status` (Status Updates)

**Description**: Provides status updates about request processing state.

**Data Type**: `StringArray`

**Status Values**:

| Status | Description |
|--------|-------------|
| `"processing"` | Request is being sent to cloud provider |
| `"complete"` | Request completed successfully |
| `"cancelled"` | Request was cancelled |
| `"timeout"` | Request timed out |
| `"error"` | General error occurred |
| `"error: <details>"` | Detailed error message |
| `"ready"` | Node is ready for new requests |
| `"reset"` | Session was reset |

**Example Flow** (Successful Request):

```python
# Downstream node receives:
[10:23:15] 📨 status → Status: processing
[10:23:17] 📨 text → Status: started, Data: "Hello"
[10:23:17] 📨 text → Status: ongoing, Data: " there"
[10:23:18] 📨 text → Status: ongoing, Data: "!"
[10:23:18] 📨 text → Status: ended, Data: ""
[10:23:18] 📨 status → Status: complete
```

**Example** (Cancellation):

```python
# Downstream node receives when cancelled:
[10:23:15] 📨 status → Status: processing
[10:23:16] 📨 text → Status: cancelled, Data: "Error: Stream cancelled by user"
[10:23:16] 📨 status → Status: cancelled
```

#### 3. `tool_calls` (MCP Tool Calls)

**Description**: Forwards tool calls from LLM to client when local MCP execution is disabled.

**Data Type**: `StringArray` (JSON format)

**Metadata Fields**: None (passthrough from triggering input)

**Data Format**:
```json
[
  {
    "id": "call_123",
    "type": "function",
    "function": {
      "name": "get_current_weather",
      "arguments": "{\"location\": \"Boston\"}"
    }
  }
]
```

**Flow**:
1. LLM generates tool calls
2. Tool calls sent on `tool_calls` output
3. Client executes tools
4. Results returned on `tool_results` input
5. Final response generated

#### 4. `log` (Internal Logging)

**Description**: Internal logging output for debugging and monitoring.

**Data Type**: `StringArray` (JSON format)

**Log Format**:
```json
{
  "node": "maas-client",
  "level": "INFO|DEBUG|WARNING|ERROR",
  "message": "Log message",
  "timestamp": 1234567890
}
```

**Usage**: Connect to monitoring or logging infrastructure for observability.

## Configuration

### Configuration File (`maas_config.toml`)

**Location**: Configured via `MAAS_CONFIG_PATH` environment variable (defaults to `maas_config.toml`)

**Format**: TOML, YAML, or JSON

**Example Configuration**:

```toml
# Default model to use
default_model = "gpt-4"

# System prompt for conversations
system_prompt = "You are a helpful assistant."

# Maximum conversation history to maintain
max_history_exchanges = 10

# Enable streaming mode (default: true)
enable_streaming = true

# Log level for the node
log_level = "INFO"

# MCP tool support
tools_enabled = false
tools_local_mcp = false

# HTTP request cancellation settings
request_timeout_secs = 30      # Request timeout in seconds
stream_timeout_secs = 120      # Streaming timeout in seconds
enable_cancellation = true     # Enable request cancellation

# Provider configuration
[[providers]]
kind = "openai"
id = "openai"
api_key = "env:OPENAI_API_KEY"  # Can use env: prefix for environment variables
api_url = "https://api.openai.com/v1"
proxy = false

# Model routing configuration
[[models]]
id = "gpt-4"
  [models.route]
  provider = "openai"
  model = "gpt-4-turbo-preview"
```

### Environment Variables

#### Required

| Variable | Description |
|----------|-------------|
| `MAAS_CONFIG_PATH` | Path to configuration file (default: `maas_config.toml`) |
| `OPENAI_API_KEY` | API key for OpenAI (or use `env:OPENAI_API_KEY` in config) |
| `GEMINI_API_KEY` | API key for Gemini (if using Gemini provider) |

#### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `MAAS_DEFAULT_MODEL` | Default model ID | From config file |
| `MAAS_LOG_LEVEL` | Log level (INFO, DEBUG, WARNING, ERROR) | "INFO" |
| `MAAS_REQUEST_TIMEOUT_SECS` | Request timeout in seconds | 30 |
| `MAAS_STREAM_TIMEOUT_SECS` | Streaming timeout in seconds | 120 |
| `MAAS_ENABLE_CANCELLATION` | Enable request cancellation | true |
| `MAAS_ENABLE_STREAMING` | Enable streaming mode | true |
| `MAAS_ENABLE_TOOLS` | Enable MCP tool support | false |
| `MAAS_ENABLE_LOCAL_MCP` | Enable local MCP tool execution | false |

### Provider Configuration

#### OpenAI Provider

```toml
[[providers]]
kind = "openai"
id = "openai"
api_key = "env:OPENAI_API_KEY"
api_url = "https://api.openai.com/v1"
proxy = false  # Set to true for proxy support
```

#### Gemini Provider

```toml
[[providers]]
kind = "gemini"
id = "gemini"
api_key = "env:GEMINI_API_KEY"
api_url = "https://generativelanguage.googleapis.com/v1"
proxy = false
```

#### Alicloud Provider (OpenAI-compatible)

```toml
[[providers]]
kind = "alicloud"
id = "alicloud"
api_key = "env:ALICLOUD_API_KEY"
api_url = "https://dashscope.aliyuncs.com/api/v1"
proxy = false
```

### Model Routing

Map model IDs to providers and actual model names:

```toml
[[models]]
id = "gpt-4"  # Your internal model ID
  [models.route]
  provider = "openai"
  model = "gpt-4-turbo-preview"  # Actual provider model name

[[models]]
id = "gemini-pro"
  [models.route]
  provider = "gemini"
  model = "gemini-pro"

[[models]]
id = "qwen-max"
  [models.route]
  provider = "alicloud"
  model = "qwen-max"
```

### MCP Configuration

```toml
[tools]
enabled = true
local_mcp = true  # Execute tools locally (true) or pass to client (false)

[tools.mcp]
[[tools.mcp.servers]]
name = "filesystem"
protocol = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/home/user"]
```

## HTTP Request Cancellation

### Overview

The MaaS client supports cancellation of ongoing HTTP requests. When a request is cancelled, downstream nodes are notified via the `session_status="cancelled"` metadata.

### Cancellation Flow

1. Request initiated with cancellation token
2. Control input can trigger cancellation (future enhancement)
3. If cancelled: `session_status="cancelled"` sent on text output
4. Downstream nodes detect cancellation and stop processing

### Detection Pattern

```python
# Downstream node (e.g., conference bridge)
event = node.next()
if event["type"] == "text":
    session_status = event["metadata"].get("session_status", "unknown")
    if session_status == "cancelled":
        print("🚨 Request cancelled - cleaning up state")
        # Clear partial data, reset for next request
```

### Cancellation Configuration

```toml
# In maas_config.toml
request_timeout_secs = 30   # Global request timeout
stream_timeout_secs = 120   # Streaming timeout
enable_cancellation = true  # Enable cancellation support
```

### Cancellation Output

When a request is cancelled:

```python
{
    "type": "text",
    "data": "Error: Stream cancelled by user",
    "metadata": {
        "session_status": "cancelled",
        "session_id": "session_123"
    }
}

# Followed by status update:
{
    "type": "status",
    "data": "cancelled",
    "metadata": {}
}
```

## Session Management

### Session Lifecycle

```
1. Session Created → First text input without session_id creates "default" session
2. Session Active → Messages added to history, API calls made
3. Session Ended → session_status="ended" signals completion
4. Session Reset → "reset" command clears history (keeps system prompt)
5. Session Removed → "exit" command removes session entirely
```

### Session Memory Management

- Sessions persist in memory for the duration of the node
- History limited by `max_history_exchanges` configuration
- System prompt always preserved
- Session state is NOT persisted across node restarts

### Multiple Sessions

Multiple concurrent sessions supported via unique `session_id` values:

```python
# Session 1
node.send_output("text", "Hello", {"session_id": "user_1"})

# Session 2
node.send_output("text", "Hi there", {"session_id": "user_2"})
```

Each session maintains independent conversation history.

## Streaming vs Non-Streaming

### Streaming Mode (`enable_streaming = true`)

**Advantages**:
- Real-time text generation visible to user
- Lower latency for long responses
- Intelligent segmentation for TTS

**Flow**:
1. Text chunks received from provider
2. Segmented into meaningful phrases (sentence/punctuation boundaries)
3. Sent with `session_status`: `"started"` → `"ongoing"` → `"ended"`
4. Empty string with `session_status="ended"` marks completion

**Use Case**: Interactive chat, real-time voice assistants

### Non-Streaming Mode (`enable_streaming = false`)

**Advantages**:
- Simpler downstream processing (single response)
- Lower overhead
- Compatible with non-streaming providers

**Flow**:
1. Complete response received from provider
2. Sent as single message with `session_status="started"`
3. Empty string with `session_status="ended"` marks completion

**Use Case**: Batch processing, simple request-response patterns

## Integration Examples

### Example 1: Basic Chatbot

```yaml
# dataflow.yml
nodes:
  - id: websocket-server
    operator:
      rust: dora-openai-websocket
    outputs:
      - text

  - id: maas-client
    operator:
      rust: dora-maas-client
    inputs:
      text: websocket-server/text
    outputs:
      - text
      - status
    env:
      MAAS_CONFIG_PATH: maas_config.toml

  - id: tts
    operator:
      python: dora-primespeech
    inputs:
      text: maas-client/text
```

### Example 2: Multi-Session Conference

```yaml
# dataflow.yml
nodes:
  - id: asr-user1
    operator:
      python: dora-asr
    env:
      SESSION_ID: user_1
    outputs:
      - text

  - id: asr-user2
    operator:
      python: dora-asr
    env:
      SESSION_ID: user_2
    outputs:
      - text

  - id: maas-client
    operator:
      rust: dora-maas-client
    inputs:
      text:
        - asr-user1/text
        - asr-user2/text
    env:
      MAAS_CONFIG_PATH: maas_config.toml

  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      text: maas-client/text
      control: maas-client/status
```

### Example 3: With MCP Tools

```yaml
# dataflow.yml
nodes:
  - id: maas-client
    operator:
      rust: dora-maas-client
    inputs:
      text: user-input/text
      tool_results: tool-executor/results
    env:
      MAAS_CONFIG_PATH: maas_config.toml
      MAAS_ENABLE_TOOLS: "true"
      MAAS_ENABLE_LOCAL_MCP: "false"  # Tools executed by client
```

### maas_config.toml for MCP:

```toml
default_model = "gpt-4"
system_prompt = "You are a helpful assistant with access to tools."
enable_tools = true
enable_local_mcp = false

[[providers]]
kind = "openai"
id = "openai"
api_key = "env:OPENAI_API_KEY"
api_url = "https://api.openai.com/v1"

[[models]]
id = "gpt-4"
  [models.route]
  provider = "openai"
  model = "gpt-4-turbo-preview"
```

## Error Handling

### Error Status Flow

```
API Error → status="error" → text=[error message] with session_status="ended"
Timeout → status="timeout" → text=[timeout message] with session_status="ended"
Cancellation → status="cancelled" → text=[cancel message] with session_status="cancelled"
```

### Error Detection (Downstream)

```python
event = node.next()
if event["type"] == "status":
    status = event["data"]
    if status == "error":
        print("API error occurred")
    elif status == "timeout":
        print("Request timed out")
    elif status == "cancelled":
        print("Request cancelled")

elif event["type"] == "text":
    session_status = event["metadata"].get("session_status")
    if session_status == "cancelled":
        print("Request was cancelled - clean up partial state")
        # Clear any accumulated data
```

### Timeout Configuration

```toml
# Timeouts are configurable
request_timeout_secs = 30   # HTTP request timeout
stream_timeout_secs = 120   # Streaming会话 timeout
```

Timeouts trigger `status="timeout"` and `session_status="ended"`.

## Performance Considerations

### Streaming Performance

- **Segmenter**: Buffers chunks until punctuation/word boundaries
- **Chunk Size**: Varies by provider (typically 1-10 tokens per chunk)
- **Latency**: First chunk typically within 100-500ms

### Session Memory

- Each session maintains message history in memory
- History limited by `max_history_exchanges`
- Token counting performed but not enforced (future enhancement)

### Connection Pooling

- Reqwest client with connection pooling enabled
- Reuses connections across requests
- Configurable via provider settings

## Best Practices

### 1. Session Management

```python
# Use meaningful session IDs for multi-user scenarios
session_id = f"user_{user_id}_session_{timestamp}"
metadata = {"session_id": session_id}
node.send_output("text", user_message, metadata)
```

### 2. Cancellation Support

```python
# Always check for cancellation
event = node.next()
if event["type"] == "text":
    session_status = event["metadata"].get("session_status")
    if session_status == "cancelled":
        # Clean up any partial state
        clear_partial_data()
        continue
```

### 3. Error Handling

```python
# Check status events for errors
event = node.next()
if event["type"] == "status":
    status = event["data"]
    if status.startswith("error"):
        log.error(f"MaaS client error: {status}")
        # Implement retry logic or fallback
```

### 4. Streaming vs Non-Streaming

```toml
# Use streaming for interactive applications
enable_streaming = true

# Use non-streaming for batch processing
enable_streaming = false
```

### 5. Tool Configuration

```toml
# For local MCP execution:
enable_tools = true
enable_local_mcp = true

# For client-side execution:
enable_tools = true
enable_local_mcp = false
```

## Troubleshooting

### Issue: "No client found for provider"

**Cause**: Provider ID in model route doesn't match provider configuration

**Solution**: Check model routing configuration matches provider IDs

### Issue: "No route found for model"

**Cause**: Model ID not configured in `[[models]]` section

**Solution**: Add model routing entry in configuration file

### Issue: Streaming doesn't work

**Cause**: Provider doesn't support streaming or streaming disabled

**Solution**:
- Check `enable_streaming = true` in config
- Verify provider supports streaming (OpenAI, Gemini do)
- Check provider API key and endpoint

### Issue: Session history not maintained

**Cause**: Different session_id per message or session reset

**Solution**: Use consistent session_id across related messages

### Issue: Cancellation not detected

**Cause**: Downstream node not checking session_status

**Solution**: Implement cancellation detection:

```python
if event["metadata"].get("session_status") == "cancelled":
    handle_cancellation()
```

### Issue: Tool calls not working

**Cause**: Tools not configured or local MCP disabled

**Solution**:
- Check `enable_tools = true`
- Verify MCP server configuration
- Check tool registration in logs

## Version History

- **v1.0**: Initial implementation with OpenAI/Gemini support
- **v1.1**: Added streaming mode and text segmentation
- **v1.2**: Added MCP tool support (local and pass-through)
- **v1.3**: Added HTTP request cancellation with session_status="cancelled"
- **v1.4**: Enhanced error handling with timeout and cancellation detection

## See Also

- [Dora Conference Bridge API](dora-conference-bridge/API.md) - For multi-participant scenarios
- [Dora ASR Integration](dora-asr/README.md) - Speech recognition integration
- [Dora TTS Integration](dora-primespeech/README.md) - Text-to-speech integration
