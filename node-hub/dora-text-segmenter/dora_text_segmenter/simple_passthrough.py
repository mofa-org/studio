#!/usr/bin/env python3
"""
Simple passthrough segmenter - just forwards text immediately
"""

import os
import time
import json
import pyarrow as pa
from dora import Node


def send_log(node, level, message, config_level="INFO"):
    """Send log message through log output channel."""
    LOG_LEVELS = {"DEBUG": 10, "INFO": 20, "WARNING": 30, "ERROR": 40}

    if LOG_LEVELS.get(level, 0) < LOG_LEVELS.get(config_level, 20):
        return

    formatted_message = f"[{level}] {message}"
    log_data = {
        "node": "text-segmenter",
        "level": level,
        "message": formatted_message,
        "timestamp": time.time()
    }
    node.send_output("log", pa.array([json.dumps(log_data)]))


def main():
    node = Node("text-segmenter")
    log_level = os.getenv("LOG_LEVEL", "INFO")

    send_log(node, "INFO", "Mode: passthrough", log_level)
    send_log(node, "INFO", "Will pass through all text immediately", log_level)

    segment_index = 0

    for event in node:
        if event["type"] == "INPUT":
            if event["id"] == "text":
                text = event["value"][0].as_py()
                metadata = event.get("metadata", {})

                send_log(node, "DEBUG", f"Received text: {len(text)} chars", log_level)
                send_log(node, "DEBUG", f"Text preview: {text[:100]}...", log_level)

                # Immediately send as segment
                out_metadata = {
                    "segment_index": segment_index,
                    "original_metadata": metadata
                }

                node.send_output(
                    "text_segment",
                    pa.array([text]),
                    metadata=out_metadata
                )

                send_log(node, "INFO", f"Sent segment {segment_index}: {len(text)} chars", log_level)
                segment_index += 1

                # Also send completion signal
                node.send_output(
                    "status",
                    pa.array(["segment_sent"]),
                    metadata={"segment_index": segment_index - 1}
                )

            elif event["id"] == "tts_complete":
                send_log(node, "DEBUG", "TTS completed", log_level)

        elif event["type"] == "STOP":
            break

    send_log(node, "INFO", "Stopped", log_level)

if __name__ == "__main__":
    main()