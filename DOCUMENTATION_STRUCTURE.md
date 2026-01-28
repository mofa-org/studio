# MoFA Studio 文档组织策略

> 📚 **文档层级说明** - 项目级与应用级文档的分工

**创建日期**: 2025-01-21
**目的**: 避免文档重复，明确文档维护责任

---

## 📂 文档结构

### 项目根目录文档（Project-Level Docs）

**目标读者**: Claude Code AI
**用途**: AI 辅助开发的快速参考和资源地图

| 文档 | 用途 | 维护者 |
|------|------|--------|
| **CLAUDE.md** | 项目资源地图、Makepad 安装位置、项目结构概览 | AI 辅助开发维护者 |
| **vibecoding.md** | Vibe Coding 开发流程、最佳实践 | 项目架构师 |
| **MAKEPAD_QUICK_REF.md** | Makepad 组件速查手册、常见模式 | AI 辅助开发维护者 |
| **ARCHITECTURE.md** | MoFA Studio 整体架构 | 项目架构师 |
| **README.md** | 项目概述、快速开始 | 项目维护者 |

### 应用级文档（App-Level Docs）

**目标读者**: 人类开发者
**用途**: 详细的应用开发文档、用户手册、架构设计

#### mofa-cast 应用文档 (`apps/mofa-cast/docs/`)

| 文档 | 用途 | 维护者 |
|------|------|--------|
| **ARCHITECTURE.md** | mofa-cast 技术架构、数据流、组件交互 | mofa-cast 开发者 |
| **DEVELOPMENT.md** | 开发工作流程、项目结构、测试指南 | mofa-cast 开发者 |
| **IMPLEMENTATION_STATUS.md** | 功能实现清单（基于代码审查） | AI 辅助开发 + mofa-cast 开发者 |
| **USER_GUIDE.md** | 用户使用手册、功能说明 | 技术写作人员 |
| **TROUBLESHOOTING.md** | 常见问题和解决方案 | mofa-cast 开发者 |
| **CHANGELOG.md** | 版本变更历史 | mofa-cast 开发者 |
| **HISTORY.md** | 开发历史记录 | mofa-cast 开发者 |
| **SCRIPT_OPTIMIZATION_GUIDE.md** | AI 脚本优化指南 | mofa-cast 开发者 |

#### mofa-fm 应用文档 (`apps/mofa-fm/docs/`)

类似结构（如果存在）

---

## 🔄 文档更新流程

### 场景 1：修改 mofa-cast 功能

**步骤**：
1. 更新应用代码
2. **应用级文档**：更新 `apps/mofa-cast/docs/ARCHITECTURE.md` 或 `DEVELOPMENT.md`
3. **AI 辅助文档**：如果涉及新 API 或模式，更新 `apps/mofa-cast/docs/IMPLEMENTATION_STATUS.md`
4. **项目级文档**：通常不需要更新，除非涉及整体架构变化

### 场景 2：发现 Makepad 组件新用法

**步骤**：
1. **项目根目录文档**：更新 `MAKEPAD_QUICK_REF.md`
2. 应用级文档：更新相关应用的 `docs/DEVELOPMENT.md`（可选）

### 场景 3：项目架构调整

**步骤**：
1. **项目根目录文档**：更新 `ARCHITECTURE.md`
2. **应用级文档**：更新所有受影响应用的 `docs/ARCHITECTURE.md`

---

## 📝 文档内容分工

### 示例：TTS 引擎说明

**项目根目录（CLAUDE.md）**：
```markdown
## 🎯 MoFA-Cast 开发快速参考

**TTS 合成** ✅
- Dora 数据流集成
- **PrimeSpeech TTS 引擎** (多语音支持)
- 多种中文声音: Luo Xiang, Yang Mi, Ma Yun, Ma Baoguo
```
→ 简洁概述，供 AI 快速了解

**应用级文档（ARCHITECTURE.md）**：
```markdown
### 3. Multi-Voice TTS Synthesis Pipeline (✅ P1.1 - v0.5.0)

- **Problem**: Single voice TTS doesn't match multi-speaker podcasts
- **Solution**: Dynamic voice routing with 3 parallel PrimeSpeech nodes
- **Configuration**: multi-voice-batch-tts.yml
- **Nodes**:
  - primespeech-luo-xiang
  - primespeech-ma-yun
  - primespeech-ma-baoguo
```
→ 详细技术实现，供人类开发者深入研究

---

## ✅ 最佳实践

### DO ✅

1. **层级清晰**：项目级文档在根目录，应用级文档在 `apps/*/docs/`
2. **避免重复**：项目根目录只保留快速参考，详细内容在应用文档
3. **交叉引用**：使用 `@` 语法引用相关文档
4. **定期同步**：AI 辅助文档应定期检查应用级文档，确保一致性

### DON'T ❌

1. **不要在项目根目录创建详细的应用文档**（应该在 `apps/*/docs/`）
2. **不要让应用文档与项目根目录文档冲突**（保持一致性）
3. **不要忽视现有文档**（创建新文档前先检查是否有重叠）

---

## 🔍 文档检查清单

创建或更新文档时，先确认：

- [ ] 是否有类似文档已存在？
- [ ] 应该放在项目根目录还是应用 `docs/` 目录？
- [ ] 目标读者是谁（AI 还是人类开发者）？
- [ ] 是否需要更新其他相关文档？
- [ ] 是否使用 `@` 语法添加交叉引用？

---

## 📚 相关资源

- **Diátaxis Framework**: https://diataxis.fr/ - 文档分类的最佳实践
  - Tutorials (教程)
  - How-to Guides (操作指南)
  - Explanation (解释)
  - Reference (参考)

**MoFA Studio 采用的简化版**：
- **项目根目录** → Reference（快速参考）
- **应用 docs/** → Explanation + How-to（详细指南）

---

**维护者**: Claude Code Assistant
**最后更新**: 2025-01-21
