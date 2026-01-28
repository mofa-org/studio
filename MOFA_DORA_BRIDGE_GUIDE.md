# MoFA Studio - Dora 数据流桥接实现详解

> **更新时间**: 2026-01-10
> **版本**: 0.1.0

---

## 目录

1. [架构概述](#架构概述)
2. [动态节点机制](#动态节点机制)
3. [核心组件详解](#核心组件详解)
4. [数据流向分析](#数据流向分析)
5. [如何为新 App 创建桥接](#如何为新-app-创建桥接)
6. [最佳实践](#最佳实践)
7. [故障排除](#故障排除)

---

## 架构概述

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                     MoFA App (Makepad UI)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ AudioPlayer  │  │ PromptInput  │  │ SystemLog    │     │
│  │   Widget     │  │   Widget     │  │   Widget     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                          ↕ (Channels)
┌─────────────────────────────────────────────────────────────┐
│                  mofa-dora-bridge (桥接层)                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              DynamicNodeDispatcher                    │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ │  │
│  │  │ AudioPlayer  │ │ PromptInput  │ │ SystemLog    │ │  │
│  │  │   Bridge     │ │   Bridge     │ │   Bridge     │ │  │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                          ↕ (DoraNode API)
┌─────────────────────────────────────────────────────────────┐
│                   Dora Dataflow (YAML)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  LLM Nodes   │  │  TTS Nodes   │  │  Controller  │     │
│  │ (Rust/Python)│  │  (Python)    │  │   (Rust)     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### 三层架构

| 层级 | 职责 | 技术 |
|------|------|------|
| **UI 层** | 用户交互、数据展示 | Makepad Widgets |
| **桥接层** | 数据转换、动态节点连接 | mofa-dora-bridge |
| **数据流层** | 业务逻辑、数据处理 | Dora Nodes (Rust/Python) |

---

## 动态节点机制

### 什么是动态节点?

**动态节点 (Dynamic Node)** 是 Dora 数据流中的一种特殊节点类型,它在**运行时**动态注册到数据流中,而不是在 YAML 配置中预定义路径。

#### 对比:静态节点 vs 动态节点

```yaml
# 静态节点 (预定义路径)
- id: my-static-node
  path: /absolute/path/to/node/executable
  inputs:
    data: upstream/output
  outputs:
    - result

# 动态节点 (运行时注册)
- id: mofa-audio-player
  path: dynamic  # 关键标识
  inputs:
    audio_student1: tts1/audio
    audio_student2: tts2/audio
  outputs:
    - buffer_status
    - session_start
```

### 动态节点的核心优势

1. **UI 直接连接**: Widget 可以直接作为数据流的一部分
2. **双向通信**: 既接收数据,也发送控制信号
3. **独立生命周期**: 每个动态节点有自己的连接状态
4. **细粒度控制**: 可以单独启动/停止特定 UI 组件的数据流

### Dora 动态节点 API

```rust
use dora_node_api::DoraNode;

// 通过节点 ID 初始化动态节点
let (mut node, mut events) = DoraNode::init_from_node_id(
    NodeId::from("mofa-audio-player".to_string())
)?;

// 事件循环
loop {
    match events.recv_timeout(Duration::from_millis(100)) {
        Some(Event::Input { id, data, metadata }) => {
            // 处理输入数据
        }
        Some(Event::Stop(_)) => break,
        None => {} // 超时继续
    }

    // 发送输出
    node.send_output(
        DataId::from("output_name".to_string()),
        metadata,
        data
    )?;
}
```

---

## 核心组件详解

### 1. DoraBridge Trait

**定义**: `mofa-dora-bridge/src/bridge.rs`

所有桥接器必须实现的核心接口:

```rust
pub trait DoraBridge: Send + Sync {
    /// 节点 ID (例如 "mofa-audio-player")
    fn node_id(&self) -> &str;

    /// 当前连接状态
    fn state(&self) -> BridgeState;

    /// 连接到数据流作为动态节点
    fn connect(&mut self) -> BridgeResult<()>;

    /// 断开连接
    fn disconnect(&mut self) -> BridgeResult<()>;

    /// 发送数据到 Dora 输出
    fn send(&self, output_id: &str, data: DoraData) -> BridgeResult<()>;

    /// 订阅桥接事件
    fn subscribe(&self) -> Receiver<BridgeEvent>;

    /// 期望的输入列表
    fn expected_inputs(&self) -> Vec<String>;

    /// 期望的输出列表
    fn expected_outputs(&self) -> Vec<String>;
}
```

#### 桥接状态机

```rust
pub enum BridgeState {
    Disconnected,  // 未连接
    Connecting,    // 连接中
    Connected,     // 已连接
    Disconnecting, // 断开中
    Error,         // 错误状态
}
```

#### 桥接事件

```rust
pub enum BridgeEvent {
    Connected,
    Disconnected,
    DataReceived {
        input_id: String,
        data: DoraData,
        metadata: EventMetadata,
    },
    Error(String),
    StateChanged(BridgeState),
}
```

### 2. DynamicNodeDispatcher

**定义**: `mofa-dora-bridge/src/dispatcher.rs`

调度器负责管理所有桥接器的生命周期:

```rust
pub struct DynamicNodeDispatcher {
    /// 数据流控制器
    controller: Arc<RwLock<DataflowController>>,

    /// 活跃的桥接器 (按 node_id 索引)
    bridges: HashMap<String, Box<dyn DoraBridge>>,

    /// Widget 绑定信息
    bindings: Vec<WidgetBinding>,

    /// 事件接收器
    event_receivers: HashMap<String, Receiver<BridgeEvent>>,
}
```

#### 核心方法

```rust
impl DynamicNodeDispatcher {
    /// 创建新的调度器
    pub fn new(controller: DataflowController) -> Self;

    /// 发现 YAML 中的 MoFA 节点
    pub fn discover_mofa_nodes(&self) -> Vec<MofaNodeSpec>;

    /// 为所有发现的节点创建桥接
    pub fn create_bridges(&mut self) -> BridgeResult<()>;

    /// 连接所有桥接到数据流
    pub fn connect_all(&mut self) -> BridgeResult<()>;

    /// 启动数据流并连接桥接
    pub fn start(&mut self) -> BridgeResult<String>;

    /// 停止数据流
    pub fn stop(&mut self) -> BridgeResult<()>;

    /// 轮询所有桥接的事件 (非阻塞)
    pub fn poll_events(&self) -> Vec<(String, BridgeEvent)>;
}
```

#### 启动流程

```rust
// 1. 解析 YAML 配置
let controller = DataflowController::new("dataflow.yml")?;

// 2. 创建调度器
let mut dispatcher = DynamicNodeDispatcher::new(controller);

// 3. 自动发现并创建桥接
dispatcher.create_bridges()?;

// 4. 启动数据流并连接
let dataflow_id = dispatcher.start()?;

// 等待数据流初始化
// (Dora 需要时间启动节点)
// 等待 TTS 模型加载
// (Python 节点需要加载 ML 模型)

// 5. 开始轮询事件
loop {
    for (node_id, event) in dispatcher.poll_events() {
        match event {
            BridgeEvent::DataReceived { input_id, data, .. } => {
                // 处理数据
            }
            // ...
        }
    }
    thread::sleep(Duration::from_millis(10));
}
```

### 3. AudioPlayerBridge 示例

**定义**: `mofa-dora-bridge/src/widgets/audio_player.rs`

这是一个完整的桥接实现示例:

```rust
pub struct AudioPlayerBridge {
    /// 节点 ID
    node_id: String,

    /// 连接状态
    state: Arc<RwLock<BridgeState>>,

    /// 事件通信
    event_sender: Sender<BridgeEvent>,
    event_receiver: Receiver<BridgeEvent>,

    /// 音频数据通道 (到 Widget)
    audio_sender: Sender<AudioData>,
    audio_receiver: Receiver<AudioData>,

    /// 缓冲区状态通道 (从 Widget)
    buffer_status_sender: Sender<f64>,
    buffer_status_receiver: Receiver<f64>,

    /// 工作线程控制
    stop_sender: Option<Sender<()>>,
    worker_handle: Option<thread::JoinHandle<()>>,
}
```

#### 后台线程事件循环

```rust
fn run_event_loop(
    node_id: String,
    state: Arc<RwLock<BridgeState>>,
    event_sender: Sender<BridgeEvent>,
    audio_sender: Sender<AudioData>,
    buffer_status_receiver: Receiver<f64>,
    stop_receiver: Receiver<()>,
) {
    // 初始化 Dora 动态节点
    let (mut node, mut events) = match DoraNode::init_from_node_id(
        NodeId::from(node_id.clone())
    ) {
        Ok(n) => n,
        Err(e) => {
            error!("Failed to init: {}", e);
            return;
        }
    };

    // 会话跟踪
    let mut session_start_sent_for: HashSet<String> = HashSet::new();

    // 事件循环
    loop {
        // 检查停止信号
        if stop_receiver.try_recv().is_ok() {
            break;
        }

        // 转发缓冲区状态 (UI -> Dora)
        while let Ok(status) = buffer_status_receiver.try_recv() {
            let _ = Self::send_buffer_status_to_dora(&mut node, status);
        }

        // 接收 Dora 事件
        match events.recv_timeout(Duration::from_millis(100)) {
            Some(Event::Input { id, data, metadata }) => {
                // 提取元数据
                let mut event_meta = EventMetadata::default();
                for (key, value) in metadata.parameters.iter() {
                    let string_value = match value {
                        Parameter::String(s) => s.clone(),
                        Parameter::Integer(i) => i.to_string(),
                        Parameter::Float(f) => f.to_string(),
                        // ... 其他类型
                    };
                    event_meta.values.insert(key.clone(), string_value);
                }

                // 处理音频输入
                if id.contains("audio") {
                    if let Some(audio_data) = Self::extract_audio(&data, &event_meta) {
                        // 发送到 UI
                        let _ = audio_sender.try_send(audio_data.clone());

                        // 发送 audio_complete 信号 (流控制)
                        let _ = Self::send_audio_complete(&mut node, &id, &event_meta);

                        // 发送 session_start 信号 (每个 question_id 一次)
                        if let Some(qid) = event_meta.get("question_id") {
                            if !session_start_sent_for.contains(qid) {
                                let _ = Self::send_session_start(&mut node, &id, &event_meta);
                                session_start_sent_for.insert(qid.to_string());
                            }
                        }
                    }
                }
            }
            Some(Event::Stop(_)) => break,
            None => {} // 超时继续
        }
    }
}
```

### 4. 数据类型系统

```rust
/// Dora 数据类型
pub enum DoraData {
    Audio(AudioData),
    Log(LogEntry),
    Chat(ChatMessage),
    Control(ControlCommand),
    Json(serde_json::Value),
    Raw(Vec<u8>),
}

/// 音频数据
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub participant_id: Option<String>,
    pub question_id: Option<String>,
}

/// 日志条目
pub struct LogEntry {
    pub level: LogLevel,
    pub node_id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// 聊天消息
pub struct ChatMessage {
    pub role: String,  // "student1", "tutor", etc.
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// 控制命令
pub struct ControlCommand {
    pub command: String,  // "start", "stop", "reset"
    pub params: HashMap<String, String>,
}
```

---

## 数据流向分析

### 完整的语音对话数据流

```
用户点击 "Start" (UI)
    ↓
DoraIntegration::start_dataflow()
    ↓
DataflowController::start()
    ↓
dora start voice-chat.yml (Dora CLI)
    ↓
[等待数据流初始化 - 2秒]
[等待 TTS 模型加载 - 3秒]
    ↓
DynamicNodeDispatcher::create_bridges()
    → 发现 YAML 中的 mofa-xxx 节点
    → 创建对应的 Bridge 实例
    ↓
DynamicNodeDispatcher::connect_all()
    → 每个 Bridge 在后台线程启动
    → 调用 DoraNode::init_from_node_id()
    → Bridge 状态变为 Connected
    ↓
UI 开始轮询事件
    ↓
[数据流运行中...]
```

### 音频数据流向详解

#### 1. LLM 生成文本

```
dora-maas-client (student1)
    ↓ output: text
bridge-to-student1 (根据 controller 信号路由)
    ↓ output: text
student1 (LLM) 输入
    ↓
[LLM 流式输出文本]
    ↓ output: text
multi-text-segmenter 输入
```

#### 2. 文本分段

```
multi-text-segmenter
    ├── 输入: student1/text
    ├── FIFO 队列
    ├── 句子分割
    ├── 等待 audio_complete 信号 (流控)
    ↓
    输出: text_segment_student1
```

#### 3. TTS 合成

```
multi-text-segmenter/text_segment_student1
    ↓
primespeech-student1 (TTS)
    ├── 加载模型 (PrimeSpeech/Kokoro)
    ├── 文本转音频
    ├── 添加元数据 (question_id, session_status)
    ↓
    输出: audio (带元数据)
```

#### 4. 音频播放

```
primespeech-student1/audio
    ↓ input: audio_student1
mofa-audio-player (动态节点)
    ├── AudioPlayerBridge 接收
    ├── 提取音频数据
    ├── 发送 session_start (每个 question_id 一次)
    ├── 发送 audio_complete (每块音频)
    ↓
AudioData 通过 channel 到 UI
    ↓
CircularAudioBuffer (UI)
    ↓
CPAL Stream → 扬声器
```

### 控制信号流向

#### session_start 信号

```
AudioPlayerBridge
    ├── 检测到新的 question_id
    ├── 第一次收到该 question_id 的音频
    ↓
node.send_output("session_start", metadata, data)
    ↓
conference-controller/session_start (输入)
    ├── 记录该 question_id 的音频已开始播放
    ├── 允许下一个说话者开始
    ↓
controller → bridge-to-student2/control
    ↓
student2 LLM 开始生成
```

#### audio_complete 信号

```
AudioPlayerBridge
    ├── 每收到一块音频
    ↓
node.send_output("audio_complete", metadata, data)
    ↓
multi-text-segmenter/audio_complete (输入)
    ├── FIFO 队列释放下一段
    ↓
text_segment_student2 → TTS
```

#### buffer_status 信号

```
CircularAudioBuffer (UI)
    ├── 每 50ms 检查缓冲区填充
    ↓
DoraIntegration::send_command(UpdateBufferStatus)
    ↓
AudioPlayerBridge
    ↓
node.send_output("buffer_status", percentage)
    ↓
conference-controller/buffer_status
multi-text-segmenter/audio_buffer_control
    ├── 低水位 (30%): 暂停发送
    ├── 高水位 (60%): 恢复发送
```

---

## 如何为新 App 创建桥接

### 步骤 1: 定义节点类型

在 `mofa-dora-bridge/src/lib.rs` 中添加:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MofaNodeType {
    // ... 现有类型

    /// 你的新节点类型
    MyCustomNode,
}

impl MofaNodeType {
    pub fn node_id(&self) -> &'static str {
        match self {
            // ... 现有类型
            MofaNodeType::MyCustomNode => "mofa-my-custom-node",
        }
    }

    pub fn from_node_id(node_id: &str) -> Option<Self> {
        match node_id {
            // ... 现有类型
            "mofa-my-custom-node" => Some(MofaNodeType::MyCustomNode),
            _ => None,
        }
    }
}
```

### 步骤 2: 创建桥接实现

创建 `mofa-dora-bridge/src/widgets/my_custom_node.rs`:

```rust
use crate::bridge::{BridgeEvent, BridgeState, DoraBridge};
use crate::data::{DoraData, EventMetadata};
use crate::error::{BridgeError, BridgeResult};
use crossbeam_channel::{bounded, Receiver, Sender};
use dora_node_api::{DoraNode, Event, IntoArrow};
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;

pub struct MyCustomNodeBridge {
    node_id: String,
    state: Arc<RwLock<BridgeState>>,
    event_sender: Sender<BridgeEvent>,
    event_receiver: Receiver<BridgeEvent>,
    stop_sender: Option<Sender<()>>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl MyCustomNodeBridge {
    pub fn new(node_id: &str) -> Self {
        let (event_tx, event_rx) = bounded(100);

        Self {
            node_id: node_id.to_string(),
            state: Arc::new(RwLock::new(BridgeState::Disconnected)),
            event_sender: event_tx,
            event_receiver: event_rx,
            stop_sender: None,
            worker_handle: None,
        }
    }

    fn run_event_loop(
        node_id: String,
        state: Arc<RwLock<BridgeState>>,
        event_sender: Sender<BridgeEvent>,
        stop_receiver: Receiver<()>,
    ) {
        // 初始化 Dora 节点
        let (mut node, mut events) = match DoraNode::init_from_node_id(
            NodeId::from(node_id)
        ) {
            Ok(n) => n,
            Err(e) => {
                error!("Failed to init: {}", e);
                *state.write() = BridgeState::Error;
                let _ = event_sender.send(BridgeEvent::Error(e.to_string()));
                return;
            }
        };

        *state.write() = BridgeState::Connected;
        let _ = event_sender.send(BridgeEvent::Connected);

        // 事件循环
        loop {
            // 检查停止信号
            if stop_receiver.try_recv().is_ok() {
                break;
            }

            // 接收 Dora 事件
            match events.recv_timeout(Duration::from_millis(100)) {
                Some(Event::Input { id, data, metadata }) => {
                    // 处理输入
                    self.handle_input(&mut node, id, data, metadata);
                }
                Some(Event::Stop(_)) => break,
                None => {}
            }
        }

        *state.write() = BridgeState::Disconnected;
        let _ = event_sender.send(BridgeEvent::Disconnected);
    }

    fn handle_input(
        &self,
        node: &mut DoraNode,
        id: DataId,
        data: dora_node_api::ArrowData,
        metadata: dora_node_api::Metadata,
    ) {
        // 提取元数据
        let mut event_meta = EventMetadata::default();
        for (key, value) in metadata.parameters.iter() {
            let string_value = match value {
                Parameter::String(s) => s.clone(),
                Parameter::Integer(i) => i.to_string(),
                // ... 处理所有类型
            };
            event_meta.values.insert(key.clone(), string_value);
        }

        // 处理数据
        match id.as_str() {
            "my_input" => {
                // 提取数据
                // 发送到 UI
                let _ = self.event_sender.send(BridgeEvent::DataReceived {
                    input_id: id.to_string(),
                    data: DoraData::Raw(data.to_vec()),
                    metadata: event_meta,
                });
            }
            _ => {}
        }
    }
}

impl DoraBridge for MyCustomNodeBridge {
    fn node_id(&self) -> &str {
        &self.node_id
    }

    fn state(&self) -> BridgeState {
        *self.state.read()
    }

    fn connect(&mut self) -> BridgeResult<()> {
        if self.is_connected() {
            return Err(BridgeError::AlreadyConnected);
        }

        *self.state.write() = BridgeState::Connecting;

        let (stop_tx, stop_rx) = bounded(1);
        self.stop_sender = Some(stop_tx);

        let node_id = self.node_id.clone();
        let state = Arc::clone(&self.state);
        let event_sender = self.event_sender.clone();

        let handle = thread::spawn(move || {
            Self::run_event_loop(node_id, state, event_sender, stop_rx);
        });

        self.worker_handle = Some(handle);
        Ok(())
    }

    fn disconnect(&mut self) -> BridgeResult<()> {
        if let Some(stop_tx) = self.stop_sender.take() {
            let _ = stop_tx.send(());
        }

        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }

        *self.state.write() = BridgeState::Disconnected;
        Ok(())
    }

    fn send(&self, output_id: &str, data: DoraData) -> BridgeResult<()> {
        // 实现发送逻辑
        Ok(())
    }

    fn subscribe(&self) -> Receiver<BridgeEvent> {
        self.event_receiver.clone()
    }

    fn expected_inputs(&self) -> Vec<String> {
        vec!["my_input".to_string()]
    }

    fn expected_outputs(&self) -> Vec<String> {
        vec!["my_output".to_string()]
    }
}

impl Drop for MyCustomNodeBridge {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}
```

### 步骤 3: 注册桥接

在 `mofa-dora-bridge/src/widgets/mod.rs` 中:

```rust
pub mod my_custom_node;

pub use my_custom_node::MyCustomNodeBridge;
```

在 `mofa-dora-bridge/src/dispatcher.rs` 中添加:

```rust
use crate::widgets::MyCustomNodeBridge;

impl DynamicNodeDispatcher {
    pub fn create_bridges(&mut self) -> BridgeResult<()> {
        // ... 现有代码

        for node_spec in mofa_nodes {
            let bridge: Box<dyn DoraBridge> = match node_spec.node_type {
                // ... 现有类型
                MofaNodeType::MyCustomNode => {
                    Box::new(MyCustomNodeBridge::new(&node_spec.id))
                }
            };

            // ... 注册代码
        }
    }
}
```

### 步骤 4: 在 YAML 中定义节点

```yaml
# dataflow/my-app.yml

nodes:
  # ... 其他节点

  - id: mofa-my-custom-node
    path: dynamic
    inputs:
      my_input:
        source: upstream-node/output
        queue_size: 1000
    outputs:
      - my_output
      - status
      - log
    env:
      LOG_LEVEL: "DEBUG"
```

### 步骤 5: 在 App 中集成

```rust
// apps/my-app/src/dora_integration.rs

use mofa_dora_bridge::{DynamicNodeDispatcher, DataflowController};
use crossbeam_channel::{Receiver, Sender};

pub struct MyAppDoraIntegration {
    dispatcher: Option<DynamicNodeDispatcher>,
    event_rx: Receiver<MyAppEvent>,
}

impl MyAppDoraIntegration {
    pub fn new() -> Self {
        let (event_tx, event_rx) = bounded(100);

        Self {
            dispatcher: None,
            event_rx,
        }
    }

    pub fn start_dataflow(&mut self, dataflow_path: PathBuf) -> Result<()> {
        // 创建数据流控制器
        let controller = DataflowController::new(&dataflow_path)?;

        // 创建调度器
        let mut dispatcher = DynamicNodeDispatcher::new(controller);

        // 启动 (自动发现并创建桥接)
        dispatcher.start()?;

        self.dispatcher = Some(dispatcher);
        Ok(())
    }

    pub fn poll_events(&self) -> Vec<MyAppEvent> {
        let mut events = Vec::new();

        if let Some(ref dispatcher) = self.dispatcher {
            for (node_id, bridge_event) in dispatcher.poll_events() {
                match bridge_event {
                    mofa_dora_bridge::BridgeEvent::DataReceived { input_id, data, .. } => {
                        // 转换为 App 事件
                        events.push(MyAppEvent::DataReceived { input_id, data });
                    }
                    // ... 处理其他事件
                }
            }
        }

        events
    }
}
```

### 步骤 6: 在 UI 中使用

```rust
// apps/my-app/src/screen.rs

impl Widget for MyAppScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // 轮询 Dora 事件
        if let Some(ref integration) = self.dora_integration {
            for event in integration.poll_events() {
                match event {
                    MyAppEvent::DataReceived { input_id, data } => {
                        // 更新 UI
                        self.update_ui(cx, data);
                    }
                }
            }
        }
    }
}
```

---

## 最佳实践

### 1. 元数据处理

**始终处理所有参数类型:**

```rust
for (key, value) in metadata.parameters.iter() {
    let string_value = match value {
        Parameter::String(s) => s.clone(),
        Parameter::Integer(i) => i.to_string(),  // question_id 是 Integer!
        Parameter::Float(f) => f.to_string(),
        Parameter::Bool(b) => b.to_string(),
        Parameter::ListInt(l) => format!("{:?}", l),
        Parameter::ListFloat(l) => format!("{:?}", l),
        Parameter::ListString(l) => format!("{:?}", l),
    };
    event_meta.values.insert(key.clone(), string_value);
}
```

### 2. 流控制

**使用 try_send() 避免阻塞:**

```rust
// ❌ 错误: 可能阻塞
audio_sender.send(audio_data)?;

// ✅ 正确: 非阻塞
if let Err(e) = audio_sender.try_send(audio_data) {
    warn!("Channel full, dropping data: {}", e);
}
```

### 3. 会话跟踪

**每个 question_id 只发送一次 session_start:**

```rust
let mut session_start_sent_for: HashSet<String> = HashSet::new();

if let Some(qid) = question_id {
    if !session_start_sent_for.contains(qid) {
        let _ = Self::send_session_start(node, input_id, &event_meta);
        session_start_sent_for.insert(qid.to_string());
    }
}
```

### 4. 错误处理

**使用 ? 传播错误,但记录日志:**

```rust
if let Err(e) = Self::send_session_start(node, input_id, &event_meta) {
    warn!("Failed to send session_start: {}", e);
    // 不要返回错误,继续处理
}
```

### 5. 资源清理

**实现 Drop trait:**

```rust
impl Drop for MyBridge {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}
```

### 6. 通道缓冲

**根据数据类型选择合适的缓冲大小:**

```rust
// 音频数据: 需要较大缓冲
let (audio_tx, audio_rx) = bounded(500);

// 控制信号: 小缓冲即可
let (control_tx, control_rx) = bounded(10);

// 事件通知: 中等缓冲
let (event_tx, event_rx) = bounded(100);
```

### 7. 超时处理

**使用 recv_timeout() 避免永久阻塞:**

```rust
match events.recv_timeout(Duration::from_millis(100)) {
    Some(event) => { /* 处理事件 */ }
    None => {} // 超时,继续循环
}
```

### 8. 日志级别

**使用适当的日志级别:**

```rust
error!("Critical error: {}", e);      // 错误
warn!("Unexpected condition: {}", x);  // 警告
info!("Connection established");        // 信息
debug!("Processing data: {:?}", data);  // 调试
```

---

## 故障排除

### 问题 1: 动态节点无法连接

**症状:**
```
Failed to init dora node: Connection refused
```

**原因:**
- 数据流未启动
- 节点 ID 不匹配
- Dora daemon 未运行

**解决方案:**
```rust
// 1. 确保数据流已启动
controller.start()?;

// 2. 等待数据流初始化
thread::sleep(Duration::from_secs(2));

// 3. 检查节点 ID 是否与 YAML 一致
assert_eq!(bridge.node_id(), "mofa-audio-player");

// 4. 检查 Dora daemon 状态
let status = dora_list()?;
```

### 问题 2: 对话在 N 轮后停止

**症状:**
- 前几轮正常
- 之后沉默

**原因:**
- `session_start` 未为新 question_id 发送
- 元数据提取不完整

**解决方案:**
```rust
// 确保处理 Integer 类型的 question_id
Parameter::Integer(i) => i.to_string(),  // 关键!

// 跟踪已发送的 question_id
let mut session_start_sent_for: HashSet<String> = HashSet::new();

if !session_start_sent_for.contains(qid) {
    send_session_start(...)?;
    session_start_sent_for.insert(qid.to_string());
}
```

### 问题 3: 音频缓冲区卡顿

**症状:**
- 音频断断续续
- LED 面板全亮

**原因:**
- 通道阻塞
- 缓冲区满

**解决方案:**
```rust
// 使用 try_send() 而非 send()
if let Err(e) = audio_sender.try_send(audio_data) {
    warn!("Audio channel full, dropping chunk: {}", e);
}

// 定期发送 buffer_status
thread::spawn(|| {
    loop {
        let fill = audio_player.buffer_fill_percentage();
        bridge.send_buffer_status(fill);
        thread::sleep(Duration::from_millis(50));
    }
});
```

### 问题 4: 元数据丢失

**症状:**
- `question_id` 为 None
- `participant_id` 未知

**原因:**
- 只处理了 String 类型的参数
- 未从 input_id 提取信息

**解决方案:**
```rust
// 1. 处理所有参数类型
let string_value = match value {
    Parameter::String(s) => s.clone(),
    Parameter::Integer(i) => i.to_string(),
    // ... 其他类型
};

// 2. 从 input_id 提取参与者
let participant_id = input_id
    .strip_prefix("audio_")
    .unwrap_or("unknown")
    .to_string();
```

### 问题 5: 内存泄漏

**症状:**
- 内存持续增长
- 性能下降

**原因:**
- HashSet 无限增长
- 通道未清理

**解决方案:**
```rust
// 限制 HashSet 大小
if session_start_sent_for.len() > 100 {
    let to_remove: Vec<_> = session_start_sent_for.iter()
        .take(50)
        .cloned()
        .collect();
    for key in to_remove {
        session_start_sent_for.remove(&key);
    }
}

// 使用 bounded channels
let (tx, rx) = bounded(100); // 而非 unbounded()
```

---

## 总结

### 关键要点

1. **动态节点** 是 UI 与数据流之间的桥梁
2. **每个 Widget** 有自己的 Bridge 实例
3. **Dispatcher** 管理所有 Bridge 的生命周期
4. **元数据** 必须处理所有参数类型
5. **流控制** 通过 `audio_complete` 和 `session_start` 实现
6. **非阻塞** 使用 `try_send()` 和 `recv_timeout()`

### 架构优势

- ✅ **解耦**: UI 与业务逻辑分离
- ✅ **灵活**: 可以独立控制每个 Widget 的数据流
- ✅ **类型安全**: Rust 类型系统保证
- ✅ **可扩展**: 易于添加新的节点类型
- ✅ **可维护**: 清晰的层次结构

### 相关文档

- [ARCHITECTURE.md](ARCHITECTURE.md) - 系统架构
- [MOFA_DORA_ARCHITECTURE.md](MOFA_DORA_ARCHITECTURE.md) - Dora 集成架构
- [apps/mofa-fm/dataflow/voice-chat.yml](apps/mofa-fm/dataflow/voice-chat.yml) - 完整数据流示例

---

*最后更新: 2026-01-10*
*作者: MoFA Studio 团队*
