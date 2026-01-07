"""
Common logging utility for Dora nodes.
Provides consistent logging across all components.
"""

import pyarrow as pa
from typing import Any, Dict


def send_log(node, level: str, message: str, node_name: str = None, config_level: str = "INFO"):
    """
    Send log message through log output channel.

    Args:
        node: Dora node instance
        level: Log level (DEBUG, INFO, WARNING, ERROR)
        message: Log message
        node_name: Name of the node (auto-detected if not provided)
        config_level: Minimum log level to output (default: INFO)
    """
    LOG_LEVELS = {"DEBUG": 10, "INFO": 20, "WARNING": 30, "ERROR": 40}

    if LOG_LEVELS.get(level, 0) < LOG_LEVELS.get(config_level, 20):
        return

    # Auto-detect node name if not provided
    if node_name is None:
        node_name = node.__class__.__name__ if hasattr(node, '__class__') else "unknown_node"

    formatted_message = f"[{level}] {message}"
    log_data = {
        "node": node_name,
        "level": level,
        "message": message
        # timestamp will be handled by viewer, not included in metadata
    }

    try:
        # Send JSON string for debate_viewer compatibility
        import json
        json_message = json.dumps(log_data)
        node.send_output("log", pa.array([json_message]), metadata=log_data)
    except Exception as e:
        # Fallback to print if logging fails
        print(f"[{node_name}] Logging failed: {e}")
        print(f"[{node_name}] {formatted_message}")


def send_status(node, status: str, details: Dict[str, Any] = None, node_name: str = None):
    """
    Send status message through status output channel.

    Args:
        node: Dora node instance
        status: Status message
        details: Additional status details as metadata
        node_name: Name of the node (auto-detected if not provided)
    """
    if node_name is None:
        node_name = node.__class__.__name__ if hasattr(node, '__class__') else "unknown_node"

    metadata = {
        "node": node_name,
        "status": status,
        "timestamp": None
    }

    if details:
        metadata.update(details)

    try:
        node.send_output("status", pa.array([status]), metadata=metadata)
    except Exception:
        # Fallback to print if status sending fails
        print(f"[{node_name}] STATUS: {status}")


def get_log_level_from_env(env_var: str = "LOG_LEVEL", default: str = "INFO") -> str:
    """
    Get log level from environment variable.

    Args:
        env_var: Environment variable name
        default: Default log level

    Returns:
        Log level string
    """
    import os
    return os.getenv(env_var, default).upper()