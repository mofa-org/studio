# Common pitfalls

- Dataflow path mismatch: app looks for `apps/<app>/dataflow/voice-chat.yml` and `dataflow/voice-chat.yml`.
- Dynamic node ID mismatch: `mofa-dora-bridge` discovers nodes by `mofa-` prefix.
- Missing `apply_over` at runtime: use `apply_over` for visibility and shader instance updates.
- `vec4` required for runtime colors: hex colors in `apply_over` do not work.
- Timers running while hidden: implement `stop_timers()`/`start_timers()` on ScreenRef.
- Hover events ignored: handle `Hit::FingerHoverIn` before `Event::Actions` early return.
