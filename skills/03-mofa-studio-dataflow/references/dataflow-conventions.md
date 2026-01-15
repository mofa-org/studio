# Dataflow conventions

## 1. Dynamic nodes
- Use `path: dynamic`.
- Node IDs must start with `mofa-` (suffix allowed):
  - `mofa-audio-player`
  - `mofa-prompt-input`
  - `mofa-system-log`

## 2. Audio inputs
- Use `audio_<participant>` naming (e.g., `audio_student1`).
- The audio bridge extracts the participant ID by stripping `audio_`.

## 3. Control wiring
- Prompt input outputs `control`.
- `conference-controller` must receive `control`, `session_start`, and `buffer_status`.

## 4. Log wiring
- System log bridge reads outputs named `*_log`, `log`, `*_status`.

## 5. Example snippet
```yaml
- id: mofa-audio-player
  path: dynamic
  inputs:
    audio_student1: { source: primespeech-student1/audio, queue_size: 1000 }
    control: { source: conference-controller/llm_control, queue_size: 10 }
  outputs: [buffer_status, session_start, audio_complete, log]
```
