#!/usr/bin/env python3
"""Entry point for running as module: python -m dora_text_segmenter

Mode is controlled by SEGMENTER_MODE env var:
- single (default): Queue-based segmenter for single conversation
- conference: Multi-participant segmenter with session management
- passthrough: Simple pass-through without buffering
- sequential: Sequential processing mode
"""

from dora_text_segmenter.main import main

if __name__ == "__main__":
    main()