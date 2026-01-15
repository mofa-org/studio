# Settings edge cases

- Invalid JSON: falls back to defaults and logs an error.
- Removing provider: only custom providers can be removed.
- Missing API key: provider remains listed but may fail at runtime.
- Device selection: persisted by name; handle device name changes gracefully.
