# MoFA Studio Vibe Coding 开发指南

> 基于 MoFA Studio 框架的规范性开发流程与最佳实践

**版本**: 1.0
**日期**: 2026-01-08
**目标读者**: 使用 MoFA Studio 框架进行 AI 应用开发的开发者

---

## 第一部分：项目架构分析

### 1.1 项目概览

MoFA Studio 是一个**模块化、插件式**的桌面应用框架，基于 Rust 和 Makepad UI 框架构建。它的核心设计理念是：

```
┌─────────────────────────────────────────────────────────────┐
│                    MoFA Studio 架构                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        mofa-studio-shell (外壳程序)                   │  │
│  │  - 窗口管理、导航、主题切换                            │  │
│  │  - App 协调与生命周期管理                              │  │
│  │  - 提供统一的运行时环境                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                           ↕                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        mofa-widgets (共享组件库)                      │  │
│  │  - MofaApp Trait (插件接口)                           │  │
│  │  - 主题系统 (字体、颜色、亮暗模式)                     │  │
│  │  - 通用组件 (波形图、LED条、日志面板)                 │  │
│  └──────────────────────────────────────────────────────┘  │
│                           ↕                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        apps/ (应用插件目录)                          │  │
│  │  ┌─────────────┐  ┌──────────────┐                  │  │
│  │  │ mofa-fm     │  │ mofa-settings│                  │  │
│  │  │ 语音对话    │  │  提供商配置   │                  │  │
│  │  └─────────────┘  └──────────────┘                  │  │
│  │  ┌─────────────┐                                     │  │
│  │  │ your-app    │  ← 你要创建的新应用                  │  │
│  │  └─────────────┘                                     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 核心设计原则

#### 原则 1: 黑盒应用 (Black-Box Apps)
每个应用是**自包含**的，不依赖于外壳的具体实现：
- ✅ 应用通过 `MofaApp` trait 声明元数据
- ✅ 应用通过 `live_design!` 注册 UI 组件
- ❌ 应用不应直接依赖外壳的内部结构
- ❌ 应用之间不应有直接依赖

#### 原则 2: 编译时组件解析 (Compile-Time Widget Resolution)
由于 Makepad 的 `live_design!` 宏要求：
- 所有 widget 类型必须在**编译时**确定
- 无法在运行时动态加载应用
- Shell 需要在 `Cargo.toml` 中声明依赖的应用
- 每个新应用需要重新编译整个 shell

#### 原则 3: 本地状态所有权 (Local State Ownership)
遵循 Makepad 的设计哲学：
- 每个 Widget 拥有自己的状态（`#[rust]` 字段）
- 不使用全局状态存储（Redux/Zustand 模式不适用）
- 父组件通过 `WidgetRef` 方法控制子组件
- 通过 `Arc<Mutex<T>>` 或 `channel` 实现跨组件通信

#### 原则 4: Dora 数据流集成 (Dora Dataflow Integration)
对于 AI/语音应用：
- 使用 Dora 作为后端数据流编排引擎
- 通过 Bridge 模式连接 UI 和 Dora 节点
- 支持实时音频流、LLM 推理、TTS 合成

---

## 第二部分：Vibe Coding 开发流程

### 2.1 开发前准备

#### 步骤 1: 环境检查

```bash
# 确认 Rust 版本
rustc --version  # 应该 >= 1.70

# 确认项目可以编译
cargo build --release

# 运行一次确认环境正常
./target/release/mofa-studio
```

#### 步骤 2: 理解现有应用

在创建新应用前，先熟悉两个示例应用：

```bash
# 阅读 mofa-fm 源码
apps/mofa-fm/src/
├── lib.rs              # App 描述符
├── screen.rs           # 主屏幕 (~1360 行)
├── mofa_hero.rs        # 状态栏组件
├── audio.rs            # 音频管理
├── dora_integration.rs # Dora 集成层
└── audio_player.rs     # 音频播放器

# 阅读 mofa-settings 源码
apps/mofa-settings/src/
├── lib.rs                  # App 描述符
├── screen.rs               # 设置界面
├── providers_panel.rs      # 提供商列表
└── provider_view.rs        # 配置编辑
```

**学习重点**：
1. 如何实现 `MofaApp` trait
2. 如何组织 `screen.rs` 和子组件
3. 如何使用共享主题和组件
4. 如何处理用户输入和事件

### 2.2 创建新应用的标准化流程

#### 案例：文章转播客生成器 (Article to Podcast Generator)

**需求描述**：
- 输入：一篇文章的文本或 URL
- 输出：两个人对话的播客内容和音频
- 技术：LLM 生成对话脚本 + TTS 合成语音

#### 步骤 1: 创建应用骨架

```bash
cd apps
cargo new mofa-podcast --lib
```

#### 步骤 2: 配置 Cargo.toml

```toml
# apps/mofa-podcast/Cargo.toml
[package]
name = "mofa-podcast"
version = "0.1.0"
edition = "2021"

[dependencies]
makepad-widgets = { workspace = true }
mofa-widgets = { path = "../../mofa-widgets" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 如果需要与后端通信
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }

# 如果需要音频处理
cpal = "0.15"
```

#### 步骤 3: 实现 App 描述符

```rust
// apps/mofa-podcast/src/lib.rs
pub mod screen;
pub mod podcast_generator;  // 核心逻辑模块
pub mod audio_export;       // 音频导出模块

use makepad_widgets::Cx;
use mofa_widgets::{MofaApp, AppInfo};

/// 应用描述符 - 必须实现 MofaApp trait
pub struct MoFaPodcastApp;

impl MofaApp for MoFaPodcastApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Podcast Generator",
            id: "mofa-podcast",
            description: "Convert articles to engaging podcast conversations",
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}

/// 向后兼容的注册函数
pub fn live_design(cx: &mut Cx) {
    MoFaPodcastApp::live_design(cx);
}
```

#### 步骤 4: 设计主屏幕 UI

```rust
// apps/mofa-podcast/src/screen.rs
use makepad_widgets::*;
use mofa_widgets::theme::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    // 导入共享主题
    use mofa_widgets::theme::FONT_REGULAR;
    use mofa_widgets::theme::FONT_MEDIUM;
    use mofa_widgets::theme::DARK_BG;
    use mofa_widgets::theme::TEXT_PRIMARY;

    pub PodcastScreen = {{PodcastScreen}} {
        width: Fill, height: Fill
        flow: Down
        padding: 20

        show_bg: true
        draw_bg: { color: (DARK_BG) }

        // 标题区域
        <View> {
            width: Fill, height: Fit
            margin: {bottom: 30}

            <Label> {
                text: "Article to Podcast Generator"
                draw_text: {
                    text_style: <FONT_MEDIUM> { font_size: 28.0, height_factor: 1.2 }
                    fn get_color(self) -> vec4 { (TEXT_PRIMARY) }
                }
            }
        }

        // 输入区域
        <View> {
            width: Fill, height: Fit
            margin: {bottom: 20}

            <Label> {
                text: "Article Content or URL"
                draw_text: {
                    text_style: <FONT_REGULAR> { font_size: 14.0 }
                    fn get_color(self) -> vec4 { (TEXT_PRIMARY) }
                }
            }

            article_input = <TextInput> {
                width: Fill, height: 150
                text: ""
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                        sdf.fill(#3);
                        sdf.stroke(#4, 1.0);
                        return sdf.result;
                    }
                }
            }
        }

        // 配置区域
        <View> {
            width: Fill, height: Fit
            margin: {bottom: 20}
            flow: Right
            spacing: 20

            voice1_dropdown = <DropDown> {
                width: Fill, height: Fit
                labels: ["Voice 1: Host (Male)", "Voice 1: Host (Female)", "Voice 1: Narrator"]
            }

            voice2_dropdown = <DropDown> {
                width: Fill, height: Fit
                labels: ["Voice 2: Guest (Male)", "Voice 2: Guest (Female)", "Voice 2: Expert"]
            }

            style_dropdown = <DropDown> {
                width: Fill, height: Fit
                labels: ["Conversation", "Interview", "Debate", "Tutorial"]
            }
        }

        // 生成按钮
        generate_btn = <Button> {
            width: Fill, height: Fit
            margin: {bottom: 20}
            text: "Generate Podcast"

            draw_bg: {
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 8.0);
                    sdf.fill(#1a73e8);
                    return sdf.result;
                }
            }

            draw_text: {
                text_style: <FONT_MEDIUM> { font_size: 16.0 }
                fn get_color(self) -> vec4 { #fff }
            }
        }

        // 进度显示
        progress_view = <View> {
            width: Fill, height: Fit
            visible: false

            progress_label = <Label> {
                text: "Generating podcast script..."
                draw_text: {
                    text_style: <FONT_REGULAR> { font_size: 14.0 }
                    fn get_color(self) -> vec4 { (TEXT_PRIMARY) }
                }
            }

            progress_bar = <View> {
                width: Fill, height: 8
                show_bg: true
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                        sdf.fill(#2);
                        return sdf.result;
                    }
                }
            }
        }

        // 输出区域
        output_view = <View> {
            width: Fill, height: Fill
            visible: false

            script_output = <TextInput> {
                width: Fill, height: Fill
                text: ""
                read_only: true
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                        sdf.fill(#1a);
                        sdf.stroke(#3, 1.0);
                        return sdf.result;
                    }
                }
            }
        }

        // 导出按钮
        export_btn = <Button> {
            width: Fill, height: Fit
            visible: false
            text: "Export Audio"

            draw_bg: {
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 8.0);
                    sdf.fill(#34a853);
                    return sdf.result;
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct PodcastScreen {
    #[deref]
    view: View,

    // 应用状态（本地所有权）
    #[rust]
    article_content: String,

    #[rust]
    generated_script: Option<String>,

    #[rust]
    is_generating: bool,

    #[rust]
    generation_progress: f32,

    // 可选：音频播放器引用
    #[rust]
    audio_player: Option<AudioPlayer>,
}

// 实现事件处理和绘制逻辑
impl Widget for PodcastScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // 处理用户输入
        // ...
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // 绘制 UI
        // ...
    }
}
```

#### 步骤 5: 集成到 Shell

```rust
// mofa-studio-shell/Cargo.toml
[dependencies]
# ... 其他依赖
mofa-podcast = { path = "../../apps/mofa-podcast", optional = true }

[features]
default = ["mofa-fm", "mofa-settings", "mofa-podcast"]
```

```rust
// mofa-studio-shell/src/app.rs
use mofa_podcast::MoFaPodcastApp;

impl App {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        // 注册新应用
        self.app_registry.register(MoFaPodcastApp::info());
    }

    fn live_register(cx: &mut Cx) {
        // 注册 UI 组件
        <MoFaPodcastApp as MofaApp>::live_design(cx);
    }
}
```

#### 步骤 6: 实现核心逻辑模块

```rust
// apps/mofa-podcast/src/podcast_generator.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PodcastConfig {
    pub voice1: String,
    pub voice2: String,
    pub style: PodcastStyle,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PodcastStyle {
    Conversation,
    Interview,
    Debate,
    Tutorial,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DialogueLine {
    pub speaker: String,
    pub text: String,
    pub timestamp: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodcastScript {
    pub title: String,
    pub lines: Vec<DialogueLine>,
}

pub struct PodcastGenerator {
    api_client: reqwest::Client,
    api_endpoint: String,
}

impl PodcastGenerator {
    pub fn new(api_endpoint: String) -> Self {
        Self {
            api_client: reqwest::Client::new(),
            api_endpoint,
        }
    }

    /// 从文章生成播客脚本
    pub async fn generate_script(
        &self,
        article: &str,
        config: &PodcastConfig,
    ) -> Result<PodcastScript, Box<dyn std::error::Error>> {
        // 调用 LLM API 生成对话
        let request = serde_json::json!({
            "article": article,
            "voice1": config.voice1,
            "voice2": config.voice2,
            "style": config.style,
        });

        let response = self.api_client
            .post(&self.api_endpoint)
            .json(&request)
            .send()
            .await?;

        let script: PodcastScript = response.json().await?;
        Ok(script)
    }

    /// 将脚本转换为 TTS 指令
    pub fn prepare_tts_commands(&self, script: &PodcastScript) -> Vec<TTSCommand> {
        script.lines.iter().map(|line| {
            TTSCommand {
                text: line.text.clone(),
                voice: line.speaker.clone(),
                timestamp: line.timestamp,
            }
        }).collect()
    }
}

#[derive(Debug)]
pub struct TTSCommand {
    pub text: String,
    pub voice: String,
    pub timestamp: f32,
}
```

#### 步骤 7: 实现 Dora 集成（可选，如果需要实时音频）

如果需要实时播放和音频处理，参考 `mofa-fm` 的 Dora 集成模式：

```rust
// apps/mofa-podcast/src/dora_integration.rs
use mofa_dora_bridge::DynamicNodeDispatcher;

pub struct PodcastDoraIntegration {
    dispatcher: Option<DynamicNodeDispatcher>,
    dataflow_path: PathBuf,
}

impl PodcastDoraIntegration {
    pub fn new(dataflow_path: PathBuf) -> Self {
        Self {
            dispatcher: None,
            dataflow_path,
        }
    }

    pub async fn start_dataflow(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 启动 Dora 数据流
        let mut dispatcher = DynamicNodeDispatcher::new();
        dispatcher.start_dataflow(&self.dataflow_path).await?;
        self.dispatcher = Some(dispatcher);
        Ok(())
    }

    pub async fn send_script_to_tts(&self, script: &PodcastScript) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref dispatcher) = self.dispatcher {
            // 发送 TTS 命令到数据流
            for line in &script.lines {
                dispatcher.send_data("tts_input", &line.text, None).await?;
            }
        }
        Ok(())
    }
}
```

#### 步骤 8: 编写 Dora 数据流配置（可选）

```yaml
# apps/mofa-podcast/dataflow/podcast-generation.yml

nodes:
  # LLM 脚本生成节点
  - id: script-generator
    build: cargo build --release --manifest-path node-hub/llm-generator/Cargo.toml
    path: node-hub/llm-generator/target/release/llm-generator
    inputs:
      article: user-input/article
    outputs:
      - script
      - status
    env:
      LLM_API_KEY: ${OPENAI_API_KEY}
      MODEL: gpt-4

  # TTS 合成节点
  - id: tts-synthesizer
    build: pip install -e ../../../node-hub/dora-primespeech
    path: dora-primespeech
    inputs:
      text: script-generator/script
    outputs:
      - audio
      - status
    env:
      VOICE_NAME: "Luo Xiang"
      PRIMESPEECH_MODEL_DIR: $HOME/.dora/models/primespeech

  # 音频输出节点
  - id: podcast-output
    path: dynamic
    inputs:
      audio: tts-synthesizer/audio
    outputs:
      - podcast_file

# 连接定义
connections:
  - user-input/article -> script-generator/article
  - script-generator/script -> tts-synthesizer/text
  - tts-synthesizer/audio -> podcast-output/audio
```

### 2.3 开发最佳实践

#### 实践 1: 使用共享主题

```rust
// ❌ 不要：硬编码颜色和字体
draw_text: {
    text_style: { font_size: 14.0, font: "Manrope" }
    fn get_color(self) -> vec4 { #333 }
}

// ✅ 应该：使用主题系统
use mofa_widgets::theme::{FONT_REGULAR, TEXT_PRIMARY};

draw_text: {
    text_style: <FONT_REGULAR> { font_size: 14.0 }
    fn get_color(self) -> vec4 { (TEXT_PRIMARY) }
}
```

#### 实践 2: 状态管理

```rust
// ❌ 不要：使用共享状态（RwLock<Arc<GlobalState>>）

// ✅ 应该：本地状态 + Channel 通信
#[derive(Live, LiveHook, Widget)]
pub struct MyScreen {
    #[rust]
    local_state: MyLocalState,

    #[rust]
    event_sender: Sender<MyEvent>,
    #[rust]
    event_receiver: Receiver<MyEvent>,
}

impl Widget for MyScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // 处理本地事件
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                MyEvent::DataUpdated => self.update_ui(cx),
            }
        }
    }
}
```

#### 实践 3: 定时器管理

```rust
// ⚠️ 注意：Makepad 没有自动清理定时器
// 必须手动管理生命周期

impl MyScreenRef {
    pub fn start_timers(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_timer = cx.start_interval(0.1); // 100ms
        }
    }

    pub fn stop_timers(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow_mut() {
            if let Some(timer) = inner.update_timer.take() {
                cx.stop_timer(timer);
            }
        }
    }
}

// 在 Shell 中切换页面时调用
impl App {
    fn switch_to_podcast(&mut self, cx: &mut Cx) {
        self.podcast_screen.start_timers(cx);
    }

    fn switch_away_from_podcast(&mut self, cx: &mut Cx) {
        self.podcast_screen.stop_timers(cx);
    }
}
```

#### 实践 4: 组件化

```rust
// ✅ 好的实践：将大型 screen 拆分为多个子组件

// 主 screen
pub struct PodcastScreen {
    #[deref]
    view: View,

    // 子组件引用
    podcast_input: PodcastInput,
    script_editor: ScriptEditor,
    audio_preview: AudioPreview,
    export_dialog: ExportDialog,
}

// 子组件 1：输入区域
#[derive(Live, Widget)]
pub struct PodcastInput {
    #[deref]
    view: View,
    #[rust]
    article_url: String,
}

// 子组件 2：脚本编辑器
#[derive(Live, Widget)]
pub struct ScriptEditor {
    #[deref]
    view: View,
    #[rust]
    dialogue_lines: Vec<DialogueLine>,
}

// 子组件 3：音频预览
#[derive(Live, Widget)]
pub struct AudioPreview {
    #[deref]
    view: View,
    #[rust]
    audio_buffer: CircularAudioBuffer,
}
```

#### 实践 5: 错误处理

```rust
// ✅ 使用 Result 类型，优雅处理错误
pub async fn generate_podcast(&self, article: &str) -> Result<Podcast, GenerateError> {
    if article.is_empty() {
        return Err(GenerateError::EmptyInput);
    }

    let script = self.generator.generate_script(article, &self.config)
        .await
        .map_err(|e| GenerateError::LLMError(e.to_string()))?;

    Ok(Podcast { script })
}

// 在 UI 中显示错误
impl Widget for PodcastScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Some(btn) = generate_btn.clicked(actions) {
            match self.generate_podcast(&self.article_content).await {
                Ok(podcast) => self.show_podcast(cx, podcast),
                Err(e) => self.show_error(cx, &format!("生成失败: {}", e)),
            }
        }
    }
}
```

### 2.4 调试和测试

#### 调试日志

```rust
// 使用 log crate
use log::{info, warn, error};

impl Widget for PodcastScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        info!("PodcastScreen: handle_event called");

        if generate_btn.clicked(actions) {
            info!("Generate button clicked, article length: {}", self.article_content.len());
        }
    }
}

// 运行时启用日志
RUST_LOG=debug cargo run --release
```

#### 单元测试

```rust
// apps/mofa-podcast/tests/generator_test.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_script_generation() {
        let generator = PodcastGenerator::new("http://localhost:8000".to_string());
        let config = PodcastConfig {
            voice1: "Host".to_string(),
            voice2: "Guest".to_string(),
            style: PodcastStyle::Conversation,
        };

        let article = "This is a test article about AI.";
        let script = generator.generate_script(article, &config).await.unwrap();

        assert!(!script.lines.is_empty());
        assert_eq!(script.lines[0].speaker, "Host");
    }
}
```

---

## 第三部分：实战案例完整代码

### 3.1 完整的 PodcastScreen 实现

由于篇幅限制，这里提供核心结构：

```rust
// apps/mofa-podcast/src/screen.rs (完整版框架)

use makepad_widgets::*;
use mofa_widgets::theme::*;
use crate::podcast_generator::{PodcastGenerator, PodcastConfig, PodcastStyle};

live_design! {
    // ... UI 定义 (见上文)
}

#[derive(Live, LiveHook, Widget)]
pub struct PodcastScreen {
    #[deref]
    view: View,

    // 状态
    #[rust]
    article_content: String,
    #[rust]
    generated_script: Option<PodcastScript>,
    #[rust]
    is_generating: bool,
    #[rust]
    generation_progress: f32,

    // 核心
    #[rust]
    generator: Option<PodcastGenerator>,

    // 定时器
    #[rust]
    update_timer: Option<Timer>,
}

impl LiveHook for PodcastScreen {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        // 初始化生成器
        self.generator = Some(PodcastGenerator::new(
            std::env::var("LLM_API_ENDPOINT").unwrap_or_else(|_| "http://localhost:8000".to_string())
        ));
    }
}

impl Widget for PodcastScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) -> Unit {
        // 事件处理逻辑
        // ...
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // 绘制逻辑
        // ...
    }
}

// 定时器控制接口（供 Shell 调用）
impl PodcastScreenRef {
    pub fn start_timers(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_timer = Some(cx.start_interval(0.1));
        }
    }

    pub fn stop_timers(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            if let Some(timer) = inner.update_timer.take() {
                cx.stop_timer(timer);
            }
        }
    }
}
```

### 3.2 构建和运行

```bash
# 1. 添加到 workspace
# 编辑 Cargo.toml (workspace root)
[workspace.dependencies]
# ... 现有依赖

# 2. 添加到 shell
# mofa-studio-shell/Cargo.toml
[dependencies]
mofa-podcast = { path = "../../apps/mofa-podcast", optional = true }

[features]
default = ["mofa-fm", "mofa-settings", "mofa-podcast"]

# 3. 重新编译
cargo build --release

# 4. 运行
./target/release/mofa-studio
```

---

## 第四部分：高级主题

### 4.1 与后端 AI 服务集成

#### 方案 A: HTTP API（推荐用于简单场景）

```rust
use reqwest::Client;
use serde_json::json;

pub async fn call_llm_api(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", std::env::var("OPENAI_API_KEY")?))
        .json(&json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    Ok(json["choices"][0]["message"]["content"].as_str().unwrap().to_string())
}
```

#### 方案 B: WebSocket（推荐用于流式输出）

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub async fn stream_llm_response(prompt: &str) -> Result<impl Stream<Item = Result<String, Error>>, Error> {
    let (ws_stream, _) = connect_async("ws://localhost:8000/stream").await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    ws_sender.send(Message::Text(prompt.to_string())).await?;

    Ok(ws_receiver.map(|msg| {
        match msg? {
            Message::Text(text) => Ok(text),
            _ => Err(Error::new("Unexpected message")),
        }
    }))
}
```

#### 方案 C: Dora 数据流（推荐用于复杂 AI 流程）

参考 `mofa-fm` 的实现，使用 Dora 编排多个 AI 节点（LLM、TTS、ASR）。

### 4.2 音频处理

```rust
// 使用 mofa-widgets 的音频播放器
use mofa_widgets::audio_player::AudioPlayer;

impl PodcastScreen {
    fn play_podcast_audio(&mut self, cx: &mut Cx, audio_data: Vec<u8>) {
        if let Some(ref mut player) = self.audio_player {
            player.load_audio(cx, audio_data, 44100);
            player.play(cx);
        }
    }
}
```

### 4.3 导出功能

```rust
use std::fs::File;
use std::io::Write;

impl PodcastScreen {
    pub fn export_script(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref script) = self.generated_script {
            let json = serde_json::to_string_pretty(script)?;
            let mut file = File::create(path)?;
            file.write_all(json.as_bytes())?;
            Ok(())
        } else {
            Err("No script generated".into())
        }
    }

    pub fn export_audio(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // 导出为 WAV 或 MP3
        // ...
    }
}
```

---

## 第五部分：常见问题和解决方案

### Q1: 编译错误 "type not found"

**问题**：
```
error[E0412]: cannot find type `PodcastScreen` in `mofa_studio_shell::app`
```

**解决**：
1. 确保在 `mofa-studio-shell/src/app.rs` 中导入：
   ```rust
   use mofa_podcast::PodcastScreen;
   ```
2. 确保在 `live_register` 中注册：
   ```rust
   fn live_register(cx: &mut Cx) {
       mofa_podcast::live_design(cx);
   }
   ```

### Q2: 主题颜色不生效

**问题**：自定义颜色没有显示，或亮暗模式切换不正常。

**解决**：
```rust
// 确保使用主题提供的颜色常量
use mofa_widgets::theme::{TEXT_PRIMARY, DARK_BG, BORDER};

draw_bg: {
    fn pixel(self) -> vec4 {
        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
        sdf.fill((DARK_BG));  // 使用主题常量
        return sdf.result;
    }
}
```

### Q3: 定时器导致 CPU 占用高

**问题**：应用切换后，定时器仍在运行。

**解决**：
```rust
// 在 Shell 中正确管理定时器生命周期
impl App {
    fn handle_app_switch(&mut self, cx: &mut Cx, from_app: &str, to_app: &str) {
        match from_app {
            "mofa-podcast" => self.podcast_screen.stop_timers(cx),
            _ => {}
        }

        match to_app {
            "mofa-podcast" => self.podcast_screen.start_timers(cx),
            _ => {}
        }
    }
}
```

### Q4: Bridge 与 Dora 节点连接失败

**问题**：`failed to init event stream`

**解决**：
1. 确保 Dora daemon 和 coordinator 正在运行
2. 确保数据流已启动：`dora start dataflow/xxx.yml`
3. 确保节点 ID 正确匹配：
   ```rust
   let node = DoraNode::init_from_node_id("mofa-podcast-node")?;
   ```

---

## 第六部分：检查清单

在提交新应用前，请确认：

### 代码质量
- [ ] 所有公共 API 有文档注释
- [ ] 错误处理完善（使用 `Result` 类型）
- [ ] 无 `unwrap()` 或 `expect()`（除测试代码）
- [ ] 无 `todo!()` 或 `unimplemented!()` 宏
- [ ] 日志记录适当（`log::info`, `log::error`）

### UI/UX
- [ ] 所有文本使用主题字体
- [ ] 亮暗模式都能正常工作
- [ ] 按钮有 hover 和 active 状态
- [ ] 加载状态有进度指示
- [ ] 错误消息友好且清晰

### 性能
- [ ] 定时器正确启动和停止
- [ ] 无内存泄漏（使用 valgrind 检查）
- [ ] 大文本输入不卡顿（使用分页或虚拟滚动）

### 集成
- [ ] 实现了 `MofaApp` trait
- [ ] 在 Shell 中正确注册
- [ ] 在侧边栏中显示图标和名称
- [ ] 切换页面时状态正确保存/恢复

### 文档
- [ ] README.md 说明用途和使用方法
- [ ] API 文档（`cargo doc` 生成完整）
- [ ] 示例配置文件（如有）

---

## 附录 A：参考资源

### 官方文档
- [Makepad 文档](https://github.com/makepad/makepad)
- [Dora 文档](https://dora.cesko.cz/docs/)
- [Rust 书籍](https://doc.rust-lang.org/book/)

### 项目内部文档
- [ARCHITECTURE.md](ARCHITECTURE.md) - 架构详解
- [APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md) - 开发指南
- [STATE_MANAGEMENT_ANALYSIS.md](STATE_MANAGEMENT_ANALYSIS.md) - 状态管理
- [MOFA_DORA_ARCHITECTURE.md](MOFA_DORA_ARCHITECTURE.md) - Dora 集成

### 示例代码
- `apps/mofa-fm/src/` - 完整的语音对话应用
- `apps/mofa-settings/src/` - 配置管理应用
- `mofa-widgets/src/` - 共享组件库

---

## 附录 B：快速参考卡片

### 创建新应用 5 步法

```bash
# 1. 创建骨架
cd apps && cargo new my-app --lib

# 2. 配置依赖
# 编辑 Cargo.toml，添加 makepad-widgets, mofa-widgets

# 3. 实现 Trait
# impl MofaApp for MyApp { fn info() -> AppInfo { ... } }

# 4. 设计 UI
# live_design! { pub MyAppScreen = {{MyAppScreen}} { ... } }

# 5. 集成 Shell
# 修改 mofa-studio-shell/Cargo.toml 和 app.rs
```

### 关键代码片段

```rust
// 1. 导入主题
use mofa_widgets::theme::{FONT_REGULAR, TEXT_PRIMARY, DARK_BG};

// 2. 定义 Widget
#[derive(Live, LiveHook, Widget)]
pub struct MyApp {
    #[deref] view: View,
    #[rust] state: MyState,
}

// 3. 实现 Widget trait
impl Widget for MyApp {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) { }
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep { }
}

// 4. 定时器控制
impl MyAppRef {
    pub fn start_timers(&self, cx: &mut Cx) { }
    pub fn stop_timers(&self, cx: &mut Cx) { }
}
```

---

**文档版本**: 1.0
**最后更新**: 2026-01-08
**维护者**: MoFA Studio Team

---

*Happy Vibe Coding! 🚀*
