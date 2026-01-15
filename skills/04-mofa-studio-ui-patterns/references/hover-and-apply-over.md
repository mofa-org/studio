# Hover and apply_over

## 1. Hover ordering
Handle hover before `Event::Actions` early return:
```rust
match event.hits(cx, widget.area()) {
    Hit::FingerHoverIn(_) => { /* apply_over hover */ }
    Hit::FingerHoverOut(_) => { /* reset */ }
    _ => {}
}

let actions = match event { Event::Actions(a) => a.as_slice(), _ => return };
```

## 2. apply_over and visibility
Use `apply_over` when toggling visibility to ensure shader instances update:
```rust
self.ui.view(ids!(content.fm_page)).apply_over(cx, live!{ visible: true });
```

## 3. vec4 for runtime colors
Hex colors do not work in `apply_over`:
```rust
self.view.apply_over(cx, live!{ draw_bg: { color: (vec4(0.12, 0.16, 0.23, 1.0)) } });
```
