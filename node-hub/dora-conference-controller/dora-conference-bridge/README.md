# Dora Conference Bridge

A Rust-based coordination node for multi-participant scenarios in Dora dataflows. Bunning messages from multiple input streams with configurable forwarding modes (bundled or cold start).

## Overview

The conference bridge coordinates multiple text inputs (e.g., from multiple ASR nodes or LLM agents) into a unified stream for downstream processing. It's designed for scenarios like:

- Multi-person voice conferences
- Debate systems with multiple participants
- Multi-agent coordination
- Hybrid ASR + LLM pipelines

## Key Features

### 🎛️ Dual Operation Modes
- **Bundled Mode**: Wait for all participants to be ready before forwarding
- **Cold Start Mode**: Forward first participant immediately, then wait for all

### 📨 Dynamic Input Ports
- Accepts any number of input ports with arbitrary names
- Automatically registers ports as they are encountered
- Maintains FIFO order for consistent message bundling

### 🎯 session_status-Based Completion
- **Explicit completion signals**: Uses `session_status="ended"` or `session_status="cancelled"` metadata
- **No inference**: Does not guess completion from empty strings or timing
- **Reliable**: Ensures deterministic behavior across all participants

### 🏷️ Question ID Management
- **Automatic increment**: Optionally increments question_id for sequential Q&A
- **Passthrough mode**: Preserve question_id from inputs for topic-based grouping

## Quick Start

### Basic Multi-Participant Configuration

```yaml
# dataflow.yml
nodes:
  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      participant1: asr1/text
      participant2: asr2/text
      participant3: asr3/text
    env:
      LOG_LEVEL: info
      COLD_START: false
      INC_QUESTION_ID: true
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `COLD_START` | Enable cold start mode (forward on first input) | `false` |
| `LOG_LEVEL` | Log level (error, warn, info, debug) | `info` |
| `STREAMING_PORTS` | Comma-separated list of streaming ports | `""` |
| `INC_QUESTION_ID` | Auto-increment question_id | `false` |

## Usage Examples

### Example 1: Voice Conference (3 participants)

```yaml
nodes:
  - id: asr-alice
    operator: python:dora-asr
    env:
      SESSION_ID: alice

  - id: asr-bob
    operator: python:dora-asr
    env:
      SESSION_ID: bob

  - id: asr-charlie
    operator: python:dora-asr
    env:
      SESSION_ID: charlie

  - id: conference-bridge
    operator: rust:dora-conference-bridge
    inputs:
      alice: asr-alice/text
      bob: asr-bob/text
      charlie: asr-charlie/text

  - id: llm
    operator: rust:dora-maas-client
    inputs:
      text: conference-bridge/text
```

### Example 2: Debate System

```yaml
nodes:
  - id: debate-bridge
    operator: rust:dora-conference-bridge
    inputs:
      debater1: debater1/text
      debater2: debater2/text
      debater3: debater3/text
    env:
      COLD_START: false      # Wait for all debaters
      INC_QUESTION_ID: true  # Track debate rounds
```

### Example 3: Hybrid Pipeline (Cold Start)

```yaml
nodes:
  - id: conference-bridge
    operator: rust:dora-conference-bridge
    inputs:
      asr: asr/text      # Fast first response
      llm: llm/text      # Slower but thoughtful
    env:
      COLD_START: true   # Forward ASR immediately
```

## API Reference

### Inputs

All inputs except `control` are treated as participant messages:

- **Dynamic ports**: Any name (except "control") accepts participant messages
- **Control port**: `"control"` accepts operational commands

### Outputs

- **`text`**: Bundled messages (concatenated with newlines)
- **`status`**: Status updates (waiting, forwarded, cancelled, reset)

### Metadata Fields

**Input Metadata**:
- `session_status`: REQUIRED. Must be `"started"`, `"ongoing"`, `"ended"`, or `"cancelled"`
- `question_id`: Optional. Used for question-based grouping
- `session_id`: Optional. Passed through to output

**Output Metadata**:
- `question_id`: Question identifier (incremented or passed through)
- `session_id`: Session identifier from inputs
- `participant_count`: Number of participants in bundle

📖 **Complete API Specification**: See [API.md](API.md) for detailed specifications, cancellation handling, configuration options, and integration examples.

## Architecture

```
┌─────────────┐     ┌──────────────────────┐     ┌─────────────┐
│ Participant 1 │───▶│                      │     │             │
├─────────────┤     │                      │     │             │
│ Participant 2 │───▶│  Conference Bridge   │────▶│  Downstream │
├─────────────┤     │                      │     │    Node     │
│ Participant 3 │───▶│                      │     │   (LLM/TTS) │
└─────────────┘     └──────────────────────┘     └─────────────┘
                            │
                            ▼
                      ┌─────────────┐
                      │   Status    │
                      │   Output    │
                      └─────────────┘
```

## Critical Rule

**ALWAYS send `session_status="ended"` or `session_status="cancelled"` to signal message completion. ** Without this metadata, the bridge will wait indefinitely and never forward messages.

```python
# ✅ CORRECT
node.send_output("text", message, {"session_status": "ended"})

# ❌ INCORRECT - will cause hang
node.send_output("text", message, {})
```

## Building

```bash
cd node-hub/dora-conference-bridge
cargo build --release
```

## Testing

```bash
# Run unit tests
cargo test

# Manual test with test dataflow
cd examples
dora start conference-bridge-test.yml
```

## License

Apache 2.0 - See LICENSE file for details
