# Dora Conference Bridge API Specification

## 🎯 Overview

The **Dora Conference Bridge** is a Rust-based node that coordinates multiple input streams in multi-participant scenarios. It collects messages from different participants, queues them per input port, and forwards bundled messages on explicit `resume` command.

**Location**: `node-hub/dora-conference-bridge/src/main.rs`

**Language**: Rust (async/await with Dora runtime)

**Primary Use Case**: Coordinating multiple inputs in conference scenarios, debates, multi-agent systems, or any workflow requiring explicit control over forwarding.

## 🎛️ Control Flow Model

### Pause/Resume Control Flow

**Default State**: **PAUSED**

The bridge operates with explicit state management:

1. **PAUSED** (default): Collects inputs into queues but does NOT forward
2. **RESUMED**: Forwards one bundled message, then auto-pauses

```
PAUSED → [receive "resume" command] → RESUMED → [forward one cycle] → PAUSED
```

**Key Features**:
- ✅ Explicit control over when forwarding occurs
- ✅ Per-port message queuing (no message loss)
- ✅ Auto-pause after one output (guaranteed)
- ✅ Controller-driven workflow

**Replaces**: The old bundled/cold-start modes

## Input API

### Input Ports

The conference bridge accepts inputs from **dynamic ports** - any port name can be used. All inputs are treated as participant messages to be bundled.

#### 1. Dynamic Participant Ports (Any Name Except "control")

**Description**: All non-control inputs are treated as participant messages. The bridge dynamically registers input ports as they are encountered.

**Data Type**: `StringArray`

**Metadata Fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `session_status` | string | **YES** | Completion signal: `"started"`, `"ongoing"`, `"ended"`, or `"cancelled"` |
| `question_id` | string | No | Question identifier for grouping related messages |
| `session_id` | string | No | Session identifier (passed through to output) |

**⚠️ CRITICAL**: The bridge **exclusively** uses `session_status` metadata to determine message completion. It does NOT infer completion from empty strings or timing.

### 2. Control Port

**Port Name**: **`control`**

**Description**: Control port for operational commands.

**Supported Commands**:

| Command | Format | Description |
|---------|--------|-------------|
| `resume` | `{"command": "resume"}` or `"resume"` | **Resume forwarding** - Send one bundled output, then auto-pause |
| `reset` | `{"command": "reset"}` or `"reset"` | ** Emergency reset ** - Clear all queues and state |
| `ready` | `{"command": "ready"}` or `"ready"` | Request status update |

** 📖 See [CONTROL_COMMANDS_SUMMARY.md](CONTROL_COMMANDS_SUMMARY.md) for detailed information **

**Example Dataflow Configuration**:

```yaml
nodes:
  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      participant1: asr1/text      # Dynamic port: participant1
      participant2: asr2/text      # Dynamic port: participant2
      participant3: asr3/text      # Dynamic port: participant3
      control: controller/command  # Control port
```

**Input Flow**:

```
1. Input received on dynamic port (e.g., "participant1")
2. Port registered if new
3. Message added to participant queue
4. session_status checked for completion
5. When complete, added to arrival queue
6. Forwarding conditions evaluated
7. Bundle forwarded when conditions met
```

#### 1. Dynamic Participant Ports

**Port Names**: Any name except "control"

**Data Type**: `StringArray`

**Example Input Events**:

```python
# Stream 1: Participant 1 speaking
{
    "type": "Input",
    "id": "participant1",
    "data": ["Hello, how are you today?"],
    "metadata": {
        "session_status": "started"
    }
}

{
    "type": "Input",
    "id": "participant1",
    "data": ["I wanted to ask about the weather."],
    "metadata": {
        "session_status": "ongoing"
    }
}

{
    "type": "Input",
    "id": "participant1",
    "data": [""],  # Empty content
    "metadata": {
        "session_status": "ended"  # Signals completion
    }
}

# Stream 2: Participant 2 speaking
{
    "type": "Input",
    "id": "participant2",
    "data": ["I'm doing great!"],
    "metadata": {
        "session_status": "started"
    }
}

{
    "type": "Input",
    "id": "participant2",
    "data": [""],
    "metadata": {
        "session_status": "ended"
    }
}
```

**Message Completion**:

The bridge considers a message complete when either:
- `session_status="ended"` (normal completion)
- `session_status="cancelled"` (cancellation)

This is CRITICAL for proper operation - the bridge does NOT infer completion from empty strings or timing.

#### 2. `control` (Control Input)

**Description**: Control port for operational commands and state management.

**Data Type**: `StringArray`

**Supported Commands**:

| Command | Format | Description |
|---------|--------|-------------|
| `reset` | Plain text: `"reset"` or JSON: `{"command": "reset"}` | Reset bridge state, clearing all inputs and queues |

**Example**:

```yaml
nodes:
  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      participant1: asr1/text
      participant2: asr2/text
      control: controller/command
```

**Control Command Examples**:

```python
# Reset conference bridge
node.send_output("control", "reset")

# Or using JSON
node.send_output("control", '{"command": "reset"}')
```

When `reset` is received:
- All input states cleared
- Arrival queue emptied
- Current question_id reset
- Status "reset" sent to output
- Conference state restored to initial configuration

## State Determination Rule (IMPORTANT)

The conference bridge **exclusively** uses `session_status` metadata to determine message completion. It does NOT:
- Use empty strings as completion signals
- Use timers or timeouts for completion
- Infer completion from lack of activity

**Correct Usage** (Upstream Nodes):

```python
# Send final message with session_status="ended"
node.send_output(
    "text",
    final_content,
    {
        "session_status": "ended",  # REQUIRED: Signals completion
        "question_id": str(question_id),
        "session_id": session_id
    }
)

# If cancelling:
node.send_output(
    "text",
    "",
    {
        "session_status": "cancelled",  # Signals cancellation
        "question_id": str(question_id),
        "session_id": session_id
    }
)
```

**Bridge Processing**:

```rust
// In metadata_indicates_completion() - lines 236-241
fn metadata_indicates_completion(metadata: &BTreeMap<String, Parameter>) -> bool {
    match metadata.get("session_status") {
        Some(Parameter::String(status)) if status == "ended" || status == "cancelled" => true,
        _ => false,  // NOT complete without explicit session_status
    }
}
```

**Consequences of Missing session_status**:

If a node sends input WITHOUT `session_status="ended"`, the conference bridge will:
- Wait indefinitely for the completion signal
- Never forward the bundle (in bundled mode)
- Cause the system to hang

## Message Queuing Per Input Port

### Overview

Each input port maintains a FIFO queue of pending messages. This allows the same port (e.g., `llm1`) to send multiple messages before previous ones are forwarded.

### Queue Behavior

**Multiple Messages from Same Port**:

```python
# llm1 sends multiple messages rapidly
node.send_output("text", "Message 1", {"session_status": "ended"})
node.send_output("text", "Message 2", {"session_status": "ended"})
node.send_output("text", "Message 3", {"session_status": "ended"})

# llm2 sends one message
node.send_output("text", "Response A", {"session_status": "ended"})
```

**Forwarding Order**:
- **Cycle 1**: Message 1 + Response A (forwarded)
- **Cycle 2**: Message 2 (waiting for llm2)
- **Cycle 3**: Message 3 (waiting for llm2)
- **...and so on**

Each port's messages are processed in strict FIFO order.

### Reset Command Clears All Queues

The `reset` control command clears ALL queued messages for ALL ports:

```python
# Send reset
node.send_output("control", "reset")

# Result:
# - All message queues cleared
# - All pending messages discarded
# - State reset to initial
```

**Use Case**: Emergency stop, conversation reset, or when upstream timestamps are detected as stale.

## Output API

### Output Ports

#### 1. `text` (Bundled Output)

**Description**: Concatenated messages from all ready participants in FIFO order.

**Data Type**: `StringArray`

**Metadata Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `question_id` | string | Question identifier (incremented if `INC_QUESTION_ID=true`) |
| `session_id` | string | Session identifier (passed through from inputs) |
| `participant_count` | string | Number of participants included in bundle |

**Output Format**:

Messages concatenated with newline (`\n`) separators between participants.

**Queuing Behavior**:

Each input port maintains a FIFO queue of pending messages:
- Messages are queued as they arrive on each port
- The bridge forwards the FIRST (oldest) message from each port per cycle
- After forwarding, that message is popped from the queue
- If more messages remain, they will be forwarded in subsequent cycles
- Queue depth is logged for debugging

**Example**:

```
Input llm1 (rapid-fire):
  T+0:00 Message 1 (session_status="ended")
  T+0:01 Message 2 (session_status="ended")
  T+0:02 Message 3 (session_status="ended")

Input llm2 (later):
  T+0:03 Message A (session_status="ended")

Cycle 1: Forward Message 1 + Message A
Cycle 2: Forward Message 2 (waiting for llm2)
Cycle 3: Forward Message 3 (waiting for llm2)
```

**Example Output**:

```yaml
# Inputs from 3 participants:
participant1: "What is AI?"
participant2: "I think AI is fascinating."
participant3: "Can you explain machine learning?"

# Output:
text: "What is AI?\nI think AI is fascinating.\nCan you explain machine learning?"
metadata:
  question_id: "1"
  participant_count: "3"
```

**Forwarding Modes**:

The bridge operates in one of two modes:

1. **Cold Start Mode** (`COLD_START=true`)
   - Forwards when ANY input is ready (first request only)
   - Subsequent requests require ALL inputs
   - Useful for reducing initial latency

2. **Bundled Mode** (`COLD_START=false`, default)
   - Forwards when ALL inputs are ready
   - Ensures synchronized multi-participant responses
   - Required for debate/conference scenarios

**Example Flow** (Bundled Mode):

```
Time  T0:   Bridge initialized (waiting)
      T1:   Input from participant1 (waiting for others)
      T2:   Input from participant2 (waiting for others)
      T3:   Input from participant3 (ALL READY → FORWARD)
      T3.1: Output bundled message
      T3.2: Reset state
      T4:   Ready for next cycle
```

**Example Flow** (Cold Start Mode):

```
Time  T0:   Bridge initialized (cold_start_used=false)
      T1:   Input from participant1 (ANY READY → FORWARD)
      T1.1: Output participant1 only
      T1.2: cold_start_used=true (switch to bundled)
      T2:   Input from participant1 (waiting for others)
      T3:   Input from participant2 (waiting)
      T4:   Input from participant3 (ALL READY → FORWARD)
      T4.1: Output all participants
```

#### 2. `status` (Status Updates)

**Description**: Provides status updates about bridge state and forwarding events.

**Data Type**: `StringArray`

**Status Values**:

| Status | Description |
|--------|-------------|
| `waiting (X/Y)` | Waiting for inputs (X ready out of Y total) |
| `forwarded` | Bundle forwarded successfully |
| `cancelled` | Cycle was cancelled by upstream |
| `reset` | State reset via control command |

**Example Status Flow**:

```python
# Successful bundled forwarding:
[10:23:45] 📨 status → Status: waiting (0/3)
[10:23:46] 📨 status → Status: waiting (1/3)
[10:23:47] 📨 status → Status: waiting (2/3)
[10:23:48] 📨 status → Status: waiting (3/3)
[10:23:48] 📨 status → Status: forwarded

# After forwarding, state reset:
[10:23:49] 📨 status → Status: waiting (0/3)

# Cancellation:
[10:23:50] 📨 status → Status: waiting (1/3)
[10:23:51] 📨 status → Status: cancelled  # Upstream sent session_status="cancelled"
[10:23:51] 📨 status → Status: waiting (0/3)  # State reset
```

## Cancellation Handling

### Overview

The conference bridge detects and handles request cancellation from upstream nodes (e.g., MaaS client) via the `session_status="cancelled"` metadata.

### Cancellation Detection

```rust
// Lines 645-667 in main()
if let Some(Parameter::String(ref session_status)) = parameters.get("session_status") {
    if session_status == "cancelled" {
        send_log(
            &mut node,
            LogLevel::Info,
            log_level,
            &format!("🛑 Input {} cancelled by upstream - clearing fragments", port_name),
        );
        bridge.was_cancelled = true;

        // Clear any pending streaming fragments for this input
        if let Some(input) = bridge.inputs.get_mut(port_name) {
            input.reset(); // Clears message_state and sets ready=false
            send_log(
                &mut node,
                LogLevel::Debug,
                log_level,
                &format!("Cleared {} pending fragments", port_name),
            );
        }
    }
}
```

**Fragment Cleanup**: When cancellation is detected, the bridge immediately calls `input.reset()` to clear any pending `MessageState::Streaming` chunks or `MessageState::Complete` content, ensuring no partial data leaks into subsequent cycles.

### Cancellation State Tracking

```rust
// In ConferenceBridge struct (line 253)
struct ConferenceBridge {
    // ... other fields ...
    was_cancelled: bool,  // Track if current cycle was cancelled
}
```

When any input has `session_status="cancelled"`, the `was_cancelled` flag is set to `true`.

### Final Status Determination

```rust
// Lines 491-493 in forward_bundle()
let final_status = if self.was_cancelled { "cancelled" } else { "forwarded" };
self.finalize_cycle(node, final_status)
```

The bridge forwards the bundle (with whatever inputs are ready) but marks the status as `cancelled` to indicate the cycle was interrupted.

### Cancellation Logging

```rust
// Lines 378-386 in finalize_cycle()
if self.was_cancelled {
    send_log(
        node,
        LogLevel::Info,
        self.log_level,
        "🛑 Conference cycle cancelled - clearing state",
    );
}
```

### Cancellation Example Flow

```python
# Upstream MaaS client detects cancellation:
{
    "type": "text",
    "id": "text",
    "data": ["Error: Stream cancelled by user"],
    "metadata": {
        "session_status": "cancelled",  # Signals cancellation
        "session_id": "session123"
    }
}

# Conference bridge:
# 1. Detects session_status="cancelled"
# 2. Calls input.reset() to clear any pending fragments
# 3. Sets bridge.was_cancelled = true

# Conference bridge forwards cancellation signal:
{
    "type": "text",
    "id": "text",
    "data": ["Error: Stream cancelled by user"],  # Forwarded as-is
    "metadata": {
        "session_status": "cancelled",  # Passed through
        "participant_count": "1"
    }
}

# Status update:
{
    "type": "status",
    "id": "status",
    "data": ["cancelled"],
    "metadata": {}
}

# State reset (including cleared fragments):
{
    "type": "status",
    "id": "status",
    "data": ["waiting (0/3)"],
    "metadata": {}
}

# Next cycle: All inputs start fresh with no pending fragments
```

**Key Points**:
- Pending chunks in `MessageState::Streaming` are discarded
- Partial messages in `MessageState::Complete` are cleared
- `arrival_queue` is cleared in `finalize_cycle()`
- Next cycle starts with clean state for all inputs

## Configuration

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `STREAMING_PORTS` | Comma-separated list of streaming input ports | `""` (all non-streaming) | `STREAMING_PORTS="asr1,asr2"` |
| `LOG_LEVEL` | Log verbosity: error, warn, info, debug | `"info"` | `LOG_LEVEL=debug` |
| `COLD_START` | Enable cold start mode (forward on any input) | `false` | `COLD_START=true` |
| `INC_QUESTION_ID` | Increment question_id after each cycle | `false` | `INC_QUESTION_ID=true` |

### Example Configurations

#### Basic Multi-Participant

```yaml
# dataflow.yml
nodes:
  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      alice: asr-alice/text
      bob: asr-bob/text
      charlie: asr-charlie/text
    env:
      LOG_LEVEL: info
      COLD_START: false
      INC_QUESTION_ID: true
```

#### Cold Start Mode

```yaml
# dataflow.yml
nodes:
  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      asr: asr/text
      llm: maas-client/text
    env:
      LOG_LEVEL: debug
      COLD_START: true  # Forward on first input
      INC_QUESTION_ID: false
```

#### Debate Scenario

```yaml
# dataflow.yml
nodes:
  - id: debate-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      debater1: debater1/text
      debater2: debater2/text
      debater3: debater3/text
    env:
      LOG_LEVEL: info
      COLD_START: false      # Wait for all debaters
      INC_QUESTION_ID: true  # Track questions
      STREAMING_PORTS: "debater1,debater2,debater3"
```

### Dynamic Node Configuration

The conference bridge can be used as a dynamic node with `--name` argument:

```bash
# Static node (from dataflow config)
dora start dataflow.yml

# Dynamic node (created at runtime)
dora-node-api \
  --name conference-bridge-1 \
  --operator rust:dora-conference-bridge \
  --inputs alice:asr-alice/text,bob:asr-bob/text
```

## Operational Modes

### Mode 1: Bundled (Default)

**Configuration**: `COLD_START=false`

**Behavior**: Wait for ALL inputs to be ready before forwarding

**Use Case**: Debates, synchronized conference responses, multi-agent consensus

**Example**:
```
Input 1: "What is AI?" (complete)
Input 2: "AI is intelligence..." (complete)
Input 3: "" (not ready)
→ WAIT (2/3 ready)

Input 3: "Machine learning is..." (complete)
→ FORWARD ALL (3/3 ready)
```

### Mode 2: Cold Start

**Configuration**: `COLD_START=true`

**Behavior**:
- First request: Forward when ANY input is ready
- Subsequent requests: Wait for ALL inputs (switches to bundled mode)

**Use Case**: Reducing initial latency, hybrid modes

**Example**:
```
First Cycle:
Input 1: "Hello" (complete)
Input 2: "" (not ready)
Input 3: "" (not ready)
→ FORWARD (Input 1 only) - cold start used

Second Cycle:
Input 1: "Question?" (complete)
Input 2: "Answer 1" (complete)
Input 3: "" (not ready)
→ WAIT (2/3 ready) - now in bundled mode
```

## Question ID Management

### Overview

The bridge supports question-based grouping with automatic or manual question_id management.

### Increment Mode (`INC_QUESTION_ID=true`)

```
Cycle 1: question_id=1 (from input) → output question_id=1
Cycle 2: question_id=1 (from input) → output question_id=2 (incremented)
Cycle 3: question_id=1 (from input) → output question_id=3 (incremented)
```

**Use Case**: Sequential Q&A where each cycle is a new question

### Passthrough Mode (`INC_QUESTION_ID=false`)

```
Cycle 1: question_id=1 (from input) → output question_id=1
Cycle 2: question_id=5 (from input) → output question_id=5
Cycle 3: question_id=5 (from input) → output question_id=5
```

**Use Case**: Debates where question_id identifies the topic, not the turn

### Question ID Extraction

```rust
// Lines 306-313 in handle_input()
if self.current_question_id == 0 {
    if let Some(Parameter::String(qid_str)) = metadata.get("question_id") {
        if let Ok(qid) = qid_str.parse::<u32>() {
            self.current_question_id = qid;
        }
    }
}
```

The bridge uses the `question_id` from the first input received in a cycle.

## Message State Machine

### Input State Transitions

```
[No Message]
    ↓ (Input received, session_status="started")
[Streaming]
    ↓ (More chunks, session_status="ongoing")
[Streaming]
    ↓ (session_status="ended" or "cancelled")
[Complete]
    ↓ (Forwarded)
[Reset]
```

### Bridge Cycle State

```
[Waiting]
    ↓ (Input 1 complete)
[Waiting (1/3)]
    ↓ (Input 2 complete)
[Waiting (2/3)]
    ↓ (Input 3 complete)
[All Ready → Forwarding]
    ↓ (Forward complete)
[Reset → Waiting]
```

## Integration Examples

### Example 1: 3-Person Conference

```yaml
# dataflow.yml
nodes:
  - id: asr-alice
    operator:
      python: dora-asr
    env:
      SESSION_ID: alice
    outputs:
      - text

  - id: asr-bob
    operator:
      python: dora-asr
    env:
      SESSION_ID: bob
    outputs:
      - text

  - id: asr-charlie
    operator:
      python: dora-asr
    env:
      SESSION_ID: charlie
    outputs:
      - text

  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      alice: asr-alice/text
      bob: asr-bob/text
      charlie: asr-charlie/text
    env:
      LOG_LEVEL: info
      COLD_START: false
      INC_QUESTION_ID: true

  - id: llm
    operator:
      rust: dora-maas-client
    inputs:
      text: conference-bridge/text
    env:
      MAAS_CONFIG_PATH: maas_config.toml
```

### Example 2: Debate with Judge

```yaml
# dataflow.yml
nodes:
  - id: debater1
    operator:
      python: debate-agent
    outputs:
      - text

  - id: debater2
    operator:
      python: debate-agent
    outputs:
      - text

  - id: debater3
    operator:
      python: debate-agent
    outputs:
      - text

  - id: debate-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      debater1: debater1/text
      debater2: debater2/text
      debater3: debater3/text
    env:
      LOG_LEVEL: debug
      COLD_START: false      # Wait for all debaters
      INC_QUESTION_ID: true  # Track debate rounds

  - id: judge-llm
    operator:
      rust: dora-maas-client
    inputs:
      text: debate-bridge/text
      control: debate-bridge/status

  - id: tts
    operator:
      python: dora-primespeech
    inputs:
      text: judge-llm/text
```

### Example 3: Hybrid ASR + LLM (Cold Start)

```yaml
# dataflow.yml
nodes:
  - id: asr
    operator:
      python: dora-asr
    outputs:
      - text

  - id: llm
    operator:
      rust: dora-maas-client
    inputs:
      text: asr/text
    env:
      MAAS_CONFIG_PATH: maas_config.toml

  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      asr: asr/text
      llm: llm/text
    env:
      LOG_LEVEL: info
      COLD_START: true  # First response from ASR only (low latency)
      INC_QUESTION_ID: false

  - id: tts
    operator:
      python: dora-primespeech
    inputs:
      text: conference-bridge/text
```

## Dora ASR Integration Example

The conference bridge integrates with Dora ASR nodes through proper use of `session_status`:

```python
# dora-asr node output
from dora import Node

node = Node()

# Send partial result
node.send_output(
    "text",
    partial_transcription,
    {"session_status": "ongoing"}
)

# Send final result with completion signal
node.send_output(
    "text",
    final_transcription,
    {"session_status": "ended"}  # CRITICAL: Signals completion
)
```

Without `session_status="ended"`, the conference bridge will never forward the message.

## Performance Considerations

### Memory Management

- Input states cleared after each forwarding cycle
- Arrival queue limited to number of active participants
- No unbounded memory growth

### Latency

**Bundled Mode**:
- Latency: max(all participants completion times)
- Deterministic: waits for slowest participant

**Cold Start Mode**:
- First cycle: min(first participant completion time)
- Subsequent cycles: same as bundled mode

### CPU Usage

- Minimal processing overhead
- No busy waiting
- Event-driven (Dora runtime)

## Troubleshooting

### Issue: Bridge never forwards messages

**Cause**: Upstream nodes not sending `session_status="ended"`

**Solution**: Ensure upstream nodes properly signal completion:

```python
# Wrong - bridge will wait forever
node.send_output("text", final_message, {})

# Correct - signals completion
node.send_output("text", final_message, {"session_status": "ended"})
```

### Issue: Messages forwarded out of order

**Cause**: Not using `session_status` correctly

**Solution**: Use FIFO queue pattern:

```python
# For each participant:
node.send_output("text", msg1, {"session_status": "started"})
node.send_output("text", msg2, {"session_status": "ongoing"})
...
node.send_output("text", final, {"session_status": "ended"})
```

### Issue: Cold start not working

**Cause**: Not enabled or already used

**Solution**: Check logs for "Cold start used" message. Cold start only works for FIRST cycle.

### Issue: Cancellation not detected

**Cause**: Bridge not checking session_status="cancelled"

**Solution**: Upstream must send cancellation signal:

```python
# When cancelling:
node.send_output(
    "text",
    "",
    {"session_status": "cancelled"}
)
```

### Issue: Question ID not incrementing

**Cause**: `INC_QUESTION_ID` not enabled

**Solution**: Set environment variable:

```yaml
env:
  INC_QUESTION_ID: "true"
```

## Logging

The bridge provides detailed logging at different levels:

**Log Levels**:
- `error`: Errors only
- `warn`: Warnings and errors
- `info`: Normal operation (default)
- `debug`: Detailed flow information

**Debug Logging** (when `LOG_LEVEL=debug`):

```
[DEBUG] Received input from participant1: 45 chars
[DEBUG] Input participant1 marked as ready
[INFO] All conditions met, ready inputs: ["participant1", "participant2"]
[INFO] Adding participant1 to bundle (45 chars)
[INFO] Adding participant2 to bundle (32 chars)
[INFO] Forwarding bundled message: 2 ports, 79 chars, question_id=1
[DEBUG] State reset complete
```

## Best Practices

### 1. Always Use session_status

```python
# ✅ CORRECT
every_output = {"session_status": "started|ongoing|ended|cancelled"}

# ❌ INCORRECT
final_output = {}  # Bridge will hang!
```

### 2. Consistent Session Management

```python
# Use consistent session_id for related messages
metadata = {
    "session_id": f"user_{user_id}",
    "question_id": str(question_id),
    "session_status": status
}
```

### 3. Handle Cancellation

```python
# Always check for cancellation
event = node.next()
if event["metadata"].get("session_status") == "cancelled":
    logger.warning("Received cancellation")
    clear_state()
```

### 4. Use Appropriate Mode

```yaml
# Debate/Conference: bundled mode
COLD_START: false

# Low-latency hybrid: cold start
COLD_START: true

# Sequential Q&A: increment question_id
INC_QUESTION_ID: true

# Topic-based: passthrough question_id
INC_QUESTION_ID: false
```

### 5. Monitor Status Updates

```python
# Connect status output for monitoring
event = node.next()
if event["type"] == "status":
    logger.info(f"Bridge status: {event['data']}")
```

## Version History

- **v1.0**: Initial implementation with bundled forwarding
- **v1.1**: Added cold start mode for reduced latency
- **v1.2**: Added question_id tracking and increment support
- **v1.3**: Enhanced logging and status reporting
- **v1.4**: Added cancellation handling with session_status="cancelled"

## API Summary

### Inputs (Dynamic Ports)

- **Any port name** (except "control"): Participant messages
  - Data: StringArray
  - Metadata: `session_status` (REQUIRED), `question_id`, `session_id`

- **`control`**: Control commands
  - Data: StringArray
  - Commands: `reset`

### Outputs

- **`text`**: Bundled messages
  - Data: StringArray (concatenated with newlines)
  - Metadata: `question_id`, `session_id`, `participant_count`

- **`status`**: Status updates
  - Data: StringArray
  - Values: `waiting (X/Y)`, `forwarded`, `cancelled`, `reset`

### Configuration

- `STREAMING_PORTS`: Comma-separated list of streaming inputs
- `LOG_LEVEL`: Log verbosity (error, warn, info, debug)
- `COLD_START`: Enable cold start mode (true/false)
- `INC_QUESTION_ID`: Auto-increment question_id (true/false)

### Critical Rule

**ALWAYS send `session_status="ended"` or `session_status="cancelled"` to signal message completion.** Without this, the bridge will not forward messages.

## See Also

- [Dora MaaS Client API](../dora-maas-client/API.md) - For upstream AI integration
- [Dora ASR Integration](../../dora-asr/README.md) - Speech recognition with proper session_status
- [Dora Dataflow Documentation](https://dora-rs.ai) - Dora framework documentation
