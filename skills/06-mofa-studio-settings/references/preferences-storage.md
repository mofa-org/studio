# Preferences storage

## 1. File location
`~/.dora/dashboard/preferences.json`

## 2. Load behavior
- Loads JSON if present.
- Merges with supported providers so defaults are always available.

## 3. Stored fields
- `providers`
- `default_chat_provider`, `default_tts_provider`, `default_asr_provider`
- `audio_input_device`, `audio_output_device`
- `dark_mode`
