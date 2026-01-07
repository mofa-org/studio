# dora-kokoro-tts

Fast, multi-language text-to-speech node for Dora using the Kokoro-82M model.

Supports **multiple backends**:
- **CPU**: Cross-platform PyTorch backend (4.1x real-time)
- **MLX**: Apple Silicon Metal GPU backend (6.6x real-time)
- **Auto**: Automatically selects the best available backend

## Features

- 🚀 **High Performance**: 4-7x faster than real-time
- 🎯 **Multiple Backends**: CPU (PyTorch) or MLX (Metal GPU)
- 🌍 **Multi-Language**: English, Chinese, Japanese, Korean
- 🎤 **Multiple Voices**: 100+ voices from Kokoro-82M
- 🔄 **Auto Backend Selection**: Automatically uses MLX on Apple Silicon
- 📊 **Performance Tracking**: Built-in statistics and RTF monitoring
- 🎛️ **PrimeSpeech Compatible**: Same interface as dora-primespeech

## Installation

### Basic Installation (CPU only)

```bash
pip install -e .
```

### With MLX Support (Apple Silicon)

```bash
pip install -e ".[mlx]"
```

This installs the `mlx-audio` package for Metal GPU acceleration.

## Environment Variables

| Variable | Values | Default | Description |
|----------|--------|---------|-------------|
| `BACKEND` | `auto`, `mlx`, `cpu` | `auto` | Which backend to use |
| `LANGUAGE` | `en`, `zh`, `ja`, `ko` | `en` | Language code |
| `VOICE` | Voice name | `af_heart` | Voice to use |
| `SPEED` | Float (0.5-2.0) | `1.0` | Speech speed |
| `LOG_LEVEL` | `DEBUG`, `INFO`, `WARNING`, `ERROR` | `INFO` | Logging level |

### Backend Selection

- **`auto`** (default): Automatically detects and uses MLX if available on macOS, otherwise falls back to CPU
- **`mlx`**: Force MLX Metal GPU backend (requires Apple Silicon and `mlx-audio`)
- **`cpu`**: Force CPU PyTorch backend (cross-platform)

## YAML Specification

### Example: Auto Backend (Recommended)

```yaml
nodes:
  - id: kokoro-tts
    operator:
      python: ../../node-hub/dora-kokoro-tts
    inputs:
      text: text-segmenter/text
      control: _/control
    outputs:
      - audio
      - segment_complete
      - log
    env:
      BACKEND: "auto"          # Auto-select best backend
      LANGUAGE: "en"
      VOICE: "af_heart"
      SPEED: "1.0"
      LOG_LEVEL: "INFO"
```

### Example: Force MLX Backend (Apple Silicon)

```yaml
nodes:
  - id: kokoro-tts-mlx
    operator:
      python: ../../node-hub/dora-kokoro-tts
    inputs:
      text: text-segmenter/text
    outputs:
      - audio
      - segment_complete
      - log
    env:
      BACKEND: "mlx"           # Force MLX GPU
      LANGUAGE: "en"
      VOICE: "af_heart"
```

### Example: Force CPU Backend (Cross-platform)

```yaml
nodes:
  - id: kokoro-tts-cpu
    operator:
      python: ../../node-hub/dora-kokoro-tts
    inputs:
      text: text-segmenter/text
    outputs:
      - audio
      - segment_complete
      - log
    env:
      BACKEND: "cpu"           # Force CPU
      LANGUAGE: "zh"
      VOICE: "af_heart"
```

## Performance Comparison

Tested on Apple Silicon Mac (15.2s English audio):

| Backend | Processing Time | RTF | Speed vs Real-Time | vs PrimeSpeech |
|---------|-----------------|-----|-------------------|----------------|
| **MLX (GPU)** | 2.32s | 0.15x | **6.6x** | **8.7x faster** |
| **CPU** | 3.70s | 0.24x | **4.1x** | **5.4x faster** |
| PrimeSpeech | 41.84s | 1.31x | 0.76x | Baseline |

**MLX provides 1.6x speedup over CPU on the same model.**

## Available Voices

Kokoro-82M supports 100+ voices across multiple languages:

### American English (lang_code="a")
- `af_heart`, `af_sky`, `af_bella`, `af_sarah`, `af_nicole`
- `am_adam`, `am_michael`, `am_leo`, `am_eric`

### British English (lang_code="b")
- `bf_alice`, `bf_lily`, `bf_isabella`, `bf_emma`
- `bm_george`, `bm_lewis`, `bm_daniel`

See [Kokoro documentation](https://github.com/hexgrad/Kokoro-82M) for full voice list.

## Usage Examples

### Basic Usage

```python
# Text input with metadata
text = "Hello, this is a test."
metadata = {
    "segment_index": 0,
    "segments_remaining": 0,
    "question_id": "test-123"
}

# Node will:
# 1. Auto-detect backend (MLX on Apple Silicon, CPU otherwise)
# 2. Synthesize audio
# 3. Send audio output
# 4. Send segment_complete signal
```

### Control Commands

Send control commands to the `control` input:

```python
# Get statistics
node.send("control", "stats")

# Reset statistics
node.send("control", "reset")
```

### Output Metadata

The `audio` output includes metadata:

```python
{
    "segment_index": 0,
    "segments_remaining": 0,
    "question_id": "test-123",
    "sample_rate": 24000,
    "duration": 3.5,
    "is_streaming": False,
    "backend": "mlx"  # or "cpu"
}
```

## Language Support

| Language | Code | Notes |
|----------|------|-------|
| English (American) | `en`, `a` | Default |
| Chinese (Mandarin) | `zh`, `z` | Requires `misaki[zh]` |
| Japanese | `ja`, `j` | Built-in support |
| Korean | `ko`, `k` | Built-in support |

**Auto-detection**: The node automatically detects Chinese characters and switches to Chinese mode.

## Inputs

- **`text`**: Text to synthesize (string)
  - Metadata: `segment_index`, `segments_remaining`, `question_id`
  - Skips punctuation-only or whitespace-only text

- **`control`**: Control commands (string)
  - `"stats"`: Print statistics
  - `"reset"`: Reset statistics

## Outputs

- **`audio`**: Synthesized audio (float32 array)
  - Sample rate: 24kHz
  - Format: Mono, float32
  - Metadata includes duration, backend, etc.

- **`segment_complete`**: Completion signal (string)
  - Values: `"completed"`, `"skipped"`, `"error"`
  - Metadata includes `segment_index`, `question_id`

- **`log`**: Structured log messages (JSON string)
  - Fields: `node`, `level`, `message`, `timestamp`

## Backend Details

### CPU Backend (`kokoro` package)
- **Pros**:
  - Cross-platform (Linux, macOS, Windows)
  - No GPU required
  - Still 5.4x faster than PrimeSpeech
  - Reliable PyTorch implementation

- **Cons**:
  - Slower than MLX on Apple Silicon
  - Higher CPU usage

### MLX Backend (`mlx-audio` package)
- **Pros**:
  - 1.6x faster than CPU
  - 8.7x faster than PrimeSpeech
  - Efficient Metal GPU usage
  - Low latency

- **Cons**:
  - Apple Silicon only
  - Requires additional package

### Auto Backend (Recommended)
- Automatically selects MLX on macOS if available
- Falls back to CPU if MLX not installed
- Best user experience

## Development

### Format with ruff

```bash
uv run ruff check . --fix
```

### Lint with ruff

```bash
uv run ruff check .
```

### Test with pytest

```bash
uv run pytest .
```

## Troubleshooting

### MLX backend not available

```
RuntimeError: MLX backend not available: No module named 'mlx_audio'
```

**Solution**: Install MLX support:
```bash
pip install -e ".[mlx]"
```

### CPU backend not available

```
RuntimeError: CPU backend not available: No module named 'kokoro'
```

**Solution**: Reinstall the package:
```bash
pip install -e .
```

### Chinese text not working

**Solution**: Ensure `misaki[zh]` is installed (included in dependencies).

## Migration from dora-mlx-kokoro

If you were using the separate `dora-mlx-kokoro` node:

1. **Switch to `dora-kokoro-tts`** with `BACKEND=mlx`
2. **Same functionality**, unified codebase
3. **Automatic backend selection** with `BACKEND=auto`

Example migration:

```yaml
# Old (dora-mlx-kokoro)
- id: tts
  operator:
    python: ../../node-hub/dora-mlx-kokoro

# New (dora-kokoro-tts with MLX)
- id: tts
  operator:
    python: ../../node-hub/dora-kokoro-tts
  env:
    BACKEND: "mlx"  # or "auto" for automatic selection
```

## License

dora-kokoro-tts's code is released under the MIT License.

## Related Files

- **Performance comparison**: `/Users/yuechen/home/fresh/dora/examples/model-manager/KOKORO_RESULTS.md`
- **Test script**: `/Users/yuechen/home/fresh/dora/examples/model-manager/test_kokoro_comparison.py`
- **Audio samples**: `/Users/yuechen/home/fresh/dora/examples/model-manager/audio_samples/`
