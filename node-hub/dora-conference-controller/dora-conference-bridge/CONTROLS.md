# Dora Conference Bridge - Control Commands Reference

## Overview

The conference bridge accepts control commands via the `control` input port. These commands allow runtime management of the bridge state, message queues, and forwarding behavior.

**Input Port**: `control`
**Data Type**: `StringArray` (JSON or plain text)
**Location**: `node-hub/dora-conference-bridge/src/main.rs:626-673`

## Control Command List

### 1. `reset` - Clear All State

**Command Format**:
- Plain text: `"reset"`
- JSON: `{"command": "reset"}`

**Description**: Performs a complete reset of the conference bridge state. This is the most comprehensive cleanup command.

**Behavior**:
- Clears ALL message queues for ALL input ports (discards pending messages)
- Empties the arrival queue (FIFO order tracking)
- Resets `current_question_id` to 0
- Sets `cold_start_used` to false (enables cold start again if enabled)
- Clears `was_cancelled` flag
- Sends status output: `"reset"`

**Logs Generated**:
```
[INFO] Reset command received - state restored to initial configuration
[INFO] Conference cycle cancelled - clearing state (if was_cancelled=true)
```

**Use Cases**:
- Emergency stop / conversation reset
- When upstream timestamps are stale
- Recovery from error states
- Restarting a conversation/debate session
- Clearing backlog when upstream is producing too fast

**Example Usage**:
```python
from dora import Node

node = Node()

# Reset via plain text
node.send_output("control", "reset")

# Reset via JSON
node.send_output("control", '{"command": "reset"}')
```

**Dataflow Configuration**:
```yaml
nodes:
  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      participant1: asr1/text
      participant2: asr2/text
      control: controller/reset_command  # Connect control port
```

**Implementation Reference**:
```rust
// src/main.rs:417-432
fn reset_state(&mut self, node: &mut DoraNode) -> Result<()> {
    for (_, input) in self.inputs.iter_mut() {
        let drain = input.is_streaming_active();
        input.reset_with_drain(drain);  // Clears message_queue
    }
    self.arrival_queue.clear();
    self.current_question_id = 0;
    self.cold_start_used = false;
    send_status(node, "reset")?;
    Ok(())
}
```

### 2. `ready` - Request Status Update

**Command Format**:
- Plain text: `"ready"`
- JSON: `{"command": "ready"}`

**Description**: Request the bridge to send a "ready" status update.

**Behavior**:
- Sends status output: `"ready"`
- Does NOT clear any state or queues
- Useful for synchronization with upstream/downstream nodes

**Use Cases**:
- Health check / liveness probe
- Initial handshake with upstream nodes
- Debugging / testing connectivity
- Confirming bridge is operational

**Example Usage**:
```python
from dora import Node

node = Node()

# Request ready status
node.send_output("control", "ready")

# Listen for response
event = node.next()
if event["type"] == "status" and event["data"] == "ready":
    print("Bridge is ready")
```

**Response Format**:
```yaml
type: status
id: status
data: ["ready"]
metadata: {}
```

### 3. `exit` - Remove Session

**Command Format**:
- Plain text: `"exit"`
- JSON: `{"command": "exit"}`

**Description**: Remove session and clean up session-specific state.

**Behavior**:
- Removes session from the `sessions` HashMap
- Logs session removal
- Does NOT clear message queues or reset bridge state
- Primarily for session management in conversation flows

**Logs Generated**:
```
[INFO] Removed session: <session_id>
```

**Use Cases**:
- User disconnects from chat
- Session timeout
- Cleanup after conversation ends
- Memory management for long-running instances

**Example Usage**:
```python
# Send exit command for session cleanup
node.send_output("control", "exit")
```

**Implementation Reference**:
```rust
// src/main.rs:1194 (in control handling)
else if control_text.eq_ignore_ascii_case("exit") {
    sessions.remove(&session_id);
    send_log(&mut node, "INFO", &format!("Removed session: {}", session_id))?;
}
```

## Command Processing Details

### Input Parsing

The bridge supports two input formats:

**1. Plain Text Format**:
```python
node.send_output("control", "reset")
```
- Direct string command
- Converted to lowercase for case-insensitive matching
- Simple and lightweight

**2. JSON Format**:
```python
node.send_output("control", '{"command": "reset"}')
```
- Structured JSON object
- `command` field contains the actual command
- Extensible for future parameters
- Example: `{"command": "reset", "param": "value"}`

**Processing Flow**:
```rust
// src/main.rs:626-650
if port_name == "control" {
    let control_payload = collect_string_data(data);
    let trimmed = control_payload.trim();

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        // JSON format
        if let Some(cmd) = value.get("command").and_then(|v| v.as_str()) {
            command = Some(cmd.to_ascii_lowercase());
        }
    } else {
        // Plain text format
        command = Some(trimmed.to_ascii_lowercase());
    }

    match command.as_deref() {
        Some("reset") => { /* ... */ }
        Some("ready") => { /* ... */ }
        Some("exit") => { /* ... */ }
        Some(other) => { /* warning */ }
        None => { /* ignore */ }
    }
}
```

### Command Validation

**Valid Commands**:
- ✓ `reset`
- ✓ `ready`
- ✓ `exit`

**Invalid Commands**:
- ✗ Returns warning log: `Unknown control command: <command>`
- ✗ No state changes
- ✗ Command is ignored

**Examples of Invalid Commands**:
```python
node.send_output("control", "invalid")  # Warning logged
node.send_output("control", "clear")    # Warning logged
node.send_output("control", "")         # Warning logged (empty)
```

## Control Input Metadata

The `control` input does NOT use or require metadata fields like `session_status`, `session_id`, or `question_id`. Control commands are processed independently of message metadata.

```python
# ✓ Correct - no metadata needed
node.send_output("control", "reset")

# ✓ Also works with metadata (but metadata is ignored for control)
node.send_output("control", "reset", {"session_id": "test"})
```

## Integration Examples

### Example 1: Emergency Reset Button

```yaml
# dataflow.yml
nodes:
  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      asr1: asr1/text
      asr2: asr2/text
      control: reset-button/command  # Physical button or UI
```

```python
# reset_button.py
from dora import Node
import gpio  # GPIO library

node = Node()

# Monitor physical button
while True:
    if gpio.read(BUTTON_PIN) == 0:  # Button pressed
        node.send_output("command", "reset")
        time.sleep(0.5)  # Debounce
```

### Example 2: Session Timeout Management

```python
# session_manager.py
from dora import Node
import time

node = Node()
session_start = time.time()
TIMEOUT_SECONDS = 300  # 5 minutes

while True:
    event = node.next(timeout=1.0)

    # Check for timeout
    if time.time() - session_start > TIMEOUT_SECONDS:
        print("Session timeout - resetting bridge")
        node.send_output("reset", "reset")
        session_start = time.time()  # Reset timer
```

### Example 3: Health Check Loop

```python
# health_check.py
from dora import Node
import time

node = Node()

while True:
    # Request ready status every 30 seconds
    node.send_output("health", "ready")

    # Wait for response
    event = node.next()
    if event["type"] == "status" and event["data"] == "ready":
        print("✅ Conference bridge healthy")
    else:
        print("❌ Conference bridge not responding")

    time.sleep(30)
```

## Comparison of Commands

| Command | Clears Queues | Sends Status | Clears Sessions | Use Case |
|---------|--------------|--------------|-----------------|----------|
| `reset` | ✓ YES | `"reset"` | ✗ No | Emergency stop, full cleanup |
| `ready` | ✗ No | `"ready"` | ✗ No | Health check, synchronization |
| `exit` | ✗ No | ✗ No | ✓ YES | Session cleanup (not bridge state) |

## Future Command Ideas

Potential future commands (not currently implemented):

- `pause` / `resume`: Temporarily stop/start forwarding
- `status`: Request detailed status report with queue depths
- `config`: Dynamically update configuration
- `skip`: Skip current waiting cycle and forward immediately
- `clear <port>`: Clear queue for specific port only

## Summary

The conference bridge currently supports **3 control commands**:

1.  **`reset`**  : Complete state reset (primary control command)
2.  **`ready`**  : Status check / synchronization
3.  **`exit`**  : Session cleanup

All commands:
- Accept both plain text and JSON formats
- Are case-insensitive
- Generate appropriate log messages
- Are processed immediately upon receipt

The `reset` command is the most important for emergency cleanup and queue management.
