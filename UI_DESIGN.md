# Mneme — UI 设计方案

> 目标:为 Mneme(稍后读 → 自生长 LLM Wiki)设计前端。
> 约束:React;动态美观、中英双语;界面干净、设置通俗易懂;配色专业沉稳;
> 字体 英文 Arial/Times New Roman、中文 宋体/微软雅黑。
> 运行环境:Tauri WebView(桌面本地)或 headless 托管(浏览器)——同一套 React。

---

## 1. 技术栈

| 关注点 | 选型 | 理由 |
|--------|------|------|
| 框架 | **React 18 + TypeScript** | 要求 React |
| 构建/dev server | **Node.js + Vite** | 标准、成熟的 React 工具链，使用 npm 管理依赖，支持热更新和生产构建 |
| 样式 | **CSS 变量(design tokens)** | CSS 变量承载主题/字体，切换不重编译；需要时可通过 Vite 引入 Tailwind CSS |
| 组件基座 | **Radix UI primitives**(可选 shadcn/ui) | 无样式、可访问性好,契合"干净、自定义沉稳风" |
| 路由 | react-router | 多视图 |
| 数据 | **TanStack Query** | 与 Agent Core 的异步数据、任务进度、缓存 |
| UI 状态 | **zustand** | 轻量(布局、面板、主题、语言) |
| 国际化 | **react-i18next** | 中英双语,字符串全外置 |
| 动画 | **Framer Motion** | "动态"——视图切换、卡片、面板过渡 |
| 布局 | **react-resizable-panels** | 可重排/可调的动态面板布局 |
| 内容渲染 | react-markdown + remark | 渲染原文与 wiki 页;脚注 → 点回原文 |
| 图谱 | react-force-graph(d3) | 实体↔实体 / 实体↔来源 图视图 |
| 通信抽象 | `src/api.ts` | 封装 Tauri `invoke` 与 HTTP+SSE,前端无感切换本地/托管 |

---

## 2. 配色:专业、沉稳(design tokens)

主推 **「Ink & Parchment(墨与纸)」**——暖中性纸感背景 + 墨色文字 + 深青强调色。长时间阅读不刺眼,气质专业克制,契合"知识/阅读"产品。全部以 CSS 变量定义,支持浅/深色。

```css
/* 浅色(默认) */
--bg:        #FAF9F7;   /* 暖纸背景 */
--surface:   #FFFFFF;   /* 卡片/面板 */
--surface-2: #F3F1EC;   /* 次级面 */
--text:      #1C1B19;   /* 墨色正文 */
--text-muted:#6B6862;   /* 次要文字 */
--border:    #E5E2DC;
--accent:    #0F6E6E;   /* 深青(主行动) */
--accent-fg: #FFFFFF;
--warn:      #B4541E;   /* 矛盾/警示(沉稳琥珀棕,非刺眼红) */
--ok:        #3F7D58;

/* 深色 */
--bg:        #16181D;
--surface:   #1E2127;
--surface-2: #23272E;
--text:      #E8E6E1;
--text-muted:#9A968E;
--border:    #2C313A;
--accent:    #4FB7B3;
--warn:      #D08A5A;
--ok:        #6BB389;
```

> 备选палитра(若想换气质):**「Slate Professional」**(冷灰 + 收敛靛蓝 #3B5BA9)、**「Graphite + Amber」**(石墨灰 + 低饱和琥珀)。三者都刻意避开高饱和、避免"科技蓝紫"的廉价感。

强调色克制使用:仅主按钮、当前选中、链接。大面积保持中性,信息靠层级而非颜色堆叠。

---

## 3. 字体(按要求,分工使用)

把两套字体**按用途分工**,而非混用——既满足要求,又有设计意图:

- **UI 界面(导航/按钮/设置/卡片)= 无衬线**:`Arial` + `微软雅黑`
- **阅读正文(原文 + wiki 页正文)= 衬线**:`Times New Roman` + `宋体`(serif 更适合长文阅读,给"文档感")

```css
--font-ui:      Arial, "Microsoft YaHei", "微软雅黑", system-ui, sans-serif;
--font-reading: "Times New Roman", "SimSun", "宋体", Georgia, serif;
--font-mono:    ui-monospace, "SFMono-Regular", Menlo, monospace;
```

字号阶梯(rem):UI 12/13/14/16;阅读正文 17–18(行高 1.7,measure ≤ 70 字符)。
设置里提供「界面字体 / 阅读字体」开关,允许用户在 sans/serif 间切换(满足"通俗易懂"的可调)。

---

## 4. 国际化(中英双语)

- `react-i18next`,locale 文件 `src/locales/{zh,en}.json`,UI 字符串全部 key 化。
- 语言切换:顶栏一键切换 + 设置页;选择持久化(localStorage / 配置)。
- **关键区分**:界面语言(中/英)与**内容语言**相互独立——wiki 内容始终保持原文语言(对应 AGENT_SPEC 的规则),界面切中文不会把英文文章翻译掉。
- 日期/数字按 locale 格式化。

---

## 5. 布局与信息架构

**App Shell(动态、可重排)**

```
┌──────────────────────────────────────────────────────────────┐
│ TopBar: [Mneme]   🔍 全局搜索        [+ 收藏] [中/EN] [☾主题] │
├───────┬──────────────────────────────────────┬───────────────┤
│ Side  │  主内容区(随视图变化)               │ 右侧上下文面板 │
│ nav   │                                      │ (可折叠)      │
│       │                                      │               │
│ 队列  │                                      │ backlinks /   │
│ Wiki  │                                      │ 溯源 /        │
│ 图谱  │                                      │ 任务进度      │
│ 检索  │                                      │               │
│ 设置  │                                      │               │
├───────┴──────────────────────────────────────┴───────────────┤
│ StatusBar: Rust Core ●在线  |  provider: llama.cpp  |  本地检索 │
└──────────────────────────────────────────────────────────────┘
```

- 侧栏可收起为图标条;主区与右侧面板用 `react-resizable-panels` 可拖拽调宽;
- 响应式:窄屏右面板转为抽屉、侧栏转底部 tab;
- 视图切换、面板展开用 Framer Motion 做轻过渡(120–180ms,缓动克制)。

---

## 6. 核心视图(对应后端能力)

### 6.1 收藏队列 Inbox / Queue
- 卡片流:标题 / 来源域名 / 阅读时长 / 标签;
- **状态徽章**:`仅原文` → `已摘要` → `已入 Wiki`(对应 ingest_state),让"消化进度"一眼可见;
- 顶部筛选:未读/在读/归档、标签、来源格式(网页/PDF/论文);
- 动态聚合:按主题/时间分组可切换。

### 6.2 阅读器 Reader(双栏)
```
┌───────────── 原文(serif)──────────┬──────── 笔记/Wiki ────────┐
│ 标题、作者、来源                     │ TL;DR                     │
│ 干净正文(可高亮)                   │ 要点                      │
│  …高亮处可点 → 右侧定位/溯源…        │ 候选实体 → [一键 Wikify]  │
│                                     │ 关联实体 [[...]]          │
└─────────────────────────────────────┴───────────────────────────┘
```
- 论断高亮 ↔ 原文字符偏移联动(脚注 `src:ID@start-end`);
- 顶部进度:`已摘要` / `Wikify 中…`(SSE 实时);

### 6.3 Wiki / 实体页
- 渲染 Summary / Facts / **History(浅灰,标"已被取代")** / **Contradictions(琥珀警示卡,"待人工裁决"+ 采纳/忽略)** / Related / Sources;
- 每条 fact 行尾小图标 → 点回原文出处;
- 右面板:**backlinks(反向链接)**——哪些页/来源指向本页;
- 双向链接 `[[slug]]` 可点跳转(Obsidian 风)。

### 6.4 图谱 Graph
- 实体↔实体(硬链接)+ 实体↔来源(提及);
- 点节点 → 高亮邻居 + 跳实体页;非炫技,作为导航与发现;
- 规模大时按当前实体的 2 跳子图渲染。

### 6.5 检索 / 问答 Research
- 对话式;答案内引用 `[[entity]]` 与 `(src:ID@…)` **可点**;
- 上方提供本地关键词检索与基于检索上下文的 Research 问答。

### 6.6 设置 Settings(通俗易懂)
分组 + 大白话说明 + 状态指示,不堆术语:
- **模型**:单选 `本地(llama.cpp)/ 云端(Anthropic)/ 演示(mock)`;选本地填地址(默认 `http://127.0.0.1:8080`),选云端填 key(存系统 keyring);旁边实时显示 `●已连接 / ○未连接`。
- **检索**：内置 Rust 本地检索，无需下载模型；
- **数据**:vault 路径(可选本地或托管同步 git remote);"导出全部数据"按钮(强调数据主权);
- **外观**:浅/深/跟随系统;界面字体(Arial/微软雅黑)、阅读字体(Times/宋体)切换;
- **语言**:中文 / English。

### 6.7 收藏入口 Add(命令面板式)
- 顶栏 `+ 收藏` 或快捷键唤出:粘贴 URL / 选文件;
- 自动识别格式(网页/PDF/论文/文本)并显示用了哪个适配器;
- 提交后回到队列,卡片即时出现并显示"摘要中…"。

---

## 7. 组件清单(首批)

`AppShell` `SideNav` `TopBar` `StatusBar` `ResizablePanels` ·
`SourceCard` `StateBadge` `FilterBar` ·
`ReaderPane` `NotePane` `HighlightLayer` `WikifyButton` ·
`EntityPage` `FactList` `HistoryList` `ContradictionCard` `BacklinkPanel` `WikiLink` ·
`GraphView` ·
`ResearchChat` `Citation` ·
`SettingsForm` `ProviderPicker` `ConnectionDot` `FontToggle` `ThemeToggle` `LangToggle` ·
`AddDialog` `CommandPalette` `TaskProgress(SSE)` `Toast`。

---

## 8. 与 Agent Core 通信(`src/api.ts`)

单一抽象,屏蔽本地/托管差异:
```ts
// 桌面(Tauri):invoke('add_source', {...})
// 托管:fetch('/api/sources', {...}) + EventSource('/api/jobs/stream')
export const api = {
  addSource, listSources, getSource, wikify,
  getEntity, query, search, listEntities,
  jobsStream,            // SSE,推送 ingest/wikify 进度
  getConfig, setConfig,  // provider / retriever / 语言 / 主题
};
```
任务进度(摘要中 / Wikify 中 / 完成)经 SSE 推到 `TaskProgress` 与卡片徽章。

---

## 9. 动态与可访问性

- **动态**:视图切换淡入滑移、卡片入场 stagger、面板拖拽实时、Wikify 进度脉冲——全部克制(短时长、低位移),专业而非花哨;尊重 `prefers-reduced-motion`。
- **可访问性**:Radix 保证键盘/焦点/ARIA;对比度满足 WCAG AA;强调色不作为唯一信息载体(矛盾卡同时有图标+文案)。
- **深色模式**:同一套 token,纸感 → 墨感平滑切换。

---

## 10. 目录结构(前端)

```
ui/
├── index.html                 # Vite HTML 入口
├── package.json               # npm scripts:dev / build / preview / tauri
├── package-lock.json          # npm 锁定依赖版本
├── src/
│   ├── main.tsx  App.tsx
│   ├── styles/tokens.css      # 配色/字体 CSS 变量(浅/深)
│   ├── api.ts                 # Tauri/HTTP 抽象
│   ├── store/                 # zustand:layout/theme/lang
│   ├── locales/{zh,en}.json   # i18n
│   ├── components/            # 见 §7
│   └── views/                 # Queue / Reader / Wiki / Graph / Research / Settings
└── tailwind.config.ts
```

> Tauri 集成：`beforeDevCommand = "npm run dev"`、`beforeBuildCommand = "npm run build"`、`devUrl` 指向 Vite dev server，`frontendDist = "../dist"`。Node.js 仅用于前端构建，运行时 Agent Core 完全位于 Rust/Tauri 进程中。

---

## 11. 落地顺序(建议)

1. **设计系统先行**:tokens.css(配色+字体)+ AppShell + 主题/语言切换 → 把"沉稳专业+双语+双字体"的地基钉死;
2. 队列 + Reader(双栏)→ 读起来;
3. 实体页(Facts/History/Contradictions/Backlinks)→ 体现 Wiki 差异化;
4. 设置页(模型/检索/外观/语言)→ 通俗易懂;
5. 图谱 + 问答 → 锦上添花。
