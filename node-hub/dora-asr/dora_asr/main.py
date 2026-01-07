"""
Dora ASR Node - Main entry point
Multi-engine ASR with task management and interruption handling.
"""

import time
import os
import sys
import json
import numpy as np
import pyarrow as pa
from dora import Node

from .config import ASRConfig
from .manager import ASRManager
from .utils import (
    calculate_audio_stats,
    normalize_transcription,
    split_audio_for_long_transcription,
    merge_transcription_chunks
)

# Add common logging to path
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..', 'dora-common'))
from dora_common.logging import send_log as common_send_log, get_log_level_from_env


def send_log(node, level, message, config_level="INFO"):
    """Wrapper for backward compatibility during migration to common logging."""
    # Convert old format to new format
    common_send_log(node, level, message, "dora-asr", config_level)


def main():
    """Main entry point for ASR node"""

    # Initialize
    node = Node()
    config = ASRConfig()
    manager = ASRManager(node)  # Pass node for logging

    # Send initialization logs
    send_log(node, "INFO", "ASR Node initialized", config.LOG_LEVEL)
    send_log(node, "INFO", f"Engine: {config.ASR_ENGINE}", config.LOG_LEVEL)
    send_log(node, "INFO", f"Language: {config.LANGUAGE}", config.LOG_LEVEL)
    send_log(node, "INFO", f"Log level: {config.LOG_LEVEL}", config.LOG_LEVEL)
    send_log(node, "DEBUG", f"Punctuation: {config.ENABLE_PUNCTUATION}", config.LOG_LEVEL)
    send_log(node, "DEBUG", f"Models directory: {config.get_models_dir()}", config.LOG_LEVEL)

    # Pre-initialize ASR engine to avoid first-call delay
    try:
        send_log(node, "INFO", "Pre-initializing ASR engine...", config.LOG_LEVEL)
        start_time = time.time()

        # Determine which engine to initialize based on config
        if config.ASR_ENGINE == 'auto':
            # Initialize the engine for the configured language
            engine_name = manager.get_engine_for_language(config.LANGUAGE)
        else:
            engine_name = config.ASR_ENGINE

        # Pre-initialize the engine
        manager.get_or_create_engine(engine_name)

        init_time = time.time() - start_time
        send_log(node, "INFO", f"ASR engine pre-initialized in {init_time:.2f}s", config.LOG_LEVEL)
    except Exception as e:
        send_log(node, "WARNING", f"Failed to pre-initialize ASR engine: {e}", config.LOG_LEVEL)
        send_log(node, "WARNING", "Engine will be initialized on first use", config.LOG_LEVEL)
    
    # Statistics
    total_segments = 0
    total_duration = 0
    
    for event in node:
        if event["type"] == "INPUT":
            input_id = event["id"]
            
            if input_id == "audio":
                # Get audio segment
                audio_array = event["value"].to_numpy()
                input_metadata = event.get("metadata", {})

                # Extract metadata - pass through all input metadata
                segment_num = input_metadata.get("segment", 0)
                sample_rate = input_metadata.get("sample_rate", config.SAMPLE_RATE)

                # Calculate audio statistics
                audio_stats = calculate_audio_stats(audio_array)
                duration = audio_stats['duration']

                question_id = input_metadata.get("question_id", "unknown")
                send_log(node, "INFO", f"Processing segment #{segment_num} (question_id={question_id})", config.LOG_LEVEL)
                send_log(node, "DEBUG", f"   Duration: {duration:.2f}s", config.LOG_LEVEL)
                
                start_time = time.time()
                
                try:
                    # Check if audio is too long and needs splitting
                    if duration > config.MAX_AUDIO_DURATION:
                        send_log(node, "WARNING", f"Audio too long ({duration:.1f}s), splitting...", config.LOG_LEVEL)
                        
                        # Split into chunks
                        chunks = split_audio_for_long_transcription(
                            audio_array,
                            sample_rate=sample_rate,
                            chunk_duration=config.MAX_AUDIO_DURATION,
                            overlap_duration=1.0
                        )
                        
                        # Transcribe each chunk
                        transcribed_chunks = []
                        for i, chunk_data in enumerate(chunks):
                            send_log(node, "DEBUG", f"Processing chunk {i+1}/{len(chunks)}...", config.LOG_LEVEL)
                            result = manager.transcribe(
                                chunk_data['audio'],
                                language=config.LANGUAGE
                            )
                            transcribed_chunks.append({
                                'text': result['text'],
                                'start_time': chunk_data['start_time'],
                                'end_time': chunk_data['end_time']
                            })
                        
                        # Merge results
                        full_text = merge_transcription_chunks(transcribed_chunks)
                        detected_language = config.LANGUAGE
                        
                    else:
                        # Normal transcription
                        result = manager.transcribe(
                            audio_array,
                            language=config.LANGUAGE
                        )
                        full_text = result['text']
                        detected_language = result['language']
                    
                    # Normalize text
                    full_text = normalize_transcription(full_text, detected_language)
                    
                    processing_time = time.time() - start_time
                    
                    # Update statistics
                    total_segments += 1
                    total_duration += duration
                    
                    # Skip empty transcriptions
                    if not full_text.strip():
                        send_log(node, "WARNING", "Empty transcription", config.LOG_LEVEL)
                        continue
                    
                    send_log(node, "INFO", f"Transcribed: {full_text[:100]}...", config.LOG_LEVEL)
                    send_log(node, "INFO", f"Language: {detected_language}", config.LOG_LEVEL)
                    send_log(node, "DEBUG", f"Processing time: {processing_time:.3f}s", config.LOG_LEVEL)
                    send_log(node, "DEBUG", f"Speed: {duration/processing_time:.1f}x realtime", config.LOG_LEVEL)

                    # Send transcription output - pass through all input metadata
                    output_metadata = input_metadata.copy()
                    output_metadata["session_status"] = "ended"  # Mark transcription complete for conference system

                    node.send_output(
                        "transcription",
                        pa.array([full_text]),
                        metadata=output_metadata
                    )
                    
                    # Send language detection if enabled
                    if config.ENABLE_LANGUAGE_DETECTION:
                        node.send_output(
                            "language_detected",
                            pa.array([detected_language]),
                            metadata={}
                        )

                    # Send processing time
                    node.send_output(
                        "processing_time",
                        pa.array([processing_time]),
                        metadata={}
                    )

                    # Send confidence if available
                    if config.ENABLE_CONFIDENCE_SCORE and result.get('confidence'):
                        node.send_output(
                            "confidence",
                            pa.array([result['confidence']]),
                            metadata={}
                        )
                    
                except Exception as e:
                    send_log(node, "ERROR", f"Transcription error: {e}", config.LOG_LEVEL)

                    # Send empty transcription on error - pass through input metadata
                    error_metadata = input_metadata.copy()
                    error_metadata["error"] = str(e)

                    node.send_output(
                        "transcription",
                        pa.array([""]),
                        metadata=error_metadata
                    )
            
            elif input_id == "control":
                # Handle control commands
                command = event["value"][0].as_py()
                
                if command == "stats":
                    # Report statistics
                    send_log(node, "INFO", "ASR Statistics:", config.LOG_LEVEL)
                    send_log(node, "INFO", f"Total segments: {total_segments}", config.LOG_LEVEL)
                    send_log(node, "INFO", f"Total duration: {total_duration:.1f}s", config.LOG_LEVEL)
                    if total_segments > 0:
                        send_log(node, "INFO", f"Average duration: {total_duration/total_segments:.1f}s", config.LOG_LEVEL)
                
                elif command == "cleanup":
                    # Cleanup resources
                    send_log(node, "INFO", "Cleaning up ASR engines...", config.LOG_LEVEL)
                    manager.cleanup()
                    send_log(node, "INFO", "Cleanup complete", config.LOG_LEVEL)

                elif command == "reset":
                    # Reset signal from websocket server - ASR is stateless so just log
                    send_log(node, "INFO", "🔄 Reset received - ASR ready for new session", config.LOG_LEVEL)
                    # Reset counters for the new session
                    total_segments = 0
                    total_duration = 0.0


if __name__ == "__main__":
    main()