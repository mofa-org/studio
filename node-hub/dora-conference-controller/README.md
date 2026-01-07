# Dora Conference Controller

A flexible conference controller for Dora that manages multi-participant speaking turns based on configurable policies. Uses the pattern string itself to determine the policy type - no separate `type` field needed!

## Features

- **Pattern-based configuration**: The pattern string determines the behavior mode
- **Three policy modes in one**: Sequential, ratio-based, and priority-based
- **Simple syntax**: Intuitive patterns like `[Judge → Defense → Prosecution]`
- **Word count tracking**: Fair turn allocation based on actual speaking time
- **Extensible**: Easy to add new policy implementations

## Quick Start

### 1. Basic Usage

```yaml
# dataflow.yml
nodes:
  - id: conference-controller
    operator:
      rust: dora-conference-controller
    env:
      # Set pattern via environment variable
      DORA_POLICY_PATTERN: "[Judge → Defense → Prosecution]"
    inputs:
      # Participant inputs (can be any number)
      llm1: maas-client-1/text
      llm2: maas-client-2/text
      llm3: maas-client-3/text
      # Control input
      control: reset-control/status
    outputs:
      - control    # Commands to conference bridge
      - status     # Controller status
```

### 2. Policy Pattern Syntax

#### Important: Use Input Port Names!

The policy pattern must use the **exact same names** as your input ports in the YAML configuration:

```yaml
inputs:
  judge: llm-judge/text        # Port name: "judge"
  defense: llm-defense/text    # Port name: "defense"

env:
  # Use those exact same names in the pattern (case-sensitive!)
  DORA_POLICY_PATTERN: "[judge → defense]"  # ✅ Correct
  # DORA_POLICY_PATTERN: "[Judge → Defense]"  # ❌ Wrong (different names)
```

#### A. Sequential Mode (→ arrows)

**Syntax**: `[Name → Name → Name]`

**Example**:
```yaml
DORA_POLICY_PATTERN: "[judge → defense → prosecution]"
```

**Behavior**:
- Speakers take turns in exact sequence
- Cycles back to start after last speaker
- Everyone gets equal opportunity

**Use case**: Courtroom debates, structured Q&A, round-table discussions

#### B. Ratio/Priority Mode (parenthesis)

**Syntax**: `[(Name, weight)]` where weight is number or *

**Priority syntax (always speak first)**: `[(Name, *)]`

**Example with priority**:
```yaml
# judge always speaks first (unless they just spoke)
# Use input port names: judge, defense, prosecution
DORA_POLICY_PATTERN: "[(judge, *), (defense, 1), (prosecution, 1)]"
```

**Example with ratios**:
```yaml
# judge gets 2x the speaking time of others
DORA_POLICY_PATTERN: "[(judge, 2), (defense, 1), (prosecution, 1)]"
```

**Behavior**:
- Priority (*) speakers get preference
- Ratios determine relative speaking time
- Fair distribution based on actual word counts

**Use case**: Moderated debates, interviews with host priority, weighted discussions

#### C. Simple Ratio Mode (just names)

**Syntax**: `[Name, Name, Name]`

**Example**:
```yaml
# All participants get equal time
# Use input port names: judge, defense, prosecution
DORA_POLICY_PATTERN: "[judge, defense, prosecution]"
```

**Behavior**:
- Equal ratio for all participants (1:1:1)
- Distribute speaking time evenly
- Same as `[(Judge, 1), (Defense, 1), (Prosecution, 1)]`

**Use case**: Equal-time debates, balanced multi-participant conversations

## Dataflow Configuration Examples

### Example 1: Courtroom Debate (Sequential)

```yaml
nodes:
  - id: conference-controller
    operator:
      rust: dora-conference-controller
    env:
      DORA_POLICY_PATTERN: "[Judge → Defense → Prosecution]"
    inputs:
      judge: llm-judge/text
      defense: llm-defense/text
      prosecution: llm-prosecution/text
      control: reset-control/status
    outputs:
      - control
      - status

  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      judge: llm-judge/text
      defense: llm-defense/text
      prosecution: llm-prosecution/text
      control: conference-controller/control   # Controlled by controller
    outputs:
      - text
```

### Example 2: Interview with Host Priority

```yaml
nodes:
  - id: conference-controller
    operator:
      rust: dora-conference-controller
    env:
      # Host (*) speaks first, then guests share equally
      DORA_POLICY_PATTERN: "[(HostAlex, *), (GuestBob, 1), (GuestCharlie, 1)]"
    inputs:
      alex: llm-alex/text
      bob: llm-bob/text
      charlie: llm-charlie/text
      control: reset-control/status
    outputs:
      - control
      - status

  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      alex: llm-alex/text
      bob: llm-bob/text
      charlie: llm-charlie/text
      control: conference-controller/control
    outputs:
      - text
```

### Example 3: Equal-Time Panel Discussion

```yaml
nodes:
  - id: conference-controller
    operator:
      rust: dora-conference-controller
    env:
      # All panelists get equal time
      DORA_POLICY_PATTERN: "[Economist, PoliticalScientist, Historian, Journalist]"
    inputs:
      economist: llm-economist/text
      politicalscientist: llm-politicalscientist/text
      historian: llm-historian/text
      journalist: llm-journalist/text
      control: reset-control/status
    outputs:
      - control
      - status

  - id: conference-bridge
    operator:
      rust: dora-conference-bridge
    inputs:
      economist: llm-economist/text
      politicalscientist: llm-politicalscientist/text
      historian: llm-historian/text
      journalist: llm-journalist/text
      control: conference-controller/control
    outputs:
      - text
```

## Advanced Configuration

### Setting Pattern Programmatically

```python
import os
from dora import Node

node = Node()

# Set pattern via DORA_POLICY_PATTERN environment variable
os.environ['DORA_POLICY_PATTERN'] = "[(Judge, *), (Defense, 1), (Prosecution, 1)]"

# Then start dora
```

### Multiple Controllers with Different Patterns

```yaml
nodes:
  # First controller: Opening statements (sequential)
  - id: opening-controller
    operator:
      rust: dora-conference-controller
    env:
      DORA_POLICY_PATTERN: "[Judge → Prosecution → Defense]"
    inputs:
      judge: judge-llm/text
      prosecution: prosecution-llm/text
      defense: defense-llm/text
    outputs:
      - control

  # Second controller: Cross-examination (priority-based)
  - id: cross-exam-controller
    operator:
      rust: dora-conference-controller
    env:
      DORA_POLICY_PATTERN: "[(Judge, *), (Prosecution, 1), (Defense, 1)]"
    inputs:
      judge: judge-llm/text
      prosecution: prosecution-llm/text
      defense: defense-llm/text
    outputs:
      - control
```

## Control Commands

### Reset
```bash
# Reset controller state and word counts
node.send_output("control", "reset")
```

### Status Check
```bash
# Get controller statistics
node.send_output("control", "stats")
```

Stats format:
```json
{
  "mode": "ratio_priority",
  "participants": ["Judge", "Defense", "Prosecution"],
  "weights": [
    {"name": "Judge", "weight": "*"},
    {"name": "Defense", "weight": 1.0},
    {"name": "Prosecution", "weight": 1.0}
  ],
  "word_counts": {
    "Judge": 450,
    "Defense": 320,
    "Prosecution": 295
  }
}
```

## Building

```bash
cd /Users/yuechen/home/fresh/dora

cargo build -p dora-conference-controller --release
```

## API

### Inputs

- **Any named input** (e.g., `llm1`, `llm2`, `participant_1`): Receives text from participants
  - Type: `StringArray`
  - Triggers speaker selection when received

- **control**: Control commands
  - Type: `StringArray`
  - Commands:
    - `reset`: Clear state and word counts
    - `ready`: Health check
    - `stats`: Request statistics

### Outputs

- **control**: Commands to conference bridge
  - Type: `StringArray`
  - Sends: `resume` commands

- **status**: Controller status and statistics
  - Type: `StringArray` (JSON)
  - Contains: Current speaker, word counts, configuration

## Design Rationale

### Why Pattern-Based Configuration?

The pattern string itself encodes the policy type:
- `→` arrows = sequential mode
- `(Name, weight)` = ratio/priority mode
- Plain names = simple ratio mode

This makes configuration:
- **Intuitive**: Pattern syntax is self-documenting
- **Compact**: No separate type field needed
- **Flexible**: Easy to switch between modes
- **Error-resistant**: Invalid patterns are rejected at startup

### Comparison: Explicit Types vs Pattern-Based

**OLD (explicit type):**
```toml
[policy]
type = "sequential"
sequence = ["Judge", "Defense", "Prosecution"]
```

**NEW (pattern-based):**
```yaml
DORA_POLICY_PATTERN: "[Judge → Defense → Prosecution]"
```

Advantages:
- ✅ 50% less configuration
- ✅ Syntax visible in pattern itself
- ✅ Single source of truth
- ✅ More ergonomic for users

## Related Documentation

- **Conference Bridge API**: [dora-conference-bridge/README.md](../dora-conference-bridge/README.md)
- **MaaS Client Integration**: [dora-maas-client/API.md](../dora-maas-client/API.md)
- **Control Commands**: [dora-conference-bridge/CONTROL_COMMANDS_SUMMARY.md](../dora-conference-bridge/CONTROL_COMMANDS_SUMMARY.md)

## Examples

See the `examples/conference-controller/` directory for complete working examples of different policy configurations.
