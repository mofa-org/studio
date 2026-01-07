# GPU Control Guide for dora-asr

## Environment Variable Control

The dora-asr node supports GPU acceleration through the `USE_GPU` environment variable.

### How It Works

1. **`USE_GPU=false`** (default)
   - Uses the original `FunASREngine` (CPU-only, ONNX backend)
   - Fastest CPU performance
   - No GPU memory usage

2. **`USE_GPU=true`**
   - Uses the enhanced `FunASRGPUEngine`
   - Automatically detects and uses CUDA if available
   - Falls back to CPU if no GPU is found
   - Supports both PyTorch and ONNX backends

### Configuration in YAML

```yaml
nodes:
  - id: asr
    operator:
      python: ../../node-hub/dora-asr
    inputs:
      audio: audio-source/audio
    outputs:
      - text
    env:
      USE_GPU: "true"              # Enable GPU acceleration
      ASR_ENGINE: "funasr"         # Use FunASR engine
      LANGUAGE: "zh"               # Chinese language
```

### Command Line Usage

```bash
# Run with GPU enabled
USE_GPU=true dora start dataflow.yml

# Run with GPU disabled (CPU only)
USE_GPU=false dora start dataflow.yml

# Or set globally
export USE_GPU=true
dora start dataflow.yml
```

### Validation Tests

Test that GPU control is working:

```bash
# Test CPU mode
USE_GPU=false python -c "
from dora_asr.manager import ASRManager
m = ASRManager()
print(f'CPU mode: {m._engine_classes[\"funasr\"].__name__}')
"

# Test GPU mode
USE_GPU=true python -c "
from dora_asr.manager import ASRManager
m = ASRManager()
print(f'GPU mode: {m._engine_classes[\"funasr\"].__name__}')
"
```

Expected output:
- CPU mode: `FunASREngine`
- GPU mode: `FunASRGPUEngine`

### Performance Comparison

Based on benchmarks with 17.35s Chinese audio on RTX 4090:

| Mode | Engine | Device | Processing Time | Real-time Speed |
|------|--------|--------|-----------------|-----------------|
| `USE_GPU=false` | FunASREngine | CPU | 0.640s | 27.1x |
| `USE_GPU=true` | FunASRGPUEngine | CUDA | 0.282s | 61.6x |

**Result**: 2.27x faster with GPU enabled

### Important Notes

1. **Accepted Values**: The environment variable accepts:
   - `"true"`, `"True"`, `"TRUE"` → GPU enabled
   - `"false"`, `"False"`, `"FALSE"` → GPU disabled
   - Any other value (including `"1"`, `"yes"`) → GPU disabled

2. **Module Reloading**: The configuration is read when modules are first imported. If testing in the same Python process, you need to restart the process or properly reload all modules.

3. **Automatic Fallback**: If `USE_GPU=true` but no GPU is available, the engine automatically falls back to CPU mode.

4. **Memory Usage**: GPU mode uses approximately 2GB of VRAM for the FunASR models.

### Troubleshooting

If GPU is not being used when expected:

1. Check CUDA availability:
```python
import torch
print(torch.cuda.is_available())  # Should be True
print(torch.cuda.get_device_name(0))  # Should show your GPU
```

2. Verify environment variable:
```bash
echo $USE_GPU  # Should show "true"
```

3. Check which engine is loaded:
```python
from dora_asr.manager import ASRManager
manager = ASRManager()
print(manager._engine_classes['funasr'].__name__)
# Should show "FunASRGPUEngine" when USE_GPU=true
```

4. Check actual device after transcription:
```python
# After running transcription
if 'funasr' in manager._engines:
    engine = manager._engines['funasr']
    print(f"Device: {engine.device}")  # Should show "cuda" or "cpu"
```