"""
Enhanced FunASR engine with GPU acceleration support.
Supports both PyTorch models and ONNX models with automatic selection.
"""

import os
import time
from typing import Optional, Dict, Any
import numpy as np
import re
import logging

logger = logging.getLogger(__name__)

# Try to import both FunASR variants
FUNASR_PYTORCH_AVAILABLE = False
FUNASR_ONNX_AVAILABLE = False

try:
    from funasr import AutoModel
    FUNASR_PYTORCH_AVAILABLE = True
    logger.info("FunASR PyTorch version available")
except ImportError:
    logger.warning("FunASR PyTorch not available. Install with: pip install funasr")

try:
    from funasr_onnx import SeacoParaformer, CT_Transformer
    FUNASR_ONNX_AVAILABLE = True
    logger.info("FunASR ONNX version available")
except ImportError:
    logger.warning("FunASR ONNX not available. Install with: pip install funasr-onnx")

# Check for GPU availability
try:
    import torch
    TORCH_AVAILABLE = True
    CUDA_AVAILABLE = torch.cuda.is_available()
    if CUDA_AVAILABLE:
        logger.info(f"CUDA available: {torch.cuda.get_device_name(0)}")
except ImportError:
    TORCH_AVAILABLE = False
    CUDA_AVAILABLE = False

from .base import ASRInterface
from ..utils import ensure_minimum_audio_duration, fix_spaced_uppercase
from ..config import ASRConfig


class FunASRGPUEngine(ASRInterface):
    """Enhanced FunASR engine with GPU acceleration"""
    
    supported_languages = ['zh', 'en', 'auto']
    
    def __init__(self):
        super().__init__()
        self.config = ASRConfig()
        self.asr_model = None
        self.punc_model = None
        self.use_pytorch = False  # Flag to track which backend is used
        self.device = None
        self.metrics = {}  # Store performance metrics
    
    def setup(self, **kwargs) -> None:
        """
        Setup FunASR models with automatic backend selection.
        Prioritizes PyTorch with GPU > ONNX with GPU > PyTorch CPU > ONNX CPU
        """
        models_dir = kwargs.get('models_dir', self.config.get_models_dir() / "funasr")
        
        # Model paths
        asr_model_name = self.config.FUNASR_ASR_MODEL
        punc_model_name = self.config.FUNASR_PUNC_MODEL
        
        asr_model_path = models_dir / asr_model_name
        punc_model_path = models_dir / punc_model_name
        
        logger.info(f"Loading FunASR models from: {models_dir}")
        
        # Check if models exist
        if not asr_model_path.exists():
            logger.error(f"ASR model not found at {asr_model_path}")
            raise FileNotFoundError(f"ASR model not found: {asr_model_path}")
        
        # Determine best backend and device
        use_gpu = self.config.USE_GPU and (CUDA_AVAILABLE or self._check_onnx_gpu())
        
        # Try PyTorch backend first (better GPU support)
        if FUNASR_PYTORCH_AVAILABLE and (asr_model_path / "model.pt").exists():
            self._setup_pytorch_backend(asr_model_path, punc_model_path, use_gpu)
        # Fallback to ONNX backend
        elif FUNASR_ONNX_AVAILABLE and (asr_model_path / "model.onnx").exists():
            self._setup_onnx_backend(asr_model_path, punc_model_path, use_gpu)
        else:
            raise RuntimeError("No suitable FunASR backend available")
        
        self.is_initialized = True
        logger.info(f"FunASR models loaded successfully (backend: {'PyTorch' if self.use_pytorch else 'ONNX'}, device: {self.device})")
    
    def _check_onnx_gpu(self) -> bool:
        """Check if ONNX Runtime has GPU support"""
        try:
            import onnxruntime as ort
            providers = ort.get_available_providers()
            return 'CUDAExecutionProvider' in providers
        except ImportError:
            return False
    
    def _setup_pytorch_backend(self, asr_model_path, punc_model_path, use_gpu):
        """Setup PyTorch-based FunASR models"""
        logger.info("Setting up FunASR PyTorch backend")
        logger.info(f"FunASR update check: {'disabled' if self.config.FUNASR_DISABLE_UPDATE else 'enabled'}")
        
        # Set device
        self.device = "cuda" if use_gpu and CUDA_AVAILABLE else "cpu"
        self.use_pytorch = True
        
        try:
            # Load ASR model
            logger.info(f"Loading ASR model with PyTorch (device: {self.device})")
            self.asr_model = AutoModel(
                model=str(asr_model_path),
                device=self.device,
                disable_update=self.config.FUNASR_DISABLE_UPDATE,
                disable_log=True
            )
            
            # Load punctuation model if enabled
            if self.config.ENABLE_PUNCTUATION and punc_model_path.exists():
                logger.info(f"Loading punctuation model with PyTorch")
                self.punc_model = AutoModel(
                    model=str(punc_model_path),
                    device=self.device,
                    disable_update=self.config.FUNASR_DISABLE_UPDATE,
                    disable_log=True
                )
            
            # Log GPU memory if using CUDA
            if self.device == "cuda":
                allocated = torch.cuda.memory_allocated() / 1024**3
                reserved = torch.cuda.memory_reserved() / 1024**3
                logger.info(f"GPU memory: {allocated:.2f}GB allocated, {reserved:.2f}GB reserved")
                
        except Exception as e:
            logger.error(f"Failed to load PyTorch models: {e}")
            raise
    
    def _setup_onnx_backend(self, asr_model_path, punc_model_path, use_gpu):
        """Setup ONNX-based FunASR models"""
        logger.info("Setting up FunASR ONNX backend")
        
        # Set device ID for ONNX
        device_id = "0" if use_gpu else "-1"
        self.device = "cuda" if use_gpu else "cpu"
        self.use_pytorch = False
        
        try:
            # Check for quantized ONNX model
            onnx_model = asr_model_path / "model_quant.onnx"
            if not onnx_model.exists():
                onnx_model = asr_model_path / "model.onnx"
            
            use_quantized = "quant" in str(onnx_model)
            
            logger.info(f"Loading ASR model with ONNX (device_id: {device_id}, quantized: {use_quantized})")
            self.asr_model = SeacoParaformer(
                str(asr_model_path),
                quantize=use_quantized,
                device_id=device_id
            )
            
            # Load punctuation model if enabled
            if self.config.ENABLE_PUNCTUATION and punc_model_path.exists():
                punc_onnx = punc_model_path / "model_quant.onnx"
                if not punc_onnx.exists():
                    punc_onnx = punc_model_path / "model.onnx"
                
                use_quantized = "quant" in str(punc_onnx)
                
                logger.info(f"Loading punctuation model with ONNX (quantized: {use_quantized})")
                self.punc_model = CT_Transformer(
                    str(punc_model_path),
                    quantize=use_quantized,
                    device_id=device_id
                )
                
        except Exception as e:
            logger.error(f"Failed to load ONNX models: {e}")
            raise
    
    def warmup(self) -> None:
        """Warmup FunASR models"""
        if not self.is_initialized:
            return
        
        logger.info("Warming up FunASR models...")
        try:
            start_time = time.time()
            result = self.transcribe(self.warmup_audiodata, language='zh')
            warmup_time = time.time() - start_time
            logger.info(f"FunASR models warmed up in {warmup_time:.3f}s")
        except Exception as e:
            logger.error(f"FunASR warmup failed: {e}")
    
    def transcribe(
        self,
        audio_array: np.ndarray,
        language: Optional[str] = None,
        **kwargs
    ) -> Dict[str, Any]:
        """
        Transcribe audio using FunASR with performance tracking.
        """
        if not self.is_initialized:
            raise RuntimeError("FunASR models not initialized")
        
        # Ensure minimum duration
        audio_array = ensure_minimum_audio_duration(
            audio_array,
            sample_rate=self.config.SAMPLE_RATE,
            min_duration=self.config.MIN_AUDIO_DURATION
        )
        
        start_time = time.time()
        audio_duration = len(audio_array) / self.config.SAMPLE_RATE
        
        try:
            # Run ASR based on backend
            if self.use_pytorch:
                text = self._transcribe_pytorch(audio_array, **kwargs)
            else:
                text = self._transcribe_onnx(audio_array, **kwargs)
            
            # Calculate metrics
            total_time = time.time() - start_time
            rtf = total_time / audio_duration
            
            # Update metrics
            self.metrics = {
                'audio_duration': audio_duration,
                'processing_time': total_time,
                'rtf': rtf,
                'backend': 'PyTorch' if self.use_pytorch else 'ONNX',
                'device': self.device
            }
            
            # Log GPU memory if using CUDA
            if self.device == "cuda" and TORCH_AVAILABLE:
                self.metrics['gpu_memory_gb'] = torch.cuda.memory_allocated() / 1024**3
            
            logger.info(f"Transcription completed in {total_time:.3f}s (RTF: {rtf:.3f})")
            
            return {
                "text": text,
                "language": language or "zh",
                "segments": [{"text": text, "start": 0, "end": audio_duration}],
                "metrics": self.metrics
            }
            
        except Exception as e:
            logger.error(f"Transcription failed: {e}")
            raise RuntimeError(f"FunASR transcription failed: {str(e)}")
    
    def _transcribe_pytorch(self, audio_array: np.ndarray, **kwargs) -> str:
        """Transcribe using PyTorch backend"""
        # Run ASR
        hotwords = kwargs.get('hotwords', '魔搭')
        result = self.asr_model.generate(
            input=audio_array,
            batch_size_s=300,
            hotword=hotwords,
            disable_pbar=True
        )
        
        # Extract text
        if isinstance(result, list) and len(result) > 0:
            text = result[0].get("text", "")
        else:
            text = str(result)
        
        # Add punctuation if available
        if self.punc_model and text:
            punc_result = self.punc_model.generate(input=text, disable_pbar=True)
            if isinstance(punc_result, list) and len(punc_result) > 0:
                text = punc_result[0].get("text", text)
        
        return text
    
    def _transcribe_onnx(self, audio_array: np.ndarray, **kwargs) -> str:
        """Transcribe using ONNX backend"""
        # Run ASR
        hotwords = kwargs.get('hotwords', '')
        segments = self.asr_model(wav_content=audio_array, hotwords=hotwords)
        
        # Process segments
        transcribed_texts = []
        for segment in segments:
            text = segment.get('text', '')
            if text:
                transcribed_texts.append(text)
        
        full_text = ' '.join(transcribed_texts)
        
        # Add punctuation if available
        if self.punc_model and full_text:
            full_text = self.punc_model(text=full_text)
        
        # Clean up text
        full_text = re.sub(r'\s+', ' ', full_text).strip()
        full_text = fix_spaced_uppercase(full_text)
        
        return full_text
    
    def get_metrics(self) -> Dict[str, Any]:
        """Get performance metrics from last transcription"""
        return self.metrics.copy()