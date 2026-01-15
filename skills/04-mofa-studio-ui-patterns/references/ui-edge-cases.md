# UI edge cases

- TextInput cursor invisible: set `draw_cursor` color.
- DropDown has no `icon_walk`: remove icon settings.
- RoundedView has no `icon_walk` or `draw_label` fields.
- Use `FingerHoverIn`/`FingerHoverOut` instead of mouse hover events.
- Missing redraw after `apply_over`: call `view.redraw(cx)`.
- Widget not rendering: ensure `live_design` registration order is correct.
