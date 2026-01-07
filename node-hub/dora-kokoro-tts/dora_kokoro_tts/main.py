"""
Enhanced Dora Kokoro TTS Node with multi-backend support.
Fast, multi-language text-to-speech with CPU and MLX GPU backends.

Environment Variables:
- BACKEND: "auto" (default), "mlx", "cpu"
- LANGUAGE: Language code (en, zh, ja, ko)
- VOICE: Voice name (e.g., zf_xiaoxiao for Chinese female)
- SPEED_FACTOR: Speech speed (default: 1.0, matches PrimeSpeech naming)
- SPEED: Legacy speech speed parameter (deprecated, use SPEED_FACTOR)
- KOKORO_MODEL_CPU: CPU model path (default: "hexgrad/Kokoro-82M")
- KOKORO_MODEL_MLX: MLX model path (default: "prince-canuma/Kokoro-82M")
- LOG_LEVEL: Logging level (DEBUG, INFO, WARNING, ERROR)
"""

import os
import re
import sys
import time
import json
import traceback
import numpy as np
import pyarrow as pa
from dora import Node

# Environment configuration
BACKEND = os.getenv("BACKEND", "auto")  # auto, mlx, cpu
LANGUAGE = os.getenv("LANGUAGE", "en")
VOICE = os.getenv("VOICE", "af_heart")
# Support both SPEED_FACTOR (matches PrimeSpeech) and SPEED (backward compatibility)
SPEED = float(os.getenv("SPEED_FACTOR", os.getenv("SPEED", "1.0")))

# Model paths - expand environment variables like $HOME
KOKORO_MODEL_CPU = os.path.expandvars(
    os.path.expanduser(os.getenv("KOKORO_MODEL_CPU", "hexgrad/Kokoro-82M"))
)
KOKORO_MODEL_MLX = os.path.expandvars(
    os.path.expanduser(os.getenv("KOKORO_MODEL_MLX", "prince-canuma/Kokoro-82M"))
)

LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO")


def send_log(node, level, message, config_level="INFO"):
    """Send log message through log output channel."""
    LOG_LEVELS = {
        "DEBUG": 10,
        "INFO": 20,
        "WARNING": 30,
        "ERROR": 40
    }

    if LOG_LEVELS.get(level, 0) < LOG_LEVELS.get(config_level, 20):
        return

    formatted_message = f"[{level}] {message}"
    # Also print to console
    try:
        print(formatted_message, file=sys.stderr if level in {"ERROR", "WARNING"} else sys.stdout, flush=True)
    except Exception:
        pass

    # Only send to node output if node is available
    if node is not None:
        log_data = {
            "node": "kokoro-tts",
            "level": level,
            "message": formatted_message,
            "timestamp": time.time()
        }
        node.send_output("log", pa.array([json.dumps(log_data)]))


def detect_backend():
    """Auto-detect the best available backend."""
    # Try MLX first (if on macOS)
    if sys.platform == "darwin":
        try:
            import mlx_audio
            send_log(None, "INFO", "MLX backend available", LOG_LEVEL)
            return "mlx"
        except ImportError:
            pass

    # Fall back to CPU
    try:
        import kokoro
        send_log(None, "INFO", "CPU backend available", LOG_LEVEL)
        return "cpu"
    except ImportError:
        pass

    raise RuntimeError("No Kokoro backend available. Install 'kokoro' or 'mlx-audio'")


class KokoroMLXBackend:
    """MLX-accelerated Kokoro backend using mlx-audio."""

    def __init__(self, model_path=None):
        from mlx_audio.tts.generate import generate_audio

        self.generate_audio = generate_audio

        # Store the model path - can be HF repo ID or local path
        # generate_audio() will handle downloading/loading internally
        self.model_path = model_path or KOKORO_MODEL_MLX
        self.backend_name = "mlx"

    def synthesize(self, text, voice, speed, lang_code):
        """Synthesize audio using MLX backend."""
        import tempfile
        import soundfile as sf
        from scipy import signal
        import contextlib
        import io

        # Create temp directory for output
        temp_dir = tempfile.mkdtemp()
        file_prefix = os.path.join(temp_dir, "mlx_output")

        try:
            # Suppress MLX-audio verbose output by redirecting stdout/stderr
            stdout_capture = io.StringIO()
            stderr_capture = io.StringIO()

            with contextlib.redirect_stdout(stdout_capture), contextlib.redirect_stderr(stderr_capture):
                # Generate audio - use the resolved model path
                self.generate_audio(
                    text=text,
                    model_path=self.model_path,
                    voice=voice,
                    speed=speed,
                    lang_code=lang_code,
                    audio_format="wav",
                    file_prefix=file_prefix,
                    join_audio=True,
                    verbose=False,
                )

            # Read the generated audio - MLX might create either mlx_output.wav or mlx_output_000.wav
            output_file = f"{file_prefix}.wav"
            if not os.path.exists(output_file):
                # Try with _000 suffix (MLX sometimes appends this)
                output_file_alt = f"{file_prefix}_000.wav"
                if os.path.exists(output_file_alt):
                    output_file = output_file_alt
                else:
                    # List what files were created for debugging
                    created_files = os.listdir(temp_dir) if os.path.exists(temp_dir) else []
                    raise RuntimeError(
                        f"MLX failed to generate audio. Expected {output_file} or {output_file_alt}. "
                        f"Files in temp dir: {created_files}"
                    )

            audio_data, sample_rate = sf.read(output_file)

            # Cleanup
            os.remove(output_file)
            os.rmdir(temp_dir)

            # Resample from 24000 Hz to 32000 Hz to match PrimeSpeech
            # Kokoro outputs at 24000 Hz, but PrimeSpeech uses 32000 Hz
            TARGET_SAMPLE_RATE = 32000
            if sample_rate != TARGET_SAMPLE_RATE:
                # Use polyphase filtering for high-quality audio resampling
                # Ratio: 32000/24000 = 4/3
                audio_data = signal.resample_poly(audio_data, up=4, down=3)
                sample_rate = TARGET_SAMPLE_RATE

            return audio_data.astype(np.float32), sample_rate

        except Exception as e:
            # Cleanup on error
            try:
                if os.path.exists(temp_dir):
                    import shutil
                    shutil.rmtree(temp_dir)
            except:
                pass
            raise e


class KokoroCPUBackend:
    """CPU-based Kokoro backend using kokoro package."""

    def __init__(self, model_path=None):
        from kokoro import KPipeline
        self.KPipeline = KPipeline
        self.model_path = model_path or KOKORO_MODEL_CPU
        self.pipeline = None
        self.current_lang = None
        self.backend_name = "cpu"

    def synthesize(self, text, voice, speed, lang_code):
        """Synthesize audio using CPU backend."""
        from scipy import signal

        # Initialize or switch pipeline if language changed
        if self.pipeline is None or self.current_lang != lang_code:
            self.pipeline = self.KPipeline(lang_code=lang_code, repo_id=self.model_path)
            self.current_lang = lang_code

        # Generate audio
        generator = self.pipeline(
            text,
            voice=voice,
            speed=speed,
            split_pattern=r"\n+",
        )

        # Collect all audio chunks
        audio_chunks = []
        for _, (_, _, audio) in enumerate(generator):
            audio_np = audio.numpy()
            audio_chunks.append(audio_np)

        if not audio_chunks:
            raise RuntimeError("No audio generated from CPU backend")

        # Concatenate
        audio_data = np.concatenate(audio_chunks)
        sample_rate = 24000  # Kokoro default

        # Resample from 24000 Hz to 32000 Hz to match PrimeSpeech
        TARGET_SAMPLE_RATE = 32000
        if sample_rate != TARGET_SAMPLE_RATE:
            # Use polyphase filtering for high-quality audio resampling
            # Ratio: 32000/24000 = 4/3
            audio_data = signal.resample_poly(audio_data, up=4, down=3)
            sample_rate = TARGET_SAMPLE_RATE

        return audio_data.astype(np.float32), sample_rate


def create_backend(backend_type):
    """Create the appropriate backend based on type."""
    if backend_type == "mlx":
        try:
            return KokoroMLXBackend()
        except ImportError as e:
            raise RuntimeError(f"MLX backend not available: {e}. Install with: pip install 'dora-kokoro-tts[mlx]'")
    elif backend_type == "cpu":
        try:
            return KokoroCPUBackend()
        except ImportError as e:
            raise RuntimeError(f"CPU backend not available: {e}. Install with: pip install kokoro")
    else:
        raise ValueError(f"Unknown backend type: {backend_type}")


def map_language_to_code(language):
    """Map language names to Kokoro language codes."""
    lang_map = {
        "zh": "z", "ch": "z", "chinese": "z", "mandarin": "z",
        "ja": "j", "japanese": "j",
        "ko": "k", "korean": "k",
        "en": "a", "english": "a", "american": "a",
    }
    return lang_map.get(language.lower(), "a")  # Default to American English


def main():
    """Main entry point for Kokoro TTS node with multi-backend support."""

    node = Node()

    # Determine backend
    backend_type = BACKEND.lower()
    if backend_type == "auto":
        backend_type = detect_backend()
        send_log(node, "INFO", f"Auto-detected backend: {backend_type}", LOG_LEVEL)
    else:
        send_log(node, "INFO", f"Using configured backend: {backend_type}", LOG_LEVEL)

    # Create backend (lazy initialization)
    backend = None
    current_backend_type = backend_type

    send_log(node, "INFO", f"Kokoro TTS Node initialized (backend: {backend_type})", LOG_LEVEL)
    send_log(node, "INFO", f"Language: {LANGUAGE}, Voice: {VOICE}, Speed: {SPEED}", LOG_LEVEL)
    if backend_type == "mlx":
        send_log(node, "INFO", f"MLX Model: {KOKORO_MODEL_MLX}", LOG_LEVEL)
    elif backend_type == "cpu":
        send_log(node, "INFO", f"CPU Model: {KOKORO_MODEL_CPU}", LOG_LEVEL)
    send_log(node, "INFO", "Using lazy initialization - backend will load on first text", LOG_LEVEL)

    # Statistics
    total_syntheses = 0
    total_duration = 0
    total_processing_time = 0

    send_log(node, "INFO", "Entering event loop, waiting for events", LOG_LEVEL)

    for event in node:
        send_log(node, "DEBUG", f"Received event: type={event['type']}, id={event.get('id', 'N/A')}", LOG_LEVEL)

        if event["type"] == "INPUT":
            input_id = event["id"]

            if input_id == "text":
                # Get text to synthesize
                text = event["value"][0].as_py()
                metadata = event.get("metadata", {})

                # Extract metadata - use safe defaults for optional fields
                question_id = metadata.get("question_id", "default") if metadata else "default"
                session_status = metadata.get("session_status", "unknown") if metadata else "unknown"
                session_id = metadata.get("session_id", "unknown") if metadata else "unknown"

                send_log(node, "DEBUG", f"Received text: '{text}' (len={len(text)})", LOG_LEVEL)

                # Skip if text is only punctuation or whitespace
                text_stripped = text.strip()
                if not text_stripped or all(c in '。！？.!?,，、；：""''（）【】《》\n\r\t ' for c in text_stripped):
                    send_log(node, "DEBUG", f"Skipped - text is only punctuation/whitespace: '{text}'", LOG_LEVEL)
                    # Send segment_complete without audio
                    node.send_output(
                        "segment_complete",
                        pa.array(["skipped"]),
                        metadata={
                            "question_id": question_id,
                            "session_status": session_status,
                            "session_id": session_id
                        }
                    )
                    continue

                send_log(node, "INFO", f"Processing text (len={len(text)})", LOG_LEVEL)

                # Lazy initialize backend on first use
                if backend is None:
                    send_log(node, "INFO", f"Initializing {current_backend_type} backend...", LOG_LEVEL)
                    try:
                        backend = create_backend(current_backend_type)
                        send_log(node, "INFO", f"✅ {current_backend_type.upper()} backend initialized", LOG_LEVEL)
                    except Exception as e:
                        send_log(node, "ERROR", f"Failed to initialize backend: {e}", LOG_LEVEL)
                        node.send_output(
                            "segment_complete",
                            pa.array(["error"]),
                            metadata={
                                "question_id": question_id,
                                "session_status": "error",
                                "session_id": session_id,
                                "error": str(e),
                                "error_stage": "backend_init"
                            }
                        )
                        continue

                # Auto-detect language from text if needed
                lang_code = map_language_to_code(LANGUAGE)
                if re.findall(r'[\u4e00-\u9fff]+', text):
                    lang_code = "z"  # Chinese detected

                # Log synthesis parameters at DEBUG level
                send_log(node, "DEBUG",
                        f"Synthesis: text='{text[:50]}...' voice={VOICE} speed={SPEED} lang={lang_code}",
                        LOG_LEVEL)

                # Synthesize speech
                start_time = time.time()

                try:
                    # Generate audio using selected backend
                    audio_array, sample_rate = backend.synthesize(text, VOICE, SPEED, lang_code)

                    synthesis_time = time.time() - start_time
                    audio_duration = len(audio_array) / sample_rate

                    total_syntheses += 1
                    total_duration += audio_duration
                    total_processing_time += synthesis_time

                    rtf = synthesis_time / audio_duration if audio_duration > 0 else 0
                    send_log(node, "INFO",
                            f"Synthesized: {audio_duration:.2f}s audio in {synthesis_time:.3f}s "
                            f"(RTF: {rtf:.3f}x, backend: {backend.backend_name})",
                            LOG_LEVEL)

                    # Send audio output with metadata
                    node.send_output(
                        "audio",
                        pa.array([audio_array]),
                        metadata={
                            "question_id": question_id,
                            "session_status": session_status,
                            "session_id": session_id,
                            "sample_rate": sample_rate,
                            "duration": audio_duration,
                            "is_streaming": False,
                            "backend": backend.backend_name,
                        }
                    )

                    # Send segment completion signal
                    node.send_output(
                        "segment_complete",
                        pa.array(["completed"]),
                        metadata={
                            "question_id": question_id,
                            "session_status": session_status,
                            "session_id": session_id
                        }
                    )
                    send_log(node, "INFO", "Sent segment_complete", LOG_LEVEL)

                except Exception as e:
                    error_details = traceback.format_exc()
                    send_log(node, "ERROR", f"Synthesis error: {e}", LOG_LEVEL)
                    send_log(node, "ERROR", f"Traceback: {error_details}", LOG_LEVEL)

                    # Send segment completion with error status
                    node.send_output(
                        "segment_complete",
                        pa.array(["error"]),
                        metadata={
                            "question_id": question_id,
                            "session_status": "error",
                            "session_id": session_id,
                            "error": str(e),
                            "error_stage": "synthesis"
                        }
                    )
                    send_log(node, "ERROR", "Sent error segment_complete", LOG_LEVEL)

            elif input_id == "control":
                # Handle control commands
                command = event["value"][0].as_py()

                if command == "reset":
                    send_log(node, "INFO", "[KokoroTTS] RESET received", LOG_LEVEL)
                    # Reset statistics
                    total_syntheses = 0
                    total_duration = 0
                    total_processing_time = 0
                    send_log(node, "INFO", "[KokoroTTS] Reset acknowledged", LOG_LEVEL)

                elif command == "stats":
                    avg_rtf = total_processing_time / total_duration if total_duration > 0 else 0
                    backend_info = backend.backend_name if backend else "not initialized"
                    send_log(node, "INFO", f"Backend: {backend_info}", LOG_LEVEL)
                    send_log(node, "INFO", f"Total syntheses: {total_syntheses}", LOG_LEVEL)
                    send_log(node, "INFO", f"Total audio duration: {total_duration:.1f}s", LOG_LEVEL)
                    send_log(node, "INFO", f"Total processing time: {total_processing_time:.1f}s", LOG_LEVEL)
                    send_log(node, "INFO", f"Average RTF: {avg_rtf:.3f}x", LOG_LEVEL)
                    if total_syntheses > 0:
                        avg_duration = total_duration / total_syntheses
                        send_log(node, "INFO", f"Average audio duration: {avg_duration:.1f}s", LOG_LEVEL)

        elif event["type"] == "STOP":
            break

    send_log(node, "INFO", "Kokoro TTS node stopped", LOG_LEVEL)


if __name__ == "__main__":
    main()
