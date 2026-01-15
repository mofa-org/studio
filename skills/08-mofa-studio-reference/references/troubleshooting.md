# Troubleshooting flow

## 1. Capture
- Run with `RUST_LOG=debug` when needed.
- Check `out/` for dora logs.

## 2. Dataflow checks
```bash
dora list
```
- Ensure dynamic nodes are connected.

## 3. UI checks
- Confirm `live_design` registration order.
- Validate hover handlers before `Event::Actions`.

## 4. Settings checks
- Confirm `preferences.json` is valid and keys exist.
