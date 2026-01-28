# Claude Code Instructions for MoFA Studio Development

> 📖 **AI 辅助开发指南** - 优化 Vibe Coding 体验

本文档为 Claude Code 提供 MoFA Studio 项目开发的快速参考和资源地图。

---

## 🚀 快速导航

- **核心开发指南**: @vibecoding.md - 完整的 Vibe Coding 开发流程和最佳实践
- **mofa-cast 应用文档**: `apps/mofa-cast/docs/` - 详细的应用开发文档
  - @IMPLEMENTATION_STATUS.md - 功能实现清单（基于代码审查）
  - @ARCHITECTURE.md - 技术架构设计
  - @DEVELOPMENT.md - 开发工作流程
  - @USER_GUIDE.md - 用户使用手册
- **Makepad 速查**: @MAKEPAD_QUICK_REF.md - Makepad 组件和模式快速参考

---

## 📍 Makepad 资源地图

### 本地安装位置

Makepad 已通过 Cargo Git 依赖安装到系统：

```bash
# 主目录
~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/

# 关键子目录
~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/widgets/    # UI 组件库
~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/examples/  # 示例代码
~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/draw/      # 绘图 API
~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/platform/  # 平台支持
```

**查看 Makepad 版本信息:**
```bash
cat Cargo.toml | grep makepad
# 当前版本: makepad-widgets = { git = "https://github.com/wyeworks/makepad", rev = "b8b65f4fa" }
```

### 关键示例项目（推荐学习顺序）

#### 1. **ui_zoo** - 🌟 最重要
**路径**: `~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/examples/ui_zoo/`

**用途**: 所有 Makepad UI 组件的完整展示和参考

**关键文件**:
- `src/tab_button.rs` - 按钮组件示例
- `src/tab_text_input.rs` - 文本输入框
- `src_tab_slider.rs` - 滑块控件
- `src/tab_dropdown.rs` - 下拉菜单
- `src/tab_scrollbar.rs` - 滚动条
- `src/tab_view.rs` - 视图布局
- `src/tab_label.rs` - 文本标签
- `src/tab_image.rs` - 图片显示
- `src/...` (30+ 个组件示例)

**何时参考**: 需要使用任何 UI 组件时，先在这里找示例

#### 2. **simple** - 入门示例
**路径**: `~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/examples/simple/`

**用途**: 理解基础应用结构和事件处理

**何时参考**: 创建新的最小化应用时

#### 3. **ironfish** - 完整应用
**路径**: `~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/examples/ironfish/`

**用途**: 学习复杂应用架构、音频处理、实时 UI 更新

**何时参考**: 构建复杂功能、音频相关应用时

#### 4. **其他有用示例**
- `examples/slides/` - 幻灯片展示
- `examples/layout/` - 布局系统
- `examples/text_flow/` - 文本流处理
- `examples/markdown/` - Markdown 渲染

### Makepad Widgets 组件库

**可用组件**（位于 `widgets/src/`）:

```rust
// 基础组件
use makepad_widgets::*;
// 可用组件:
//   button, label, image, icon, slider, text_input
//   check_box, radio_button, drop_down, popup_menu

// 布局组件
//   view, scroll_bar, splitter, stack_navigation
//   portal_list, slide_panel, expandable_panel

// 高级组件
//   dock, tab_bar, tab_close_button
//   color_picker, file_tree, slides_view
//   web_view, video, keyboard_view

// 数据展示
//   markdown, html, text_flow
//   multi_image, rotated_image, image_blend

// 反馈组件
//   modal, tooltip, popup_notification
//   loading_spinner
```

**组件源码位置**:
```bash
~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/widgets/src/
```

---

## 🛠️ MoFA Studio 项目结构

### 工作空间（Workspace）

```
mofa-studio/
├── mofa-studio-shell/        # 主程序（外壳）
├── mofa-widgets/             # 共享组件库
├── mofa-dora-bridge/         # Dora 数据流集成
├── apps/                     # 应用插件目录
│   ├── mofa-fm/             # 语音对话应用
│   ├── mofa-settings/       # 提供商配置应用
│   ├── mofa-debate/         # 辩论游戏应用
│   └── mofa-cast/           # 播客生成应用 ← 主要开发目标
└── Cargo.toml               # 工作空间配置
```

### mofa-cast 应用架构

```
apps/mofa-cast/
├── src/
│   ├── lib.rs                  # App 描述符和导出
│   ├── screen/
│   │   ├── mod.rs              # 屏幕模块定义
│   │   ├── main.rs             # 主屏幕 UI (~1580 行)
│   │   └── design.rs           # 设计模式屏幕（TODO）
│   ├── transcript_parser.rs    # 脚本解析器
│   ├── tts_batch.rs            # 批量 TTS 合成
│   ├── audio_mixer.rs          # 音频混音和导出
│   ├── dora_integration.rs     # Dora 数据流集成
│   ├── dora_process_manager.rs # Dora 进程管理
│   ├── recent_files.rs         # 最近文件管理
│   └── script_templates.rs     # 脚本模板
├── dataflow/                   # Dora 数据流配置
│   ├── multi-voice-batch-tts.yml  # 多语音 TTS（主配置）
│   ├── test-primespeech-simple.yml
│   └── batch-tts.yml
├── test_samples/              # 测试样本文件
└── Cargo.toml
```

### mofa-widgets 共享组件库

```
mofa-widgets/
├── src/
│   ├── lib.rs                  # 库入口
│   ├── app.rs                  # MofaApp trait 定义
│   ├── theme.rs                # 主题系统（颜色、字体）
│   ├── waveform_view.rs        # 波形图组件
│   ├── participant_panel.rs    # 参与者面板
│   ├── led_gauge.rs            # LED 条形图
│   └── log_panel.rs            # 日志面板
└── Cargo.toml
```

---

## 🎯 MoFA-Cast 开发快速参考

### 已实现功能概览

**脚本处理** ✅
- 多格式导入: PlainText, JSON, Markdown
- 自动格式检测
- 脚本模板: 2人访谈、3人讨论、叙事
- 内置编辑器 + 外部编辑器集成
- 自动文件变更检测
- 最近文件管理（最多5个，持久化）

**TTS 合成** ✅
- Dora 数据流集成
- **PrimeSpeech TTS 引擎** (多语音支持)
- 多种中文声音: Luo Xiang (主持人), Yang Mi (女声), Ma Yun (男声), Ma Baoguo (特色)
- 智能说话人映射
- 批量并行合成
- 实时进度显示
- 音频段自动保存

**音频导出** ✅
- WAV 格式（无损）
- MP3 格式（4种比特率: 128/192/256/320 kbps）
- 音频混音（0.5秒静音间隔）
- 音量标准化（EBU R128 -14dB）
- 元数据嵌入

**UI/UX** ✅
- 说话人列表和颜色编码
- 系统日志面板（带级别过滤）
- 亮暗主题支持
- 实时状态更新
- 音频播放控制

### 常用开发命令

```bash
# 构建 mofa-cast
cd apps/mofa-cast
cargo build

# 运行完整 shell（包含所有 apps）
cargo run --bin mofa-studio

# 运行测试
cargo test --package mofa-cast

# 查看日志
RUST_LOG=debug cargo run --bin mofa-studio
```

### 环境变量

```bash
# Rust 日志级别
RUST_LOG=error        # 仅错误
RUST_LOG=warn         # 警告及以上
RUST_LOG=info         # 信息及以上（推荐）
RUST_LOG=debug        # 调试信息
RUST_LOG=trace        # 详细跟踪
```

**注意**: mofa-cast 使用 **PrimeSpeech TTS** 引擎（在数据流配置中设置），不需要环境变量切换。

---

## 🎨 Makepad 开发模式

### live_design! 宏

```rust
live_design! {
    use makepad_widgets::*;

    // 使用共享主题
    use mofa_widgets::theme::FONT_REGULAR;
    use mofa_widgets::theme::DARK_BG;
    use mofa_widgets::theme::TEXT_PRIMARY;

    pub MyScreen = {{MyScreen}} {
        width: Fill, height: Fill
        flow: Down
        padding: 20

        show_bg: true
        draw_bg: { color: (DARK_BG) }

        // 标题
        <View> {
            width: Fill, height: Fit
            margin: {bottom: 20}

            <Label> {
                text: "Hello, Makepad!"
                draw_text: {
                    text_style: <FONT_REGULAR> { font_size: 18.0 }
                    fn get_color(self) -> vec4 { (TEXT_PRIMARY) }
                }
            }
        }

        // 按钮
        <Button> {
            text: "Click Me"
            icon: IconId(MofaIconPlay)
        }
    }
}
```

### 事件处理

```rust
impl Widget for CastScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // 匹配按钮点击
        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => return,
        };

        // 按钮 ID 来自 live_design! 中的定义
        if self.button(ids!(my_button)).clicked(actions) {
            // 处理点击
            self.handle_button_click(cx);
        }

        // 下拉菜单
        if let Some(selected) = self.drop_down(ids!(my_dropdown)).selected(actions) {
            // 处理选择
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
```

### 主题和样式

**使用共享主题**:
```rust
// 在 live_design! 中导入
use mofa_widgets::theme::*;
use mofa_widgets::theme::FONT_REGULAR;
use mofa_widgets::theme::DARK_BG;
use mofa_widgets::theme::TEXT_PRIMARY;
use mofa_widgets::theme::ACCENT_BLUE;

// 定义颜色
fn get_color(self) -> vec4 {
    match self.color_theme {
        ColorTheme::Light => vec3(1.0, 1.0, 1.0),
        ColorTheme::Dark => vec3(0.1, 0.1, 0.1),
    }
}
```

**常用颜色常量**（定义在 `mofa-widgets/src/theme.rs`）:
- `DARK_BG`: 深色背景 (#1a1a1a)
- `TEXT_PRIMARY`: 主要文本
- `TEXT_SECONDARY`: 次要文本
- `ACCENT_BLUE`: 蓝色强调
- `ACCENT_GREEN`: 绿色成功
- `ACCENT_RED`: 红色错误

---

## 📚 学习资源

### 官方资源

- **Makepad Discord**: https://discord.gg/adqBRq7Ece
- **Makepad GitHub**: https://github.com/wyeworks/makepad
- **Makepad 文档**: https://makepad.nl/

### 项目内资源

- **vibecoding.md** - 完整开发流程和最佳实践
- **ARCHITECTURE.md** - MoFA Studio 架构设计
- **MOFA_DORA_ARCHITECTURE.md** - Dora 集成架构
- **各应用源码** - 最佳实践参考

### 快速查找代码

```bash
# 查找特定组件用法
grep -r "Button" apps/mofa-fm/src/

# 查找事件处理模式
grep -r "handle_event" apps/mofa-cast/src/

# 查找 live_design! 模式
grep -r "live_design!" mofa-widgets/src/
```

---

## 🔧 调试技巧

### 查看 Makepad 组件源码

```bash
# 查看组件实现
less ~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/widgets/src/button.rs

# 查看示例用法
less ~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/examples/ui_zoo/src/tab_button.rs
```

### 日志调试

```rust
// 使用 Makepad 日志
use makepad_widgets::log;

log!("Info message");
log::warn!("Warning message");
log::error!("Error message");

// 在 UI 中显示日志
self.add_log(cx, "[INFO] Something happened");
```

### 运行 Makepad 示例

```bash
# 运行 ui_zoo 查看所有组件
cd ~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/examples/ui_zoo
cargo run

# 运行其他示例
cd ~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/examples/simple
cargo run
```

---

## ✅ 开发工作流

### 添加新功能的步骤

1. **查看现有示例**
   ```bash
   # 在 ui_zoo 中找类似组件
   ls ~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/examples/ui_zoo/src/
   ```

2. **参考项目内类似实现**
   ```bash
   # 在 mofa-fm 或 mofa-cast 中搜索
   grep -r "similar_feature" apps/
   ```

3. **复制模式并修改**
   - 复制 `live_design!` 结构
   - 复制事件处理逻辑
   - 修改为你的需求

4. **测试**
   ```bash
   cargo build
   cargo run --bin mofa-studio
   ```

### 常见问题

**Q: 如何使用某个 Makepad 组件？**
A: 先查看 `ui_zoo` 示例，再查看组件源码

**Q: 如何实现某个 UI 效果？**
A: 在 `examples/` 目录中搜索类似效果，或参考现有应用

**Q: 如何调试 Makepad 宏错误？**
A: 检查 `live_design!` 语法，查看 `examples/` 中的正确用法

**Q: 如何查看所有可用组件？**
A: 运行 `ui_zoo` 示例，或查看 `widgets/src/` 目录

---

## 📝 总结

本文档提供了 MoFA Studio 开发的核心资源地图：

1. **Makepad 资源**: 快速找到本地安装的 Makepad 源码和示例
2. **项目结构**: 理解 mofa-studio 和 mofa-cast 的组织方式
3. **开发模式**: 常用的 Makepad 代码模式和最佳实践
4. **快速查找**: 如何快速找到需要的代码和示例

**Vibe Coding 的关键**: 减少上下文切换，直接指向本地资源，快速找到可复用模式。

---

**更新日期**: 2025-01-21
**Makepad 版本**: b8b65f4fa
**维护者**: Claude Code Assistant
