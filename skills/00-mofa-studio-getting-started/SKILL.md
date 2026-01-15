---
name: 00-mofa-studio-getting-started
description: Onboard to the MoFA Studio repository, run the app and dataflows, and choose the right MoFA Studio skill. Use when setting up the workspace, building/running, or deciding which part of the system to edit (app, dataflow, UI, audio, settings, deployment, troubleshooting).
---

# MoFA Studio Getting Started

## 1. Overview
Use this skill to orient in the repo and pick the next specialized skill. Keep changes small and move to the focused skill once you know the task.

## 2. Quick start map
1. Run the app or dataflow: see references/quickstart.md
2. Understand repo layout: see references/repo-map.md
3. Pick the focus skill:
   - Architecture and boundaries -> 01-mofa-studio-core
   - Add or change an app -> 02-mofa-studio-app-development
   - Edit Dora dataflows -> 03-mofa-studio-dataflow
   - UI layout, events, shaders -> 04-mofa-studio-ui-patterns
   - Audio pipeline -> 05-mofa-studio-audio
   - Settings/providers -> 06-mofa-studio-settings
   - Run/deploy -> 07-mofa-studio-deployment
   - Debug or investigate failures -> 08-mofa-studio-reference
   - Roadmap/refactor planning -> 99-mofa-studio-evolution

## 3. Guardrails
- Prefer ASCII in new files unless the target file already uses non-ASCII.
- When in doubt about dataflow naming or signals, stop and read the dataflow skill references.
- Keep app changes self-contained; avoid shell coupling beyond the documented 4 points.

## 4. References
- references/quickstart.md
- references/repo-map.md
- references/pitfalls.md
