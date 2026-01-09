# MoFA Studio

> AI-powered desktop voice chat application built with Rust and Makepad

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)

MoFA Studio is a modern, GPU-accelerated desktop application for AI voice chat and model management. Built entirely in Rust using the [Makepad](https://github.com/makepad/makepad) UI framework, it provides a beautiful, responsive interface with native performance.

![MoFA Studio](mofa-studio-shell/resources/mofa-logo.png)

## ✨ Features

- **🎨 Beautiful UI** - GPU-accelerated rendering with smooth animations
- **🌓 Dark Mode** - Seamless light/dark theme switching with animated transitions
- **🎙️ Audio Management** - Real-time microphone monitoring and device selection
- **🔌 Modular Architecture** - Plugin-based app system for extensibility
- **⚙️ Provider Configuration** - Manage multiple AI service providers (OpenAI, DeepSeek, Alibaba Cloud)
- **📊 Real-time Metrics** - CPU, memory, and audio buffer monitoring
- **🚀 Native Performance** - Built with Rust for maximum efficiency

## 🏗️ Architecture

MoFA Studio uses a modular workspace structure:

```
mofa-studio/
├── mofa-studio-shell/      # Main application shell
├── mofa-widgets/           # Shared reusable widgets
└── apps/
    ├── mofa-fm/            # Voice chat interface
    └── mofa-settings/      # Provider configuration
```

### Key Design Principles

- **Plugin System** - Apps implement the `MofaApp` trait for standardized integration
- **Black-Box Apps** - Apps are self-contained with no shell coupling
- **Theme System** - Centralized color and font management
- **Makepad Native** - Leverages Makepad's GPU-accelerated immediate-mode UI

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed system design.

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.70+ (2021 edition)
- **Cargo** package manager
- **Git** for cloning the repository

### Voice Chat Prerequisites

To run the voice chat dataflow, you need to set up the Python environment and download the required AI models.

#### 1. Environment Setup

```bash
cd models/setup-local-models
./setup_isolated_env.sh
```

This creates a conda environment `mofa-studio` with:
- Python 3.12
- PyTorch 2.2.0, NumPy 1.26.4, Transformers 4.45.0

#### 2. Install All Packages

After the conda environment is created, install all Python and Rust components:

```bash
conda activate mofa-studio
./install_all_packages.sh
```

This installs:
- Shared library: `dora-common`
- Python nodes: `dora-asr`, `dora-primespeech`, `dora-speechmonitor`, `dora-text-segmenter`
- Rust nodes: `dora-maas-client`, `dora-conference-bridge`, `dora-conference-controller`
- Dora CLI

Verify installation:

```bash
python test_dependencies.py
```

#### 3. Model Downloads

```bash
cd models/model-manager

# ASR models (FunASR Paraformer + punctuation)
python download_models.py --download funasr

# PrimeSpeech TTS (base + voices)
python download_models.py --download primespeech

# List available voices
python download_models.py --list-voices

# Download specific voice
python download_models.py --voice "Luo Xiang"
```

Models are stored in:
| Location | Contents |
|----------|----------|
| `~/.dora/models/asr/funasr/` | FunASR ASR models |
| `~/.dora/models/primespeech/` | PrimeSpeech TTS base + voices |

#### 4. API Keys (Optional)

For LLM inference, set your API keys in the MoFA Settings app or via environment variables:
(You may also enter it in the MoFA Studio's Settings UI page later)

```bash
export OPENAI_API_KEY="your-key"
export DEEPSEEK_API_KEY="your-key"
export ALIBABA_CLOUD_API_KEY="your-key"
```

### Build & Run

```bash
# Clone the repository
git clone https://github.com/mofa-org/mofa-studio.git
cd mofa-studio

# Build in release mode
cargo build --release

# Run the application
cargo run --release
```

The application window will open at 1400x900 pixels by default.

### Development Build

```bash
# Fast debug build
cargo build

# Run with debug logging
RUST_LOG=debug cargo run
```

### Run Voice Chat Dataflow

MoFA Studio uses [Dora](https://github.com/dora-rs/dora) for voice chat dataflow orchestration. Each app can have its own dataflow configuration.

```bash
# Navigate to app's dataflow directory
cd apps/mofa-fm/dataflow

# Start the Dora daemon
dora up

# Start the dataflow (packages already installed via install_all_packages.sh)
dora start voice-chat.yml

# Check running dataflows
dora list

# Stop dataflow
dora stop <dataflow-id>
```

The `node-hub/` directory contains all Dora nodes used by the dataflows:

| Node | Type | Description |
|------|------|-------------|
| `dora-maas-client` | Rust | LLM inference via MaaS APIs |
| `dora-conference-bridge` | Rust | Text routing between participants |
| `dora-conference-controller` | Rust | Turn-taking and policy management |
| `dora-primespeech` | Python | TTS synthesis with multiple voices |
| `dora-text-segmenter` | Python | Text segmentation for TTS |
| `dora-asr` | Python | Speech recognition (Whisper/FunASR) |
| `dora-common` | Python | Shared logging utilities |

## 📦 Project Structure

MoFA Studio is organized as a Cargo workspace with 5 crates:

| Crate | Type | Description |
|-------|------|-------------|
| `mofa-studio-shell` | Binary | Main application shell with window chrome and navigation |
| `mofa-widgets` | Library | Shared UI components (theme, audio player, waveforms, etc.) |
| `mofa-fm` | Library | Voice chat interface app |
| `mofa-settings` | Library | Provider configuration app |

### Key Files

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Complete system architecture guide
- **[APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md)** - How to create new apps
- **[STATE_MANAGEMENT_ANALYSIS.md](STATE_MANAGEMENT_ANALYSIS.md)** - State management patterns
- **[CHECKLIST.md](CHECKLIST.md)** - Refactoring roadmap and completion status

## 🎯 Current Status

MoFA Studio is currently a **UI prototype** with working components:

### ✅ Implemented
- Full UI navigation and theming
- Audio device selection and monitoring
- Provider configuration persistence
- Dark/light mode with animations
- Plugin app system

### 🚧 Planned
- WebSocket client for AI service integration
- Live ASR (speech recognition) integration
- Live TTS (text-to-speech) integration
- LLM chat completion
- Real-time conversation flow

## 🛠️ Creating a New App

MoFA Studio's plugin system makes it easy to add new functionality:

```rust
// 1. Implement the MofaApp trait
impl MofaApp for MyApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "My App",
            id: "my-app",
            description: "My custom app"
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}

// 2. Create your screen widget
live_design! {
    pub MyAppScreen = {{MyAppScreen}} {
        width: Fill, height: Fill
        // Your UI here
    }
}
```

See [APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md) for step-by-step instructions.

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | System architecture, widget hierarchy, best practices |
| [APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md) | Creating apps, plugin system, dark mode support |
| [STATE_MANAGEMENT_ANALYSIS.md](STATE_MANAGEMENT_ANALYSIS.md) | Why Redux/Zustand don't work in Makepad |
| [CHECKLIST.md](CHECKLIST.md) | P0-P3 refactoring roadmap (all complete) |

## 🔧 Technology Stack

- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[Makepad](https://github.com/makepad/makepad)** - GPU-accelerated UI framework
- **[CPAL](https://github.com/RustAudio/cpal)** - Cross-platform audio I/O
- **[Tokio](https://tokio.rs/)** - Async runtime
- **[Serde](https://serde.rs/)** - Serialization framework

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Test thoroughly (`cargo test`, `cargo build`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📝 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

```
Copyright 2026 MoFA Studio Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0
```

## 🙏 Acknowledgments

- **[Makepad](https://github.com/makepad/makepad)** - For the incredible GPU-accelerated UI framework
- **[Dora Robotics Framework](https://github.com/dora-rs/dora)** - Original inspiration for voice chat architecture
- **Rust Community** - For excellent tooling and libraries

## 📧 Contact

- **Repository**: https://github.com/mofa-org/mofa-studio
- **Issues**: https://github.com/mofa-org/mofa-studio/issues

---

*Built with ❤️ using Rust and Makepad*
