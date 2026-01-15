# App edge cases

- Missing dataflow file: handle with a log + connection status failed.
- Prompt sent while dataflow stopped: decide whether to queue or warn.
- Timers continue while hidden: ensure ScreenRef exposes `stop_timers()`/`start_timers()`.
- Dark mode not updating: check `update_dark_mode()` is called from shell.
- `apply_over` vs `set_visible`: prefer `apply_over` for visibility toggles.
- New app dataflow directory: add to `flake.nix` checks if using Nix.
- Export WidgetRefExt in `lib.rs` if shell needs timer control.
