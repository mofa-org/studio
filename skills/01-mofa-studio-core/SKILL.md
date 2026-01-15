---
name: 01-mofa-studio-core
description: Core architecture of MoFA Studio: plugin system, shell/app boundaries, state coordination, theme and dark mode, and timer/event lifecycles. Use when changing architecture, refactoring, or adding cross-cutting behavior.
---

# MoFA Studio Core

## 1. Overview
Follow the black-box app principle and keep shell/app coupling limited to the four required points. Use the references for details and examples.

## 2. Core rules
- Keep apps self-contained; shell does not reach into app internals.
- Register widgets in the correct order in `LiveRegister`.
- Use `MofaApp` and `AppRegistry` for metadata and registration.
- Propagate shared state via WidgetRef methods, not a global store.

## 3. Workflow for architectural changes
1. Identify the coupling point you need to touch (import, register, instantiate, visibility).
2. Update the app crate first, then wire the shell.
3. Verify dark mode and timer lifecycle hooks.
4. Update dataflow wiring if the app uses Dora.

## 4. References
- references/architecture.md
- references/state-and-theme.md
- references/timers-and-events.md
