#!/usr/bin/env python3
"""
Batch synthesis script for Kokoro TTS (standalone, no Dora required).

This script provides a simple command-line interface for synthesizing
text to speech using the Kokoro-82M model, without requiring a Dora dataflow.

Usage:
    python batch_synthesize.py --text "Hello world" --output output.wav --voice af_heart
"""

import argparse
import sys
import os
import tempfile
import contextlib
import io


def detect_backend(backend_choice):
    """Detect which backend to use."""
    if backend_choice == "cpu":
        try:
            import kokoro
            return "cpu"
        except ImportError:
            raise RuntimeError("CPU backend not available. Install 'kokoro' package")

    elif backend_choice == "mlx":
        try:
            import mlx_audio
            return "mlx"
        except ImportError:
            raise RuntimeError("MLX backend not available. Install 'mlx-audio' package")

    elif backend_choice == "auto":
        # Try MLX first (macOS)
        if sys.platform == "darwin":
            try:
                import mlx_audio
                return "mlx"
            except ImportError:
                pass

        # Fall back to CPU
        try:
            import kokoro
            return "cpu"
        except ImportError:
            raise RuntimeError("No Kokoro backend available. Install 'kokoro' or 'mlx-audio'")

    else:
        raise ValueError(f"Invalid backend: {backend_choice}")


def synthesize_mlx(text, voice, speed, lang_code, output_path):
    """Synthesize using MLX backend."""
    from mlx_audio.tts.generate import generate_audio

    # Create temp directory for output
    temp_dir = tempfile.mkdtemp()
    file_prefix = os.path.join(temp_dir, "mlx_output")

    try:
        # Suppress MLX-audio verbose output
        stdout_capture = io.StringIO()
        stderr_capture = io.StringIO()

        with contextlib.redirect_stdout(stdout_capture), contextlib.redirect_stderr(stderr_capture):
            # Generate audio
            generate_audio(
                text=text,
                model_path="prince-canuma/Kokoro-82M",  # MLX model
                voice=voice,
                speed=speed,
                lang_code=lang_code,
                audio_format="wav",
                file_prefix=file_prefix,
                join_audio=True,
                verbose=False,
            )

        # Find the generated file
        output_file = f"{file_prefix}.wav"
        if not os.path.exists(output_file):
            output_file_alt = f"{file_prefix}_000.wav"
            if os.path.exists(output_file_alt):
                output_file = output_file_alt
            else:
                created_files = os.listdir(temp_dir) if os.path.exists(temp_dir) else []
                raise RuntimeError(f"MLX did not create expected output file. Created: {created_files}")

        # Copy to final output path
        import shutil
        shutil.move(output_file, output_path)

        print(f"Successfully generated audio: {output_path}", file=sys.stderr)

    finally:
        # Cleanup temp directory
        import shutil
        if os.path.exists(temp_dir):
            shutil.rmtree(temp_dir)


def synthesize_cpu(text, voice, speed, lang_code, output_path):
    """Synthesize using CPU backend."""
    import kokoro

    # Generate audio
    audio_array, sample_rate = kokoro.generate(
        text=text,
        voice=voice,
        speed=speed,
        lang_code=lang_code,
    )

    # Save to file
    import soundfile as sf
    sf.write(output_path, audio_array, sample_rate)

    print(f"Successfully generated audio: {output_path}", file=sys.stderr)


def main():
    parser = argparse.ArgumentParser(
        description="Batch text-to-speech synthesis using Kokoro-82M",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )

    parser.add_argument("--text", required=True, help="Text to synthesize")
    parser.add_argument("--output", required=True, help="Output WAV file path")
    parser.add_argument("--voice", default="af_heart", help="Voice name (default: af_heart)")
    parser.add_argument("--language", default="en", help="Language code (en/zh/ja/ko)")
    parser.add_argument("--speed", type=float, default=1.0, help="Speed factor (0.5-2.0)")
    parser.add_argument("--backend", default="auto", choices=["auto", "mlx", "cpu"],
                       help="Backend to use (default: auto)")

    args = parser.parse_args()

    # Detect and use backend
    backend = detect_backend(args.backend)

    # Map language to lang_code
    # American English (a), British English (b), Chinese (z), Japanese (j), Korean (k)
    lang_map = {
        "en": "a",  # Default to American English
        "zh": "z",
        "ja": "j",
        "ko": "k",
    }
    lang_code = lang_map.get(args.language, "a")

    print(f"[INFO] Using {backend.upper()} backend for Kokoro TTS", file=sys.stderr)
    print(f"[INFO] Text: {args.text[:100]}{'...' if len(args.text) > 100 else ''}", file=sys.stderr)
    print(f"[INFO] Voice: {args.voice}, Language: {args.language} ({lang_code}), Speed: {args.speed}", file=sys.stderr)

    try:
        if backend == "mlx":
            synthesize_mlx(args.text, args.voice, args.speed, lang_code, args.output)
        elif backend == "cpu":
            synthesize_cpu(args.text, args.voice, args.speed, lang_code, args.output)

        return 0

    except Exception as e:
        print(f"[ERROR] Synthesis failed: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
