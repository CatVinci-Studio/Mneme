<p align="center">
  <img src="assets/mneme-logo.svg" alt="Mneme" width="128" height="128" />
</p>

<h1 align="center">Mneme</h1>

<p align="center">
  <strong>保存读过的内容，让它生长成真正属于你的 Wiki。</strong><br>
  一款本地优先、由原生 Rust Agent 驱动、事实可追溯的稍后读与个人知识库应用。
</p>

<p align="center">
  <a href="https://github.com/CatVinci-Studio/Mneme/releases/latest"><strong>下载</strong></a> ·
  <a href="./README.md">English</a>
</p>

---

## Mneme 是什么

Mneme（Μνήμη，希腊语“记忆”）把网页、PDF 和文本保存为不可变的本地快照。Rust Agent 会抽取有来源依据的事实，并持续合并到 Markdown Wiki 中：新事实追加，更精确信息保留溯源，失效事实进入历史，互相冲突的说法并列展示、等待人工判断。

它不只是一个阅读队列，而是把你收藏的内容转化为普通文件、Git 历史和长期可携带的个人知识库。

## 为什么做它

普通稍后读产品很容易变成第二个收件箱：内容不断进入，却很少再次被看见。托管知识工具还可能将数据锁在数据库里，服务停止后很难完整迁出。

Mneme 坚持三个原则：

1. **原文不可覆盖**：网页内容变化时创建新快照，不改写旧证据。
2. **事实必须可追溯**：每条事实都记录来源 ID 和精确字符区间。
3. **文件就是产品**：Vault 是可直接阅读的 Markdown 和 Git 仓库，不依赖私有数据库。

## 安装

通过 Homebrew 安装 macOS Apple Silicon 版本：

```sh
brew install --cask catvinci-studio/tap/mneme
```

也可以从 [Releases](https://github.com/CatVinci-Studio/Mneme/releases/latest) 下载 `Mneme_X.Y.Z_aarch64.dmg`。

> 当前版本暂未进行 Apple 公证。如果 macOS 阻止首次启动，请右键点击 **Mneme.app** 并选择“打开”，或执行 `xattr -cr /Applications/Mneme.app`。

## 快速开始

1. 启动 Mneme，进入 **设置 → AI 服务**。
2. 选择 OpenAI、DeepSeek、ChatGLM、Qwen、llama.cpp 或自定义 OpenAI-compatible 服务。API Key 保存在系统凭据库中。
3. 点击 **收藏**，粘贴网页地址或文本，Mneme 会自动生成笔记和 Wiki 实体页。
4. 点击事实旁的 ↩，可以返回对应的原文片段。
5. 使用全局搜索、知识图谱和 Research 问答探索已经积累的知识。

选择 **Demo** 模型可以完全离线体验流程，不需要 API Key。

## 主要能力

- 网页、PDF 和文本收藏
- 基于内容哈希的不可变原文快照
- AI 摘要、实体抽取和事实级 Wikify
- 追加、细化、历史取代和矛盾标记
- 与 WebView 一致的 UTF-16 原文定位
- YAML frontmatter + Markdown 实体页
- Rust 本地搜索、Research、图谱和健康检查
- 自动 Git 提交与可选私有远端备份
- 系统凭据库密钥存储
- 私网 URL 阻止、DNS 固定、逐次重定向检查、流式大小限制与超时
- 中英文、深浅主题、响应式布局与键盘操作

## 技术架构

```text
React + TypeScript + Vite
          │ Tauri invoke
          ▼
原生 Rust Agent Core
  ├── Ingest / Wikify / Research / Janitor
  ├── OpenAI-compatible Providers
  ├── Markdown Wiki Writer
  ├── 本地搜索与知识图谱
  ├── 系统凭据库
  └── Git 版本与备份
```

Node.js 只负责构建前端。安装后的应用没有 Node.js 后端、本地 HTTP API、CORS 边界或 sidecar 进程。

## 从源码构建

需要 Node.js 22+、稳定版 Rust 和 Xcode Command Line Tools。

```bash
git clone https://github.com/CatVinci-Studio/Mneme.git
cd Mneme/ui
npm install
npm run tauri dev
```

检查与构建：

```bash
npm run build
npm run tauri build
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

## 当前范围

0.2 版本首先支持 macOS Apple Silicon。Rust 原生向量检索、后台任务崩溃恢复、动态网页抓取、Apple 签名公证和 Windows/Linux 验证列在 [TASKS.md](TASKS.md) 中。

## License

[MIT](./LICENSE) © 2026 CatVinci Studio
