# MoFA Studio

> 基于 Rust 和 Makepad 构建的 AI 驱动桌面语音对话应用

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)

MoFA Studio 是一个现代化的、GPU 加速的桌面 AI 语音对话和模型管理应用程序。完全使用 Rust 语言和 [Makepad](https://github.com/makepad/makepad) UI 框架构建，提供美观、响应式的高性能界面。

![MoFA Studio](mofa-studio-shell/resources/mofa-logo.png)

## ✨ 特性

- **🎨 精美界面** - GPU 加速渲染，流畅动画
- **🌓 深色模式** - 带动画过渡的亮色/深色主题无缝切换
- **🎙️ 音频管理** - 实时麦克风监控和设备选择
- **🔌 模块化架构** - 基于插件的应用系统，易于扩展
- **⚙️ 提供商配置** - 管理多个 AI 服务提供商（OpenAI、DeepSeek、阿里云）
- **📊 实时指标** - CPU、内存和音频缓冲区监控
- **🚀 原生性能** - 使用 Rust 构建以实现最高效率

## 🏗️ 架构

MoFA Studio 采用模块化工作空间结构：

```
mofa-studio/
├── mofa-studio-shell/      # 主应用程序外壳
├── mofa-widgets/           # 共享可复用组件
└── apps/
    ├── mofa-fm/            # 语音对话界面
    └── mofa-settings/      # 提供商配置
```

### 核心设计原则

- **插件系统** - 应用实现 `MofaApp` trait 以进行标准化集成
- **黑盒应用** - 应用自包含，无外壳耦合
- **主题系统** - 集中式颜色和字体管理
- **Makepad Native** - 利用 Makepad 的 GPU 加速即时模式 UI

详见 [ARCHITECTURE.md](ARCHITECTURE.md) 了解详细的系统设计。

## 🚀 快速开始

### 前置要求

- **Rust** 1.70+ (2021 edition)
- **Cargo** 包管理器
- **Git** 用于克隆仓库

### 语音对话前置要求

要运行语音对话数据流，您需要设置 Python 环境并下载所需的 AI 模型。

#### 1. 环境设置

```bash
cd models/setup-local-models
./setup_isolated_env.sh
```

这将创建一个 conda 环境 `mofa-studio`，包含：
- Python 3.12
- PyTorch 2.2.0, NumPy 1.26.4, Transformers 4.45.0
- 所有语音对话 Python 节点（ASR、PrimeSpeech、文本分段器）

激活环境：

```bash
conda activate mofa-studio
python test_dependencies.py  # 验证安装
```

#### 2. 模型下载

```bash
cd models/model-manager

# ASR 模型（FunASR Paraformer + 标点）
python download_models.py --download funasr

# PrimeSpeech TTS（基础模型 + 声音）
python download_models.py --download primespeech

# 列出可用声音
python download_models.py --list-voices

# 下载特定声音
python download_models.py --voice "Luo Xiang"
```

模型存储位置：
| 位置 | 内容 |
|----------|----------|
| `~/.dora/models/asr/funasr/` | FunASR ASR 模型 |
| `~/.dora/models/primespeech/` | PrimeSpeech TTS 基础模型 + 声音 |

#### 3. API 密钥（可选）

对于 LLM 推理，在 MoFA 设置应用中设置您的 API 密钥或通过环境变量：

```bash
export OPENAI_API_KEY="your-key"
export DEEPSEEK_API_KEY="your-key"
export ALIBABA_CLOUD_API_KEY="your-key"
```

### 构建和运行

```bash
# 克隆仓库
git clone https://github.com/YOUR_ORG/mofa-studio.git
cd mofa-studio

# Release 模式构建
cargo build --release

# 运行应用
cargo run --release
```

应用窗口默认以 1400x900 像素打开。

### 开发构建

```bash
# 快速调试构建
cargo build

# 运行并启用调试日志
RUST_LOG=debug cargo run
```

### 构建应用特定数据流

MoFA Studio 使用 [Dora](https://github.com/dora-rs/dora) 进行语音对话数据流编排。每个应用都有自己的数据流配置。

```bash
# 导航到应用的数据流目录
cd apps/mofa-fm/dataflow

# 构建所有节点（Rust 和 Python）
dora build voice-chat.yml

# 启动数据流
dora start voice-chat.yml

# 检查运行中的数据流
dora list

# 停止数据流
dora stop <dataflow-id>
```

`node-hub/` 目录包含数据流使用的所有 Dora 节点：

| 节点 | 类型 | 描述 |
|------|------|-------------|
| `dora-maas-client` | Rust | 通过 MaaS API 进行 LLM 推理 |
| `dora-conference-bridge` | Rust | 参与者之间的文本路由 |
| `dora-conference-controller` | Rust | 轮流发言和策略管理 |
| `dora-primespeech` | Python | 多声音 TTS 合成 |
| `dora-text-segmenter` | Python | 用于 TTS 的文本分段 |
| `dora-asr` | Python | 语音识别（Whisper/FunASR） |
| `dora-common` | Python | 共享日志工具 |

## 📦 项目结构

MoFA Studio 组织为包含 5 个 crate 的 Cargo 工作空间：

| Crate | 类型 | 描述 |
|-------|------|-------------|
| `mofa-studio-shell` | 二进制 | 主应用程序外壳，包含窗口装饰和导航 |
| `mofa-widgets` | 库 | 共享 UI 组件（主题、音频播放器、波形等） |
| `mofa-fm` | 库 | 语音对话界面应用 |
| `mofa-settings` | 库 | 提供商配置应用 |

### 关键文件

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - 完整的系统架构指南
- **[APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md)** - 如何创建新应用
- **[STATE_MANAGEMENT_ANALYSIS.md](STATE_MANAGEMENT_ANALYSIS.md)** - 状态管理模式
- **[CHECKLIST.md](CHECKLIST.md)** - 重构路线图和完成状态

## 🎯 当前状态

MoFA Studio 目前是一个 **UI 原型**，具有可工作的组件：

### ✅ 已实现
- 完整的 UI 导航和主题
- 音频设备选择和监控
- 提供商配置持久化
- 带动画的深色/亮色模式
- 插件应用系统

### 🚧 计划中
- 用于 AI 服务集成的 WebSocket 客户端
- 实时 ASR（语音识别）集成
- 实时 TTS（文本转语音）集成
- LLM 聊天补全
- 实时对话流程

## 🛠️ 创建新应用

MoFA Studio 的插件系统使添加新功能变得简单：

```rust
// 1. 实现 MofaApp trait
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

// 2. 创建屏幕组件
live_design! {
    pub MyAppScreen = {{MyAppScreen}} {
        width: Fill, height: Fill
        // 您的 UI 代码
    }
}
```

详见 [APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md) 的分步说明。

## 📚 文档

| 文档 | 描述 |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | 系统架构、组件层次、最佳实践 |
| [APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md) | 创建应用、插件系统、深色模式支持 |
| [STATE_MANAGEMENT_ANALYSIS.md](STATE_MANAGEMENT_ANALYSIS.md) | 为什么 Redux/Zustand 在 Makepad 中不工作 |
| [CHECKLIST.md](CHECKLIST.md) | P0-P3 重构路线图（全部完成） |

## 🔧 技术栈

- **[Rust](https://www.rust-lang.org/)** - 系统编程语言
- **[Makepad](https://github.com/makepad/makepad)** - GPU 加速 UI 框架
- **[CPAL](https://github.com/RustAudio/cpal)** - 跨平台音频 I/O
- **[Tokio](https://tokio.rs/)** - 异步运行时
- **[Serde](https://serde.rs/)** - 序列化框架

## 🤝 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解指南。

### 开发设置

1. Fork 仓库
2. 创建特性分支（`git checkout -b feature/amazing-feature`）
3. 进行更改
4. 彻底测试（`cargo test`、`cargo build`）
5. 提交更改（`git commit -m 'Add amazing feature'`）
6. 推送到分支（`git push origin feature/amazing-feature`）
7. 打开 Pull Request

## 📝 许可证

本项目在 Apache License 2.0 下许可 - 详见 [LICENSE](LICENSE) 文件。

```
Copyright 2026 MoFA Studio Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0
```

## 🙏 致谢

- **[Makepad](https://github.com/makepad/makepad)** - 感谢不可思议的 GPU 加速 UI 框架
- **[Dora Robotics Framework](https://github.com/dora-rs/dora)** - 语音对话架构的原始灵感
- **Rust 社区** - 感谢出色的工具和库

## 📧 联系方式

- **仓库**: https://github.com/YOUR_ORG/mofa-studio
- **问题**: https://github.com/YOUR_ORG/mofa-studio/issues

---

*使用 Rust 和 Makepad 用 ❤️ 构建*
