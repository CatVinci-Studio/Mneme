import { createContext, useContext, useState, type ReactNode } from "react";

// 轻量 i18n:界面语言中/英;内容语言与之解耦(wiki 内容保持原文)。
const DICT = {
  zh: {
    queue: "队列", wiki: "Wiki", research: "问答", settings: "设置",
    add: "收藏", search: "搜索…", empty: "还没有内容,点「收藏」添加。",
    addTitle: "收藏内容", url: "网址", orPasteText: "或粘贴文本", titleOpt: "标题(可选)",
    cancel: "取消", save: "收藏并消化", processing: "处理中…",
    summarized: "已摘要", reading: "原文", note: "笔记", wikify: "提升为 Wiki", wikified: "已入 Wiki",
    facts: "事实", history: "历史(已被取代)", contradictions: "矛盾(待人工裁决)", related: "相关", sources: "来源", backlinks: "反向链接",
    keyPoints: "要点", candidates: "候选实体", tldr: "摘要",
    ask: "提问…", answer: "回答", askPlaceholder: "问问你读过的东西……",
    provider: "模型", language: "语言", theme: "主题", light: "浅色", dark: "深色", appearance: "外观", uiLang: "界面语言",
    connected: "已连接", noEntities: "还没有实体页。入库并 Wikify 后会自动生长。",
    graph: "图谱", graphEmpty: "实体还太少,建立链接后这里会显示关系图。",
    aiProvider: "AI 服务", apiKey: "API Key", apiKeySet: "已设置(留空则不变)", model: "模型(可选)", baseUrl: "接口地址", saveCfg: "保存", saved: "已保存",
    keyHint: "Key 存储在系统凭据库中，不进入 vault 或 Git。",  custom: "自定义", demo: "演示(离线,无需 Key)",
    retrieval: "检索方式", builtin: "Rust 本地检索（无需下载）",
    health: "健康检查", runLint: "扫描", orphans: "孤儿页", contradictionsN: "矛盾页", entitiesN: "实体", factsN: "事实",
    backup: "同步 / 备份", gitRemote: "Git 远端地址", backupNow: "立即备份", backupHint: "vault 即 git 仓库,每次写入自动提交;备份会推送到你的私有远端(API Key 不会被提交)。",
    loading: "加载中…", loadFailed: "加载失败", retry: "重试", back: "返回", mainNavigation: "主导航",
    checking: "正在连接…", serviceOnline: "Mneme 服务在线", serviceOffline: "Mneme 服务离线",
    addedSuccess: "内容已收藏并完成消化", wikifySuccess: "已更新 Wiki", referenceExcerpt: "引用原文",
    searchKnowledge: "搜索知识库", searchPlaceholder: "搜索实体、事实和摘要…", searchAction: "搜索", searching: "搜索中…", noResults: "没有找到结果", searchHint: "输入关键词搜索 Wiki，按 Esc 关闭。", sourceNotFound: "无法加载来源", entityNotFound: "无法加载实体", askFailed: "问答失败", words: "字", nodes: "个节点", links: "条链接",
  },
  en: {
    queue: "Queue", wiki: "Wiki", research: "Research", settings: "Settings",
    add: "Add", search: "Search…", empty: "Nothing yet — click Add to save something.",
    addTitle: "Add content", url: "URL", orPasteText: "or paste text", titleOpt: "Title (optional)",
    cancel: "Cancel", save: "Save & digest", processing: "Processing…",
    summarized: "Summarized", reading: "Original", note: "Note", wikify: "Promote to Wiki", wikified: "In Wiki",
    facts: "Facts", history: "History (superseded)", contradictions: "Contradictions (needs review)", related: "Related", sources: "Sources", backlinks: "Backlinks",
    keyPoints: "Key points", candidates: "Candidate entities", tldr: "TL;DR",
    ask: "Ask…", answer: "Answer", askPlaceholder: "Ask about what you've read…",
    provider: "Model", language: "Language", theme: "Theme", light: "Light", dark: "Dark", appearance: "Appearance", uiLang: "Interface language",
    connected: "connected", noEntities: "No entity pages yet. They grow as you ingest and wikify.",
    graph: "Graph", graphEmpty: "Too few entities yet — the link graph shows up once pages connect.",
    aiProvider: "AI provider", apiKey: "API Key", apiKeySet: "set (leave blank to keep)", model: "Model (optional)", baseUrl: "Base URL", saveCfg: "Save", saved: "Saved",
    keyHint: "The key is stored in the system credential store and never enters the vault or Git.", custom: "Custom", demo: "Demo (offline, no key)",
    retrieval: "Retrieval", builtin: "Native Rust search (no download)",
    health: "Health check", runLint: "Scan", orphans: "Orphans", contradictionsN: "Conflicting", entitiesN: "Entities", factsN: "Facts",
    backup: "Sync / Backup", gitRemote: "Git remote URL", backupNow: "Back up now", backupHint: "The vault is a git repo; every write auto-commits. Backup pushes to your private remote (the API key is never committed).",
    loading: "Loading…", loadFailed: "Could not load", retry: "Retry", back: "Back", mainNavigation: "Main navigation",
    checking: "Connecting…", serviceOnline: "Mneme service online", serviceOffline: "Mneme service offline",
    addedSuccess: "Saved and digested", wikifySuccess: "Wiki updated", referenceExcerpt: "Referenced passage",
    searchKnowledge: "Search knowledge", searchPlaceholder: "Search entities, facts, and summaries…", searchAction: "Search", searching: "Searching…", noResults: "No results", searchHint: "Search the Wiki. Press Esc to close.", sourceNotFound: "Could not load source", entityNotFound: "Could not load entity", askFailed: "Research failed", words: "words", nodes: "nodes", links: "links",
  },
};

export type Lang = keyof typeof DICT;
type Key = keyof (typeof DICT)["en"];

const Ctx = createContext<{ lang: Lang; setLang: (l: Lang) => void; t: (k: Key) => string }>({
  lang: "zh", setLang: () => {}, t: (k) => k,
});

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(() => localStorage.getItem("mneme-lang") === "en" ? "en" : "zh");
  const setLang = (next: Lang) => { localStorage.setItem("mneme-lang", next); setLangState(next); };
  const t = (k: Key) => DICT[lang][k] ?? k;
  return <Ctx.Provider value={{ lang, setLang, t }}>{children}</Ctx.Provider>;
}

export const useI18n = () => useContext(Ctx);
