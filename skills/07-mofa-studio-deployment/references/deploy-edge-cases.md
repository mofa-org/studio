# Deployment edge cases

- Stale venv path: delete `.venv-mofa` and rerun.
- dora-rs version mismatch: rerun without `MOFA_SKIP_BOOTSTRAP`.
- Missing dataflow directory: add `apps/<app>/dataflow` and update `flake.nix`.
- Offline machine: use `MOFA_SKIP_BOOTSTRAP=1` only after dependencies exist.
