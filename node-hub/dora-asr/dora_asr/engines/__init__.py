"""
ASR engines module.
"""

from .base import ASRInterface
from .whisper import WhisperEngine
from .funasr import FunASREngine

# Try to import GPU-enhanced version
try:
    from .funasr_gpu import FunASRGPUEngine
    __all__ = ['ASRInterface', 'WhisperEngine', 'FunASREngine', 'FunASRGPUEngine']
except ImportError:
    __all__ = ['ASRInterface', 'WhisperEngine', 'FunASREngine']