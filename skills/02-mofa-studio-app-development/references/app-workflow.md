# App workflow

## 1. Scaffold a new app
1. Create crate:
   ```bash
   cd apps
   cargo new mofa-myapp --lib
   ```
2. Add dependencies in `apps/mofa-myapp/Cargo.toml`:
   ```toml
   [dependencies]
   makepad-widgets.workspace = true
   mofa-widgets = { path = "../../mofa-widgets" }
   mofa-dora-bridge = { path = "../../mofa-dora-bridge" }
   mofa-settings = { path = "../mofa-settings" }
   ```
3. Implement `MofaApp` in `apps/mofa-myapp/src/lib.rs`:
   ```rust
   pub struct MoFaMyApp;
   impl MofaApp for MoFaMyApp { /* info + live_design */ }
   ```
4. Create `apps/mofa-myapp/src/screen/mod.rs` with `live_design!` and `Widget` impl.

## 2. Optional: clone an existing app
- Copy `apps/mofa-debate` and rename the crate and types.
- Update dataflow paths and node IDs.
