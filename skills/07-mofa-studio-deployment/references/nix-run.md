# Nix run

## 1. Recommended flow
```bash
./run.sh
```

## 2. Alternative
```bash
nix --extra-experimental-features 'nix-command flakes' run .
```

## 3. Environment variables
- `MOFA_STUDIO_DIR` - repo root override
- `MOFA_STATE_DIR` - dora-cli cache dir
- `MOFA_VENV_DIR` - python venv dir
- `MOFA_DORA_RS_VERSION` - pin dora-rs version
- `MOFA_SKIP_BOOTSTRAP=1` - skip dependency install
- `MOFA_DRY_RUN=1` - run checks only
