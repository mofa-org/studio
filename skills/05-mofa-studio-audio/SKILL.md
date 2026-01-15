---
name: 05-mofa-studio-audio
description: Audio pipeline details for MoFA Studio: device selection, mic monitoring, AudioPlayer buffer behavior, and participant tracking. Use when modifying audio code or diagnosing audio issues.
---

# MoFA Studio Audio

## 1. Overview
MoFA uses cpal for input/output and a circular buffer AudioPlayer for playback. Keep sample rate and metadata consistent with dataflow.

## 2. Audio workflow
1. Initialize `AudioManager` and populate device dropdowns.
2. Start mic monitoring and update UI on timer.
3. Send buffer status to Dora from actual buffer fill.
4. Write audio with participant and question_id.
5. Use smart reset to discard stale audio.

## 3. References
- references/audio-pipeline.md
- references/audio-player-contracts.md
- references/audio-edge-cases.md
