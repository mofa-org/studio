#!/usr/bin/env python3
"""
Voice Router for mofa-cast

Parses JSON input containing voice routing information and routes text
to different TTS nodes based on voice_name.

Input format (JSON):
{
  "speaker": "host",
  "text": "Hello world",
  "voice_name": "Luo Xiang",
  "speed": 1.0
}

Output: Routes to different outputs based on voice_name:
- text_luo_xiang: for "Luo Xiang" voice
- text_ma_yun: for "Ma Yun" voice
- text_ma_baoguo: for "Ma Baoguo" voice
- text_fallback: for unknown voices
"""

import json
import os
import pyarrow as pa
from dora import Node
from typing import Optional


def send_log(node, level, message):
    """Send log message through log output channel."""
    log_data = {
        "node": "voice-router",
        "level": level,
        "message": message
    }
    node.send_output("log", pa.array([json.dumps(log_data)]))


def main():
    """Main entry point for voice router"""
    node = Node()

    # Log level from environment
    log_level = os.getenv("LOG_LEVEL", "INFO")

    # Voice routing map (voice_name -> output_id)
    # These must match the PrimeSpeech node IDs in the dataflow
    voice_outputs = {
        "Luo Xiang": "text_luo_xiang",
        "Ma Yun": "text_ma_yun",
        "Ma Baoguo": "text_ma_baoguo",
    }

    send_log(node, "INFO", f"Voice router initialized with {len(voice_outputs)} voices")
    send_log(node, "DEBUG", f"Available voices: {list(voice_outputs.keys())}")

    while True:
        # Wait for input
        event = node.next()

        if event["type"] == "INPUT":
            input_id = event["id"]
            data = event["value"]

            # Only process text input
            if input_id != "text":
                continue

            # Parse JSON input
            try:
                # Get text from PyArrow array
                if len(data) == 0:
                    send_log(node, "WARNING", "Received empty input")
                    continue

                text_input = data[0].as_py()
                send_log(node, "DEBUG", f"Received input: {text_input[:100]}...")

                # Parse JSON
                segment_data = json.loads(text_input)

                speaker = segment_data.get("speaker", "unknown")
                text = segment_data.get("text", "")
                voice_name = segment_data.get("voice_name", "Luo Xiang")
                speed = segment_data.get("speed", 1.0)

                send_log(node, "INFO", f"Routing: speaker='{speaker}', voice='{voice_name}', speed={speed}, text_len={len(text)}")

                # Determine output based on voice_name
                output_id = voice_outputs.get(voice_name, "text_fallback")

                if output_id == "text_fallback":
                    send_log(node, "WARNING", f"Unknown voice '{voice_name}', using fallback")

                # Route text to appropriate TTS node
                # Note: We send just the text, not the full JSON
                node.send_output(output_id, pa.array([text]))

                send_log(node, "DEBUG", f"Routed to {output_id}")

            except json.JSONDecodeError as e:
                send_log(node, "ERROR", f"Invalid JSON input: {e}")
            except Exception as e:
                send_log(node, "ERROR", f"Error processing input: {e}")


if __name__ == "__main__":
    main()
