"""
Configuration for ASR node.
"""

import os
from pathlib import Path


class ASRConfig:
    """ASR configuration class"""
    
    def __init__(self):
        # Engine selection
        self.ASR_ENGINE = os.getenv("ASR_ENGINE", "auto")  # auto/whisper/funasr
        self.LANGUAGE = os.getenv("LANGUAGE", "auto")  # auto/zh/en
        
        # Model configuration
        self.WHISPER_MODEL = os.getenv("WHISPER_MODEL", "medium-q5_0")
        self.FUNASR_ASR_MODEL = os.getenv("FUNASR_ASR_MODEL", "speech_seaco_paraformer_large_asr_nat-zh-cn-16k-common-vocab8404-pytorch")
        self.FUNASR_PUNC_MODEL = os.getenv("FUNASR_PUNC_MODEL", "punc_ct-transformer_cn-en-common-vocab471067-large")
        
        # Audio processing
        self.MIN_AUDIO_DURATION = float(os.getenv("MIN_AUDIO_DURATION", "0.5"))  # seconds
        self.MAX_AUDIO_DURATION = float(os.getenv("MAX_AUDIO_DURATION", "30"))  # seconds
        self.SAMPLE_RATE = int(os.getenv("SAMPLE_RATE", "16000"))
        
        # Features
        self.ENABLE_PUNCTUATION = os.getenv("ENABLE_PUNCTUATION", "true").lower() == "true"
        self.ENABLE_LANGUAGE_DETECTION = os.getenv("ENABLE_LANGUAGE_DETECTION", "true").lower() == "true"
        self.ENABLE_CONFIDENCE_SCORE = os.getenv("ENABLE_CONFIDENCE_SCORE", "false").lower() == "true"
        
        # Performance
        self.USE_GPU = os.getenv("USE_GPU", "false").lower() == "true"
        self.NUM_THREADS = int(os.getenv("NUM_THREADS", "4"))
        
        # FunASR specific settings
        self.FUNASR_DISABLE_UPDATE = os.getenv("FUNASR_DISABLE_UPDATE", "true").lower() == "true"
        
        # Logging
        self.LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO").upper()  # DEBUG, INFO, WARNING, ERROR
    
    # Model paths
    @staticmethod
    def get_models_dir() -> Path:
        """Get models directory path"""
        # Check environment variable first
        models_dir = os.getenv("ASR_MODELS_DIR")
        if models_dir:
            # Expand '~' and environment variables to avoid Docker HOME issues
            return Path(os.path.expandvars(models_dir)).expanduser()
        
        # Default to home directory
        home = Path.home()
        default_dir = home / ".dora" / "models" / "asr"
        default_dir.mkdir(parents=True, exist_ok=True)
        return default_dir
    
        
    # Language mappings (class variable)
    LANGUAGE_TO_ENGINE = {
        'zh': 'funasr',   # Chinese optimized with FunASR
        'en': 'whisper',  # English with Whisper
        'auto': 'whisper' # Auto-detect with Whisper
    }
