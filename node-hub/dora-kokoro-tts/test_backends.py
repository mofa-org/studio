#!/usr/bin/env python3
"""
Test backend switching in dora-kokoro-tts.
"""

import os
import sys

# Test backend detection
print("Testing Backend Detection")
print("=" * 60)

# Test 1: Auto detection (should select MLX on macOS)
os.environ["BACKEND"] = "auto"
from dora_kokoro_tts.main import detect_backend
backend = detect_backend()
print(f"✅ Auto-detected backend: {backend}")

# Test 2: Force MLX
try:
    os.environ["BACKEND"] = "mlx"
    from dora_kokoro_tts.main import create_backend
    mlx_backend = create_backend("mlx")
    print(f"✅ MLX backend created: {mlx_backend.backend_name}")
except Exception as e:
    print(f"❌ MLX backend failed: {e}")

# Test 3: Force CPU
try:
    cpu_backend = create_backend("cpu")
    print(f"✅ CPU backend created: {cpu_backend.backend_name}")
except Exception as e:
    print(f"❌ CPU backend failed: {e}")

print("\n" + "=" * 60)
print("Backend Switching Test Complete")
print("=" * 60)
