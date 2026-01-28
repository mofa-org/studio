# Dora Voice Router

Voice routing node for mofa-cast multi-voice TTS synthesis.

## Purpose

Routes text segments to different TTS nodes based on voice_name in JSON input.

## Input Format

JSON with voice routing information:
```json
{
  "speaker": "host",
  "text": "Hello world",
  "voice_name": "Luo Xiang",
  "speed": 1.0
}
```

## Outputs

Routes to different outputs based on voice_name:
- `text_luo_xiang`: for "Luo Xiang" voice
- `text_yang_mi`: for "Yang Mi" voice
- `text_ma_yun`: for "Ma Yun" voice
- `text_fallback`: for unknown voices
