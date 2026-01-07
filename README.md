# Dora Common Utilities

Shared utilities for Dora nodes to ensure consistent logging and status reporting across all components.

## Features

- **Consistent Logging**: Standardized logging function with configurable log levels
- **Status Reporting**: Common status output format
- **Environment Integration**: Automatic log level detection from environment variables

## Installation

```bash
cd node-hub/dora-common
pip install -e .
```

## Usage

### Basic Logging

```python
from dora_common.logging import send_log, get_log_level_from_env

# Get log level from environment (LOG_LEVEL env var, defaults to INFO)
log_level = get_log_level_from_env()

# Send log messages
send_log(node, "INFO", "Node started successfully", "my-node")
send_log(node, "DEBUG", "Processing input data", "my-node", log_level)
send_log(node, "WARNING", "Input queue is nearly full", "my-node")
send_log(node, "ERROR", "Failed to process input", "my-node")
```

### Status Reporting

```python
from dora_common.logging import send_status

# Send status updates with optional metadata
send_status(node, "ready", {"queue_size": 10, "processing_rate": 5.0})
send_status(node, "processing", {"current_item": "audio_data.wav"})
send_status(node, "error", {"error_code": 500, "error_message": "Connection failed"})
```

### Environment Variables

- `LOG_LEVEL`: Set minimum log level (DEBUG, INFO, WARNING, ERROR). Defaults to `INFO`.

Example:
```bash
export LOG_LEVEL=DEBUG
python my_dora_node.py
```

## Functions

### `send_log(node, level, message, node_name=None, config_level="INFO")`

Send log message through the log output channel.

**Parameters:**
- `node`: Dora node instance
- `level`: Log level ("DEBUG", "INFO", "WARNING", "ERROR")
- `message`: Log message
- `node_name`: Name of the node (auto-detected if not provided)
- `config_level`: Minimum log level to output (default: "INFO")

### `send_status(node, status, details=None, node_name=None)`

Send status message through the status output channel.

**Parameters:**
- `node`: Dora node instance
- `status`: Status message
- `details`: Additional status details as metadata (optional)
- `node_name`: Name of the node (auto-detected if not provided)

### `get_log_level_from_env(env_var="LOG_LEVEL", default="INFO")`

Get log level from environment variable.

**Parameters:**
- `env_var`: Environment variable name (default: "LOG_LEVEL")
- `default`: Default log level (default: "INFO")

**Returns:** Log level string

## Migration Guide

To migrate existing nodes to use the common logging:

1. Replace local `send_log` function:
```python
# Old
from dora import Node
import pyarrow as pa

def send_log(node, level, message, config_level="INFO"):
    # ... local implementation

# New
from dora import Node
from dora_common.logging import send_log, get_log_level_from_env
```

2. Update log calls:
```python
# Old
send_log(node, "INFO", "Processing complete", "INFO")

# New
log_level = get_log_level_from_env()
send_log(node, "INFO", "Processing complete", "my-node", log_level)
```

## Benefits

- **Consistency**: All nodes use the same logging format
- **Maintainability**: Single source of truth for logging logic
- **Flexibility**: Easy to add new logging features
- **Environment Integration**: Automatic log level configuration
- **Fallback Support**: Graceful fallback to print() if logging fails