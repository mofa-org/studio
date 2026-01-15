# State and theme

## 1. State coordination
- Shell owns cross-cutting state and propagates via WidgetRef methods.
- Avoid global stores or Arc<Mutex<T>> for widget state.

Example:
```rust
self.ui.mo_fa_fmscreen(ids!(...fm_page)).update_dark_mode(cx, dark_mode);
```

## 2. Dark mode
- Use `instance dark_mode: 0.0` in shaders and `mix()` for colors.
- Update with `apply_over`.

```rust
inner.view.apply_over(cx, live!{
    draw_bg: { dark_mode: (dark_mode) }
});
```

## 3. Runtime colors
- Use `vec4()` in `apply_over`.
- Hex literals in `apply_over` do not work.

```rust
self.view.apply_over(cx, live!{ draw_bg: { color: (vec4(0.12, 0.16, 0.23, 1.0)) } });
```
