---
name: 03-mofa-studio-dataflow
description: Dora dataflow authoring and wiring for MoFA Studio dynamic nodes. Use when editing voice-chat.yml, adding dynamic nodes, or debugging dataflow connections and signals.
---

# MoFA Studio Dataflow

## 1. Overview
MoFA apps rely on Dora dataflows with dynamic nodes. Follow naming and signal contracts so the bridge layer can discover and connect.

## 2. Dataflow workflow
1. Start from an existing `voice-chat.yml` (fm or debate).
2. Keep dynamic node IDs with `mofa-` prefix; add suffixes if needed.
3. Wire control and audio signals to `conference-controller`.
4. Ensure log outputs use `*_log` or `*_status` naming.
5. Validate with `dora up` and check UI connection status.

## 3. References
- references/dataflow-conventions.md
- references/signal-contracts.md
- references/dataflow-debugging.md
