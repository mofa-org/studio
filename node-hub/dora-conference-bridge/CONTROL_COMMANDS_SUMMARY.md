# Dora Conference Bridge - Control Commands Summary

## Quick Reference

The conference bridge accepts control commands via the `control` input port:

| Command | Format | Description |
|---------|--------|-------------|
| **reset** | `{"command": "reset"}` or `"reset"` | **Emergency reset** - Clears ALL queues and state |
| **resume** | `{"command": "resume"}` or `"resume"` | **Resume forwarding** - Send one bundled output, then auto-pause |
| **ready** | `{"command": "ready"}` or `"ready"` | Request status update (bridge responds with "ready") |

## Bridge State Model

**Default State**: **PAUSED**

The bridge always starts in PAUSED state and only forwards messages when explicitly RESUMED.

### State Transitions

```
PAUSED → [receive "resume"] → RESUMED → [forward one cycle] → PAUSED
PAUSED → [receive "reset"] → PAUSED
RESUMED → [receive "resume"] → RESUMED (no-op)
```

## Primary Command: `resume` ⏯️

**Purpose**: Trigger one forwarding cycle, then auto-pause

**What it does**:
- ✓ Changes state from PAUSED → RESUMED
- ✓ Checks if forwarding conditions are met
- ✓ If ready: Forwards bundled message, then auto-pauses
- ✓ If not ready: Waits for inputs, then forwards when ready, then auto-pauses
- ✓ Sends status: `"resumed"` or `"forwarded-from-resume"`

**Key Feature**: **Auto-pause** - After one output is sent, bridge automatically returns to PAUSED state

**When to use**:
- **Primary control mechanism** for the bridge
- Triggered by conference controller based on logic
- Sequential Q&A scenarios
- Debate turn management
- Any workflow requiring explicit control

**Example Workflow**:
```python
from dora import Node

node = Node()

# Bridge is PAUSED - collecting inputs but not forwarding

# Controller decides it's time to forward
node.send_output("control", "resume")

# Bridge will:
# 1. Change state to RESUMED
# 2. Wait for all inputs to be ready (if not already)
# 3. Forward bundled message
# 4. Automatically return to PAUSED
# 5. Wait for next "resume" command

# Send another resume for next cycle
node.send_output("control", "resume")
```

**Behavior Details**:

**If inputs are ready when resume received**:
```
T+0.00: resume received
T+0.01: forward_bundle() called
T+0.02: output sent
T+0.03: auto-paused (state=PAUSED)
```

**If inputs are NOT ready when resume received**:
```
T+0.00: resume received
T+0.01: status="resumed (waiting for inputs)"
T+1.50: final input arrives
T+1.51: forward_bundle() called
T+1.52: output sent
T+1.53: auto-paused (state=PAUSED)
```

## Secondary Command: `reset` 🔄

**Purpose**: Emergency stop and complete state cleanup

**What it does**:
- ✓ Clears ALL message queues (all pending messages discarded)
- ✓ Empties arrival queue (FIFO order tracking)
- ✓ Resets question_id to 0
- ✓ Re-enables cold start mode
- ✓ Clears cancellation flags
- ✓ Sends status: `"reset"`
- ✓ State set to PAUSED

**When to use**:
- Emergency stop
- Conversation restart
- Queue overflow recovery
- State corruption recovery
- Test cleanup between scenarios

**Example**:
```python
from dora import Node

node = Node()
node.send_output("control", "reset")
# or
node.send_output("control", '{"command": "reset"}')
```

## Tertiary Command: `ready` ✅

**Purpose**: Health check and synchronization

**What it does**:
- ✓ Sends status: `"ready"` immediately
- ✗ Does NOT clear any state
- ✗ Does NOT modify queues
- ✗ Does NOT change state

**When to use**:
- Health check / liveness probe
- Initial handshake
- Confirm bridge is operational

**Example**:
```python
from dora import Node

node = Node()
node.send_output("control", "ready")

# Wait for response
event = node.next()
if event["type"] == "status" and event["data"] == "ready":
    print("✅ Bridge is operational")
```

## Message Queue Management

The `reset` command is the **only** way to clear queued messages:

```python
# Scenario: llm1 sent 5 messages, llm2 sent 1 message
# Queues: llm1=[msg1, msg2, msg3, msg4, msg5], llm2=[msgA]

node.send_output("control", "reset")

# Result: llm1=[], llm2=[] (all cleared)
```

## Comparison

| Feature | reset | ready |
|---------|-------|-------|
| Clears message queues | ✓ YES | ✗ NO |
| Sends status | ✓ "reset" | ✓ "ready" |
| Clears arrival queue | ✓ YES | ✗ NO |
| Resets question_id | ✓ YES | ✗ NO |
| For operational use | Emergency only | Routine checks |

## Implementation

**Location**: `src/main.rs:626-673`

**Parsing**:
- Accepts plain text: `"reset"`
- Accepts JSON: `{"command": "reset"}`
- Case-insensitive matching

**Supported Commands**:
- `"reset"` / `{"command": "reset"}`
- `"ready"` / `{"command": "ready"}`
- `"exit"` / `{"command": "exit"}` (for session cleanup)

## Dataflow Configuration

```yaml
# dataflow.yml
nodes:
  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      # Participant inputs (any names except "control")
      llm1: maas-client-1/text
      llm2: maas-client-2/text
      asr1: asr-1/text

      # Control input (MUST be named "control")
      control: controller/command
```

## Best Practices

1. **Use `reset` for emergency cleanup only**
   - Potentially destructive (drops all pending messages)
   - Last resort for recovery

2. **Use `ready` for routine health checks**
   - Safe, non-destructive
   - Can be called frequently

3. **Connect control port in production**
   - Essential for emergency recovery
   - Consider adding a monitoring dashboard button

4. **Monitor queue depths**
   - Conference bridge logs queue depth when forwarding
   - If queues grow too fast, consider throttling upstream or using reset

## Related Documentation

- **Complete API Spec**: [API.md](API.md)
- **Detailed Controls**: [CONTROLS.md](CONTROLS.md) - Full documentation with examples
- **MaaS Client**: [../dora-maas-client/API.md](../dora-maas-client/API.md) - Upstream AI integration
