# Repo map

## 1. Top-level directories
- `apps/` - app crates (mofa-fm, mofa-debate, mofa-settings)
- `mofa-studio-shell/` - shell app and navigation
- `mofa-widgets/` - shared widgets and theme
- `mofa-dora-bridge/` - dynamic node bridges and dataflow parsing
- `node-hub/` - Dora nodes (Rust/Python)
- `models/` - model downloads and setup scripts

## 2. New app touch points
- `apps/<app>/` - new crate and UI
- `mofa-studio-shell/Cargo.toml` - add dependency + feature
- `mofa-studio-shell/src/app.rs` - register app + timers
- `mofa-studio-shell/src/widgets/dashboard.rs` - add page widget
- `mofa-studio-shell/src/widgets/sidebar.rs` - add sidebar entry
- `flake.nix` - dataflow directory checks

## 3. Key docs
- `ARCHITECTURE.md` and `APP_DEVELOPMENT_GUIDE.md`
- `MOFA_DORA_ARCHITECTURE.md` and `mofa-studio-dora-integration-checklist.md`
- `DEPLOY_WITH_NIX.md`
