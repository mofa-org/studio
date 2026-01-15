# Dataflow debugging

## 1. Quick check
```bash
dora list
```

## 2. Start/stop
```bash
dora up
dora start voice-chat.yml
dora stop <id>
```

## 3. Typical failures
- Dataflow not found: verify `apps/<app>/dataflow/voice-chat.yml`.
- Dynamic nodes not connected: check `mofa-` prefix and `path: dynamic`.
- No chat text: ensure prompt input `llm*_text` inputs are wired.
- No audio: confirm `audio_*` inputs wired to TTS outputs.
- No logs: outputs must end in `_log` or `_status`, or be `log`.

## 4. UI-side signals
- Connection status in hero should change to Connected on `DataflowStarted`.
- System log should show bridge connected/disconnected messages.
