# UI workflow

## 1. Layout
- Define widget hierarchy in `live_design!`.
- Use `View` + `RoundedView` for layout; keep IDs in snake_case.
- Import `mofa_widgets::theme::*` for fonts/colors.

## 2. Event handling
- Always call `self.view.handle_event(cx, event, scope)`.
- Handle hover and press events via `event.hits`.

## 3. Runtime updates
- Prefer `apply_over` to change `draw_bg` or `draw_text` values.
- Use `set_visible` only for simple show/hide where no shader state is involved.
