#!/usr/bin/env python3
"""
Benchmark script to compare FunASR performance with and without GPU acceleration.
Tests both the original and GPU-enhanced implementations.
"""

import os
import sys
import time
import json
import argparse
import numpy as np
from pathlib import Path
from typing import Dict, List
import librosa

# Ensure dora-asr is in path
sys.path.insert(0, str(Path(__file__).parent))

# Configure environment
os.environ['ASR_ENGINE'] = 'funasr'
os.environ['ASR_MODELS_DIR'] = str(Path.home() / ".dora" / "models" / "asr")


def benchmark_original_funasr(audio_path: str, use_gpu: bool = False) -> Dict:
    """Benchmark the original FunASR implementation"""
    print("\n" + "="*60)
    print("Benchmarking Original FunASR Implementation")
    print("="*60)
    
    os.environ['USE_GPU'] = str(use_gpu).lower()
    
    from dora_asr.engines.funasr import FunASREngine
    
    # Load audio
    audio_data, sr = librosa.load(audio_path, sr=16000)
    audio_duration = len(audio_data) / sr
    
    # Initialize engine
    print("Initializing engine...")
    init_start = time.time()
    engine = FunASREngine()
    engine.setup()
    init_time = time.time() - init_start
    
    # Warmup
    print("Warming up...")
    warmup_start = time.time()
    engine.warmup()
    warmup_time = time.time() - warmup_start
    
    # Run transcription multiple times
    print("Running transcriptions...")
    transcription_times = []
    results = []
    
    for i in range(3):
        start = time.time()
        result = engine.transcribe(audio_data, language='zh')
        elapsed = time.time() - start
        transcription_times.append(elapsed)
        results.append(result)
        print(f"  Run {i+1}: {elapsed:.3f}s")
    
    avg_time = np.mean(transcription_times)
    std_time = np.std(transcription_times)
    rtf = avg_time / audio_duration
    
    return {
        'implementation': 'Original FunASR',
        'backend': 'ONNX',
        'device': 'GPU (ONNX)' if use_gpu else 'CPU',
        'init_time': init_time,
        'warmup_time': warmup_time,
        'audio_duration': audio_duration,
        'avg_time': avg_time,
        'std_time': std_time,
        'rtf': rtf,
        'speed_x': 1.0 / rtf,
        'transcription': results[0]['text'] if results else '',
        'all_times': transcription_times
    }


def benchmark_gpu_enhanced_funasr(audio_path: str, use_gpu: bool = True) -> Dict:
    """Benchmark the GPU-enhanced FunASR implementation"""
    print("\n" + "="*60)
    print(f"Benchmarking GPU-Enhanced FunASR ({'GPU' if use_gpu else 'CPU'})")
    print("="*60)
    
    # Set environment variable BEFORE importing (important for config)
    os.environ['USE_GPU'] = 'true' if use_gpu else 'false'
    
    # Force reload of modules to pick up new environment variable
    import importlib
    import sys
    if 'dora_asr.config' in sys.modules:
        importlib.reload(sys.modules['dora_asr.config'])
    if 'dora_asr.engines.funasr_gpu' in sys.modules:
        importlib.reload(sys.modules['dora_asr.engines.funasr_gpu'])
    
    try:
        from dora_asr.engines.funasr_gpu import FunASRGPUEngine
    except ImportError:
        print("GPU-enhanced engine not available")
        return None
    
    # Load audio
    audio_data, sr = librosa.load(audio_path, sr=16000)
    audio_duration = len(audio_data) / sr
    
    # Initialize engine
    print("Initializing engine...")
    init_start = time.time()
    engine = FunASRGPUEngine()
    engine.setup()
    init_time = time.time() - init_start
    
    # Get backend info
    backend = 'PyTorch' if engine.use_pytorch else 'ONNX'
    device = engine.device
    
    # Warmup
    print("Warming up...")
    warmup_start = time.time()
    engine.warmup()
    warmup_time = time.time() - warmup_start
    
    # Run transcription multiple times
    print("Running transcriptions...")
    transcription_times = []
    results = []
    metrics_list = []
    
    for i in range(3):
        start = time.time()
        result = engine.transcribe(audio_data, language='zh')
        elapsed = time.time() - start
        transcription_times.append(elapsed)
        results.append(result)
        metrics_list.append(engine.get_metrics())
        print(f"  Run {i+1}: {elapsed:.3f}s (RTF: {engine.get_metrics()['rtf']:.3f})")
    
    avg_time = np.mean(transcription_times)
    std_time = np.std(transcription_times)
    rtf = avg_time / audio_duration
    
    # Get GPU memory if available
    gpu_memory = None
    if metrics_list and 'gpu_memory_gb' in metrics_list[0]:
        gpu_memory = metrics_list[0]['gpu_memory_gb']
    
    return {
        'implementation': 'GPU-Enhanced FunASR',
        'backend': backend,
        'device': device,
        'init_time': init_time,
        'warmup_time': warmup_time,
        'audio_duration': audio_duration,
        'avg_time': avg_time,
        'std_time': std_time,
        'rtf': rtf,
        'speed_x': 1.0 / rtf,
        'gpu_memory_gb': gpu_memory,
        'transcription': results[0]['text'] if results else '',
        'all_times': transcription_times
    }


def print_results(results: List[Dict]):
    """Print benchmark results in a formatted table"""
    print("\n" + "="*80)
    print("BENCHMARK RESULTS")
    print("="*80)
    
    # Header
    print(f"{'Implementation':<25} {'Backend':<10} {'Device':<10} {'Avg Time':<10} {'RTF':<8} {'Speed':<10} {'GPU Mem'}")
    print("-"*80)
    
    # Results
    for r in results:
        if r is None:
            continue
        gpu_mem = f"{r.get('gpu_memory_gb', 0):.2f}GB" if r.get('gpu_memory_gb') else "N/A"
        print(f"{r['implementation']:<25} {r['backend']:<10} {r['device']:<10} "
              f"{r['avg_time']:.3f}s     {r['rtf']:.3f}    {r['speed_x']:.1f}x      {gpu_mem}")
    
    print("\n" + "="*80)
    print("PERFORMANCE COMPARISON")
    print("="*80)
    
    if len(results) >= 2 and results[0] and results[1]:
        baseline = results[0]
        for i, r in enumerate(results[1:], 1):
            if r is None:
                continue
            speedup = baseline['avg_time'] / r['avg_time']
            rtf_improvement = baseline['rtf'] / r['rtf']
            
            print(f"\n{r['implementation']} vs {baseline['implementation']}:")
            print(f"  • Processing time: {r['avg_time']:.3f}s vs {baseline['avg_time']:.3f}s")
            print(f"  • Speedup: {speedup:.2f}x faster")
            print(f"  • RTF improvement: {rtf_improvement:.2f}x better")
            print(f"  • Real-time speed: {r['speed_x']:.1f}x vs {baseline['speed_x']:.1f}x")
            
            if r.get('gpu_memory_gb'):
                print(f"  • GPU memory used: {r['gpu_memory_gb']:.2f} GB")
    
    # Transcription quality check
    print("\n" + "="*80)
    print("TRANSCRIPTION QUALITY")
    print("="*80)
    
    transcriptions = [r['transcription'] for r in results if r and r.get('transcription')]
    if transcriptions:
        print(f"\nTranscription: {transcriptions[0][:100]}...")
        
        # Check if all transcriptions match
        if all(t == transcriptions[0] for t in transcriptions):
            print("✅ All implementations produced identical transcriptions")
        else:
            print("⚠️  Transcriptions differ between implementations")
            for i, r in enumerate(results):
                if r and r.get('transcription'):
                    print(f"\n{r['implementation']}: {r['transcription'][:100]}...")


def main():
    parser = argparse.ArgumentParser(description="Benchmark FunASR GPU acceleration")
    parser.add_argument("--audio", type=str, default="asr.wav", help="Path to audio file")
    parser.add_argument("--skip-original", action="store_true", help="Skip original implementation")
    parser.add_argument("--cpu-only", action="store_true", help="Test CPU performance only")
    args = parser.parse_args()
    
    # Check audio file
    audio_path = Path(args.audio)
    if not audio_path.exists():
        print(f"Audio file not found: {audio_path}")
        sys.exit(1)
    
    print(f"Audio file: {audio_path}")
    
    # Check GPU availability
    try:
        import torch
        has_gpu = torch.cuda.is_available()
        if has_gpu:
            print(f"GPU detected: {torch.cuda.get_device_name(0)}")
        else:
            print("No GPU detected, will test CPU only")
            args.cpu_only = True
    except ImportError:
        print("PyTorch not available, cannot check GPU")
        args.cpu_only = True
    
    results = []
    
    # Benchmark original implementation
    if not args.skip_original:
        try:
            result = benchmark_original_funasr(str(audio_path), use_gpu=False)
            results.append(result)
        except Exception as e:
            print(f"Original implementation failed: {e}")
            results.append(None)
    
    # Benchmark GPU-enhanced implementation (CPU)
    try:
        result = benchmark_gpu_enhanced_funasr(str(audio_path), use_gpu=False)
        results.append(result)
    except Exception as e:
        print(f"GPU-enhanced (CPU) failed: {e}")
        results.append(None)
    
    # Benchmark GPU-enhanced implementation (GPU)
    if not args.cpu_only:
        try:
            result = benchmark_gpu_enhanced_funasr(str(audio_path), use_gpu=True)
            results.append(result)
        except Exception as e:
            print(f"GPU-enhanced (GPU) failed: {e}")
            results.append(None)
    
    # Print results
    print_results(results)
    
    # Save results to JSON
    output_file = "benchmark_results.json"
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to {output_file}")


if __name__ == "__main__":
    main()