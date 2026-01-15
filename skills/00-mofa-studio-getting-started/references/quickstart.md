# Quickstart

## 1. Nix run (recommended)
1. From repo root:
   ```bash
   ./run.sh
   ```
2. Alternative:
   ```bash
   nix --extra-experimental-features 'nix-command flakes' run .
   ```

## 2. Manual build/run (Rust only)
1. Build and run:
   ```bash
   cargo build --release
   cargo run --release
   ```
2. For debug logging:
   ```bash
   RUST_LOG=debug cargo run
   ```

## 3. Run a dataflow
```bash
cd apps/mofa-fm/dataflow

dora up

dora start voice-chat.yml

dora list

dora stop <dataflow-id>
```

## 4. Edge cases and fixes
- Nix installed but flakes not enabled: add `experimental-features = nix-command flakes` to `~/.config/nix/nix.conf`.
- dora not found: run `models/setup-local-models/install_all_packages.sh` or use Nix.
- Dataflow not found: verify `apps/<app>/dataflow/voice-chat.yml` exists.
- API keys missing: UI can start but LLM nodes will fail; set keys in settings or env.
- No audio device: cpal will error; test with a known output device.
