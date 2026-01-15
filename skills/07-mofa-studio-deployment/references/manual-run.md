# Manual run

## 1. Build and run
```bash
cargo build --release
cargo run --release
```

## 2. Python node setup
```bash
cd models/setup-local-models
./setup_isolated_env.sh
./install_all_packages.sh
```

## 3. Dataflow
```bash
cd apps/mofa-fm/dataflow

dora up
dora start voice-chat.yml
```
