# Makepad 组件速查手册

> 🎨 **快速参考** - Makepad UI 组件库使用指南

本文档提供 Makepad widgets 的快速参考，帮助你在 MoFA Studio 开发中快速查找和使用组件。

**更新日期**: 2025-01-21
**Makepad 版本**: b8b65f4fa
**ui_zoo 示例数**: 28 个组件示例

---

## 📚 目录

- [基础组件](#基础组件)
- [输入组件](#输入组件)
- [布局组件](#布局组件)
- [数据展示](#数据展示)
- [反馈组件](#反馈组件)
- [高级组件](#高级组件)
- [常见模式](#常见模式)
- [快速查找](#快速查找)

---

## 🎯 基础组件

### Button 按钮

**源码**: `widgets/src/button.rs`
**示例**: `examples/ui_zoo/src/tab_button.rs`

**基础用法**:
```rust
live_design! {
    use makepad_widgets::*;

    <Button> {
        text: "Click Me"
        icon: IconId(MofaIconPlay)
    }
}
```

**事件处理**:
```rust
if self.button(ids!(my_button)).clicked(actions) {
    // 处理点击
}
```

**样式选项**:
- `text`: 按钮文本
- `icon`: 图标 ID
- `enabled`: 是否可用

---

### Label 标签

**源码**: `widgets/src/label.rs`
**示例**: `examples/ui_zoo/src/tab_label.rs`

**基础用法**:
```rust
<Label> {
    text: "Hello, Makepad!"
    draw_text: {
        text_style: <FONT_REGULAR> {
            font_size: 18.0
            height_factor: 1.2
        }
        fn get_color(self) -> vec4 {
            (TEXT_PRIMARY)
        }
    }
}
```

**动态更新**:
```rust
self.label(ids!(my_label)).set_text(cx, "New text");
```

---

### Image 图片

**源码**: `widgets/src/image.rs`
**示例**: `examples/ui_zoo/src/tab_image.rs`

**基础用法**:
```rust
<Image> {
    source: DepOfBuf(assets::IMAGE_NAME),
    width: 100.0,
    height: 100.0
}
```

**加载图片**:
```rust
// 需要在 assets 中声明
```

---

### Icon 图标

**源码**: `widgets/src/icon.rs`
**示例**: `examples/ui_zoo/src/tab_icon.rs`

**基础用法**:
```rust
<Icon> {
    icon: IconId(MofaIconPlay),
    draw_icon: {
        fn get_color(self) -> vec4 {
            (ACCENT_BLUE)
        }
    }
}
```

---

## 📝 输入组件

### TextInput 文本输入框

**源码**: `widgets/src/text_input.rs`
**示例**: `examples/ui_zoo/src/tab_textinput.rs`

**基础用法**:
```rust
<TextInput> {
    text: "Placeholder text"
    draw_bg: {
        color: (DARK_BG)
    }
}
```

**获取输入**:
```rust
let text = self.text_input(ids!(my_input)).text(cx);
```

**设置文本**:
```rust
self.text_input(ids!(my_input)).set_text(cx, "New text");
```

---

### Slider 滑块

**源码**: `widgets/src/slider.rs`
**示例**: `examples/ui_zoo/src/tab_slider.rs`

**基础用法**:
```rust
<Slider> {
    min: 0.0,
    max: 100.0,
    step: 1.0,
    value: 50.0
}
```

**事件处理**:
```rust
if let Some(val) = self.slider(ids!(my_slider)).changed(actions) {
    // val 是 f64 类型
}
```

---

### CheckBox 复选框

**源码**: `widgets/src/check_box.rs`
**示例**: `examples/ui_zoo/src/tab_checkbox.rs`

**基础用法**:
```rust
<CheckBox> {
    text: "Check me"
    checked: true
}
```

**事件处理**:
```rust
if let Some(checked) = self.check_box(ids!(my_checkbox)).changed(actions) {
    // checked 是 bool 类型
}
```

---

### RadioButton 单选按钮

**源码**: `widgets/src/radio_button.rs`
**示例**: `examples/ui_zoo/src/tab_radiobutton.rs`

**基础用法**:
```rust
<RadioButton> {
    text: "Option 1"
    selected: true
}
```

---

### DropDown 下拉菜单

**源码**: `widgets/src/drop_down.rs`
**示例**: `examples/ui_zoo/src/tab_dropdown.rs`

**基础用法**:
```rust
<DropDown> {
    labels: <["Option 1", "Option 2", "Option 3"]>
    selected: 0
}
```

**事件处理**:
```rust
if let Some(selected) = self.drop_down(ids!(my_dropdown)).selected(actions) {
    // selected 是 usize 类型 (索引)
}
```

**设置选项**:
```rust
// 在 live_design! 中使用 labels 参数
labels: <["Item 1", "Item 2", "Item 3"]>
```

---

## 📐 布局组件

### View 视图容器

**源码**: `widgets/src/view.rs`
**示例**: `examples/ui_zoo/src/tab_view.rs`

**基础用法**:
```rust
<View> {
    width: Fill,
    height: Fill,
    flow: Down,       // 或 Right
    padding: 20.0,
    margin: {top: 10.0, bottom: 10.0}
    spacing: 10.0     // 子元素间距
}
```

**布局模式**:
- `flow: Down` - 垂直布局
- `flow: Right` - 水平布局
- `width: Fill` - 填充父容器
- `width: Fit` - 适应内容
- `height: Fill` - 填充父容器
- `height: Fit` - 适应内容

**常用属性**:
- `padding`: 内边距
- `margin`: 外边距
- `spacing`: 子元素间距
- `align`: 对齐方式

---

### ScrollBar 滚动条

**源码**: `widgets/src/scroll_bar.rs`
**示例**: `examples/ui_zoo/src/tab_scrollbar.rs`

**基础用法**:
```rust
<ScrollView> {
    width: Fill,
    height: Fill,

    <View> {
        // 内容
    }
}
```

---

### Splitter 分割器

**源码**: `widgets/src/splitter.rs`
**示例**: `examples/ui_zoo/src/...`

**基础用法**:
```rust
<Splitter> {
    axis: Horizontal,  // 或 Vertical
    min_a: 100.0,      // A 面板最小宽度
    min_b: 100.0,      // B 面板最小宽度
}
```

---

### PortalList 列表视图

**源码**: `widgets/src/portal_list.rs`
**示例**: `examples/ui_zoo/src/tab_portallist.rs`

**用途**: 高性能滚动列表

**基础用法**:
```rust
<PortalList> {
    width: Fill,
    height: Fill
    // 需要配合数据源使用
}
```

---

## 📊 数据展示

### Markdown 渲染

**源码**: `widgets/src/markdown.rs`
**示例**: `examples/ui_zoo/src/tab_markdown.rs`

**基础用法**:
```rust
<Markdown> {
    width: Fill,
    height: Fill,
    // 内容通过 set_text() 设置
}
```

**设置内容**:
```rust
self.markdown(ids!(my_markdown)).set_text(cx, "# Title\n\nContent...");
```

---

### HTML 渲染

**源码**: `widgets/src/html.rs`
**示例**: `examples/ui_zoo/src/tab_html.rs`

**基础用法**:
```rust
<Html> {
    width: Fill,
    height: Fill
}
```

---

### FileTree 文件树

**源码**: `widgets/src/file_tree.rs`
**示例**: `examples/ui_zoo/src/tab_filetree.rs`

**基础用法**:
```rust
<FileTree> {
    width: Fill,
    height: Fill
}
```

---

## 🔔 反馈组件

### PopupMenu 弹出菜单

**源码**: `widgets/src/popup_menu.rs`
**示例**: `examples/ui_zoo/src/tab_*.rs` (各个组件示例中)

**基础用法**:
```rust
// 在事件处理中弹出
self.popup_menu(ids!(my_menu)).show_menu(cx);
```

---

### Modal 模态对话框

**源码**: `widgets/src/modal.rs`
**示例**: `examples/...`

**基础用法**:
```rust
<Modal> {
    // 对话框内容
}
```

---

### Tooltip 工具提示

**源码**: `widgets/src/tooltip.rs`
**示例**: `examples/...`

**基础用法**:
```rust
// 在组件上添加 tooltip
```

---

### PopupNotification 通知

**源码**: `widgets/src/popup_notification.rs`
**示例**: `examples/...`

**基础用法**:
```rust
<PopupNotification> {
    // 通知内容
}
```

---

### LoadingSpinner 加载动画

**源码**: `widgets/src/loading_spinner.rs`
**示例**: `examples/...`

**基础用法**:
```rust
<LoadingSpinner> {
    // 加载动画
}
```

---

## 🚀 高级组件

### Dock 停靠面板

**源码**: `widgets/src/dock.rs`
**示例**: `examples/...`

**基础用法**:
```rust
<Dock> {
    // 停靠面板布局
}
```

---

### TabBar 标签栏

**源码**: `widgets/src/tab_bar.rs`
**示例**: `examples/...`

**基础用法**:
```rust
<TabBar> {
    // 标签栏
}
```

---

### ColorPicker 颜色选择器

**源码**: `widgets/src/color_picker.rs`
**示例**: `examples/ui_zoo/src/...` (如果存在)

**基础用法**:
```rust
<ColorPicker> {
    // 颜色选择
}
```

---

### WebView 网页视图

**源码**: `widgets/src/web_view.rs`
**示例**: `examples/...`

**注意**: 仅在某些平台支持

---

## 🎨 常见模式

### 1. 使用共享主题

```rust
live_design! {
    use makepad_widgets::*;
    use mofa_widgets::theme::*;

    pub MyScreen = {{MyScreen}} {
        // 使用预定义颜色和字体
        draw_bg: { color: (DARK_BG) }

        <Label> {
            draw_text: {
                text_style: <FONT_REGULAR> { font_size: 14.0 }
                fn get_color(self) -> vec4 { (TEXT_PRIMARY) }
            }
        }
    }
}
```

### 2. 响应式布局

```rust
<View> {
    width: Fill,
    height: Fill,
    flow: Down,

    // 顶部固定高度
    <View> {
        height: Fit,
        margin: {bottom: 20}
    }

    // 中间填充剩余空间
    <View> {
        height: Fill
    }

    // 底部固定高度
    <View> {
        height: Fit
    }
}
```

### 3. 动态显示/隐藏

```rust
// Rust 代码
self.view(ids!(my_panel)).set_visible(cx, true);
self.view(ids!(my_panel)).set_visible(cx, false);
```

### 4. 动态样式更新

```rust
// 使用 apply_over 更新样式
self.view(ids!(my_view)).apply_over(cx, live!{
    draw_bg: {
        color: vec3(1.0, 0.0, 0.0)  // 红色
    }
});
```

### 5. 自定义绘制

```rust
draw_bg: {
    uniform opacity: 1.0,
    uniform border_radius: 4.0,

    fn pixel(self) -> vec4 {
        let color = self.color;
        let alpha = self.opacity;

        // 自定义绘制逻辑
        return vec4(color.rgb * alpha, alpha);
    }
}
```

### 6. 动画和过渡

```rust
// 使用 shader 动画
draw_bg: {
    fn pixel(self) -> vec4 {
        let t = self.time * 0.001;  // 时间（秒）
        let pulse = sin(t) * 0.5 + 0.5;

        return vec4(self.color.rgb * pulse, 1.0);
    }
}
```

### 7. 多个事件处理

```rust
impl Widget for MyScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => return,
        };

        // 按钮 1
        if self.button(ids!(button1)).clicked(actions) {
            self.handle_button1(cx);
        }

        // 按钮 2
        if self.button(ids!(button2)).clicked(actions) {
            self.handle_button2(cx);
        }

        // 下拉菜单
        if let Some(selected) = self.drop_down(ids!(my_dropdown)).selected(actions) {
            self.handle_dropdown(cx, selected);
        }
    }
}
```

---

## 🔍 快速查找

### 按组件类型查找

**基础**: Button, Label, Image, Icon
**输入**: TextInput, Slider, CheckBox, RadioButton, DropDown
**布局**: View, ScrollBar, Splitter, PortalList, StackNavigation
**数据**: Markdown, Html, FileTree, TextFlow
**反馈**: PopupMenu, Modal, Tooltip, Notification, LoadingSpinner
**高级**: Dock, TabBar, ColorPicker, WebView, Video

### 按功能需求查找

**需要按钮** → Button
**需要文本输入** → TextInput
**需要显示多行文本** → Label 或 Markdown
**需要选择** → DropDown, RadioButton, CheckBox
**需要列表** → PortalList, FlatList
**需要滚动** → ScrollView
**需要布局** → View (flow: Down/Right)
**需要弹出** → PopupMenu, Modal
**需要通知** → PopupNotification
**需要加载状态** → LoadingSpinner

### 按示例位置查找

```bash
# 查看所有 ui_zoo 示例
ls ~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/examples/ui_zoo/src/

# 查看组件源码
ls ~/.cargo/git/checkouts/makepad-721ba110953b28bc/b8b65f4/widgets/src/
```

### 示例文件映射

| 组件 | 示例文件 |
|------|----------|
| Button | `tab_button.rs` |
| Label | `tab_label.rs` |
| TextInput | `tab_textinput.rs` |
| Slider | `tab_slider.rs` |
| DropDown | `tab_dropdown.rs` |
| CheckBox | `tab_checkbox.rs` |
| RadioButton | `tab_radiobutton.rs` |
| ScrollBar | `tab_scrollbar.rs` |
| View | `tab_view.rs` |
| Image | `tab_image.rs` |
| Icon | `tab_icon.rs` |
| Markdown | `tab_markdown.rs` |
| HTML | `tab_html.rs` |
| FileTree | `tab_filetree.rs` |
| PortalList | `tab_portallist.rs` |
| ... | ... |

---

## 📚 相关资源

### 官方文档

- **Makepad GitHub**: https://github.com/wyeworks/makepad
- **Makepad 文档**: https://makepad.nl/
- **Makepad Discord**: https://discord.gg/adqBRq7Ece

### 项目内文档

- **CLAUDE.md** - 项目资源地图
- **vibecoding.md** - 开发流程和最佳实践
- **MOFA_CAST_DEV_CHECKLIST.md** - mofa-cast 功能清单

### 学习路径

1. **入门**: 运行 `examples/simple/`
2. **组件学习**: 运行 `examples/ui_zoo/` 并查看所有组件
3. **完整应用**: 学习 `examples/ironfish/`
4. **实际项目**: 参考项目内 `apps/mofa-fm/` 和 `apps/mofa-cast/`

---

## 💡 最佳实践

### DO ✅

- **参考 ui_zoo**: 所有组件都有示例
- **使用共享主题**: 保持一致的视觉风格
- **简洁的事件处理**: 每个事件一个方法
- **响应式布局**: 使用 Fill/Fit 适应不同屏幕
- **错误处理**: 使用 Result 类型处理错误

### DON'T ❌

- **不要重复造轮子**: 先查找是否已有组件
- **不要硬编码样式**: 使用主题常量
- **不要忽略错误**: 使用 `?` 或 `unwrap_or()`
- **不要过度嵌套 View**: 保持布局层次简洁
- **不要忘记重绘**: 修改状态后调用 `self.view.redraw(cx)`

---

## 🎯 快速提示

### 如何使用新组件？

1. **查找示例**: `ls examples/ui_zoo/src/`
2. **查看源码**: `cat widgets/src/<component>.rs`
3. **复制模式**: 复制 live_design! 和事件处理
4. **修改适配**: 根据需求调整

### 如何调试样式？

1. **使用 apply_over**: 动态修改样式查看效果
2. **检查 live_design!**: 确保语法正确
3. **查看示例**: 对比工作示例
4. **简化测试**: 从最小示例开始

### 如何优化性能？

1. **PortalList**: 大量数据使用 PortalList
2. **避免频繁重绘**: 只在必要时调用 redraw
3. **使用 CachedWidget**: 缓存复杂组件
4. **减少层级**: 避免过深的 View 嵌套

---

**维护者**: Claude Code Assistant
**最后更新**: 2025-01-21
**Makepad 版本**: b8b65f4fa
