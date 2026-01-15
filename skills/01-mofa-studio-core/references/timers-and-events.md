# Timers and events

## 1. Timer lifecycle
- Store timers as fields and provide `stop_timers()`/`start_timers()` on ScreenRef.
- Shell must call these when switching pages or overlays.

```rust
impl MyScreenRef {
    pub fn stop_timers(&self, cx: &mut Cx) { /* stop timers */ }
    pub fn start_timers(&self, cx: &mut Cx) { /* start timers */ }
}
```

## 2. Animations
- Prefer shader time or `Event::NextFrame` for UI animations.
- Call `cx.new_next_frame()` while animation is active.

## 3. Event ordering
- Always call `self.view.handle_event(cx, event, scope)`.
- Handle hover before early-returning on `Event::Actions`.
