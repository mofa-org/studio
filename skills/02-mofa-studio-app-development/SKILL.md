---
name: 02-mofa-studio-app-development
description: Create or modify MoFA Studio apps, screens, and shell integration. Use when adding a new app, changing an existing app, or wiring app lifecycle (timers, dark mode, dataflow).
---

# MoFA Studio App Development

## 1. Overview
Build apps as self-contained crates under `apps/`. Use `MofaApp` for registration and keep shell coupling minimal.

## 2. New app workflow
1. Scaffold the app crate (copy mofa-fm or `cargo new`).
2. Implement `MofaApp` and export the screen type.
3. Create the screen with `live_design!` and `Widget` impl.
4. Register the app in the shell (Cargo features, `LiveRegister`, `LiveHook`).
5. Add the dashboard page and sidebar entry.
6. Add timer and dark mode hooks if needed.
7. Add dataflow wiring if the app uses Dora.

## 3. Existing app changes
- Keep UI changes inside `apps/<app>/src/screen/*`.
- Keep dataflow changes inside `apps/<app>/dataflow`.
- Update shared widgets in `mofa-widgets` only when multiple apps need it.

## 4. References
- references/app-workflow.md
- references/shell-integration.md
- references/app-edge-cases.md
