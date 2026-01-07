#!/usr/bin/env python3
"""
Text Segmenter Node - Configurable entry point for different segmentation modes.

Supported modes (via SEGMENTER_MODE env var):
- single (default): Queue-based segmenter for single conversation
- conference: Multi-participant segmenter with session management
- passthrough: Simple pass-through without buffering
- sequential: Sequential processing mode

Example:
    SEGMENTER_MODE=conference dora start dataflow.yml
"""

import os


def main():
    """Main entry point - dispatches to appropriate segmenter based on SEGMENTER_MODE."""
    mode = os.getenv("SEGMENTER_MODE", "single").lower()

    if mode == "single":
        from .queue_based_segmenter import main as segmenter_main
    elif mode == "conference":
        from .multi_participant_segmenter import main as segmenter_main
    elif mode == "passthrough":
        from .simple_passthrough import main as segmenter_main
    elif mode == "sequential":
        from .main_sequential import main as segmenter_main
    else:
        # Default to single mode
        from .queue_based_segmenter import main as segmenter_main

    segmenter_main()


if __name__ == "__main__":
    main()