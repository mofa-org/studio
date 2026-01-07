#!/usr/bin/env python3
"""
Queue-based Text Segmenter with Intelligent Buffering

Features:
1. Segments streaming LLM text by punctuation marks
2. Buffers incomplete text fragments across chunks
3. Combines "北京的" + "天气好，但是不稳定" → "北京的天气好，" + buffer("但是不稳定")
4. No deadlock - first complete segment sent immediately
5. Backpressure control - sends one at a time, triggered by TTS completion
6. Skips punctuation-only segments
7. Smart reset based on question_id
"""

import os
import time
import re
import json
from typing import Iterable, List, Tuple
import pyarrow as pa
from dora import Node
from collections import deque


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

def parse_int_env(name: str, default: int) -> int:
    """Safely parse integer environment variables with fallback."""
    value = os.getenv(name)
    if value is None:
        return default

    try:
        return int(value)
    except ValueError:
        return default


def remove_speaker_id(text, node=None, log_level="INFO"):
    """Remove speaker names enclosed in square brackets like [Student1], [Tutor], [孙老师], etc.

    Args:
        text: Input text that may contain speaker IDs
        node: Dora node for logging (optional)
        log_level: Log level for filtering

    Returns:
        Text with speaker IDs removed
    """
    # Pattern: [any text] ONLY at the beginning of the string
    # Examples: [Student1], [Tutor], [孙老师], [亦菲], etc.
    # ^: match at start of string
    # \[: literal opening bracket
    # [^\]]+: one or more non-bracket characters
    # \]: literal closing bracket
    # \s*: optional whitespace after bracket
    pattern = r'^\[[^\]]+\]\s*'

    cleaned_text = re.sub(pattern, '', text)

    if node and cleaned_text != text:
        send_log(node, "DEBUG", f"Removed speaker ID: '{text}' → '{cleaned_text}'", log_level)

    return cleaned_text


def should_skip_segment(text, punctuation_marks="。！？.!?", node=None, log_level="INFO"):
    """Check if segment should be skipped (only punctuation or numbers)

    Args:
        text: Text segment to check
        punctuation_marks: String of punctuation marks to consider (configurable via env var)
        node: Dora node for logging (optional)
        log_level: Log level for filtering
    """
    # Remove whitespace for checking
    text_stripped = text.strip()

    # Skip if empty
    if not text_stripped:
        if node:
            send_log(node, "DEBUG", f"Filter: SKIP empty: '{text}' (len={len(text)})", log_level)
        return True

    # Build pattern dynamically from configured punctuation marks
    # Escape special regex characters in punctuation marks
    escaped_punctuation = re.escape(punctuation_marks)

    # Pattern: only whitespace + numbers + configured punctuation marks
    # This allows filtering based on user-configured punctuation
    skip_pattern = f'^[\\s\\d{escaped_punctuation}]+$'

    matched = re.match(skip_pattern, text_stripped)
    if matched:
        if node:
            send_log(node, "DEBUG", f"Filter: SKIP punctuation: '{text}' (len={len(text)}, pattern matched)", log_level)
        return True

    if node:
        send_log(node, "DEBUG", f"Filter: KEEP: '{text}' (len={len(text)})", log_level)
    return False


def find_split_index(text: str, max_length: int, split_marks: Iterable[str]) -> int:
    """Find a split index at or before max_length using provided marks or whitespace."""
    if max_length <= 0:
        return -1

    limit = min(len(text), max_length)

    if split_marks:
        for idx in range(limit, 0, -1):
            if text[idx - 1] in split_marks:
                return idx

    for idx in range(limit, 0, -1):
        if text[idx - 1].isspace():
            return idx

    return -1


def split_segment_to_max(
    segment: str,
    max_length: int,
    min_length: int,
    split_marks: Iterable[str],
    node=None,
    log_level: str = "INFO",
) -> Tuple[List[str], str]:
    """Split segment into chunks that respect configured boundaries."""
    if max_length <= 0 or len(segment) <= max_length:
        return [segment], ""

    chunks: List[str] = []
    remainder = segment

    if node:
        send_log(
            node,
            "DEBUG",
            f"Splitting long segment (len={len(segment)}) with max={max_length}",
            log_level,
        )

    while remainder:
        if len(remainder) <= max_length:
            chunks.append(remainder)
            break

        split_idx = find_split_index(remainder, max_length, split_marks)
        if split_idx == -1:
            split_idx = max_length

        chunk = remainder[:split_idx]
        if chunk.strip():
            chunks.append(chunk)

        remainder = remainder[split_idx:]
        remainder = remainder.lstrip()

    if chunks:
        last_chunk = chunks[-1]
        if len(last_chunk.strip()) < max(min_length, 1):
            tail = chunks.pop()
            if node:
                send_log(
                    node,
                    "DEBUG",
                    f"Holding short tail for buffer (len={len(tail.strip())}): '{tail}'",
                    log_level,
                )
            return chunks, tail

    return chunks, ""


def segment_by_punctuation(
    text,
    punctuation_marks,
    max_length,
    min_length,
    fallback_split_marks,
    node=None,
    log_level="INFO",
):
    """Segment text by punctuation marks, respecting MAX_SEGMENT_LENGTH when possible.

    Logic:
    - Find all punctuation marks in the text
    - If a segment is <= MAX_SEGMENT_LENGTH, keep it as-is
    - If a segment is > MAX_SEGMENT_LENGTH, split it at intermediate punctuation marks
    - Never split mid-sentence (always split at punctuation boundaries)
    """
    if not text:
        return [], "", False

    escaped_punctuation = re.escape(punctuation_marks)
    pattern = f'[^{escaped_punctuation}]+[{escaped_punctuation}]'

    segments: List[str] = []
    last_end = 0
    accumulator = ""

    for match in re.finditer(pattern, text):
        segment_text = match.group().strip()
        if not segment_text:
            continue

        # Add to accumulator
        if accumulator:
            combined = accumulator + segment_text
        else:
            combined = segment_text

        # Check if we should flush the accumulator
        if max_length > 0 and len(combined) > max_length:
            # Combined segment is too long
            # Flush the accumulator (if not empty) as a separate segment
            if accumulator:
                segments.append(accumulator)
                if node:
                    send_log(
                        node,
                        "DEBUG",
                        f"Segmentation: Flushed segment at max_length: '{accumulator}' (len={len(accumulator)})",
                        log_level,
                    )
                accumulator = segment_text  # Start new accumulator with current segment
            else:
                # Current segment alone is longer than max_length
                # Send it anyway (can't split mid-sentence)
                segments.append(segment_text)
                if node:
                    send_log(
                        node,
                        "DEBUG",
                        f"Segmentation: Segment exceeds max_length: '{segment_text}' (len={len(segment_text)})",
                        log_level,
                    )
                accumulator = ""
        else:
            # Combined segment is within limit, keep accumulating
            accumulator = combined

        last_end = match.end()

    # Flush any remaining accumulator
    if accumulator:
        segments.append(accumulator)
        if node:
            send_log(
                node,
                "DEBUG",
                f"Segmentation: Final segment: '{accumulator}' (len={len(accumulator)})",
                log_level,
            )

    # Anything left over is incomplete (no ending punctuation)
    incomplete = text[last_end:].strip()

    if node and incomplete:
        send_log(node, "DEBUG", f"Segmentation: Incomplete text buffered: '{incomplete}'", log_level)

    return segments, incomplete, False

def main():
    node = Node("text-segmenter")

    # Configuration from environment
    punctuation_marks = os.getenv("PUNCTUATION_MARKS", "。！？.!?，,、；：""''（）【】《》")
    log_level = os.getenv("LOG_LEVEL", "INFO")
    segment_mode = os.getenv("SEGMENT_MODE", "sentence").lower()
    min_segment_length = max(1, parse_int_env("MIN_SEGMENT_LENGTH", 5))
    max_segment_length = parse_int_env("MAX_SEGMENT_LENGTH", 100)
    enable_backpressure = os.getenv("ENABLE_BACKPRESSURE", "true").lower() not in {"0", "false", "no"}
    remove_speaker_id_enabled = os.getenv("REMOVE_SPEAKER_ID", "false").lower() in {"1", "true", "yes"}

    fallback_split_marks = {"，", ",", "、", "；", ";", "：", ":"}

    if not punctuation_marks:
        punctuation_marks = "。！？.!?"

    if segment_mode == "punctuation":
        punctuation_marks = "".join(dict.fromkeys(punctuation_marks + "".join(fallback_split_marks)))

    send_log(node, "INFO", "Mode: single (queue-based)", log_level)
    send_log(
        node,
        "INFO",
        (
            "Configured — segment_mode: %s, min: %d, max: %s, punctuation: '%s', backpressure: %s, remove_speaker_id: %s"
            % (
                segment_mode,
                min_segment_length,
                "∞" if max_segment_length <= 0 else str(max_segment_length),
                punctuation_marks,
                str(enable_backpressure),
                str(remove_speaker_id_enabled),
            )
        ),
        log_level,
    )

    # Simple queue for segments
    segment_queue = deque()
    is_sending = False

    # Removed segment counter - no longer needed

    # Track current question_id for smart reset
    current_question_id = None

    # Text buffer for incomplete segments (accumulates across LLM chunks)
    text_buffer = ""

    # Track pending session_ended signal (when it arrives while is_sending=True)
    pending_session_end = False
    pending_session_end_metadata = {}

    send_log(node, "INFO", "Text Segmenter started with punctuation-based segmentation", log_level)
    
    for event in node:
        if event["type"] == "INPUT":
            if event["id"] == "text":
                # Received text from LLM
                text = event["value"][0].as_py()
                metadata = event.get("metadata", {})

                send_log(node, "INFO", f"🔵 RAW LLM INPUT: '{text}' (len={len(text)})", log_level)

                # Check for session_status: "ended" signal
                session_status = metadata.get("session_status", "")
                if session_status == "ended":
                    send_log(node, "INFO", f"🏁 SESSION ENDED signal received", log_level)

                    # If there's buffered text, flush it with "ended" status
                    if text_buffer.strip():
                        send_log(node, "INFO", f"🏁 Flushing buffer on session end: '{text_buffer}'", log_level)
                        segment_queue.append({
                            "text": text_buffer.strip(),
                            "metadata": {**metadata, "session_status": "ended"},
                        })
                        text_buffer = ""

                    # If queue has items, mark the last one as "ended"
                    if segment_queue:
                        segment_queue[-1]["metadata"]["session_status"] = "ended"
                        send_log(node, "INFO", f"🏁 Marked last queued segment as ended", log_level)

                    # If currently sending, the TTS will get the ended status from the queue
                    # If not sending and queue has items, send now
                    if not is_sending and segment_queue:
                        segment = segment_queue.popleft()
                        send_log(node, "INFO", f"🏁 Sending final segment: '{segment['text']}' with session_status=ended", log_level)
                        node.send_output(
                            "text_segment",
                            pa.array([segment["text"]]),
                            metadata=segment["metadata"]
                        )
                        is_sending = True
                    elif is_sending and not segment_queue:
                        # TTS is busy and no queued segments - remember to send session_ended later
                        pending_session_end = True
                        pending_session_end_metadata = metadata.copy()
                        send_log(node, "INFO", f"🏁 TTS busy, queuing session_ended for later", log_level)

                    # Skip normal text processing for empty "ended" message
                    if not text.strip():
                        continue

                # Remove speaker ID if enabled
                if remove_speaker_id_enabled:
                    original_text = text
                    text = remove_speaker_id(text, node, log_level)
                    if original_text != text:
                        send_log(node, "INFO", f"🔵 AFTER SPEAKER REMOVAL: '{text}' (len={len(text)})", log_level)

                # Extract question_id from metadata (passed from ASR via LLM)
                question_id = metadata.get("question_id", None)

                # Update current question_id
                if question_id is not None:
                    current_question_id = question_id

                # Combine with buffered text from previous chunk
                combined_text = text_buffer + text

                if text_buffer:
                    send_log(node, "DEBUG", f"Combined buffered '{text_buffer}' + new '{text}' = '{combined_text}'", log_level)

                # Segment the combined text by punctuation
                send_log(node, "INFO", f"🟡 COMBINED TEXT (buffer + new): '{combined_text}' (len={len(combined_text)})", log_level)

                complete_segments, incomplete_text, keep_incomplete = segment_by_punctuation(
                    combined_text,
                    punctuation_marks,
                    max_segment_length,
                    min_segment_length,
                    fallback_split_marks,
                    node,
                    log_level,
                )

                send_log(node, "INFO", f"🟢 SEGMENTATION OUTPUT: {len(complete_segments)} segments, incomplete: '{incomplete_text}' (len={len(incomplete_text)})", log_level)
                for i, seg in enumerate(complete_segments):
                    send_log(node, "INFO", f"🟢   Segment {i}: '{seg}' (len={len(seg)})", log_level)

                # Handle standalone punctuation in buffer
                # If incomplete_text is ONLY punctuation/whitespace, don't buffer it
                # (This happens when LLM sends standalone punctuation after a complete segment)
                if incomplete_text:
                    if not keep_incomplete and should_skip_segment(incomplete_text, punctuation_marks, node, log_level):
                        send_log(node, "DEBUG", f"Discarding standalone punctuation buffer: '{incomplete_text}'", log_level)
                        text_buffer = ""
                    else:
                        text_buffer = incomplete_text
                else:
                    text_buffer = ""

                # Queue all complete segments
                for segment_text in complete_segments:
                    # Check if we should skip this segment (punctuation-only filter)
                    if not should_skip_segment(segment_text, punctuation_marks, node, log_level):
                        # Valid segment - metadata already contains question_id
                        segment_queue.append({
                            "text": segment_text,
                            "metadata": metadata,
                        })

                        send_log(node, "DEBUG", f"Queued segment: '{segment_text}' (total: {len(segment_queue)})", log_level)
                    else:
                        send_log(node, "DEBUG", f"Skipped punctuation-only segment: '{segment_text}'", log_level)

                # Try to send a segment if not currently sending
                # This happens whether we queued segments or not
                # Ensures no deadlock even if first segments are all punctuation
                if not is_sending and segment_queue:
                    segment = segment_queue.popleft()

                    send_log(node, "INFO", f"Sending first to TTS: '{segment['text']}' (len={len(segment['text'])})", log_level)

                    # Send segment to TTS with metadata
                    node.send_output(
                        "text_segment",
                        pa.array([segment["text"]]),
                        metadata={
                            **segment["metadata"]  # Just pass through original metadata
                        }
                    )

                    send_log(node, "DEBUG", "First segment sent, setting is_sending=True", log_level)
                    is_sending = True
                    
            elif event["id"] == "tts_complete":
                # TTS completed a segment

                # Send next segment if available
                if segment_queue:
                    segment = segment_queue.popleft()

                    send_log(node, "INFO", f"Sending to TTS: '{segment['text']}' (len={len(segment['text'])})", log_level)

                    node.send_output(
                        "text_segment",
                        pa.array([segment["text"]]),
                        metadata={
                            **segment["metadata"]  # Just pass through original metadata
                        }
                    )
                    send_log(node, "DEBUG", "send_output() completed, setting is_sending=True", log_level)
                else:
                    # No more segments to send
                    is_sending = False

                    # Check if we have a pending session_ended signal to propagate
                    if pending_session_end:
                        send_log(node, "INFO", f"🏁 TTS done, sending pending session_ended signal", log_level)
                        # Send empty segment with session_status="ended" to signal completion
                        node.send_output(
                            "text_segment",
                            pa.array([""]),
                            metadata={**pending_session_end_metadata, "session_status": "ended"}
                        )
                        pending_session_end = False
                        pending_session_end_metadata = {}
                    
            elif event["id"] == "control":
                # Reset command
                command = event["value"][0].as_py()
                if command == "reset":
                    cleared_segments = len(segment_queue)
                    cleared_buffer = len(text_buffer) > 0
                    segment_queue.clear()
                    text_buffer = ""
                    is_sending = False
                    pending_session_end = False
                    pending_session_end_metadata = {}
                    # segment_counter removed
                    send_log(node, "INFO", f"Reset: Cleared {cleared_segments} queued segments and text buffer (buffer had text: {cleared_buffer})", log_level)

            elif event["id"] == "reset":
                # Reset signal - clear only segments from OLD questions (different question_id)
                # Ignore "resume" commands - only process actual reset commands
                command = event["value"][0].as_py() if event.get("value") else None

                # Ignore "resume" commands - these are for bridges, not segmenter
                if command == "resume":
                    send_log(node, "DEBUG", f"Ignoring 'resume' command on reset input", log_level)
                    continue

                metadata = event.get("metadata", {})
                incoming_question_id = metadata.get("question_id", None)

                if incoming_question_id is None:
                    # No question_id in reset signal - clear all (backward compatibility)
                    cleared_count = len(segment_queue)
                    cleared_buffer = len(text_buffer) > 0
                    segment_queue.clear()
                    text_buffer = ""
                    is_sending = False
                    pending_session_end = False
                    pending_session_end_metadata = {}
                    # segment_counter removed
                    send_log(node, "INFO", f"Reset: Cleared {cleared_count} queued segments and text buffer (no question_id)", log_level)
                else:
                    # Smart reset - only clear segments from different question_id
                    original_count = len(segment_queue)
                    new_queue = deque()
                    cleared_count = 0

                    for segment in segment_queue:
                        seg_question_id = segment["metadata"].get("question_id", None)

                        # Keep segment if:
                        # 1. It has the same question_id as the incoming reset, OR
                        # 2. It has no question_id (assume it's new content)
                        if seg_question_id == incoming_question_id or seg_question_id is None:
                            # Keep this segment
                            new_queue.append(segment)
                        else:
                            # This segment is from a different (old) question - discard it
                            cleared_count += 1

                    segment_queue = new_queue
                    # segment_counter removed

                    # Clear text buffer ONLY when question_id changes
                    # Keep buffer for same question_id to avoid losing incomplete text
                    buffer_was_cleared = False
                    if current_question_id != incoming_question_id:
                        buffer_was_cleared = len(text_buffer) > 0
                        text_buffer = ""
                        # Also clear pending session_ended from old question
                        pending_session_end = False
                        pending_session_end_metadata = {}

                    # Update current_question_id to the new question
                    current_question_id = incoming_question_id

                    # Reset is_sending if queue is empty
                    if len(segment_queue) == 0:
                        is_sending = False

                    send_log(node, "INFO", f"Smart reset: Cleared {cleared_count}/{original_count} old segments, kept {len(segment_queue)} from new question_id={incoming_question_id}, cleared buffer: {buffer_was_cleared}", log_level)

        elif event["type"] == "STOP":
            break

if __name__ == "__main__":
    main()
