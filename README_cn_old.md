# MoFA Studio

> 基于 Rust 和 Makepad 构建的 AI 驱动桌面语音聊天应用

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)

MoFA Studio 是一个现代化的、GPU 加速的桌面应用程序，用于 AI 语音聊天和模型管理。它完全使用 Rust 语言构建，采用 [Makepad](https://github.com/makepad/makepad) UI 框架，提供了美观、响应式的界面和原生性能。

![MoFA Studio](mofa-studio-shell/resources/mofa-logo.png)

## ✨ 特性

- **🎨 美观的 UI** - GPU 加速渲染，流畅的动画效果
- **🌓 深色模式** - 无缝切换亮色/深色主题，带有动画过渡效果
- **🎙️ 音频管理** - 实时麦克风监控和设备选择
- **🔌 模块化架构** - 基于插件的应用系统，易于扩展
- **⚙️ 服务商配置** - 管理多个 AI 服务提供商（OpenAI、DeepSeek、阿里云）
- **📊 实时指标** - CPU、内存和音频缓冲区监控
- **🚀 原生性能** - 使用 Rust 构建，实现最高效率

## 🏗️ 架构

MoFA Studio 使用模块化的工作空间结构：

```
mofa-studio/
├── mofa-studio-shell/      # 主应用程序外壳
├── mofa-widgets/           # 共享的可复用组件
└── apps/
    ├── mofa-fm/            # 语音聊天界面
    └── mofa-settings/      # 服务商配置
```

### 核心设计原则

- **插件系统** - 应用实现 `MofaApp` trait 以进行标准化集成
- **黑盒应用** - 应用是自包含的，无外壳耦合
- **主题系统** - 集中化的颜色和字体管理
- **Makepad 原生** - 利用 Makepad 的 GPU 加速即时模式 UI

详见 [ARCHITECTURE.md](ARCHITECTURE.md) 了解详细的系统设计。

## 🚀 快速开始

### 前置要求

- **Rust** 1.70+ (2021 edition)
- **Cargo** 包管理器
- **Git** 用于克隆仓库

### 构建和运行

```bash
# 克隆仓库
git clone https://github.com/YOUR_ORG/mofa-studio.git
cd mofa-studio

# 以 release 模式构建
cargo build --release

# 运行应用程序
cargo run --release
```

应用程序窗口默认以 1400x900 像素打开。

### 开发构建

```bash
# 快速调试构建
cargo build

# 运行并启用调试日志
RUST_LOG=debug cargo run
```

## 📦 项目结构

MoFA Studio 组织为一个包含 5 个 crate 的 Cargo 工作空间：

| Crate | 类型 | 描述 |
|-------|------|-------------|
| `mofa-studio-shell` | 二进制 | 主应用程序外壳，包含窗口框架和导航 |
| `mofa-widgets` | 库 | 共享的 UI 组件（主题、音频播放器、波形图等） |
| `mofa-fm` | 库 | 语音聊天界面应用 |
| `mofa-settings` | 库 | 服务商配置应用 |

### 关键文件

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - 完整的系统架构指南
- **[APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md)** - 如何创建新应用
- **[STATE_MANAGEMENT_ANALYSIS.md](STATE_MANAGEMENT_ANALYSIS.md)** - 状态管理模式
- **[CHECKLIST.md](CHECKLIST.md)** - 重构路线图和完成状态

## 🎯 当前状态

MoFA Studio 目前是一个 **UI 原型**，具有可工作的组件：

### ✅ 已实现
- 完整的 UI 导航和主题系统
- 音频设备选择和监控
- 服务商配置持久化
- 带动画的深色/亮色模式
- 插件应用系统

### 🚧 计划中
- WebSocket 客户端用于 AI 服务集成
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

// 2. 创建你的屏幕组件
live_design! {
    pub MyAppScreen = {{MyAppScreen}} {
        width: Fill, height: Fill
        // 在这里编写你的 UI
    }
}
```

详见 [APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md) 获取分步说明。

## 📚 文档

| 文档 | 描述 |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | 系统架构、组件层级、最佳实践 |
| [APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md) | 创建应用、插件系统、深色模式支持 |
| [STATE_MANAGEMENT_ANALYSIS.md](STATE_MANAGEMENT_ANALYSIS.md) | 为什么 Redux/Zustand 不适用于 Makepad |
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
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 进行更改
4. 充分测试 (`cargo test`, `cargo build`)
5. 提交更改 (`git commit -m 'Add amazing feature'`)
6. 推送到分支 (`git push origin feature/amazing-feature`)
7. 打开 Pull Request

## 📝 许可证

本项目采用 Apache License 2.0 许可 - 详见 [LICENSE](LICENSE) 文件。

```
Copyright 2026 MoFA Studio Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0
```

## 🙏 致谢

- **[Makepad](https://github.com/makepad/makepad)** - 感谢令人惊叹的 GPU 加速 UI 框架
- **[Dora Robotics Framework](https://github.com/dora-rs/dora)** - 语音聊天架构的最初灵感来源
- **Rust 社区** - 感谢优秀的工具和库

## 📧 联系方式

- **仓库**: https://github.com/YOUR_ORG/mofa-studio
- **问题**: https://github.com/YOUR_ORG/mofa-studio/issues

---

*使用 Rust 和 Makepad 构建 ❤️*
