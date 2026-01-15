---
name: 06-mofa-studio-settings
description: Provider settings and user preferences in MoFA Studio. Use when editing provider UI, saving preferences, or wiring API keys and defaults.
---

# MoFA Studio Settings

## 1. Overview
Settings are stored in `~/.dora/dashboard/preferences.json` and merged with supported providers on load.

## 2. Settings workflow
1. Load Preferences at app start.
2. Update Provider data and call `save()`.
3. Propagate changes to UI and other apps.

## 3. References
- references/provider-workflow.md
- references/preferences-storage.md
- references/settings-edge-cases.md
