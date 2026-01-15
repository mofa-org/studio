# Refactor playbook

## 1. Plan
1. Identify the smallest viable change.
2. Locate coupling points and update one at a time.
3. Preserve black-box app boundaries.

## 2. Execute
- Update code in one area (app, shell, widgets, dataflow).
- Run builds or smoke tests where possible.
- Update docs if behavior changes.

## 3. Validate
- Check UI behavior, dataflow connectivity, and log output.
- Ensure timers stop/start on navigation.
