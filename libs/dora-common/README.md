# dora-common

Common utilities for Dora nodes in MoFA Studio.

## Overview

This package provides shared utilities used across multiple Dora nodes, ensuring consistent logging and status reporting throughout the voice chat pipeline.

## Installation

```bash
pip install -e libs/dora-common
```

## Usage

### Logging

```python
from dora_common.logging import send_log, get_log_level_from_env

# Get log level from environment
log_level = get_log_level_from_env()  # reads LOG_LEVEL env var, defaults to "INFO"

# Send log messages
send_log(node, "INFO", "Processing started", config_level=log_level)
send_log(node, "DEBUG", "Detailed debug info", config_level=log_level)
send_log(node, "ERROR", "Something went wrong", config_level=log_level)
```

### Status Updates

```python
from dora_common.logging import send_status

# Send status updates
send_status(node, "ready")
send_status(node, "processing", details={"progress": 50})
send_status(node, "error", details={"error": "Connection failed"})
```

## API Reference

### `send_log(node, level, message, node_name=None, config_level="INFO")`

Send log message through the `log` output channel.

**Parameters:**

- `node`: Dora node instance
- `level`: Log level (`DEBUG`, `INFO`, `WARNING`, `ERROR`)
- `message`: Log message string
- `node_name`: Optional node name (auto-detected if not provided)
- `config_level`: Minimum log level to output (default: `INFO`)

### `send_status(node, status, details=None, node_name=None)`

Send status message through the `status` output channel.

**Parameters:**

- `node`: Dora node instance
- `status`: Status string
- `details`: Optional dict with additional status details
- `node_name`: Optional node name (auto-detected if not provided)

### `get_log_level_from_env(env_var="LOG_LEVEL", default="INFO")`

Get log level from environment variable.

**Parameters:**

- `env_var`: Environment variable name (default: `LOG_LEVEL`)
- `default`: Default log level if env var not set (default: `INFO`)

**Returns:** Log level string (uppercase)

## Log Levels

| Level   | Value | Description                |
| ------- | ----- | -------------------------- |
| DEBUG   | 10    | Detailed debug information |
| INFO    | 20    | General information        |
| WARNING | 30    | Warning messages           |
| ERROR   | 40    | Error messages             |

## Dependencies

- `dora-rs==0.3.12`
- `pyarrow>=10.0.0`
