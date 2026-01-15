# Common errors and fixes

- "Dataflow not found": verify `apps/<app>/dataflow/voice-chat.yml`.
- "No audio output device": ensure an output device exists and is selected.
- "Failed to parse preferences": delete or fix `~/.dora/dashboard/preferences.json`.
- `icon_walk` field error: remove from widgets that do not support icons.
- Missing cursor: set `draw_cursor` on TextInput.
