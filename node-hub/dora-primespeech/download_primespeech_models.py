#!/usr/bin/env python3
"""Download GPT-SoVITS pretrained models for PrimeSpeech TTS"""

import os
import sys
from pathlib import Path

# Try to import requests
try:
    import requests
except ImportError:
    print("Installing requests...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "requests"])
    import requests

def download_file(url: str, dest_path: Path):
    """Download a file from URL to destination path"""
    dest_path.parent.mkdir(parents=True, exist_ok=True)

    print(f"Downloading {url}")
    print(f"To: {dest_path}")

    response = requests.get(url, stream=True, timeout=120)
    response.raise_for_status()

    total_size = int(response.headers.get('content-length', 0))
    block_size = 8192
    downloaded = 0

    with open(dest_path, 'wb') as f:
        for chunk in response.iter_content(chunk_size=block_size):
            if chunk:
                f.write(chunk)
                downloaded += len(chunk)
                if total_size > 0:
                    percent = downloaded / total_size * 100
                    print(f"\rProgress: {percent:.1f}%", end='')

    print(f"\n✅ Downloaded: {dest_path}")

def main():
    """Main download function"""
    # Base directory
    base_dir = Path(__file__).parent / "dora_primespeech" / "moyoyo_tts" / "pretrained_models"
    pretrained_dir = base_dir / "gsv-v2final-pretrained"

    print(f"Target directory: {pretrained_dir}")
    pretrained_dir.mkdir(parents=True, exist_ok=True)

    # Models to download from HuggingFace
    base_url = "https://huggingface.co/lj1995/GPT-SoVITS/resolve/main/gsv-v2final-pretrained"

    models = [
        "s1bert25hz-5kh-longer-epoch=12-step=369668.ckpt",  # T2S model (148 MB)
        "s2G2333k.pth",  # VITS G model (101 MB)
        "s2D2333k.pth",  # VITS D model (89 MB) - optional
    ]

    print("=" * 60)
    print("GPT-SoVITS Pretrained Models Download")
    print("=" * 60)
    print("\nThis will download approximately 338 MB of model files.")
    print("(148 MB + 101 MB + 89 MB)")
    print("\nThese models are required for TTS synthesis.")
    print("\nStarting download in 3 seconds...")
    print("Press Ctrl+C to cancel.")

    import time
    for i in range(3, 0, -1):
        print(f"{i}...", flush=True)
        time.sleep(1)

    for model in models:
        url = f"{base_url}/{model}"
        dest = pretrained_dir / model

        if dest.exists():
            print(f"\n✓ Already exists: {model}")
            continue

        try:
            download_file(url, dest)
        except Exception as e:
            print(f"\n❌ Failed to download {model}: {e}")
            print("\nYou can download manually from:")
            print(f"  {url}")
            print(f"\nAnd place in: {dest}")
            sys.exit(1)

    print("\n" + "=" * 60)
    print("✅ All pretrained models downloaded successfully!")
    print("=" * 60)
    print(f"\nLocation: {pretrained_dir}")
    print("\nYou can now run PrimeSpeech TTS with real voice synthesis!")

if __name__ == "__main__":
    main()
